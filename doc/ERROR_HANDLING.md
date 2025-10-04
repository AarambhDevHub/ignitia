# Error Handling

A comprehensive guide to error handling in the Ignitia web framework v0.2.4+.

## Table of Contents

- [Overview](#overview)
- [What's New in v0.2.4](#whats-new-in-v024)
- [Error Types](#error-types)
- [IntoResponse for Errors](#intoresponse-for-errors)
- [Custom Errors](#custom-errors)
- [Error Responses](#error-responses)
- [Error Constructors](#error-constructors)
- [Error Handling Patterns](#error-handling-patterns)
- [Usage Examples](#usage-examples)
- [Best Practices](#best-practices)
- [Testing Errors](#testing-errors)
- [Migration from v0.2.3](#migration-from-v023)

## Overview

Ignitia provides a comprehensive error handling system that allows you to:

- Handle different types of errors gracefully
- Create custom error types with structured metadata
- Automatically convert errors to appropriate HTTP responses via `IntoResponse`
- Provide consistent error formatting across your application
- Use the `?` operator seamlessly with automatic error conversion
- Return errors directly from handlers without explicit conversion

## What's New in v0.2.4

### Major Changes

1. **IntoResponse Trait**: Errors now automatically implement `IntoResponse`
   - Errors convert to HTTP responses automatically
   - No need for explicit error handling middleware
   - Seamless integration with the `?` operator

2. **Removed ErrorHandlerMiddleware**: Replaced by `IntoResponse` trait
   - Error handling is now built into the framework
   - Cleaner, more ergonomic API
   - Better type safety and compile-time guarantees

3. **Simplified Response API**: `Response::json()` is now infallible
   - Old: `Ok(Response::json(data)?)`
   - New: `Ok(Response::json(data))`

### Benefits

- **Less Boilerplate**: Return errors directly without wrapping
- **Better Ergonomics**: Natural Rust error handling with `?`
- **Type Safety**: Compile-time guarantees for error conversion
- **Performance**: Zero-cost error conversion

## Error Types

Ignitia defines several built-in error types in the `Error` enum:

```rust
use ignitia::{Error, Result, Response};

// Core HTTP errors
Error::NotFound(String)           // 404 - Resource not found
Error::MethodNotAllowed(String)   // 405 - Method not allowed
Error::BadRequest(String)         // 400 - Bad request
Error::Unauthorized               // 401 - Authentication required
Error::Forbidden                  // 403 - Access denied

// Application errors
Error::Validation(String)         // 400 - Validation failed
Error::Internal(String)           // 500 - Internal server error
Error::Database(String)           // 500 - Database error
Error::ExternalService(String)    // 500 - External service error

// System errors (with automatic From implementations)
Error::Io(std::io::Error)         // I/O errors
Error::Hyper(hyper::Error)        // HTTP client/server errors
Error::Json(serde_json::Error)    // JSON parsing errors

// Custom errors
Error::Custom(Box<dyn CustomError>) // User-defined errors
```

## IntoResponse for Errors

**New in v0.2.4**: All errors automatically convert to HTTP responses.

### Automatic Error Conversion

```rust
use ignitia::prelude::*;

// Errors automatically convert to responses
async fn handler() -> Result<impl IntoResponse, Error> {
    let data = fetch_data().await?;  // Errors convert automatically
    Ok(Response::json(data))
}

// Even simpler: use the Result type alias
async fn handler() -> Result<Response> {
    let data = fetch_data().await?;
    Ok(Response::json(data))
}
```

### IntoResponse Implementation

The `Error` enum implements `IntoResponse`:

```rust
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            error: status.canonical_reason().unwrap_or("Error"),
            message: self.to_string(),
            status: status.as_u16(),
            error_type: self.error_type(),
            error_code: self.error_code(),
            metadata: self.metadata(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        Response::json(error_response).with_status(status)
    }
}
```

### Error Response Format

Errors automatically convert to structured JSON:

```rust
{
  "error": "Bad Request",
  "message": "Invalid user input",
  "status": 400,
  "error_type": "validation_error",
  "error_code": "VALIDATION_FAILED",
  "metadata": {
    "field": "email",
    "reason": "invalid_format"
  },
  "timestamp": "2025-10-01T12:00:00Z"
}
```

## Custom Errors

### Implementing CustomError

Create your own error types by implementing the `CustomError` trait:

```rust
use ignitia::{CustomError, Error};
use http::StatusCode;
use serde_json::json;

#[derive(Debug)]
pub struct UserError {
    pub message: String,
    pub user_id: u64,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User error for user {}: {}", self.user_id, self.message)
    }
}

impl std::error::Error for UserError {}

impl CustomError for UserError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_type(&self) -> &'static str {
        "user_error"
    }

    fn error_code(&self) -> Option<String> {
        Some("USER_VALIDATION_FAILED".to_string())
    }

    fn metadata(&self) -> Option<serde_json::Value> {
        Some(json!({
            "user_id": self.user_id,
            "category": "user_validation"
        }))
    }
}

// Convert to Ignitia Error
impl From<UserError> for Error {
    fn from(err: UserError) -> Self {
        Error::Custom(Box::new(err))
    }
}

// Usage in handlers
async fn handler() -> Result<Response> {
    let user_error = UserError {
        message: "Invalid operation".to_string(),
        user_id: 123,
    };
    Err(user_error.into())  // Automatically converts to Response
}
```

### Advanced Custom Error with IntoResponse

```rust
use ignitia::{CustomError, IntoResponse, Response};
use http::StatusCode;

#[derive(Debug)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
    pub status: StatusCode,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

// Implement IntoResponse directly for custom format
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        Response::json(serde_json::json!({
            "success": false,
            "error_code": self.code,
            "message": self.message,
        }))
        .with_status(self.status)
    }
}

// Usage
async fn handler() -> Result<Response, ApiError> {
    Err(ApiError {
        code: "RATE_LIMIT_EXCEEDED",
        message: "Too many requests".to_string(),
        status: StatusCode::TOO_MANY_REQUESTS,
    })  // Automatically converts via IntoResponse
}
```

## Error Responses

### Error Methods

```rust
impl Error {
    pub fn status_code(&self) -> StatusCode
    pub fn error_type(&self) -> &'static str
    pub fn error_code(&self) -> Option<String>
    pub fn metadata(&self) -> Option<serde_json::Value>
}
```

### Creating Responses from Errors

```rust
use ignitia::{Response, Error};

// Method 1: Return errors directly (v0.2.4+)
async fn handler() -> Result<Response> {
    Err(Error::validation("Email format is invalid"))  // Auto-converts
}

// Method 2: Explicit conversion
async fn handler2() -> impl IntoResponse {
    let error = Error::bad_request("Missing required field");
    error.into_response()
}

// Method 3: Early return with ?
async fn handler3() -> Result<Response> {
    let user = database::find_user(id)
        .await
        .map_err(|_| Error::not_found("User not found"))?;

    Ok(Response::json(user))
}
```

## Error Constructors

Ignitia provides fast path constructors for common errors:

```rust
use ignitia::{Error, Result, Response};

async fn example_handler() -> Result<Response> {
    // Quick error constructors
    return Err(Error::not_found("/api/users/123"));
    return Err(Error::bad_request("Invalid JSON payload"));
    return Err(Error::validation("Password too weak"));
    return Err(Error::unauthorized());
    return Err(Error::forbidden());
    return Err(Error::internal("Database connection failed"));
}
```

### Error Constructor Methods

```rust
impl Error {
    pub fn not_found(path: &str) -> Self
    pub fn bad_request(msg: impl Into<String>) -> Self
    pub fn validation(msg: impl Into<String>) -> Self
    pub fn unauthorized() -> Self
    pub fn forbidden() -> Self
    pub fn internal(msg: impl Into<String>) -> Self
    pub fn database(msg: impl Into<String>) -> Self
    pub fn external_service(msg: impl Into<String>) -> Self
}
```

## Error Handling Patterns

### Pattern 1: Using the ? Operator (Recommended)

```rust
use ignitia::prelude::*;

async fn get_user(Path(user_id): Path<u64>) -> Result<Response> {
    // ? operator automatically converts errors
    let user = database::find_user(user_id)
        .await
        .map_err(|_| Error::not_found(&format!("User {} not found", user_id)))?;

    Ok(Response::json(user))
}
```

### Pattern 2: Early Returns

```rust
async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // Validate and return early
    if !payload.email.contains('@') {
        return Err(Error::validation("Invalid email format"));
    }

    if database::user_exists(&payload.email).await {
        return Err(Error::bad_request("User already exists"));
    }

    let user = database::create_user(payload).await
        .map_err(|e| Error::internal(format!("Failed to create user: {}", e)))?;

    Ok(Response::json(user))
}
```

### Pattern 3: Match Expressions

```rust
async fn handler() -> Result<Response> {
    match database::find_user(id).await {
        Ok(user) => Ok(Response::json(user)),
        Err(DatabaseError::NotFound) => {
            Err(Error::not_found(&format!("User {} not found", id)))
        }
        Err(e) => Err(Error::database(format!("Database error: {}", e))),
    }
}
```

### Pattern 4: Multiple Validation Errors

```rust
#[derive(Serialize)]
struct ValidationErrors {
    errors: Vec<String>,
}

async fn validate_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    let mut errors = Vec::new();

    if payload.email.is_empty() {
        errors.push("Email is required".to_string());
    } else if !payload.email.contains('@') {
        errors.push("Invalid email format".to_string());
    }

    if payload.name.len() < 2 {
        errors.push("Name must be at least 2 characters".to_string());
    }

    if !errors.is_empty() {
        return Ok(Response::json(ValidationErrors { errors })
            .with_status(StatusCode::BAD_REQUEST));
    }

    let user = database::create_user(payload).await?;
    Ok(Response::json(user))
}
```

### Pattern 5: Custom Error Middleware (using from_fn)

**New in v0.2.4**: Create custom error handling with `from_fn`:

```rust
use ignitia::middleware::from_fn;
use tracing;

let error_logger = from_fn(|req, next| async move {
    let response = next.run(req).await;

    // Log errors based on status code
    if response.status.is_server_error() {
        tracing::error!(
            "Server error: {} - {}",
            response.status,
            response.status.canonical_reason().unwrap_or("Unknown")
        );
    } else if response.status.is_client_error() {
        tracing::warn!(
            "Client error: {} - {}",
            response.status,
            response.status.canonical_reason().unwrap_or("Unknown")
        );
    }

    response
});

let router = Router::new()
    .middleware(error_logger)
    .get("/api/users", get_users);
```

## Usage Examples

### Complete Handler Examples

```rust
use ignitia::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    email: String,
    name: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    email: String,
    name: String,
}

// Example 1: Simple GET with error handling
async fn get_user(Path(user_id): Path<u64>) -> Result<Response> {
    let user = database::find_user(user_id)
        .await
        .ok_or_else(|| Error::not_found(&format!("User {} not found", user_id)))?;

    Ok(Response::json(user))
}

// Example 2: POST with validation
async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // Validate input
    if !payload.email.contains('@') {
        return Err(Error::validation("Invalid email format"));
    }

    // Check for duplicates
    if database::user_exists(&payload.email).await {
        return Err(Error::bad_request("User already exists"));
    }

    // Create user
    let user = database::create_user(payload)
        .await
        .map_err(|e| Error::internal(format!("Failed to create user: {}", e)))?;

    Ok(Response::json(user).with_status(StatusCode::CREATED))
}

// Example 3: Using IntoResponse for multiple return types
async fn flexible_handler(Path(id): Path<u64>) -> Result<impl IntoResponse> {
    if id == 0 {
        return Err(Error::bad_request("ID cannot be zero"));
    }

    let user = database::find_user(id)
        .await
        .map_err(|_| Error::not_found("User not found"))?;

    Ok(Response::json(user))
}

// Example 4: Handler with state
async fn handler_with_state(
    State(db): State<Arc<DatabasePool>>,
    Path(id): Path<u64>,
) -> Result<Response> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(db.as_ref())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::not_found("User not found"),
            _ => Error::database(format!("Database error: {}", e)),
        })?;

    Ok(Response::json(user))
}
```

### External API Integration

```rust
use reqwest;

async fn fetch_external_data() -> Result<Response> {
    let response = reqwest::get("https://api.example.com/data")
        .await
        .map_err(|e| Error::external_service(format!("API request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(Error::external_service(
            format!("API returned status: {}", response.status())
        ));
    }

    let data = response.json::<serde_json::Value>()
        .await
        .map_err(|e| Error::external_service(format!("Invalid JSON response: {}", e)))?;

    Ok(Response::json(data))
}
```

## Best Practices

### 1. Use Specific Error Types

```rust
// Good: Specific error with context
return Err(Error::not_found(&format!("User {} not found", user_id)));

// Bad: Generic error
return Err(Error::internal("Something went wrong"));
```

### 2. Include Helpful Context

```rust
// Good: Include operation context
database::update_user(id, data).await
    .map_err(|e| Error::database(format!("Failed to update user {}: {}", id, e)))?;

// Bad: No context
database::update_user(id, data).await
    .map_err(|e| Error::database(e.to_string()))?;
```

### 3. Use the ? Operator

```rust
// Good: Clean and idiomatic
async fn handler() -> Result<Response> {
    let user = fetch_user().await?;
    let profile = fetch_profile(user.id).await?;
    Ok(Response::json(profile))
}

// Avoid: Excessive unwrap
async fn handler() -> Result<Response> {
    let user = fetch_user().await.unwrap();  // Don't do this!
    Ok(Response::json(user))
}
```

### 4. Provide Client-Friendly Messages

```rust
// Good: User-friendly message
return Err(Error::bad_request("Email address is already registered"));

// Bad: Technical implementation details
return Err(Error::bad_request("UNIQUE constraint failed: users.email"));
```

### 5. Log Errors Appropriately

```rust
use tracing;

async fn handler() -> Result<Response> {
    match database::find_user(id).await {
        Ok(user) => Ok(Response::json(user)),
        Err(e) => {
            // Log the detailed error
            tracing::error!("Database error while fetching user {}: {}", id, e);

            // Return user-friendly error
            Err(Error::internal("Failed to fetch user"))
        }
    }
}
```

### 6. Use Custom Errors for Domain Logic

```rust
#[derive(Debug)]
enum OrderError {
    InsufficientStock { product_id: u64, requested: u32, available: u32 },
    InvalidDiscount { code: String },
    PaymentFailed { reason: String },
}

impl std::fmt::Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientStock { product_id, requested, available } => {
                write!(f, "Product {} has only {} items, requested {}",
                    product_id, available, requested)
            }
            Self::InvalidDiscount { code } => write!(f, "Invalid discount code: {}", code),
            Self::PaymentFailed { reason } => write!(f, "Payment failed: {}", reason),
        }
    }
}

impl CustomError for OrderError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InsufficientStock { .. } => StatusCode::CONFLICT,
            Self::InvalidDiscount { .. } => StatusCode::BAD_REQUEST,
            Self::PaymentFailed { .. } => StatusCode::PAYMENT_REQUIRED,
        }
    }

    fn error_type(&self) -> &'static str {
        match self {
            Self::InsufficientStock { .. } => "insufficient_stock",
            Self::InvalidDiscount { .. } => "invalid_discount",
            Self::PaymentFailed { .. } => "payment_failed",
        }
    }
}
```

## Testing Errors

### Unit Testing Error Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ignitia::{Error, StatusCode};

    #[tokio::test]
    async fn test_user_not_found() {
        let result = get_user(Path(999)).await;

        assert!(result.is_err());

        if let Err(Error::NotFound(message)) = result {
            assert!(message.contains("999"));
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_validation_error() {
        let payload = CreateUser {
            email: "invalid-email".to_string(),
            name: "".to_string(),
        };

        let result = create_user(Json(payload)).await;

        match result {
            Err(Error::Validation(_)) => {}, // Expected
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let error = Error::bad_request("Test error");
        let response = error.into_response();

        assert_eq!(response.status, StatusCode::BAD_REQUEST);

        // Parse JSON body
        let body: serde_json::Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(body["status"], 400);
        assert_eq!(body["error_type"], "bad_request");
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use ignitia::{Router, Server};
    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_error_responses() {
        let app = Router::new()
            .get("/users/{id}", get_user);

        let server = Server::new(app, "127.0.0.1:0".parse().unwrap());

        // Test 404 error
        let response = reqwest::get("http://localhost:3000/users/999")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let error_body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(error_body["status"], 404);
        assert_eq!(error_body["error_type"], "not_found");
        assert!(error_body["message"].as_str().unwrap().contains("999"));
    }
}
```

## Migration from v0.2.3

### Old Error Handling (v0.2.3)

```rust
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }
    async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        Ok(())
    }
}

// Old handler with double Result
async fn handler() -> Result<Response> {
    Ok(Response::json(data)?)  // Double Result
}

// Old error middleware
let app = Router::new()
    .middleware(ErrorHandlerMiddleware::new())
    .get("/", handler);
```

### New Error Handling (v0.2.4+)

```rust
pub trait Middleware: Send + Sync {
    async fn handle(&self, req: Request, next: Next) -> Response;
}

// New handler with IntoResponse
async fn handler() -> Result<Response> {
    Ok(Response::json(data))  // Single Result, infallible
}

// Or with IntoResponse
async fn handler() -> Result<impl IntoResponse> {
    Ok(Response::json(data))
}

// Error handling via from_fn
let error_logger = from_fn(|req, next| async move {
    let response = next.run(req).await;
    if response.status.is_server_error() {
        tracing::error!("Error: {}", response.status);
    }
    response
});

let app = Router::new()
    .middleware(error_logger)
    .get("/", handler);
```

### Migration Checklist

- [ ] Remove `ErrorHandlerMiddleware` usage
- [ ] Update `Response::json()` calls to remove `?` operator
- [ ] Change handler return types to use `IntoResponse` where beneficial
- [ ] Replace error middleware with `from_fn` if needed
- [ ] Update middleware from `before/after` to `handle(req, next)`
- [ ] Remove explicit error-to-response conversions (now automatic)

***

This error handling system provides a robust, ergonomic foundation for building reliable web applications with Ignitia v0.2.4+. The combination of automatic error conversion through `IntoResponse`, structured error types, and seamless `?` operator integration makes error handling both powerful and pleasant to work with.
