use std::net::SocketAddr;

use ignitia::{
    middleware::{from_fn, CorsMiddleware, LoggerMiddleware, Next},
    response::IntoResponse,
    Error, Request, Response, Result, Router, Server,
};
use tracing_subscriber;

// Custom Bearer token authentication middleware using Next pattern
async fn bearer_auth_middleware(req: Request, next: Next) -> Response {
    let path = req.uri.path();

    // Only protect /protected path
    if path.starts_with("/protected") {
        // Check for Authorization header
        match req.header("authorization") {
            Some(auth_header) => {
                // Validate Bearer token
                if auth_header.starts_with("Bearer ") {
                    let token = auth_header.trim_start_matches("Bearer ");

                    if token == "secret-token" {
                        // Token is valid, proceed to next middleware/handler
                        println!("✅ Valid Bearer token for {}", path);
                        return next.run(req).await;
                    } else {
                        println!("❌ Invalid Bearer token for {}", path);
                        return Error::Unauthorized("Invalid token".to_string()).into_response();
                    }
                } else {
                    println!("❌ Invalid Authorization header format for {}", path);
                    return Error::Unauthorized(
                        "Invalid authorization format. Use: Bearer <token>".to_string(),
                    )
                    .into_response();
                }
            }
            None => {
                println!("❌ Missing Authorization header for {}", path);
                return Error::Unauthorized("Authorization header required".to_string())
                    .into_response();
            }
        }
    }

    // Not a protected path, proceed
    next.run(req).await
}

// Custom request ID middleware
async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    use uuid::Uuid;

    let request_id = Uuid::new_v4().to_string();

    // Add request ID to request headers
    req.headers.insert(
        http::HeaderName::from_static("x-request-id"),
        http::HeaderValue::from_str(&request_id).unwrap(),
    );

    println!("🆔 Request ID: {}", request_id);

    // Process request
    let mut response = next.run(req).await;

    // Add request ID to response headers
    response.headers.insert(
        http::HeaderName::from_static("x-request-id"),
        http::HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}

// Custom rate limiting middleware (simple example)
async fn rate_limit_middleware(req: Request, next: Next) -> Response {
    // In a real app, you'd track requests per IP/user with timestamps
    // For demo purposes, we'll just log and pass through

    let client_ip = req
        .header("x-forwarded-for")
        .or_else(|| req.header("x-real-ip"))
        .unwrap_or("unknown");

    println!("📊 Rate limit check for IP: {}", client_ip);

    // In production: check rate limits here
    // if rate_limit_exceeded() {
    //     return Response::text("Too Many Requests")
    //         .with_status(StatusCode::TOO_MANY_REQUESTS);
    // }

    next.run(req).await
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        // Apply middleware in order using Next pattern
        .middleware(LoggerMiddleware)
        .middleware(CorsMiddleware::new().build()?) // or use .new() for defaults
        .middleware(from_fn(request_id_middleware))
        .middleware(from_fn(rate_limit_middleware))
        .middleware(from_fn(bearer_auth_middleware))
        // Routes using modern extractor pattern
        .get("/public", public_route)
        .get("/protected", protected_route)
        .get("/", home_page);

    let addr: SocketAddr = "127.0.0.1:3001".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🛡️ Server with Next middleware running on http://127.0.0.1:3001");
    println!("📋 Middleware stack (Axum-style Next pattern):");
    println!("   ✅ Logger middleware - logs all requests");
    println!("   ✅ CORS middleware - handles cross-origin requests");
    println!("   ✅ Request ID middleware - adds unique request IDs");
    println!("   ✅ Rate limit middleware - tracks request rates");
    println!("   ✅ Auth middleware - protects /protected route");
    println!();
    println!("🔗 Available routes:");
    println!("   GET / - Home page with information");
    println!("   GET /public - Public route (no auth required)");
    println!("   GET /protected - Protected route (requires Bearer token)");
    println!();
    println!("🔑 To test protected route:");
    println!("   curl -H \"Authorization: Bearer secret-token\" http://127.0.0.1:3001/protected");

    server.ignitia().await.unwrap();
    Ok(())
}

// Simple handler with no extractors - just returns response
async fn public_route() -> Result<Response> {
    Ok(Response::html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Public Route</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; max-width: 600px; margin: 40px auto; }
                .public { background: #d4edda; padding: 20px; border-radius: 10px; }
            </style>
        </head>
        <body>
            <div class="public">
                <h1>🌍 Public Route</h1>
                <p>This is a public route - no authentication required!</p>
                <p><strong>Middleware applied (Next pattern):</strong></p>
                <ul>
                    <li>✅ Logger middleware (request logged)</li>
                    <li>✅ CORS middleware (CORS headers added)</li>
                    <li>✅ Request ID middleware (unique ID assigned)</li>
                    <li>✅ Rate limit middleware (request tracked)</li>
                    <li>⏭️ Auth middleware (skipped - not protected path, called next.run())</li>
                </ul>
                <p>Check the response headers for <code>x-request-id</code>!</p>
                <p><a href="/">← Back to Home</a></p>
            </div>
        </body>
        </html>
        "#,
    ))
}

// Protected route - auth middleware will validate token before this handler runs
async fn protected_route() -> Result<Response> {
    Ok(Response::html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Protected Route</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; max-width: 600px; margin: 40px auto; }
                .protected { background: #fff3cd; padding: 20px; border-radius: 10px; }
            </style>
        </head>
        <body>
            <div class="protected">
                <h1>🔒 Protected Route</h1>
                <p>Congratulations! You're authenticated and can access this protected resource.</p>
                <p><strong>Middleware chain executed (Next pattern):</strong></p>
                <ol>
                    <li>✅ Logger middleware → called next.run()</li>
                    <li>✅ CORS middleware → called next.run()</li>
                    <li>✅ Request ID middleware → called next.run()</li>
                    <li>✅ Rate limit middleware → called next.run()</li>
                    <li>✅ Auth middleware → validated token, called next.run()</li>
                    <li>✅ Handler executed!</li>
                </ol>
                <p>Your Bearer token was validated by the custom auth middleware using the Next pattern.</p>
                <p>Each middleware called <code>next.run(req).await</code> to continue the chain.</p>
                <p><a href="/">← Back to Home</a></p>
            </div>
        </body>
        </html>
        "#,
    ))
}

// Home page with information about the middleware demo
async fn home_page() -> Result<Response> {
    Ok(Response::html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Middleware Demo - Next Pattern</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; max-width: 800px; margin: 40px auto; }
                h1 { color: #333; }
                .middleware-info { background: #f0f8ff; padding: 20px; border-radius: 10px; margin: 20px 0; }
                .routes { background: #f8f9fa; padding: 20px; border-radius: 10px; margin: 20px 0; }
                .test-info { background: #e7f3ff; padding: 20px; border-radius: 10px; margin: 20px 0; }
                .next-pattern { background: #e8f5e9; padding: 20px; border-radius: 10px; margin: 20px 0; }
                .action-buttons { margin: 20px 0; }
                .action-buttons a {
                    display: inline-block;
                    padding: 10px 20px;
                    margin: 5px;
                    text-decoration: none;
                    border-radius: 5px;
                    color: white;
                }
                .btn-public { background: #28a745; }
                .btn-protected { background: #007acc; }
                code { background: #f1f1f1; padding: 2px 6px; border-radius: 3px; }
            </style>
        </head>
        <body>
            <h1>🛡️ Middleware Demo - Axum-Style Next Pattern</h1>

            <div class="next-pattern">
                <h3>🔄 Next Pattern Explained:</h3>
                <p>Each middleware has access to a <code>Next</code> object that represents the rest of the chain:</p>
                <pre>async fn my_middleware(req: Request, next: Next) -> Response {
    // Do something before
    println!("Before handler");

    // Call next middleware/handler
    let response = next.run(req).await;

    // Do something after
    println!("After handler");

    response
}</pre>
                <p>This is exactly how Axum middleware works!</p>
            </div>

            <div class="middleware-info">
                <h3>🔧 Active Middleware Stack (in execution order):</h3>
                <ol>
                    <li><strong>LoggerMiddleware:</strong> Logs all incoming requests and responses</li>
                    <li><strong>CorsMiddleware:</strong> Adds CORS headers for cross-origin requests</li>
                    <li><strong>Request ID Middleware:</strong> Adds unique <code>x-request-id</code> to requests/responses</li>
                    <li><strong>Rate Limit Middleware:</strong> Tracks request rates per IP</li>
                    <li><strong>Bearer Auth Middleware:</strong> Protects /protected route with Bearer token validation</li>
                </ol>
                <p>Each middleware uses <code>next.run(req).await</code> to pass control to the next layer.</p>
                <p>Middleware can short-circuit by returning a response without calling <code>next.run()</code>.</p>
            </div>

            <div class="routes">
                <h3>📍 Available Routes:</h3>
                <div class="action-buttons">
                    <a href="/public" class="btn-public">🌍 Public Route</a>
                    <a href="/protected" class="btn-protected">🔒 Protected Route (try in browser - will fail)</a>
                </div>
                <ul>
                    <li><strong>GET /</strong> - This home page (all middleware except auth)</li>
                    <li><strong>GET /public</strong> - Public route (all middleware, auth skips via next.run())</li>
                    <li><strong>GET /protected</strong> - Protected route (all middleware + auth validates token)</li>
                </ul>
            </div>

            <div class="test-info">
                <h3>🧪 Testing Authentication:</h3>
                <p><strong>Without token (will fail with 401):</strong></p>
                <pre>curl http://127.0.0.1:3001/protected</pre>

                <p><strong>With correct token (will succeed with 200):</strong></p>
                <pre>curl -H "Authorization: Bearer secret-token" http://127.0.0.1:3001/protected</pre>

                <p><strong>With wrong token (will fail with 401):</strong></p>
                <pre>curl -H "Authorization: Bearer wrong-token" http://127.0.0.1:3001/protected</pre>

                <p><strong>Test public route (no auth required):</strong></p>
                <pre>curl http://127.0.0.1:3001/public</pre>

                <p><strong>Check for request ID in response:</strong></p>
                <pre>curl -i http://127.0.0.1:3001/public | grep x-request-id</pre>
            </div>

            <h3>🛠️ Features Demonstrated:</h3>
            <ul>
                <li>✅ Axum-style Next pattern middleware</li>
                <li>✅ Middleware composition and ordering</li>
                <li>✅ Path-specific middleware logic (auth only for /protected)</li>
                <li>✅ Bearer token authentication with proper error responses</li>
                <li>✅ CORS header injection</li>
                <li>✅ Request/response logging</li>
                <li>✅ Unique request ID generation and propagation</li>
                <li>✅ Rate limiting (demo - logs only)</li>
                <li>✅ Custom middleware using <code>from_fn()</code></li>
                <li>✅ Clean handler functions with no manual middleware logic</li>
                <li>✅ Automatic error handling with <code>IntoResponse</code></li>
                <li>✅ Middleware can short-circuit the chain</li>
            </ul>

            <h3>📖 How Next Pattern Works:</h3>
            <ul>
                <li>Request flows through middleware in ORDER: 1 → 2 → 3 → Handler</li>
                <li>Each middleware calls <code>next.run(req).await</code> to continue</li>
                <li>Handler executes and returns Response</li>
                <li>Response flows back through middleware in REVERSE: Handler → 3 → 2 → 1</li>
                <li>Middleware can modify request before calling next, or response after</li>
                <li>Middleware can return early (short-circuit) without calling next</li>
            </ul>
        </body>
        </html>
        "#,
    ))
}
