# Responses

This guide covers how to create, customize, and work with HTTP responses in Ignitia.

## Table of Contents
- [Basic Responses](#basic-responses)
- [Response Builder](#response-builder)
- [Content Types](#content-types)
- [Status Codes](#status-codes)
- [Headers](#headers)
- [Cookies](#cookies)
- [Error Responses](#error-responses)
- [Advanced Response Handling](#advanced-response-handling)

## Basic Responses

### Creating Simple Responses

```rust
use ignitia::{Response, Router, Result};
use http::StatusCode;

async fn hello() -> Result<Response> {
    Ok(Response::ok())
}

async fn not_found() -> Result<Response> {
    Ok(Response::not_found())
}

async fn server_error() -> Result<Response> {
    Ok(Response::internal_error())
}

// Custom status code
async fn teapot() -> Result<Response> {
    Ok(Response::new(StatusCode::IM_A_TEAPOT))
}
```

### Quick Response Methods

```rust
// Text response
async fn text_response() -> Result<Response> {
    Ok(Response::text("Hello, World!"))
}

// HTML response
async fn html_response() -> Result<Response> {
    Ok(Response::html("<h1>Hello, World!</h1>"))
}

// JSON response
async fn json_response() -> Result<Response> {
    let data = serde_json::json!({
        "message": "Hello, World!",
        "status": "success"
    });
    Response::json(data)
}
```

## Response Builder

The `ResponseBuilder` provides a fluent interface for constructing complex responses:

### Basic Builder Usage

```rust
use ignitia::{ResponseBuilder, Result};
use http::StatusCode;

async fn custom_response() -> Result<Response> {
    let response = ResponseBuilder::new()
        .status(StatusCode::CREATED)
        .header("Location", "/api/users/123")
        .header("X-Request-ID", "abc-123")
        .json(&serde_json::json!({
            "id": 123,
            "message": "User created successfully"
        }))?
        .build();

    Ok(response)
}
```

### Builder Methods

```rust
async fn builder_examples() -> Result<Response> {
    // Text with custom status
    let response = ResponseBuilder::new()
        .status_code(201)  // Using numeric status code
        .text("Resource created")
        .build();

    // HTML with headers
    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Cache-Control", "no-cache")
        .html("<h1>Dynamic Content</h1>")
        .build();

    // Binary response
    let data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
    let response = ResponseBuilder::new()
        .header("Content-Type", "application/octet-stream")
        .body(data)
        .build();

    Ok(response)
}
```

## Content Types

### JSON Responses

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

async fn get_user() -> Result<Response> {
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    let response = ApiResponse {
        success: true,
        data: Some(user),
        message: "User retrieved successfully".to_string(),
    };

    Response::json(response)
}

// Using builder for custom JSON response
async fn custom_json() -> Result<Response> {
    let data = serde_json::json!({
        "timestamp": chrono::Utc::now(),
        "version": "1.0.0"
    });

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("X-API-Version", "v1")
        .json(&data)?
        .build();

    Ok(response)
}
```

### HTML Responses

```rust
async fn html_page() -> Result<Response> {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Ignitia App</title>
        <meta charset="utf-8">
    </head>
    <body>
        <h1>Welcome to Ignitia!</h1>
        <p>A blazing fast web framework for Rust</p>
    </body>
    </html>
    "#;

    Ok(Response::html(html))
}

// Template rendering (pseudo-code)
async fn template_response() -> Result<Response> {
    let template_data = serde_json::json!({
        "title": "User Dashboard",
        "user": {
            "name": "John Doe",
            "role": "admin"
        }
    });

    // Assuming you have a template engine
    let rendered = render_template("dashboard.html", &template_data)?;

    Ok(Response::html(rendered))
}
```

### Text and Binary Responses

```rust
use bytes::Bytes;

async fn plain_text() -> Result<Response> {
    Ok(Response::text("This is plain text content"))
}

async fn binary_data() -> Result<Response> {
    let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // Example binary data

    let response = ResponseBuilder::new()
        .header("Content-Type", "application/octet-stream")
        .header("Content-Disposition", "attachment; filename=\"data.bin\"")
        .body(Bytes::from(data))
        .build();

    Ok(response)
}

async fn file_download() -> Result<Response> {
    // Read file content (in practice, stream large files)
    let file_content = std::fs::read("document.pdf")?;

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/pdf")
        .header("Content-Disposition", "attachment; filename=\"document.pdf\"")
        .header("Content-Length", file_content.len().to_string())
        .body(file_content)
        .build();

    Ok(response)
}
```

## Status Codes

### Using Status Code Extensions

```rust
use ignitia::response::status::StatusCodeExt;
use http::StatusCode;

async fn status_examples() -> Result<Response> {
    let status = StatusCode::CREATED;

    // Check status categories
    if status.is_success() {
        println!("Success status");
    }

    if status.is_client_error() {
        println!("Client error");
    }

    if status.is_server_error() {
        println!("Server error");
    }

    Ok(Response::new(status))
}

// Common status codes
async fn various_statuses() -> Result<Response> {
    // Success responses
    let ok = Response::new(StatusCode::OK);                    // 200
    let created = Response::new(StatusCode::CREATED);          // 201
    let accepted = Response::new(StatusCode::ACCEPTED);        // 202
    let no_content = Response::new(StatusCode::NO_CONTENT);    // 204

    // Redirection responses
    let moved = Response::new(StatusCode::MOVED_PERMANENTLY);      // 301
    let found = Response::new(StatusCode::FOUND);                  // 302
    let not_modified = Response::new(StatusCode::NOT_MODIFIED);    // 304

    // Client error responses
    let bad_request = Response::new(StatusCode::BAD_REQUEST);      // 400
    let unauthorized = Response::new(StatusCode::UNAUTHORIZED);    // 401
    let forbidden = Response::new(StatusCode::FORBIDDEN);          // 403
    let not_found = Response::new(StatusCode::NOT_FOUND);          // 404

    // Server error responses
    let internal_error = Response::new(StatusCode::INTERNAL_SERVER_ERROR); // 500
    let not_implemented = Response::new(StatusCode::NOT_IMPLEMENTED);      // 501
    let service_unavailable = Response::new(StatusCode::SERVICE_UNAVAILABLE); // 503

    Ok(ok)
}
```

### Custom Status Codes

```rust
async fn custom_status() -> Result<Response> {
    // Using numeric status code
    let response = ResponseBuilder::new()
        .status_code(418)  // I'm a teapot
        .text("I'm a teapot! ☕")
        .build();

    Ok(response)
}
```

## Headers

### Setting Headers

```rust
use http::{HeaderMap, HeaderName, HeaderValue};

async fn header_examples() -> Result<Response> {
    let response = ResponseBuilder::new()
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=3600")
        .header("X-Custom-Header", "custom-value")
        .header("Access-Control-Allow-Origin", "*")
        .json(&serde_json::json!({"message": "Hello"}))?
        .build();

    Ok(response)
}

// Working with headers directly
async fn direct_headers() -> Result<Response> {
    let mut response = Response::json(serde_json::json!({"data": "test"}))?;

    // Add headers after creation
    response.headers.insert("X-Processing-Time", "150ms".parse().unwrap());
    response.headers.insert("X-Server-ID", "server-01".parse().unwrap());

    Ok(response)
}
```

### Common Header Patterns

```rust
async fn security_headers() -> Result<Response> {
    let response = ResponseBuilder::new()
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "DENY")
        .header("X-XSS-Protection", "1; mode=block")
        .header("Strict-Transport-Security", "max-age=31536000")
        .header("Content-Security-Policy", "default-src 'self'")
        .html("<h1>Secure Page</h1>")
        .build();

    Ok(response)
}

async fn api_headers() -> Result<Response> {
    let response = ResponseBuilder::new()
        .header("X-API-Version", "v1")
        .header("X-Rate-Limit-Remaining", "99")
        .header("X-Response-Time", "23ms")
        .header("ETag", "\"12345678\"")
        .json(&serde_json::json!({"result": "success"}))?
        .build();

    Ok(response)
}

async fn caching_headers() -> Result<Response> {
    let response = ResponseBuilder::new()
        .header("Cache-Control", "public, max-age=3600")
        .header("ETag", "\"version-123\"")
        .header("Last-Modified", "Wed, 21 Oct 2015 07:28:00 GMT")
        .header("Expires", "Thu, 22 Oct 2015 07:28:00 GMT")
        .text("Cached content")
        .build();

    Ok(response)
}
```

## Cookies

### Setting Cookies

```rust
use ignitia::{Cookie, SameSite};
use std::time::{Duration, SystemTime};

async fn cookie_examples() -> Result<Response> {
    let session_cookie = Cookie::new("session_id", "abc123")
        .path("/")
        .http_only()
        .secure()
        .same_site(SameSite::Strict);

    let preferences_cookie = Cookie::new("theme", "dark")
        .path("/")
        .max_age(30 * 24 * 3600) // 30 days
        .same_site(SameSite::Lax);

    let response = Response::text("Cookies set!")
        .add_cookie(session_cookie)
        .add_cookie(preferences_cookie);

    Ok(response)
}

// Multiple cookies at once
async fn multiple_cookies() -> Result<Response> {
    let cookies = vec![
        Cookie::new("user_id", "12345"),
        Cookie::new("language", "en")
            .max_age(365 * 24 * 3600), // 1 year
        Cookie::new("remember_me", "true")
            .max_age(30 * 24 * 3600)   // 30 days
            .secure()
            .http_only(),
    ];

    let response = Response::json(serde_json::json!({"status": "logged_in"}))?
        .add_cookies(cookies);

    Ok(response)
}

// Removing cookies
async fn logout() -> Result<Response> {
    let response = Response::text("Logged out")
        .remove_cookie("session_id")
        .remove_cookie("user_id")
        .remove_cookie("remember_me");

    Ok(response)
}
```

### Advanced Cookie Configuration

```rust
async fn advanced_cookies() -> Result<Response> {
    // Secure authentication cookie
    let auth_cookie = Cookie::new("auth_token", "jwt_token_here")
        .domain("example.com")
        .path("/")
        .max_age(24 * 3600) // 24 hours
        .secure()
        .http_only()
        .same_site(SameSite::Strict);

    // Tracking cookie with expiration date
    let expires = SystemTime::now() + Duration::from_secs(7 * 24 * 3600); // 7 days
    let tracking_cookie = Cookie::new("tracking_id", "track_123")
        .expires(expires)
        .path("/")
        .same_site(SameSite::None); // For cross-site requests

    let response = Response::json(serde_json::json!({"authenticated": true}))?
        .add_cookie(auth_cookie)
        .add_cookie(tracking_cookie);

    Ok(response)
}
```

## Error Responses

### Standard Error Responses

```rust
use ignitia::{Error, Response, Result};
use http::StatusCode;

async fn error_examples() -> Result<Response> {
    // Using error conversion
    let error = Error::not_found("/api/users/999");
    Ok(Response::from(error))
}

// Custom error responses
async fn custom_errors() -> Result<Response> {
    // Validation error
    let errors = vec![
        "Email is required".to_string(),
        "Password must be at least 8 characters".to_string(),
    ];

    Response::validation_error(errors)
}

// API error format
async fn api_error() -> Result<Response> {
    let error_data = serde_json::json!({
        "error": {
            "code": "RESOURCE_NOT_FOUND",
            "message": "The requested user was not found",
            "details": {
                "user_id": 999,
                "timestamp": chrono::Utc::now()
            }
        }
    });

    let response = ResponseBuilder::new()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "application/json")
        .json(&error_data)?
        .build();

    Ok(response)
}
```

### Error Response Patterns

```rust
// RESTful API error responses
async fn rest_errors() -> Result<Response> {
    // 400 Bad Request
    let bad_request = serde_json::json!({
        "error": "Invalid request format",
        "code": "INVALID_REQUEST",
        "timestamp": chrono::Utc::now()
    });

    // 401 Unauthorized
    let unauthorized = serde_json::json!({
        "error": "Authentication required",
        "code": "UNAUTHORIZED",
        "message": "Please provide a valid authentication token"
    });

    // 403 Forbidden
    let forbidden = serde_json::json!({
        "error": "Access denied",
        "code": "FORBIDDEN",
        "message": "You don't have permission to access this resource"
    });

    // 404 Not Found
    let not_found = serde_json::json!({
        "error": "Resource not found",
        "code": "NOT_FOUND",
        "path": "/api/users/999"
    });

    // 500 Internal Server Error
    let server_error = serde_json::json!({
        "error": "Internal server error",
        "code": "INTERNAL_ERROR",
        "request_id": "req_123456789"
    });

    Ok(Response::json(bad_request)?)
}
```

## Advanced Response Handling

### Streaming Responses

```rust
use futures::stream::{self, StreamExt};

async fn streaming_response() -> Result<Response> {
    // For large data that should be streamed
    let data = "This is a large response that should be streamed...".repeat(1000);

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .header("Transfer-Encoding", "chunked")
        .body(data)
        .build();

    Ok(response)
}
```

### Conditional Responses

```rust
use ignitia::{Request, Path, Query};

async fn conditional_response(
    Path(user_id): Path<u32>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    // Simulate user lookup
    let user_exists = user_id <= 1000;

    if !user_exists {
        return Ok(Response::new(StatusCode::NOT_FOUND));
    }

    // Check for specific format request
    match params.get("format").map(|s| s.as_str()) {
        Some("json") => {
            let user_data = serde_json::json!({
                "id": user_id,
                "name": "John Doe",
                "email": "john@example.com"
            });
            Response::json(user_data)
        }
        Some("xml") => {
            let xml_data = format!(
                "<user><id>{}</id><name>John Doe</name></user>",
                user_id
            );
            let response = ResponseBuilder::new()
                .header("Content-Type", "application/xml")
                .body(xml_data)
                .build();
            Ok(response)
        }
        _ => {
            // Default to HTML
            let html = format!("<h1>User {}</h1><p>John Doe</p>", user_id);
            Ok(Response::html(html))
        }
    }
}
```

### Response Middleware Integration

```rust
// Responses work seamlessly with middleware
async fn middleware_compatible() -> Result<Response> {
    // This response will be processed by middleware
    // (compression, security headers, etc.)
    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("X-Custom-Header", "value")
        .json(&serde_json::json!({
            "message": "This response will be processed by middleware",
            "timestamp": chrono::Utc::now()
        }))?
        .build();

    Ok(response)
}
```

### Response Utilities

```rust
// Helper functions for common response patterns
fn success_response<T: Serialize>(data: T) -> Result<Response> {
    let response_body = serde_json::json!({
        "success": true,
        "data": data,
        "timestamp": chrono::Utc::now()
    });
    Response::json(response_body)
}

fn error_response(message: &str, code: StatusCode) -> Result<Response> {
    let error_body = serde_json::json!({
        "success": false,
        "error": {
            "message": message,
            "code": code.as_u16()
        },
        "timestamp": chrono::Utc::now()
    });

    let response = ResponseBuilder::new()
        .status(code)
        .json(&error_body)?
        .build();

    Ok(response)
}

fn paginated_response<T: Serialize>(
    data: Vec<T>,
    page: u32,
    limit: u32,
    total: u64,
) -> Result<Response> {
    let response_body = serde_json::json!({
        "data": data,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "pages": (total + limit as u64 - 1) / limit as u64
        }
    });
    Response::json(response_body)
}
```

## Best Practices

1. **Use appropriate status codes** - Be precise with HTTP status codes to help clients understand the response

2. **Set proper Content-Type headers** - Always set the correct Content-Type for your response data

3. **Handle errors gracefully** - Provide meaningful error messages and appropriate error codes

4. **Use response builders for complex responses** - The builder pattern makes complex responses more readable

5. **Consider security headers** - Set appropriate security headers for web applications

6. **Be consistent with API responses** - Use consistent response formats across your API

7. **Use cookies securely** - Always set `Secure`, `HttpOnly`, and `SameSite` attributes appropriately

8. **Stream large responses** - For large data, consider streaming instead of loading everything into memory

This covers the comprehensive response handling capabilities in Ignitia. The framework provides flexible and powerful tools for creating any type of HTTP response your application needs.
