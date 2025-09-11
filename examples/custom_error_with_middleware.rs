// examples/custom_error_with_middleware.rs

use ignitia::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Same custom errors as before
define_error! {
    AppError {
        UserNotFound(StatusCode::NOT_FOUND, "user_not_found", "USER_NOT_FOUND"),
        InvalidEmail(StatusCode::BAD_REQUEST, "invalid_email", "INVALID_EMAIL"),
        DatabaseConnection(StatusCode::INTERNAL_SERVER_ERROR, "database_error", "DB_CONNECTION_FAILED"),
        Unauthorized(StatusCode::UNAUTHORIZED, "unauthorized", "AUTH_REQUIRED"),
        ValidationFailed(StatusCode::UNPROCESSABLE_ENTITY, "validation_error", "VALIDATION_FAILED"),
        RateLimited(StatusCode::TOO_MANY_REQUESTS, "rate_limited", "RATE_LIMIT_EXCEEDED"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// Enhanced UserService with more error scenarios
struct UserService;

impl UserService {
    fn get_user(id: u32) -> Result<User> {
        match id {
            1 => Ok(User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            }),
            2 => Err(AppError::UserNotFound(format!("User with ID {} not found", id)).into()),
            3 => Err(AppError::Unauthorized("Insufficient permissions".to_string()).into()),
            4 => Err(AppError::RateLimited("Too many requests".to_string()).into()),
            999 => Err(AppError::DatabaseConnection("Database unavailable".to_string()).into()),
            _ => Err(Error::internal(format!("Unexpected error for user {}", id))),
        }
    }

    fn create_user(req: CreateUserRequest) -> Result<User> {
        if !req.email.contains('@') {
            return Err(AppError::InvalidEmail(format!("Invalid email: {}", req.email)).into());
        }

        if req.name.trim().is_empty() {
            return Err(AppError::ValidationFailed("Name is required".to_string()).into());
        }

        Ok(User {
            id: 42,
            name: req.name,
            email: req.email,
        })
    }
}

#[derive(Deserialize)]
struct UserId {
    id: u32,
}

// Simpler handlers - errors are handled by middleware
async fn get_user_handler(Path(user): Path<UserId>) -> Result<Response> {
    let user = UserService::get_user(user.id)?;
    Response::json(&user)
}

async fn create_user_handler(Json(req): Json<CreateUserRequest>) -> Result<Response> {
    let user = UserService::create_user(req)?;
    Response::json(&user)
}

async fn simulate_500_error() -> Result<Response> {
    Err(Error::internal("Something went wrong internally"))
}

async fn simulate_validation_error() -> Result<Response> {
    Err(AppError::ValidationFailed("Multiple validation errors occurred".to_string()).into())
}

async fn simulate_rate_limit() -> Result<Response> {
    Err(AppError::RateLimited("API rate limit exceeded".to_string()).into())
}

// Custom middleware that adds request ID to errors
struct RequestIdMiddleware;

#[async_trait::async_trait]
impl Middleware for RequestIdMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Add a request ID extension
        let request_id = uuid::Uuid::new_v4().to_string();
        req.insert_extension(request_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    let router = Router::new()
        // Add request ID middleware first
        .middleware(RequestIdMiddleware)

        // Add comprehensive error handling middleware
        .middleware(
            ErrorHandlerMiddleware::new()
                .with_details(true)              // Include detailed error info
                .with_stack_trace(true)          // Include stack traces in debug
                .with_logging(true)              // Log all errors
                .with_error_log_threshold(500)   // Log 5xx as errors, 4xx as warnings
                .with_custom_error_page(
                    StatusCode::NOT_FOUND,
                    r#"
                    <html>
                        <head><title>404 - Page Not Found</title></head>
                        <body>
                            <h1>🔍 Oops! Page not found</h1>
                            <p>The page you're looking for doesn't exist.</p>
                            <a href="/">Go Home</a>
                        </body>
                    </html>
                    "#.to_string()
                )
                .with_custom_error_page(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    r#"
                    <html>
                        <head><title>500 - Server Error</title></head>
                        <body>
                            <h1>💥 Something went wrong</h1>
                            <p>We're experiencing technical difficulties. Please try again later.</p>
                            <a href="/">Go Home</a>
                        </body>
                    </html>
                    "#.to_string()
                )
        )

        // Add CORS and logging middleware
        .middleware(LoggerMiddleware)
        .middleware(CorsMiddleware::new()
            .allowed_origin(&"http://localhost:8000")
            .allowed_methods(&[Method::GET, Method::POST, Method::OPTIONS])
            .build()?
        )

        // Routes - errors are automatically handled by middleware
        .get("/users/:id", get_user_handler)
        .post("/users", create_user_handler)
        .get("/500-error", simulate_500_error)
        .get("/validation-error", simulate_validation_error)
        .get("/rate-limit", simulate_rate_limit)

        // API routes that return JSON errors
        .get("/api/users/:id", get_user_handler)
        .post("/api/users", create_user_handler)

        // Root handler
        .get("/", || async {
            Ok::<Response, Error>(Response::html(
                r#"
                <h1>Custom Error Handling Demo (With Middleware)</h1>
                <h2>HTML Endpoints (Custom Error Pages)</h2>
                <ul>
                    <li><a href="/users/1">GET /users/1</a> - Success</li>
                    <li><a href="/users/2">GET /users/2</a> - User not found (404)</li>
                    <li><a href="/users/3">GET /users/3</a> - Unauthorized (401)</li>
                    <li><a href="/users/4">GET /users/4</a> - Rate limited (429)</li>
                    <li><a href="/users/999">GET /users/999</a> - Database error (500)</li>
                    <li><a href="/500-error">GET /500-error</a> - Internal server error</li>
                    <li><a href="/validation-error">GET /validation-error</a> - Validation error</li>
                    <li><a href="/rate-limit">GET /rate-limit</a> - Rate limit error</li>
                    <li><a href="/nonexistent">GET /nonexistent</a> - 404 Not Found</li>
                </ul>

                <h2>API Endpoints (JSON Errors)</h2>
                <ul>
                    <li><a href="/api/users/1">GET /api/users/1</a> - Success (JSON)</li>
                    <li><a href="/api/users/2">GET /api/users/2</a> - User not found (JSON)</li>
                    <li><a href="/api/users/999">GET /api/users/999</a> - Database error (JSON)</li>
                </ul>

                <h2>Test with curl:</h2>
                <pre>
# Test JSON API endpoints
curl -H "Accept: application/json" http://localhost:3001/api/users/2

# Test POST with invalid data
curl -X POST http://localhost:3001/api/users \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{"name":"","email":"invalid"}'

# Test HTML error pages
curl -H "Accept: text/html" http://localhost:3001/users/2
                </pre>
                "#
            ))
        });

    let addr: SocketAddr = "127.0.0.1:3001".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🔥 Server with error middleware running on http://{}", addr);
    println!("📖 Try the endpoints listed at http://{}", addr);

    server.ignitia().await.unwrap();

    Ok(())
}

// Test commands:
// curl http://localhost:3001/api/users/2 -H "Accept: application/json"
// curl http://localhost:3001/users/2 -H "Accept: text/html"
// curl -X POST http://localhost:3001/api/users -H "Content-Type: application/json" -d '{"name":"","email":"bad"}' -H "Accept: application/json"
