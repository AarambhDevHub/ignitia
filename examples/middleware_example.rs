use ignitia::{
    middleware::{AuthMiddleware, CorsMiddleware, LoggerMiddleware},
    Response, Result, Router, Server,
};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        // Apply middleware in order
        .middleware(LoggerMiddleware)
        .middleware(CorsMiddleware::new())
        .middleware(AuthMiddleware::new("secret-token").protect_path("/protected"))
        // Routes using modern extractor pattern
        .get("/public", public_route)
        .get("/protected", protected_route)
        .get("/", home_page);

    let addr: SocketAddr = "127.0.0.1:3001".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🛡️ Server with middleware running on http://{}", addr);
    println!("📋 Middleware stack:");
    println!("   ✅ Logger middleware - logs all requests");
    println!("   ✅ CORS middleware - handles cross-origin requests");
    println!("   ✅ Auth middleware - protects /protected route");
    println!();
    println!("🔗 Available routes:");
    println!("   GET / - Home page with information");
    println!("   GET /public - Public route (no auth required)");
    println!("   GET /protected - Protected route (requires Bearer token)");
    println!();
    println!("🔑 To test protected route:");
    println!("   curl -H \"Authorization: Bearer secret-token\" http://127.0.0.1:3001/protected");

    server.run().await.unwrap();
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
                <p><strong>Middleware applied:</strong></p>
                <ul>
                    <li>✅ Logger middleware (request logged)</li>
                    <li>✅ CORS middleware (CORS headers added)</li>
                    <li>⏭️ Auth middleware (skipped - not protected path)</li>
                </ul>
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
                <p><strong>Middleware applied:</strong></p>
                <ul>
                    <li>✅ Logger middleware (request logged)</li>
                    <li>✅ CORS middleware (CORS headers added)</li>
                    <li>✅ Auth middleware (token validated successfully)</li>
                </ul>
                <p>Your Bearer token was validated by the auth middleware.</p>
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
            <title>Middleware Demo</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; max-width: 800px; margin: 40px auto; }
                h1 { color: #333; }
                .middleware-info { background: #f0f8ff; padding: 20px; border-radius: 10px; margin: 20px 0; }
                .routes { background: #f8f9fa; padding: 20px; border-radius: 10px; margin: 20px 0; }
                .test-info { background: #e7f3ff; padding: 20px; border-radius: 10px; margin: 20px 0; }
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
                .btn-test { background: #6c757d; }
            </style>
        </head>
        <body>
            <h1>🛡️ Mini Web Framework - Middleware Demo</h1>

            <div class="middleware-info">
                <h3>🔧 Active Middleware Stack:</h3>
                <ol>
                    <li><strong>LoggerMiddleware:</strong> Logs all incoming requests and responses</li>
                    <li><strong>CorsMiddleware:</strong> Adds CORS headers for cross-origin requests</li>
                    <li><strong>AuthMiddleware:</strong> Protects /protected route with Bearer token</li>
                </ol>
                <p>Middleware is applied in order for requests and reverse order for responses.</p>
            </div>

            <div class="routes">
                <h3>📍 Available Routes:</h3>
                <div class="action-buttons">
                    <a href="/public" class="btn-public">🌍 Public Route</a>
                    <a href="/protected" class="btn-protected">🔒 Protected Route</a>
                </div>
                <ul>
                    <li><strong>GET /</strong> - This home page (no middleware restrictions)</li>
                    <li><strong>GET /public</strong> - Public route (logger + CORS middleware)</li>
                    <li><strong>GET /protected</strong> - Protected route (all middleware + auth required)</li>
                </ul>
            </div>

            <div class="test-info">
                <h3>🧪 Testing Authentication:</h3>
                <p><strong>Without token (will fail):</strong></p>
                <pre>curl http://127.0.0.1:3001/protected</pre>

                <p><strong>With correct token (will succeed):</strong></p>
                <pre>curl -H "Authorization: Bearer secret-token" http://127.0.0.1:3001/protected</pre>

                <p><strong>With wrong token (will fail):</strong></p>
                <pre>curl -H "Authorization: Bearer wrong-token" http://127.0.0.1:3001/protected</pre>
            </div>

            <h3>🛠️ Features Demonstrated:</h3>
            <ul>
                <li>✅ Middleware composition and ordering</li>
                <li>✅ Path-specific middleware application</li>
                <li>✅ Bearer token authentication</li>
                <li>✅ CORS header injection</li>
                <li>✅ Request/response logging</li>
                <li>✅ Clean handler functions with no manual middleware logic</li>
                <li>✅ Automatic error handling and status codes</li>
            </ul>
        </body>
        </html>
        "#,
    ))
}
