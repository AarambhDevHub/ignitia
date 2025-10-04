# HTTP Responses in Ignitia 🔥

A comprehensive guide to creating, customizing, and optimizing HTTP responses in Ignitia v0.2.4+ - from simple text responses to complex streaming APIs.

## Table of Contents

- [What's New in v0.2.4](#whats-new-in-v024)
- [Quick Start](#quick-start)
- [IntoResponse Trait](#intoresponse-trait)
- [Basic Responses](#basic-responses)
- [Response Builder](#response-builder)
- [Content Types](#content-types)
- [Status Codes](#status-codes)
- [Headers Management](#headers-management)
- [Cookie Handling](#cookie-handling)
- [Error Responses](#error-responses)
- [Advanced Response Patterns](#advanced-response-patterns)
- [Performance Optimization](#performance-optimization)
- [Security Best Practices](#security-best-practices)
- [Testing Responses](#testing-responses)
- [Migration from v0.2.3](#migration-from-v023)
- [Production Examples](#production-examples)

## What's New in v0.2.4

### Major Changes

1. **IntoResponse Trait**: New trait for automatic response conversion
   - Return any type that implements `IntoResponse`
   - Automatic error-to-response conversion
   - Cleaner, more ergonomic handler signatures

2. **Infallible Response Methods**: `Response::json()`, `Response::html()`, and `Response::text()` now return `Response` directly
   - Old: `Ok(Response::json(data)?)`
   - New: `Ok(Response::json(data))`
   - JSON serialization errors handled internally

3. **Simplified Handler Returns**: Multiple ways to return responses
   - `Result<Response>` - Traditional approach
   - `impl IntoResponse` - Most ergonomic
   - Direct types: `String`, `&str`, tuples, etc.

### Benefits

- **Less Boilerplate**: No more double `Result` wrapping
- **Better Ergonomics**: Natural Rust patterns
- **Type Safety**: Compile-time guarantees
- **Zero Cost**: No runtime overhead

## Quick Start

Get started with Ignitia responses in seconds:

```rust
use ignitia::prelude::*;

// Simple text response (v0.2.4+ IntoResponse)
async fn hello() -> &'static str {
    "Hello, Ignitia! 🔥"
}

// JSON API response (infallible)
async fn api_response() -> impl IntoResponse {
    Response::json(serde_json::json!({
        "message": "Welcome to Ignitia",
        "version": "0.2.4",
        "status": "success"
    }))
}

// HTML page response
async fn homepage() -> impl IntoResponse {
    Response::html(r#"
        <!DOCTYPE html>
        <html>
            <head><title>Ignitia App</title></head>
            <body><h1>Powered by Ignitia 🔥</h1></body>
        </html>
    "#)
}

// With status code tuple
async fn created() -> impl IntoResponse {
    (StatusCode::CREATED, "Resource created successfully")
}

// Traditional Result approach
async fn with_result() -> Result<Response> {
    let data = fetch_data().await?;
    Ok(Response::json(data))  // No ? operator needed!
}
```

## IntoResponse Trait

**New in v0.2.4**: The `IntoResponse` trait provides automatic response conversion for any type.

### Trait Definition

```rust
pub trait IntoResponse {
    fn into_response(self) -> Response;
}
```

### Built-in Implementations

```rust
// Strings
impl IntoResponse for String
impl IntoResponse for &'static str
impl IntoResponse for Cow<'static, str>

// Tuples (status + body)
impl IntoResponse for (StatusCode, String)
impl IntoResponse for (StatusCode, &'static str)

// Tuples (headers + body)
impl IntoResponse for (HeaderMap, String)

// Tuples (status + headers + body)
impl IntoResponse for (StatusCode, HeaderMap, String)

// Results
impl<T: IntoResponse, E: Into<Error>> IntoResponse for Result<T, E>

// Response itself
impl IntoResponse for Response

// Numbers (as plain text)
impl IntoResponse for i32, i64, u32, u64, f32, f64

// Unit type (empty response)
impl IntoResponse for ()
```

### Usage Examples

```rust
// Return plain strings
async fn handler1() -> impl IntoResponse {
    "Hello, World!"
}

// Return with status code
async fn handler2() -> impl IntoResponse {
    (StatusCode::CREATED, "Resource created")
}

// Return JSON
async fn handler3() -> impl IntoResponse {
    Response::json(serde_json::json!({
        "status": "success"
    }))
}

// Return with headers
async fn handler4() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom", "value".parse().unwrap());
    (headers, "Response with headers")
}

// Return Result (errors auto-convert)
async fn handler5() -> Result<impl IntoResponse> {
    let data = database_operation().await?;
    Ok(Response::json(data))
}

// Return numbers
async fn handler6() -> impl IntoResponse {
    42  // Converts to "42" text response
}

// Empty response
async fn handler7() -> impl IntoResponse {
    ()  // 200 OK with empty body
}
```

### Custom IntoResponse Implementation

```rust
#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

impl IntoResponse for User {
    fn into_response(self) -> Response {
        Response::json(self)
            .with_header("X-Resource-Type", "user")
    }
}

// Usage
async fn get_user() -> impl IntoResponse {
    User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }  // Automatically converts to JSON response
}

// Domain-specific response types
#[derive(Debug)]
enum ApiResult<T> {
    Success(T),
    NotFound,
    Unauthorized,
}

impl<T: Serialize> IntoResponse for ApiResult<T> {
    fn into_response(self) -> Response {
        match self {
            ApiResult::Success(data) => Response::json(data),
            ApiResult::NotFound => Response::not_found(),
            ApiResult::Unauthorized => Response::unauthorized(),
        }
    }
}

async fn api_handler() -> impl IntoResponse {
    let user = find_user(123).await;
    match user {
        Some(u) => ApiResult::Success(u),
        None => ApiResult::NotFound,
    }
}
```

## Basic Responses

### Simple Response Creation

Ignitia provides convenient methods for creating common response types:

```rust
use ignitia::{Response, StatusCode, Result};

async fn basic_responses() -> impl IntoResponse {
    // Status-only responses
    let ok_response = Response::ok();                    // 200 OK
    let created_response = Response::created();          // 201 Created
    let accepted_response = Response::accepted();        // 202 Accepted
    let no_content_response = Response::no_content();    // 204 No Content

    // Error responses
    let bad_request = Response::bad_request();           // 400 Bad Request
    let unauthorized = Response::unauthorized();         // 401 Unauthorized
    let forbidden = Response::forbidden();               // 403 Forbidden
    let not_found = Response::not_found();              // 404 Not Found
    let internal_error = Response::internal_error();     // 500 Internal Server Error

    // Custom status code
    let teapot = Response::new().with_status(StatusCode::IM_A_TEAPOT); // 418

    ok_response
}

// Method chaining for quick customization
async fn quick_customization() -> impl IntoResponse {
    Response::ok()
        .with_header("X-Custom", "value")
        .with_body("Custom response body")
}
```

### Convenience Methods (v0.2.4: Infallible)

```rust
// Text responses (no Result needed!)
async fn text_responses() -> impl IntoResponse {
    // Plain text
    Response::text("This is plain text")
}

// Multiple response variants
async fn variant_responses(format: &str) -> impl IntoResponse {
    match format {
        "text" => Response::text("Plain text response"),
        "html" => Response::html("<h1>HTML response</h1>"),
        "json" => Response::json(serde_json::json!({"format": "json"})),
        _ => Response::text("Unknown format").with_status(StatusCode::BAD_REQUEST),
    }
}

// HTML responses
async fn html_responses() -> impl IntoResponse {
    Response::html("<h1>Hello World</h1>")
}

// With status codes
async fn with_status() -> impl IntoResponse {
    Response::text("Created successfully!")
        .with_status(StatusCode::CREATED)
}
```

## Response Builder

The `ResponseBuilder` provides a fluent interface for constructing complex responses:

### Basic Builder Usage

```rust
use ignitia::{Response, StatusCode};

async fn builder_basics() -> impl IntoResponse {
    Response::new()
        .with_status(StatusCode::CREATED)
        .with_header("Content-Type", "application/json")
        .with_header("Location", "/api/resources/123")
        .with_header("X-Request-ID", "req_abc123")
        .with_body(serde_json::to_string(&serde_json::json!({
            "id": 123,
            "message": "Resource created successfully"
        })).unwrap())
}

// Conditional response building
async fn conditional_builder(user_role: &str) -> impl IntoResponse {
    let mut response = Response::ok();

    // Add role-specific headers
    match user_role {
        "admin" => {
            response = response
                .with_header("X-Admin-Access", "true")
                .with_header("X-Rate-Limit", "10000");
        }
        "premium" => {
            response = response
                .with_header("X-Premium-Features", "enabled")
                .with_header("X-Rate-Limit", "1000");
        }
        _ => {
            response = response.with_header("X-Rate-Limit", "100");
        }
    }

    let data = serde_json::json!({
        "user_role": user_role,
        "features": get_features_for_role(user_role)
    });

    response.with_body(serde_json::to_string(&data).unwrap())
}
```

## Content Types

### JSON Responses (v0.2.4: Infallible)

**Major Change**: `Response::json()` is now infallible - it returns `Response` directly, not `Result<Response>`.

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
}

// Old way (v0.2.3)
async fn old_json() -> Result<Response> {
    let user = User { /* ... */ };
    Ok(Response::json(user)?)  // Double Result!
}

// New way (v0.2.4+)
async fn new_json() -> impl IntoResponse {
    let user = User {
        id: 1,
        username: "johndoe".to_string(),
        email: "john@example.com".to_string(),
    };
    Response::json(user)  // Infallible!
}

// Or with Result for error handling
async fn json_with_result() -> Result<Response> {
    let user = fetch_user(123).await?;
    Ok(Response::json(user))  // No ? operator needed!
}

// Standard API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

async fn api_response() -> impl IntoResponse {
    let response = ApiResponse {
        success: true,
        data: Some(User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        }),
        message: "User retrieved successfully".to_string(),
    };

    Response::json(response)
}
```

### HTML Responses (v0.2.4: Infallible)

```rust
// Simple HTML (infallible)
async fn html_responses() -> impl IntoResponse {
    Response::html("<h1>Hello, World!</h1>")
}

// Complete HTML document
async fn full_page() -> impl IntoResponse {
    Response::html(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Ignitia Application</title>
        </head>
        <body>
            <h1>Welcome to Ignitia! 🔥</h1>
        </body>
        </html>
    "#)
}

// With custom headers
async fn html_with_headers() -> impl IntoResponse {
    Response::html("<h1>Created</h1>")
        .with_status(StatusCode::CREATED)
        .with_header("Location", "/users/123")
}
```

### Text Responses (v0.2.4: Infallible)

```rust
// Plain text (infallible)
async fn text_responses() -> impl IntoResponse {
    Response::text("This is plain text")
}

// Or even simpler with IntoResponse
async fn simple_text() -> &'static str {
    "Hello, World!"
}

// Dynamic text
async fn dynamic_text(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

### Binary and File Responses

```rust
use bytes::Bytes;
use tokio::fs;

async fn binary_responses() -> impl IntoResponse {
    let binary_data = vec![0xFF, 0xD8, 0xFF, 0xE0];

    Response::new()
        .with_status(StatusCode::OK)
        .with_header("Content-Type", "application/octet-stream")
        .with_header("Content-Disposition", r#"attachment; filename="data.bin""#)
        .with_body(Bytes::from(binary_data))
}

async fn file_download(file_path: &str) -> Result<impl IntoResponse> {
    let file_content = fs::read(file_path).await?;

    Ok(Response::new()
        .with_status(StatusCode::OK)
        .with_header("Content-Type", "application/pdf")
        .with_header("Content-Disposition", r#"attachment; filename="document.pdf""#)
        .with_body(Bytes::from(file_content)))
}
```

## Status Codes

### HTTP Status Code Usage

```rust
// Success responses (2xx)
async fn success_responses() -> impl IntoResponse {
    // Most handlers return these
    Response::ok()                          // 200
    Response::created()                     // 201
    Response::accepted()                    // 202
    Response::no_content()                  // 204
}

// Redirection responses (3xx)
async fn redirect_responses() -> impl IntoResponse {
    Response::new()
        .with_status(StatusCode::MOVED_PERMANENTLY)
        .with_header("Location", "/new-location")
}

// Client error responses (4xx)
async fn client_errors() -> impl IntoResponse {
    Response::bad_request()                 // 400
    Response::unauthorized()                // 401
    Response::forbidden()                   // 403
    Response::not_found()                   // 404
}

// Server error responses (5xx)
async fn server_errors() -> impl IntoResponse {
    Response::internal_error()              // 500
    Response::new().with_status(StatusCode::SERVICE_UNAVAILABLE) // 503
}

// Using tuples with IntoResponse
async fn tuple_status() -> impl IntoResponse {
    (StatusCode::CREATED, "Resource created")
}
```

## Headers Management

### Essential HTTP Headers

```rust
async fn essential_headers() -> impl IntoResponse {
    Response::json(serde_json::json!({"status": "success"}))
        // Content headers
        .with_header("Content-Type", "application/json; charset=utf-8")
        .with_header("Content-Language", "en-US")

        // Caching headers
        .with_header("Cache-Control", "public, max-age=3600")
        .with_header("ETag", r#""v1.2.3""#)

        // Security headers
        .with_header("X-Content-Type-Options", "nosniff")
        .with_header("X-Frame-Options", "SAMEORIGIN")
        .with_header("Strict-Transport-Security", "max-age=31536000")

        // API headers
        .with_header("X-API-Version", "v2.4")
        .with_header("X-Rate-Limit-Remaining", "999")
}

// Using header tuples
async fn header_tuple() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom", "value".parse().unwrap());
    (headers, "Response with headers")
}
```

## Cookie Handling

### Basic Cookie Management

```rust
use ignitia::{Cookie, SameSite};

async fn cookie_basics() -> impl IntoResponse {
    let session_cookie = Cookie::new("session_id", "sess_abc123")
        .path("/")
        .http_only()
        .secure()
        .same_site(SameSite::Strict);

    Response::ok()
        .with_cookie(session_cookie)
        .with_body("Cookies set")
}
```

## Error Responses

### Structured Error Handling (v0.2.4)

**Major Change**: Errors now automatically convert to responses via `IntoResponse`.

```rust
use ignitia::{Error, IntoResponse};

// Errors automatically convert
async fn error_handler() -> Result<impl IntoResponse> {
    // Any error automatically converts to a response
    Err(Error::not_found("Resource not found"))
}

// Custom error type
#[derive(Debug)]
struct ApiError {
    code: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        Response::json(serde_json::json!({
            "error": self.code,
            "message": self.message
        }))
        .with_status(StatusCode::BAD_REQUEST)
    }
}

async fn custom_error() -> Result<impl IntoResponse, ApiError> {
    Err(ApiError {
        code: "INVALID_INPUT".to_string(),
        message: "The input is invalid".to_string(),
    })
}
```

## Migration from v0.2.3

### Response API Changes

```rust
// Old (v0.2.3)
async fn old_handler() -> Result<Response> {
    let data = MyData { field: "value" };
    Response::json(data)?  // Double Result
}

// New (v0.2.4) - Option 1: Infallible json()
async fn new_handler1() -> Result<Response> {
    let data = MyData { field: "value" };
    Ok(Response::json(data))  // No ? needed!
}

// New (v0.2.4) - Option 2: IntoResponse
async fn new_handler2() -> impl IntoResponse {
    let data = MyData { field: "value" };
    Response::json(data)  // Even simpler!
}

// New (v0.2.4) - Option 3: Direct return
async fn new_handler3() -> impl IntoResponse {
    MyData { field: "value" }  // If MyData implements IntoResponse
}
```

### Migration Checklist

- [ ] Remove `?` from all `Response::json()` calls
- [ ] Remove `?` from all `Response::html()` calls
- [ ] Remove `?` from all `Response::text()` calls
- [ ] Update handler return types to `impl IntoResponse` where beneficial
- [ ] Implement `IntoResponse` for domain types
- [ ] Update error handling to leverage automatic conversion

## Performance Optimization

### Response Compression

```rust
async fn compression_optimization(content: &str) -> impl IntoResponse {
    // Framework handles compression automatically via middleware
    Response::text(content)
        .with_header("Content-Type", "text/plain")
}
```

### Response Time Monitoring

```rust
async fn timed_response() -> impl IntoResponse {
    let start = std::time::Instant::now();

    let data = expensive_operation().await;

    let duration = start.elapsed();

    Response::json(data)
        .with_header("X-Response-Time", format!("{}ms", duration.as_millis()))
}
```

## Security Best Practices

### Secure Response Headers

```rust
async fn secure_response(content: serde_json::Value) -> impl IntoResponse {
    Response::json(content)
        // Content security
        .with_header("X-Content-Type-Options", "nosniff")
        .with_header("X-Frame-Options", "DENY")

        // HTTPS enforcement
        .with_header("Strict-Transport-Security", "max-age=31536000; includeSubDomains")

        // XSS protection
        .with_header("X-XSS-Protection", "1; mode=block")

        // Privacy
        .with_header("Referrer-Policy", "strict-origin-when-cross-origin")
}
```

## Testing Responses

### Response Testing (v0.2.4)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_response() {
        let response = Response::json(serde_json::json!({
            "message": "test"
        }));

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn test_into_response() {
        let response = "Hello".into_response();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_tuple_response() {
        let response = (StatusCode::CREATED, "Created").into_response();
        assert_eq!(response.status, StatusCode::CREATED);
    }
}
```

## Production Examples

### Complete API Handler (v0.2.4)

```rust
use ignitia::prelude::*;

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn production_handler() -> Result<impl IntoResponse> {
    let data = fetch_data().await?;

    let response = ApiResponse {
        success: true,
        data: Some(data),
        timestamp: chrono::Utc::now(),
    };

    Ok(Response::json(response)
        .with_header("X-API-Version", "2.4")
        .with_header("Cache-Control", "private, max-age=300"))
}
```

## Best Practices Summary

### v0.2.4 Best Practices

1. **Use IntoResponse**: Simplifies handler signatures
2. **No Result Wrapping**: `Response::json()` is infallible
3. **Automatic Errors**: Errors convert to responses automatically
4. **Type Safety**: Implement `IntoResponse` for domain types
5. **Simple Returns**: Use tuples for status + body responses

***

This updated guide covers all response handling in Ignitia v0.2.4+, featuring the new `IntoResponse` trait and infallible response methods for cleaner, more ergonomic code.

**🔥 Build blazing-fast APIs with Ignitia v0.2.4!**
