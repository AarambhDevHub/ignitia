# Error Handling

A comprehensive guide to error handling in the Ignitia web framework.

## Table of Contents

- [Overview](#overview)
- [Error Types](#error-types)
- [Custom Errors](#custom-errors)
- [Error Responses](#error-responses)
- [Error Constructors](#error-constructors)
- [Error Middleware](#error-middleware)
- [Usage Examples](#usage-examples)
- [Best Practices](#best-practices)
- [Testing Errors](#testing-errors)

## Overview

Ignitia provides a comprehensive error handling system that allows you to:

- Handle different types of errors gracefully
- Create custom error types with structured metadata
- Automatically convert errors to appropriate HTTP responses
- Log errors with configurable levels
- Provide consistent error formatting across your application

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

// System errors
Error::Io(std::io::Error)         // I/O errors
Error::Hyper(hyper::Error)        // HTTP client/server errors
Error::Json(serde_json::Error)    // JSON parsing errors

// Custom errors
Error::Custom(Box<dyn CustomError>) // User-defined errors
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
```

### Using the define_error! Macro

For simpler custom errors, use the `define_error!` macro:

```rust
use ignitia::define_error;
use http::StatusCode;

define_error! {
    ApiError {
        RateLimitExceeded(StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded", "RATE_LIMIT"),
        InvalidToken(StatusCode::UNAUTHORIZED, "invalid_token", "INVALID_TOKEN"),
        ResourceLocked(StatusCode::LOCKED, "resource_locked", "RESOURCE_LOCKED"),
        PaymentRequired(StatusCode::PAYMENT_REQUIRED, "payment_required")
    }
}

// Usage in handlers
async fn protected_handler() -> Result<Response> {
    return Err(ApiError::InvalidToken("Token expired".into()).into());
}
```

## Error Responses

### ErrorResponse Structure

Errors are automatically converted to structured JSON responses:

```json
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
  "timestamp": "2023-01-01T12:00:00Z"
}
```

### Creating Custom Error Responses

```rust
use ignitia::{Response, Error};

// Method 1: Using Error constructors
async fn handler() -> Result<Response> {
    Err(Error::validation("Email format is invalid"))
}

// Method 2: Using Response::error_json
async fn handler2() -> Result<Response> {
    Response::error_json(Error::bad_request("Missing required field"))
}

// Method 3: Validation errors with multiple messages
async fn handler3() -> Result<Response> {
    let validation_errors = vec![
        "Email is required".to_string(),
        "Password must be at least 8 characters".to_string(),
    ];
    Response::validation_error(validation_errors)
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

### ErrorExt Trait

Convert standard errors using the `ErrorExt` trait:

```rust
use ignitia::{ErrorExt, Result, Response};

async fn database_handler() -> Result<Response> {
    let user = sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(&pool)
        .await
        .internal_error()?; // Converts sqlx::Error to Error::Internal

    let email: String = user.email
        .parse()
        .validation_error()?; // Converts parse error to Error::Validation

    Ok(Response::json(user)?)
}
```

## Error Middleware

### Basic Error Handler Middleware

```rust
use ignitia::{Router, ErrorHandlerMiddleware};

let app = Router::new()
    .middleware(ErrorHandlerMiddleware::new())
    .get("/", handler);
```

### Configuring Error Middleware

```rust
use ignitia::ErrorHandlerMiddleware;

let error_middleware = ErrorHandlerMiddleware::new()
    .with_details(true)                    // Include error details
    .with_stack_trace(cfg!(debug_assertions)) // Stack traces in debug
    .with_logging(true)                    // Enable error logging
    .with_error_log_threshold(500);        // Log 5xx as errors, 4xx as warnings

let app = Router::new()
    .middleware(error_middleware)
    .get("/api/users", get_users)
    .post("/api/users", create_user);
```

### Custom Error Pages

```rust
use ignitia::{ErrorHandlerMiddleware, StatusCode};

let error_middleware = ErrorHandlerMiddleware::new()
    .with_custom_error_page(
        StatusCode::NOT_FOUND,
        include_str!("../templates/404.html").to_string()
    )
    .with_custom_error_page(
        StatusCode::INTERNAL_SERVER_ERROR,
        include_str!("../templates/500.html").to_string()
    );

let app = Router::new()
    .middleware(error_middleware)
    .get("/*path", catch_all_handler);
```

## Usage Examples

### Handler Error Patterns

```rust
use ignitia::{Router, Response, Json, Path, Error, Result};
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

// Pattern 1: Early returns with ? operator
async fn get_user(Path(user_id): Path<u64>) -> Result<Response> {
    let user = database::find_user(user_id)
        .await
        .ok_or_else(|| Error::not_found(&format!("/users/{}", user_id)))?;

    Response::json(user)
}

// Pattern 2: Validation with custom errors
async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // Validate email
    if !payload.email.contains('@') {
        return Err(Error::validation("Invalid email format"));
    }

    // Check if user exists
    if database::user_exists(&payload.email).await {
        return Err(Error::bad_request("User already exists"));
    }

    let user = database::create_user(payload).await
        .map_err(|e| Error::internal(format!("Failed to create user: {}", e)))?;

    Response::json(user)
}

// Pattern 3: Multiple validation errors
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
        return Response::validation_error(errors);
    }

    let user = database::create_user(payload).await?;
    Response::json(user)
}
```

### Error Handling with State

```rust
use ignitia::{Router, Response, State, Error, Result};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    database: DatabasePool,
    cache: RedisClient,
}

async fn handler_with_state(
    State(state): State<Arc<AppState>>
) -> Result<Response> {
    // Handle database errors
    let data = sqlx::query!("SELECT * FROM items")
        .fetch_all(&state.database)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::not_found("No items found"),
            _ => Error::database(format!("Database error: {}", e)),
        })?;

    // Handle cache errors (non-fatal)
    if let Err(e) = state.cache.set("last_fetch", "now").await {
        tracing::warn!("Cache error: {}", e);
        // Continue without failing the request
    }

    Response::json(data)
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
database::update_user(id, data).await?;
```

### 3. Handle Expected Errors Gracefully

```rust
async fn get_user_profile(user_id: u64) -> Result<Response> {
    match database::find_user(user_id).await {
        Ok(user) => Response::json(user),
        Err(DatabaseError::NotFound) => {
            Err(Error::not_found(&format!("User {} not found", user_id)))
        }
        Err(e) => Err(Error::internal(format!("Database error: {}", e))),
    }
}
```

### 4. Use Middleware for Cross-Cutting Concerns

```rust
// Global error handling
let app = Router::new()
    .middleware(ErrorHandlerMiddleware::new().with_logging(true))
    .middleware(RequestIdMiddleware::new()) // For error tracing
    .route("/api/users", get_users);
```

### 5. Provide Client-Friendly Messages

```rust
// Good: User-friendly message
return Err(Error::bad_request("Email address is already registered"));

// Bad: Technical implementation details
return Err(Error::bad_request("UNIQUE constraint failed: users.email"));
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
}
```

### Integration Testing with Test Client

```rust
#[cfg(test)]
mod integration_tests {
    use ignitia::{Router, Server};
    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_error_responses() {
        let app = Router::new()
            .middleware(ErrorHandlerMiddleware::new())
            .get("/users/:id", get_user);

        let server = Server::new(app, "127.0.0.1:0".parse().unwrap());

        // Test 404 error
        let response = reqwest::get("http://localhost:3000/users/999").await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let error_body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(error_body["status"], 404);
        assert_eq!(error_body["error_type"], "not_found");
    }
}
```

***

This error handling system provides a robust foundation for building reliable web applications with Ignitia. The combination of structured errors, automatic response formatting, and comprehensive middleware makes it easy to handle both expected and unexpected errors gracefully.
