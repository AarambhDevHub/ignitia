// examples/custom_error_no_middleware.rs

use ignitia::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Define custom application errors using the macro from your framework
define_error! {
    AppError {
        UserNotFound(StatusCode::NOT_FOUND, "user_not_found", "USER_NOT_FOUND"),
        InvalidEmail(StatusCode::BAD_REQUEST, "invalid_email", "INVALID_EMAIL"),
        DatabaseConnection(StatusCode::INTERNAL_SERVER_ERROR, "database_error", "DB_CONNECTION_FAILED"),
        Unauthorized(StatusCode::UNAUTHORIZED, "unauthorized", "AUTH_REQUIRED"),
        ValidationFailed(StatusCode::UNPROCESSABLE_ENTITY, "validation_error", "VALIDATION_FAILED"),
    }
}

// Custom user struct for examples
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

#[derive(Debug, Deserialize)]
struct UserQuery {
    include_inactive: Option<bool>,
}

// Simulate a user service with potential errors
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
            999 => Err(AppError::DatabaseConnection("Connection timeout".to_string()).into()),
            _ => Err(AppError::Unauthorized("Access denied".to_string()).into()),
        }
    }

    fn create_user(req: CreateUserRequest) -> Result<User> {
        // Validate email
        if !req.email.contains('@') {
            return Err(
                AppError::InvalidEmail(format!("Invalid email format: {}", req.email)).into(),
            );
        }

        // Simulate validation error
        if req.name.is_empty() {
            return Err(AppError::ValidationFailed("Name cannot be empty".to_string()).into());
        }

        Ok(User {
            id: 42,
            name: req.name,
            email: req.email,
        })
    }
}

// Handler that demonstrates different error scenarios
async fn get_user_handler(
    Path(user_id): Path<u32>,
    Query(query): Query<UserQuery>,
) -> Result<Response> {
    println!("🔍 Getting user {} with query: {:?}", user_id, query);

    // This will return different errors based on the user_id
    let user = UserService::get_user(user_id)?;

    // If successful, return JSON response
    Ok(Response::json(&user))
}

// Handler for creating users with validation
async fn create_user_handler(Json(req): Json<CreateUserRequest>) -> Result<Response> {
    println!("🔍 Creating user: {:?}", req);

    let user = UserService::create_user(req)?;

    Ok(Response::json(&user))
}

// Handler that always returns a custom error for demonstration
async fn error_demo_handler() -> Result<Response> {
    // You can create custom errors on the fly
    Err(Error::Custom(Box::new(AppError::DatabaseConnection(
        "This is a demo error".to_string(),
    ))))
}

// Handler that shows manual error response creation
async fn manual_error_handler() -> Result<Response> {
    // Create a custom JSON error response manually
    let error_response = serde_json::json!({
        "success": false,
        "error": "custom_business_logic_error",
        "message": "This operation is not allowed at this time",
        "code": "OPERATION_NOT_ALLOWED",
        "details": {
            "reason": "Maintenance mode active",
            "retry_after": "2024-01-01T10:00:00Z"
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Response::json(error_response).with_status(StatusCode::SERVICE_UNAVAILABLE))
}

// Handler that demonstrates validation errors
async fn validate_user_handler(Json(req): Json<CreateUserRequest>) -> Result<Response> {
    let mut errors = Vec::new();

    // Manual validation
    if req.name.trim().is_empty() {
        errors.push("Name is required".to_string());
    }

    if req.name.len() < 2 {
        errors.push("Name must be at least 2 characters".to_string());
    }

    if !req.email.contains('@') {
        errors.push("Invalid email format".to_string());
    }

    if req.email.len() < 5 {
        errors.push("Email is too short".to_string());
    }

    // If there are validation errors, return them
    if !errors.is_empty() {
        return Response::validation_error(errors);
    }

    // If validation passes
    let user = User {
        id: 100,
        name: req.name,
        email: req.email,
    };

    Ok(Response::json(&user))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    let router = Router::new()
        // Different error scenarios
        .get("/users/{id}", get_user_handler) // Try /users/1 (success), /users/2 (not found), /users/999 (db error)
        .post("/users", create_user_handler) // Send JSON with invalid data
        .get("/error-demo", error_demo_handler) // Always returns error
        .get("/manual-error", manual_error_handler) // Custom error response
        .post("/validate", validate_user_handler) // Validation error example
        // Root handler for testing
        .get("/", || async {
            Ok::<Response, Error>(Response::html(
                r#"
                <h1>Custom Error Handling Demo (No Middleware)</h1>
                <p>Try these endpoints:</p>
                <ul>
                    <li><a href="/users/1">GET /users/1</a> - Success</li>
                    <li><a href="/users/2">GET /users/2</a> - User not found</li>
                    <li><a href="/users/999">GET /users/999</a> - Database error</li>
                    <li><a href="/users/3">GET /users/3</a> - Unauthorized</li>
                    <li><a href="/error-demo">GET /error-demo</a> - Custom error</li>
                    <li><a href="/manual-error">GET /manual-error</a> - Manual error response</li>
                </ul>
                <p>POST endpoints (use curl or Postman):</p>
                <ul>
                    <li>POST /users - Create user</li>
                    <li>POST /validate - Validation demo</li>
                </ul>
                "#,
            ))
        });

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🔥 Server running on http://{}", addr);
    println!("📖 Try the endpoints listed at http://{}", addr);

    server.ignitia().await.unwrap();
    Ok(())
}

// Test with curl commands:
// curl http://localhost:3000/users/1
// curl http://localhost:3000/users/2
// curl http://localhost:3000/users/999
// curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d '{"name":"","email":"invalid"}'
// curl -X POST http://localhost:3000/validate -H "Content-Type: application/json" -d '{"name":"A","email":"bad"}'
