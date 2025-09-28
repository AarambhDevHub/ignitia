# HTTP Responses in Ignitia 🔥

A comprehensive guide to creating, customizing, and optimizing HTTP responses in Ignitia - from simple text responses to complex streaming APIs.

## Table of Contents

- [Quick Start](#quick-start)
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
- [Production Examples](#production-examples)

## Quick Start

Get started with Ignitia responses in seconds:

```rust
use ignitia::prelude::*;

// Simple text response
async fn hello() -> Result<Response> {
    Ok(Response::text("Hello, Ignitia! 🔥"))
}

// JSON API response
async fn api_response() -> Result<Response> {
    Response::json(serde_json::json!({
        "message": "Welcome to Ignitia",
        "version": "0.2.3",
        "status": "success"
    }))
}

// HTML page response
async fn homepage() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
            <head><title>Ignitia App</title></head>
            <body><h1>Powered by Ignitia 🔥</h1></body>
        </html>
    "#))
}
```

## Basic Responses

### Simple Response Creation

Ignitia provides convenient methods for creating common response types:

```rust
use ignitia::{Response, StatusCode, Result};

async fn basic_responses() -> Result<Response> {
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
    let teapot = Response::new(StatusCode::IM_A_TEAPOT); // 418 I'm a teapot

    Ok(ok_response)
}

// Method chaining for quick customization
async fn quick_customization() -> Result<Response> {
    Ok(Response::ok()
        .with_header("X-Custom", "value")
        .with_body("Custom response body"))
}
```

### Convenience Methods

```rust
// Text responses with automatic Content-Type
async fn text_responses() -> Result<Response> {
    // Plain text
    let plain = Response::text("This is plain text");

    // Text with custom status
    let custom_text = Response::text_with_status(
        "Created successfully!",
        StatusCode::CREATED
    );

    // Text with encoding specified
    let encoded_text = Response::text("UTF-8 encoded: 🚀🔥⚡")
        .with_header("Content-Type", "text/plain; charset=utf-8");

    Ok(plain)
}

// HTML responses with proper Content-Type
async fn html_responses() -> Result<Response> {
    let simple_html = Response::html("<h1>Hello World</h1>");

    let full_page = Response::html(include_str!("../templates/index.html"));

    // HTML with custom status and headers
    let custom_html = Response::html("<h1>Created</h1>")
        .with_status(StatusCode::CREATED)
        .with_header("Location", "/users/123");

    Ok(simple_html)
}
```

## Response Builder

The `ResponseBuilder` provides a fluent interface for constructing complex responses with full control over all aspects:

### Basic Builder Usage

```rust
use ignitia::{ResponseBuilder, StatusCode, Result};

async fn builder_basics() -> Result<Response> {
    let response = ResponseBuilder::new()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .header("Location", "/api/resources/123")
        .header("X-Request-ID", "req_abc123")
        .header("X-Processing-Time", "45ms")
        .json(&serde_json::json!({
            "id": 123,
            "message": "Resource created successfully",
            "timestamp": chrono::Utc::now(),
            "links": {
                "self": "/api/resources/123",
                "collection": "/api/resources"
            }
        }))?
        .build();

    Ok(response)
}

// Conditional response building
async fn conditional_builder(user_role: &str) -> Result<Response> {
    let mut builder = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json");

    // Add role-specific headers
    match user_role {
        "admin" => {
            builder = builder
                .header("X-Admin-Access", "true")
                .header("X-Rate-Limit", "10000");
        }
        "premium" => {
            builder = builder
                .header("X-Premium-Features", "enabled")
                .header("X-Rate-Limit", "1000");
        }
        _ => {
            builder = builder.header("X-Rate-Limit", "100");
        }
    }

    let data = serde_json::json!({
        "user_role": user_role,
        "features": get_features_for_role(user_role)
    });

    Ok(builder.json(&data)?.build())
}
```

### Advanced Builder Patterns

```rust
async fn advanced_builder_patterns() -> Result<Response> {
    // API response with comprehensive metadata
    let api_response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("X-API-Version", "v2")
        .header("X-Rate-Limit-Remaining", "99")
        .header("X-Response-Time", "23ms")
        .header("ETag", r#""v2.1.123""#)
        .header("Cache-Control", "private, max-age=300")
        .header("Vary", "Accept-Encoding, Authorization")
        .json(&serde_json::json!({
            "data": {
                "users": [],
                "total": 0
            },
            "meta": {
                "page": 1,
                "per_page": 20,
                "total_pages": 0
            },
            "links": {
                "self": "/api/v2/users?page=1",
                "next": null,
                "prev": null
            }
        }))?
        .build();

    // File download response
    let file_content = tokio::fs::read("report.pdf").await?;
    let file_response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/pdf")
        .header("Content-Disposition", r#"attachment; filename="monthly_report.pdf""#)
        .header("Content-Length", file_content.len().to_string())
        .header("Cache-Control", "private, no-cache")
        .header("X-File-Size", format!("{} bytes", file_content.len()))
        .body(file_content)
        .build();

    // Redirect response with metadata
    let redirect_response = ResponseBuilder::new()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header("Location", "https://newdomain.com/api/v2/users")
        .header("X-Redirect-Reason", "API_MIGRATION")
        .header("X-New-Endpoint", "/api/v2/users")
        .json(&serde_json::json!({
            "message": "This endpoint has moved permanently",
            "new_url": "https://newdomain.com/api/v2/users",
            "migration_date": "2024-01-01",
            "support_contact": "api-support@example.com"
        }))?
        .build();

    Ok(api_response)
}
```

## Content Types

### JSON Responses

JSON is the primary format for modern APIs. Ignitia provides comprehensive JSON response capabilities:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
    full_name: Option<String>,
    avatar_url: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
    role: UserRole,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum UserRole {
    Admin,
    User,
    Guest,
}

// Standard API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
    timestamp: DateTime<Utc>,
    request_id: String,
}

impl<T> ApiResponse<T> {
    fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            timestamp: Utc::now(),
            request_id: generate_request_id(),
        }
    }

    fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: message.into(),
            timestamp: Utc::now(),
            request_id: generate_request_id(),
        }
    }
}

async fn json_responses() -> Result<Response> {
    let user = User {
        id: 12345,
        username: "johndoe".to_string(),
        email: "john@example.com".to_string(),
        full_name: Some("John Doe".to_string()),
        avatar_url: Some("https://example.com/avatars/johndoe.jpg".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_active: true,
        role: UserRole::User,
    };

    let response = ApiResponse::success(user, "User retrieved successfully");

    let json_response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("X-Content-Version", "v2")
        .json(&response)?
        .build();

    Ok(json_response)
}

// Paginated JSON responses
#[derive(Serialize)]
struct PaginatedResponse<T> {
    data: Vec<T>,
    pagination: PaginationMeta,
    links: PaginationLinks,
}

#[derive(Serialize)]
struct PaginationMeta {
    current_page: u32,
    per_page: u32,
    total_items: u64,
    total_pages: u32,
    has_next: bool,
    has_prev: bool,
}

#[derive(Serialize)]
struct PaginationLinks {
    #[serde(rename = "self")]
    self_link: String,
    next: Option<String>,
    prev: Option<String>,
    first: String,
    last: String,
}

async fn paginated_json_response(
    users: Vec<User>,
    page: u32,
    per_page: u32,
    total: u64
) -> Result<Response> {
    let total_pages = ((total + per_page as u64 - 1) / per_page as u64) as u32;

    let paginated = PaginatedResponse {
        data: users,
        pagination: PaginationMeta {
            current_page: page,
            per_page,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
        links: PaginationLinks {
            self_link: format!("/api/users?page={}&per_page={}", page, per_page),
            next: if page < total_pages {
                Some(format!("/api/users?page={}&per_page={}", page + 1, per_page))
            } else { None },
            prev: if page > 1 {
                Some(format!("/api/users?page={}&per_page={}", page - 1, per_page))
            } else { None },
            first: format!("/api/users?page=1&per_page={}", per_page),
            last: format!("/api/users?page={}&per_page={}", total_pages, per_page),
        },
    };

    Response::json(paginated)
}
```

### HTML Responses and Template Integration

```rust
// Static HTML responses
async fn html_responses() -> Result<Response> {
    // Simple HTML
    let simple_html = Response::html("<h1>Hello, World!</h1>");

    // Complete HTML document
    let full_page = Response::html(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Ignitia Application</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; }
                .header { color: #ff6b35; }
                .content { background: #f8f9fa; padding: 20px; border-radius: 8px; }
            </style>
        </head>
        <body>
            <h1 class="header">Welcome to Ignitia! 🔥</h1>
            <div class="content">
                <p>A blazing fast web framework for Rust.</p>
                <ul>
                    <li>High Performance</li>
                    <li>Type Safety</li>
                    <li>Modern Async</li>
                </ul>
            </div>
        </body>
        </html>
    "#);

    Ok(full_page)
}

// Template-based HTML (using a template engine like Handlebars)
async fn template_html_response(user_name: &str, posts: Vec<BlogPost>) -> Result<Response> {
    // Pseudo-code for template rendering
    let template_data = serde_json::json!({
        "user": {
            "name": user_name,
            "post_count": posts.len()
        },
        "posts": posts,
        "generated_at": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    });

    // In practice, you'd use a template engine like:
    // let rendered = handlebars.render("dashboard", &template_data)?;
    let rendered = format!(r#"
        <html>
        <head><title>Dashboard - {}</title></head>
        <body>
            <h1>Welcome back, {}!</h1>
            <p>You have {} posts</p>
            <p>Generated at: {}</p>
        </body>
        </html>
    "#, user_name, user_name, posts.len(), chrono::Utc::now());

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "private, no-cache")
        .header("X-Generated-At", chrono::Utc::now().to_rfc3339())
        .body(rendered)
        .build();

    Ok(response)
}
```

### Binary and File Responses

```rust
use bytes::Bytes;
use tokio::fs;

async fn binary_responses() -> Result<Response> {
    // Raw binary data
    let binary_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]; // JPEG header
    let binary_response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header("Content-Disposition", r#"attachment; filename="data.bin""#)
        .header("Content-Length", binary_data.len().to_string())
        .body(Bytes::from(binary_data))
        .build();

    Ok(binary_response)
}

async fn file_download_response(file_path: &str) -> Result<Response> {
    // Secure file path validation
    let safe_path = std::path::Path::new(file_path);
    if !safe_path.starts_with("./uploads/") {
        return Err(ignitia::Error::Forbidden("Invalid file path".to_string()));
    }

    // Read file metadata
    let metadata = fs::metadata(safe_path).await
        .map_err(|_| ignitia::Error::NotFound("File not found".to_string()))?;

    let file_size = metadata.len();
    let file_content = fs::read(safe_path).await
        .map_err(|_| ignitia::Error::Internal("Failed to read file".to_string()))?;

    // Determine content type based on file extension
    let content_type = match safe_path.extension()
        .and_then(|ext| ext.to_str()) {
        Some("pdf") => "application/pdf",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("txt") => "text/plain",
        Some("csv") => "text/csv",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };

    let filename = safe_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download");

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Content-Disposition", format!(r#"attachment; filename="{}""#, filename))
        .header("Content-Length", file_size.to_string())
        .header("Cache-Control", "private, no-cache")
        .header("X-File-Name", filename)
        .body(file_content)
        .build();

    Ok(response)
}

// Image responses with proper headers
async fn image_response() -> Result<Response> {
    let image_data = fs::read("./assets/logo.png").await?;

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .header("Content-Length", image_data.len().to_string())
        .header("Cache-Control", "public, max-age=31536000") // 1 year cache
        .header("ETag", r#""logo-v1.2.3""#)
        .body(image_data)
        .build();

    Ok(response)
}
```

### Streaming Responses

```rust
use futures::stream::{self, StreamExt};

async fn streaming_json_response() -> Result<Response> {
    // For large datasets that should be streamed
    let large_data = (1..=10000).map(|i| serde_json::json!({
        "id": i,
        "name": format!("Item {}", i),
        "timestamp": chrono::Utc::now()
    })).collect::<Vec<_>>();

    // Convert to newline-delimited JSON (NDJSON)
    let ndjson_data = large_data.iter()
        .map(|item| serde_json::to_string(item).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-ndjson")
        .header("Transfer-Encoding", "chunked")
        .header("X-Total-Items", "10000")
        .body(ndjson_data)
        .build();

    Ok(response)
}

// Server-Sent Events (SSE) response
async fn sse_response() -> Result<Response> {
    let sse_data = format!(
        "data: {}\n\n",
        serde_json::json!({
            "event": "welcome",
            "message": "Connected to live updates",
            "timestamp": chrono::Utc::now()
        })
    );

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .header("Access-Control-Allow-Origin", "*")
        .header("X-Accel-Buffering", "no") // Disable nginx buffering
        .body(sse_data)
        .build();

    Ok(response)
}
```

## Status Codes

### HTTP Status Code Best Practices

Ignitia provides comprehensive status code support with utilities for proper HTTP semantics:

```rust
use ignitia::StatusCode;

async fn status_code_examples() -> Result<Response> {
    // Success responses (2xx)
    let ok = Response::new(StatusCode::OK);                          // 200 - Standard success
    let created = Response::new(StatusCode::CREATED);                // 201 - Resource created
    let accepted = Response::new(StatusCode::ACCEPTED);              // 202 - Accepted for processing
    let no_content = Response::new(StatusCode::NO_CONTENT);          // 204 - Success with no body
    let reset_content = Response::new(StatusCode::RESET_CONTENT);    // 205 - Reset document view
    let partial_content = Response::new(StatusCode::PARTIAL_CONTENT); // 206 - Range request

    // Redirection responses (3xx)
    let moved_permanently = Response::new(StatusCode::MOVED_PERMANENTLY);    // 301 - Permanent redirect
    let found = Response::new(StatusCode::FOUND);                           // 302 - Temporary redirect
    let see_other = Response::new(StatusCode::SEE_OTHER);                   // 303 - See other resource
    let not_modified = Response::new(StatusCode::NOT_MODIFIED);             // 304 - Not modified (cache)
    let temporary_redirect = Response::new(StatusCode::TEMPORARY_REDIRECT); // 307 - Temporary redirect
    let permanent_redirect = Response::new(StatusCode::PERMANENT_REDIRECT); // 308 - Permanent redirect

    // Client error responses (4xx)
    let bad_request = Response::new(StatusCode::BAD_REQUEST);              // 400 - Invalid request
    let unauthorized = Response::new(StatusCode::UNAUTHORIZED);            // 401 - Authentication required
    let payment_required = Response::new(StatusCode::PAYMENT_REQUIRED);    // 402 - Payment required
    let forbidden = Response::new(StatusCode::FORBIDDEN);                  // 403 - Access denied
    let not_found = Response::new(StatusCode::NOT_FOUND);                  // 404 - Resource not found
    let method_not_allowed = Response::new(StatusCode::METHOD_NOT_ALLOWED); // 405 - Invalid HTTP method
    let not_acceptable = Response::new(StatusCode::NOT_ACCEPTABLE);        // 406 - Unacceptable content
    let conflict = Response::new(StatusCode::CONFLICT);                    // 409 - Resource conflict
    let gone = Response::new(StatusCode::GONE);                           // 410 - Resource no longer available
    let length_required = Response::new(StatusCode::LENGTH_REQUIRED);      // 411 - Content-Length required
    let precondition_failed = Response::new(StatusCode::PRECONDITION_FAILED); // 412 - Precondition failed
    let payload_too_large = Response::new(StatusCode::PAYLOAD_TOO_LARGE);  // 413 - Request too large
    let unsupported_media_type = Response::new(StatusCode::UNSUPPORTED_MEDIA_TYPE); // 415 - Unsupported content type
    let range_not_satisfiable = Response::new(StatusCode::RANGE_NOT_SATISFIABLE); // 416 - Invalid range
    let expectation_failed = Response::new(StatusCode::EXPECTATION_FAILED); // 417 - Expectation failed
    let im_a_teapot = Response::new(StatusCode::IM_A_TEAPOT);             // 418 - I'm a teapot (RFC 2324)
    let unprocessable_entity = Response::new(StatusCode::UNPROCESSABLE_ENTITY); // 422 - Validation errors
    let too_many_requests = Response::new(StatusCode::TOO_MANY_REQUESTS);  // 429 - Rate limited

    // Server error responses (5xx)
    let internal_server_error = Response::new(StatusCode::INTERNAL_SERVER_ERROR); // 500 - Server error
    let not_implemented = Response::new(StatusCode::NOT_IMPLEMENTED);      // 501 - Not implemented
    let bad_gateway = Response::new(StatusCode::BAD_GATEWAY);              // 502 - Bad gateway
    let service_unavailable = Response::new(StatusCode::SERVICE_UNAVAILABLE); // 503 - Service unavailable
    let gateway_timeout = Response::new(StatusCode::GATEWAY_TIMEOUT);      // 504 - Gateway timeout
    let http_version_not_supported = Response::new(StatusCode::HTTP_VERSION_NOT_SUPPORTED); // 505 - HTTP version not supported

    Ok(ok)
}

// Status code utilities and helpers
async fn status_code_utilities() -> Result<Response> {
    let status = StatusCode::CREATED;

    // Status code categories
    if status.is_informational() {   // 1xx
        println!("Informational response");
    }
    if status.is_success() {         // 2xx
        println!("Success response");
    }
    if status.is_redirection() {     // 3xx
        println!("Redirection response");
    }
    if status.is_client_error() {    // 4xx
        println!("Client error response");
    }
    if status.is_server_error() {    // 5xx
        println!("Server error response");
    }

    // Status code conversion
    let status_u16: u16 = status.as_u16();              // Get numeric value
    let status_str: &str = status.as_str();             // Get string representation
    let canonical_reason = status.canonical_reason();    // Get canonical reason phrase

    println!("Status: {} {} {}", status_u16, status_str, canonical_reason.unwrap_or("Unknown"));

    Ok(Response::new(status))
}

// REST API status code patterns
async fn rest_status_patterns(operation: &str, success: bool) -> Result<Response> {
    let response = match (operation, success) {
        // GET operations
        ("get", true) => Response::new(StatusCode::OK),
        ("get", false) => Response::new(StatusCode::NOT_FOUND),

        // POST operations (create)
        ("create", true) => ResponseBuilder::new()
            .status(StatusCode::CREATED)
            .header("Location", "/api/resource/123")
            .build(),
        ("create", false) => Response::new(StatusCode::BAD_REQUEST),

        // PUT operations (update/replace)
        ("update", true) => Response::new(StatusCode::OK),
        ("replace", true) => Response::new(StatusCode::OK),
        ("update", false) | ("replace", false) => Response::new(StatusCode::NOT_FOUND),

        // PATCH operations (partial update)
        ("patch", true) => Response::new(StatusCode::OK),
        ("patch", false) => Response::new(StatusCode::NOT_FOUND),

        // DELETE operations
        ("delete", true) => Response::new(StatusCode::NO_CONTENT), // Successful deletion, no body
        ("delete", false) => Response::new(StatusCode::NOT_FOUND),

        _ => Response::new(StatusCode::METHOD_NOT_ALLOWED),
    };

    Ok(response)
}
```

### Custom Status Codes and Extensions

```rust
// Custom status code helpers
impl ResponseBuilder {
    // Custom success responses
    fn accepted_async(self) -> Self {
        self.status(StatusCode::ACCEPTED)
            .header("X-Processing-Status", "accepted")
    }

    fn created_with_location(self, location: &str) -> Self {
        self.status(StatusCode::CREATED)
            .header("Location", location)
    }

    // Custom error responses
    fn validation_error(self) -> Self {
        self.status(StatusCode::UNPROCESSABLE_ENTITY)
            .header("X-Error-Type", "validation")
    }

    fn rate_limited(self, retry_after: u32) -> Self {
        self.status(StatusCode::TOO_MANY_REQUESTS)
            .header("Retry-After", retry_after.to_string())
            .header("X-Rate-Limit-Exceeded", "true")
    }

    fn maintenance_mode(self) -> Self {
        self.status(StatusCode::SERVICE_UNAVAILABLE)
            .header("Retry-After", "3600") // 1 hour
            .header("X-Maintenance-Mode", "true")
    }
}

async fn custom_status_examples() -> Result<Response> {
    // Using custom status helpers
    let async_response = ResponseBuilder::new()
        .accepted_async()
        .json(&serde_json::json!({
            "message": "Request accepted for processing",
            "job_id": "job_123456",
            "status_url": "/api/jobs/123456"
        }))?
        .build();

    let created_response = ResponseBuilder::new()
        .created_with_location("/api/users/789")
        .json(&serde_json::json!({
            "id": 789,
            "message": "User created successfully"
        }))?
        .build();

    let validation_response = ResponseBuilder::new()
        .validation_error()
        .json(&serde_json::json!({
            "error": "Validation failed",
            "details": {
                "email": ["Email is required", "Email must be valid"],
                "password": ["Password must be at least 8 characters"]
            }
        }))?
        .build();

    Ok(async_response)
}
```

## Headers Management

### Essential HTTP Headers

```rust
async fn essential_headers() -> Result<Response> {
    let response = ResponseBuilder::new()
        .status(StatusCode::OK)

        // Content headers
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Content-Language", "en-US")
        .header("Content-Encoding", "gzip")
        .header("Content-Length", "1234")

        // Caching headers
        .header("Cache-Control", "public, max-age=3600, s-maxage=7200")
        .header("ETag", r#""v1.2.3-abc123""#)
        .header("Last-Modified", "Wed, 21 Oct 2015 07:28:00 GMT")
        .header("Expires", "Thu, 22 Oct 2015 07:28:00 GMT")
        .header("Vary", "Accept-Encoding, Accept-Language")

        // Security headers
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "SAMEORIGIN")
        .header("X-XSS-Protection", "1; mode=block")
        .header("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
        .header("Content-Security-Policy", "default-src 'self'; script-src 'self' 'unsafe-inline'")
        .header("Referrer-Policy", "strict-origin-when-cross-origin")
        .header("Permissions-Policy", "geolocation=(), microphone=(), camera=()")

        // API headers
        .header("X-API-Version", "v2.1")
        .header("X-Request-ID", "req_abc123def456")
        .header("X-Rate-Limit-Limit", "1000")
        .header("X-Rate-Limit-Remaining", "999")
        .header("X-Rate-Limit-Reset", "1640995200")
        .header("X-Response-Time", "45ms")

        // CORS headers
        .header("Access-Control-Allow-Origin", "https://example.com")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With")
        .header("Access-Control-Expose-Headers", "X-Total-Count, X-Rate-Limit-Remaining")
        .header("Access-Control-Max-Age", "86400")
        .header("Access-Control-Allow-Credentials", "true")

        .json(&serde_json::json!({"status": "success"}))?
        .build();

    Ok(response)
}

// Dynamic header management
async fn dynamic_headers(user_agent: &str, accept_language: &str) -> Result<Response> {
    let mut builder = ResponseBuilder::new()
        .status(StatusCode::OK);

    // Add headers based on user agent
    if user_agent.contains("Mobile") {
        builder = builder
            .header("X-Mobile-Optimized", "true")
            .header("Cache-Control", "no-cache"); // Mobile users get fresh content
    } else {
        builder = builder
            .header("Cache-Control", "public, max-age=3600");
    }

    // Add language-specific headers
    let content_language = if accept_language.contains("es") {
        "es"
    } else if accept_language.contains("fr") {
        "fr"
    } else {
        "en"
    };

    builder = builder
        .header("Content-Language", content_language)
        .header("Vary", "Accept-Language, User-Agent");

    let localized_content = match content_language {
        "es" => serde_json::json!({"message": "¡Hola mundo!"}),
        "fr" => serde_json::json!({"message": "Bonjour le monde!"}),
        _ => serde_json::json!({"message": "Hello world!"}),
    };

    Ok(builder.json(&localized_content)?.build())
}
```

### Performance and Caching Headers

```rust
// Cache control strategies
async fn caching_strategies(resource_type: &str) -> Result<Response> {
    let (cache_control, etag) = match resource_type {
        "static_asset" => (
            "public, max-age=31536000, immutable", // 1 year for static assets
            format!(r#""static-{}-v1.2.3""#, resource_type)
        ),
        "api_data" => (
            "private, max-age=300, must-revalidate", // 5 minutes for API data
            format!(r#""api-{}-{}""#, resource_type, chrono::Utc::now().timestamp())
        ),
        "user_content" => (
            "private, no-cache, must-revalidate", // No caching for sensitive content
            format!(r#""user-content-{}""#, generate_etag())
        ),
        "public_content" => (
            "public, max-age=3600, s-maxage=86400", // 1 hour client, 1 day CDN
            format!(r#""public-{}-{}""#, resource_type, chrono::Utc::now().timestamp() / 3600)
        ),
        _ => (
            "no-cache, no-store, must-revalidate", // No caching by default
            format!(r#""default-{}""#, generate_etag())
        ),
    };

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Cache-Control", cache_control)
        .header("ETag", etag)
        .header("Vary", "Accept-Encoding")
        .json(&serde_json::json!({
            "resource_type": resource_type,
            "cached_at": chrono::Utc::now()
        }))?
        .build();

    Ok(response)
}

// Conditional response headers
async fn conditional_response(
    if_none_match: Option<&str>,
    if_modified_since: Option<&str>,
    current_etag: &str,
    last_modified: &str
) -> Result<Response> {
    // Check If-None-Match (ETag)
    if let Some(client_etag) = if_none_match {
        if client_etag == current_etag || client_etag == "*" {
            return Ok(ResponseBuilder::new()
                .status(StatusCode::NOT_MODIFIED)
                .header("ETag", current_etag)
                .header("Cache-Control", "public, max-age=3600")
                .build());
        }
    }

    // Check If-Modified-Since
    if let Some(since) = if_modified_since {
        if since == last_modified {
            return Ok(ResponseBuilder::new()
                .status(StatusCode::NOT_MODIFIED)
                .header("Last-Modified", last_modified)
                .header("ETag", current_etag)
                .build());
        }
    }

    // Return full response
    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("ETag", current_etag)
        .header("Last-Modified", last_modified)
        .header("Cache-Control", "public, max-age=3600")
        .json(&serde_json::json!({
            "content": "Full response content",
            "etag": current_etag,
            "last_modified": last_modified
        }))?
        .build())
}
```

### Security Headers

```rust
// Comprehensive security headers
async fn security_headers() -> Result<Response> {
    let response = ResponseBuilder::new()
        .status(StatusCode::OK)

        // Prevent MIME type sniffing
        .header("X-Content-Type-Options", "nosniff")

        // Prevent clickjacking
        .header("X-Frame-Options", "SAMEORIGIN")

        // XSS protection (legacy, but still useful)
        .header("X-XSS-Protection", "1; mode=block")

        // HTTPS enforcement
        .header("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload")

        // Content Security Policy
        .header("Content-Security-Policy",
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' https://trusted-scripts.com; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: https:; \
                 font-src 'self' https://fonts.gstatic.com; \
                 connect-src 'self' wss: https:; \
                 frame-ancestors 'none'; \
                 base-uri 'self'")

        // Referrer policy
        .header("Referrer-Policy", "strict-origin-when-cross-origin")

        // Permissions policy (feature policy)
        .header("Permissions-Policy",
                "geolocation=(), \
                 microphone=(), \
                 camera=(), \
                 magnetometer=(), \
                 gyroscope=(), \
                 payment=()")

        // Prevent DNS prefetching
        .header("X-DNS-Prefetch-Control", "off")

        // Prevent download of potentially malicious content
        .header("X-Download-Options", "noopen")

        .json(&serde_json::json!({
            "message": "Secure response with comprehensive headers",
            "security_level": "high"
        }))?
        .build();

    Ok(response)
}

// Environment-specific security headers
async fn environment_security_headers(is_production: bool) -> Result<Response> {
    let mut builder = ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "SAMEORIGIN");

    if is_production {
        // Production security headers
        builder = builder
            .header("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload")
            .header("Content-Security-Policy", "default-src 'self'; script-src 'self'")
            .header("X-Environment", "production");
    } else {
        // Development headers (more permissive)
        builder = builder
            .header("Content-Security-Policy", "default-src 'self' 'unsafe-inline' 'unsafe-eval'")
            .header("X-Environment", "development")
            .header("X-Debug-Mode", "enabled");
    }

    Ok(builder
        .json(&serde_json::json!({
            "environment": if is_production { "production" } else { "development" },
            "security_applied": true
        }))?
        .build())
}
```

## Cookie Handling

### Basic Cookie Management

```rust
use ignitia::{Cookie, CookieJar, SameSite};
use std::time::{Duration, SystemTime};

async fn cookie_basics() -> Result<Response> {
    // Create a simple cookie
    let session_cookie = Cookie::new("session_id", "sess_abc123def456")
        .path("/")
        .http_only()  // Prevent JavaScript access
        .secure()     // HTTPS only
        .same_site(SameSite::Strict); // CSRF protection

    // Create a persistent cookie
    let remember_cookie = Cookie::new("remember_token", "rem_xyz789")
        .path("/")
        .max_age(Duration::from_secs(30 * 24 * 3600)) // 30 days
        .http_only()
        .secure()
        .same_site(SameSite::Lax);

    // Create a preference cookie
    let theme_cookie = Cookie::new("theme", "dark")
        .path("/")
        .max_age(Duration::from_secs(365 * 24 * 3600)) // 1 year
        .same_site(SameSite::Lax); // Allow some cross-site usage

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .cookie(session_cookie)
        .cookie(remember_cookie)
        .cookie(theme_cookie)
        .json(&serde_json::json!({
            "message": "Cookies set successfully",
            "session_active": true
        }))?
        .build();

    Ok(response)
}

// Advanced cookie configuration
async fn advanced_cookies(domain: &str, is_secure: bool) -> Result<Response> {
    // Authentication cookie with domain and security settings
    let auth_cookie = Cookie::build("auth_token", generate_jwt_token())
        .domain(domain)
        .path("/")
        .max_age(Duration::from_secs(24 * 3600)) // 24 hours
        .http_only()
        .secure(is_secure)
        .same_site(if is_secure { SameSite::Strict } else { SameSite::Lax })
        .finish();

    // User preferences with longer expiration
    let preferences = serde_json::json!({
        "language": "en",
        "timezone": "UTC",
        "notifications": true
    });
    let prefs_cookie = Cookie::build("user_prefs", serde_json::to_string(&preferences).unwrap())
        .path("/")
        .max_age(Duration::from_secs(365 * 24 * 3600)) // 1 year
        .same_site(SameSite::Lax)
        .finish();

    // CSRF protection token
    let csrf_token = generate_csrf_token();
    let csrf_cookie = Cookie::build("csrf_token", csrf_token.clone())
        .path("/")
        .http_only()
        .secure(is_secure)
        .same_site(SameSite::Strict)
        .finish();

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .cookie(auth_cookie)
        .cookie(prefs_cookie)
        .cookie(csrf_cookie)
        .header("X-CSRF-Token", csrf_token) // Also provide as header
        .json(&serde_json::json!({
            "authenticated": true,
            "csrf_token": csrf_token,
            "domain": domain
        }))?
        .build();

    Ok(response)
}
```

### Cookie Jar Management

```rust
// Managing multiple cookies with CookieJar
async fn cookie_jar_management() -> Result<Response> {
    let mut jar = CookieJar::new();

    // Add multiple cookies to the jar
    jar.add(Cookie::new("session_id", "sess_123"));
    jar.add(Cookie::new("user_id", "user_456"));
    jar.add(Cookie::new("preferences", "theme=dark,lang=en"));

    // Add secure cookies
    jar.add(Cookie::build("secure_token", "token_789")
        .secure()
        .http_only()
        .same_site(SameSite::Strict)
        .finish());

    // Convert jar to headers for response
    let mut response_builder = ResponseBuilder::new().status(StatusCode::OK);

    for cookie in jar.iter() {
        response_builder = response_builder.cookie(cookie.clone());
    }

    let response = response_builder
        .json(&serde_json::json!({
            "message": "Multiple cookies set via jar",
            "cookie_count": jar.iter().count()
        }))?
        .build();

    Ok(response)
}

// Cookie removal and cleanup
async fn logout_cookies() -> Result<Response> {
    // Create expired cookies to remove them
    let session_removal = Cookie::build("session_id", "")
        .path("/")
        .max_age(Duration::from_secs(0)) // Immediate expiration
        .finish();

    let auth_removal = Cookie::build("auth_token", "")
        .path("/")
        .expires(SystemTime::UNIX_EPOCH) // Past expiration
        .finish();

    let remember_removal = Cookie::build("remember_token", "")
        .path("/")
        .max_age(Duration::from_secs(0))
        .finish();

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .cookie(session_removal)
        .cookie(auth_removal)
        .cookie(remember_removal)
        .header("Clear-Site-Data", r#""cookies", "storage""#) // Modern browser cleanup
        .json(&serde_json::json!({
            "message": "Logged out successfully",
            "cookies_cleared": true
        }))?
        .build();

    Ok(response)
}
```

### Secure Cookie Patterns

```rust
// GDPR-compliant cookie handling
async fn gdpr_compliant_cookies(user_consent: bool) -> Result<Response> {
    let mut builder = ResponseBuilder::new().status(StatusCode::OK);

    // Essential cookies (always set)
    let essential_cookie = Cookie::build("essential_session", generate_session_id())
        .path("/")
        .http_only()
        .secure()
        .same_site(SameSite::Strict)
        .finish();

    builder = builder.cookie(essential_cookie);

    // Optional cookies (only with consent)
    if user_consent {
        let analytics_cookie = Cookie::build("analytics_id", generate_analytics_id())
            .path("/")
            .max_age(Duration::from_secs(365 * 24 * 3600)) // 1 year
            .same_site(SameSite::Lax)
            .finish();

        let marketing_cookie = Cookie::build("marketing_prefs", "enabled")
            .path("/")
            .max_age(Duration::from_secs(90 * 24 * 3600)) // 90 days
            .same_site(SameSite::Lax)
            .finish();

        builder = builder
            .cookie(analytics_cookie)
            .cookie(marketing_cookie);
    }

    let response = builder
        .json(&serde_json::json!({
            "message": "Cookie consent processed",
            "consent_given": user_consent,
            "essential_cookies": true,
            "optional_cookies": user_consent
        }))?
        .build();

    Ok(response)
}

// Session management with secure cookies
async fn secure_session_management(user_id: u64) -> Result<Response> {
    let session_data = SessionData {
        user_id,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        permissions: vec!["read", "write"],
    };

    // Encrypt session data
    let encrypted_session = encrypt_session_data(&session_data)?;

    // Create secure session cookie
    let session_cookie = Cookie::build("secure_session", encrypted_session)
        .path("/")
        .max_age(Duration::from_secs(24 * 3600)) // 24 hours
        .http_only()
        .secure()
        .same_site(SameSite::Strict)
        .finish();

    // Create CSRF token cookie
    let csrf_token = generate_csrf_token();
    let csrf_cookie = Cookie::build("csrf_token", csrf_token.clone())
        .path("/")
        .http_only()
        .secure()
        .same_site(SameSite::Strict)
        .finish();

    let response = ResponseBuilder::new()
        .status(StatusCode::OK)
        .cookie(session_cookie)
        .cookie(csrf_cookie)
        .header("X-CSRF-Token", csrf_token)
        .header("X-Session-Expires", session_data.expires_at.to_rfc3339())
        .json(&serde_json::json!({
            "session_created": true,
            "user_id": user_id,
            "expires_at": session_data.expires_at,
            "csrf_token": csrf_token
        }))?
        .build();

    Ok(response)
}

#[derive(Debug, Serialize)]
struct SessionData {
    user_id: u64,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    permissions: Vec<&'static str>,
}
```

## Error Responses

### Structured Error Handling

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct ApiError {
    error: ErrorDetails,
    request_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Debug)]
struct ErrorDetails {
    code: String,
    message: String,
    details: Option<serde_json::Value>,
    help: Option<String>,
}

impl ApiError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetails {
                code: code.into(),
                message: message.into(),
                details: None,
                help: None,
            },
            request_id: generate_request_id(),
            timestamp: chrono::Utc::now(),
        }
    }

    fn with_details(mut self, details: serde_json::Value) -> Self {
        self.error.details = Some(details);
        self
    }

    fn with_help(mut self, help: impl Into<String>) -> Self {
        self.error.help = Some(help.into());
        self
    }
}

// Standard error responses
async fn error_response_examples() -> Result<Response> {
    // Validation error
    let validation_error = ApiError::new(
        "VALIDATION_FAILED",
        "The request contains invalid data"
    ).with_details(serde_json::json!({
        "fields": {
            "email": ["Email is required", "Email format is invalid"],
            "password": ["Password must be at least 8 characters"],
            "age": ["Age must be a positive number"]
        }
    })).with_help("Please correct the highlighted fields and try again");

    let validation_response = ResponseBuilder::new()
        .status(StatusCode::UNPROCESSABLE_ENTITY)
        .header("Content-Type", "application/json")
        .header("X-Error-Type", "validation")
        .json(&validation_error)?
        .build();

    // Authentication error
    let auth_error = ApiError::new(
        "AUTHENTICATION_REQUIRED",
        "Valid authentication credentials are required"
    ).with_help("Please provide a valid API key or authentication token");

    let auth_response = ResponseBuilder::new()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Bearer realm=\"API\"")
        .json(&auth_error)?
        .build();

    // Authorization error
    let authz_error = ApiError::new(
        "INSUFFICIENT_PERMISSIONS",
        "You do not have permission to access this resource"
    ).with_details(serde_json::json!({
        "required_permission": "admin",
        "current_permissions": ["user", "read_only"]
    }));

    let authz_response = ResponseBuilder::new()
        .status(StatusCode::FORBIDDEN)
        .json(&authz_error)?
        .build();

    // Rate limiting error
    let rate_limit_error = ApiError::new(
        "RATE_LIMIT_EXCEEDED",
        "Too many requests. Please slow down."
    ).with_details(serde_json::json!({
        "limit": 1000,
        "remaining": 0,
        "reset_time": chrono::Utc::now() + chrono::Duration::hours(1)
    })).with_help("Wait before making additional requests or upgrade your plan");

    let rate_limit_response = ResponseBuilder::new()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("Retry-After", "3600")
        .header("X-Rate-Limit-Limit", "1000")
        .header("X-Rate-Limit-Remaining", "0")
        .header("X-Rate-Limit-Reset", "1640995200")
        .json(&rate_limit_error)?
        .build();

    Ok(validation_response)
}

// Business logic errors
async fn business_logic_errors() -> Result<Response> {
    // Resource conflict
    let conflict_error = ApiError::new(
        "RESOURCE_CONFLICT",
        "The resource cannot be updated due to a conflict"
    ).with_details(serde_json::json!({
        "conflict_type": "version_mismatch",
        "current_version": 5,
        "provided_version": 3,
        "last_modified_by": "user@example.com",
        "last_modified_at": "2023-01-15T10:30:00Z"
    })).with_help("Fetch the latest version and retry your update");

    let conflict_response = ResponseBuilder::new()
        .status(StatusCode::CONFLICT)
        .header("ETag", r#""version-5""#)
        .json(&conflict_error)?
        .build();

    // Resource gone
    let gone_error = ApiError::new(
        "RESOURCE_PERMANENTLY_DELETED",
        "This resource has been permanently deleted and cannot be recovered"
    ).with_details(serde_json::json!({
        "deleted_at": "2023-01-01T00:00:00Z",
        "deleted_by": "admin@example.com",
        "reason": "GDPR_REQUEST"
    }));

    let gone_response = ResponseBuilder::new()
        .status(StatusCode::GONE)
        .json(&gone_error)?
        .build();

    Ok(conflict_response)
}

// Server errors
async fn server_error_responses() -> Result<Response> {
    // Internal server error
    let internal_error = ApiError::new(
        "INTERNAL_SERVER_ERROR",
        "An unexpected error occurred while processing your request"
    ).with_details(serde_json::json!({
        "incident_id": "inc_123456789",
        "support_reference": "Please include this reference when contacting support"
    })).with_help("If this error persists, please contact our support team");

    let internal_response = ResponseBuilder::new()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("X-Incident-ID", "inc_123456789")
        .json(&internal_error)?
        .build();

    // Service unavailable
    let maintenance_error = ApiError::new(
        "SERVICE_TEMPORARILY_UNAVAILABLE",
        "The service is temporarily unavailable for maintenance"
    ).with_details(serde_json::json!({
        "maintenance_window": {
            "start": "2023-01-15T02:00:00Z",
            "end": "2023-01-15T04:00:00Z"
        },
        "estimated_downtime": "2 hours"
    })).with_help("Please try again after the maintenance window");

    let maintenance_response = ResponseBuilder::new()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header("Retry-After", "7200") // 2 hours
        .header("X-Maintenance-Mode", "true")
        .json(&maintenance_error)?
        .build();

    Ok(internal_response)
}
```

### Error Response Middleware Integration

```rust
// Custom error handler for automatic error response formatting
pub struct ErrorResponseMiddleware;

impl ErrorResponseMiddleware {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_error(error: ignitia::Error) -> Response {
        match error {
            ignitia::Error::ValidationError(details) => {
                let api_error = ApiError::new("VALIDATION_FAILED", "Request validation failed")
                    .with_details(serde_json::json!({ "validation_errors": details }));

                ResponseBuilder::new()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .json(&api_error)
                    .unwrap() // Safe because we control the JSON
                    .build()
            }

            ignitia::Error::NotFound(resource) => {
                let api_error = ApiError::new("RESOURCE_NOT_FOUND", format!("Resource not found: {}", resource));

                ResponseBuilder::new()
                    .status(StatusCode::NOT_FOUND)
                    .json(&api_error)
                    .unwrap()
                    .build()
            }

            ignitia::Error::Unauthorized => {
                let api_error = ApiError::new("AUTHENTICATION_REQUIRED", "Valid authentication is required");

                ResponseBuilder::new()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("WWW-Authenticate", "Bearer")
                    .json(&api_error)
                    .unwrap()
                    .build()
            }

            ignitia::Error::Forbidden(reason) => {
                let api_error = ApiError::new("ACCESS_DENIED", format!("Access denied: {}", reason));

                ResponseBuilder::new()
                    .status(StatusCode::FORBIDDEN)
                    .json(&api_error)
                    .unwrap()
                    .build()
            }

            _ => {
                let api_error = ApiError::new("INTERNAL_SERVER_ERROR", "An unexpected error occurred")
                    .with_details(serde_json::json!({
                        "incident_id": generate_incident_id(),
                        "timestamp": chrono::Utc::now()
                    }));

                ResponseBuilder::new()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(&api_error)
                    .unwrap()
                    .build()
            }
        }
    }
}

// Usage in router
fn build_router() -> Router {
    Router::new()
        .middleware(ErrorResponseMiddleware::new())
        .get("/api/users/:id", get_user_handler)
        .post("/api/users", create_user_handler)
}
```

## Advanced Response Patterns

### Content Negotiation

```rust
use ignitia::{Headers, Response, Result};

async fn content_negotiation(headers: Headers) -> Result<Response> {
    let accept_header = headers.get("accept").unwrap_or("application/json");

    let user_data = User {
        id: 123,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    match accept_header {
        accept if accept.contains("application/json") => {
            Response::json(&user_data)
        }

        accept if accept.contains("application/xml") => {
            let xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
                <user>
                    <id>{}</id>
                    <name>{}</name>
                    <email>{}</email>
                </user>"#,
                user_data.id, user_data.name, user_data.email
            );

            Ok(ResponseBuilder::new()
                .status(StatusCode::OK)
                .header("Content-Type", "application/xml; charset=utf-8")
                .body(xml)
                .build())
        }

        accept if accept.contains("text/csv") => {
            let csv = format!("id,name,email\n{},{},{}",
                            user_data.id, user_data.name, user_data.email);

            Ok(ResponseBuilder::new()
                .status(StatusCode::OK)
                .header("Content-Type", "text/csv; charset=utf-8")
                .header("Content-Disposition", r#"attachment; filename="user.csv""#)
                .body(csv)
                .build())
        }

        accept if accept.contains("text/html") => {
            let html = format!(
                r#"<!DOCTYPE html>
                <html>
                <head><title>User Profile</title></head>
                <body>
                    <h1>User Profile</h1>
                    <p><strong>ID:</strong> {}</p>
                    <p><strong>Name:</strong> {}</p>
                    <p><strong>Email:</strong> {}</p>
                </body>
                </html>"#,
                user_data.id, user_data.name, user_data.email
            );

            Ok(Response::html(html))
        }

        _ => {
            let error = ApiError::new("UNSUPPORTED_MEDIA_TYPE", "The requested media type is not supported")
                .with_details(serde_json::json!({
                    "supported_types": ["application/json", "application/xml", "text/csv", "text/html"],
                    "requested_type": accept_header
                }));

            Ok(ResponseBuilder::new()
                .status(StatusCode::NOT_ACCEPTABLE)
                .json(&error)?
                .build())
        }
    }
}

// Language negotiation
async fn language_negotiation(headers: Headers) -> Result<Response> {
    let accept_language = headers.get("accept-language").unwrap_or("en");

    let (language, content) = if accept_language.contains("es") {
        ("es", serde_json::json!({
            "message": "¡Hola mundo!",
            "description": "Una aplicación web rápida construida con Ignitia"
        }))
    } else if accept_language.contains("fr") {
        ("fr", serde_json::json!({
            "message": "Bonjour le monde!",
            "description": "Une application web rapide construite avec Ignitia"
        }))
    } else if accept_language.contains("de") {
        ("de", serde_json::json!({
            "message": "Hallo Welt!",
            "description": "Eine schnelle Webanwendung mit Ignitia gebaut"
        }))
    } else {
        ("en", serde_json::json!({
            "message": "Hello world!",
            "description": "A fast web application built with Ignitia"
        }))
    };

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Language", language)
        .header("Vary", "Accept-Language")
        .json(&content)?
        .build())
}
```

### Streaming and Chunked Responses

```rust
use bytes::Bytes;
use futures::stream::{self, StreamExt};

// Large dataset streaming
async fn stream_large_dataset() -> Result<Response> {
    // Simulate large dataset
    let total_records = 100_000;
    let batch_size = 1000;

    // Create NDJSON stream for large datasets
    let mut ndjson_lines = Vec::new();

    for batch in (0..total_records).step_by(batch_size) {
        let batch_end = (batch + batch_size).min(total_records);
        let batch_data: Vec<_> = (batch..batch_end)
            .map(|i| serde_json::json!({
                "id": i,
                "name": format!("Record {}", i),
                "timestamp": chrono::Utc::now(),
                "batch": batch / batch_size
            }))
            .collect();

        for item in batch_data {
            ndjson_lines.push(serde_json::to_string(&item).unwrap());
        }
    }

    let ndjson_content = ndjson_lines.join("\n");

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-ndjson")
        .header("Transfer-Encoding", "chunked")
        .header("X-Total-Records", total_records.to_string())
        .header("X-Batch-Size", batch_size.to_string())
        .body(ndjson_content)
        .build())
}

// Real-time data streaming (Server-Sent Events)
async fn server_sent_events() -> Result<Response> {
    // Initial connection message
    let initial_message = format!(
        "data: {}\n\n",
        serde_json::json!({
            "type": "connection",
            "message": "Connected to live data stream",
            "timestamp": chrono::Utc::now(),
            "client_id": generate_client_id()
        })
    );

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .header("Connection", "keep-alive")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "Cache-Control")
        .header("X-Accel-Buffering", "no") // Disable nginx buffering
        .body(initial_message)
        .build())
}

// Progress tracking response
async fn progress_response(job_id: &str, progress: f64) -> Result<Response> {
    let progress_data = serde_json::json!({
        "job_id": job_id,
        "progress": progress,
        "status": if progress < 100.0 { "in_progress" } else { "completed" },
        "estimated_remaining": if progress < 100.0 {
            Some(((100.0 - progress) / progress * 30.0) as u32) // Estimated seconds
        } else {
            None
        },
        "updated_at": chrono::Utc::now()
    });

    let status_code = if progress < 100.0 {
        StatusCode::ACCEPTED
    } else {
        StatusCode::OK
    };

    Ok(ResponseBuilder::new()
        .status(status_code)
        .header("X-Progress", format!("{:.1}%", progress))
        .header("X-Job-Status", if progress < 100.0 { "in_progress" } else { "completed" })
        .json(&progress_data)?
        .build())
}
```

### Response Caching and ETags

```rust
// ETag generation and validation
async fn etag_response(content: &str, if_none_match: Option<&str>) -> Result<Response> {
    // Generate ETag from content hash
    let etag = format!(r#""{}""#, calculate_content_hash(content));

    // Check if client has current version
    if let Some(client_etag) = if_none_match {
        if client_etag == etag || client_etag == "*" {
            return Ok(ResponseBuilder::new()
                .status(StatusCode::NOT_MODIFIED)
                .header("ETag", &etag)
                .header("Cache-Control", "public, max-age=3600")
                .build());
        }
    }

    // Return full content with ETag
    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("ETag", &etag)
        .header("Cache-Control", "public, max-age=3600")
        .header("Vary", "Accept-Encoding")
        .body(content.to_string())
        .build())
}

// Last-Modified response
async fn last_modified_response(
    content: &str,
    last_modified: chrono::DateTime<chrono::Utc>,
    if_modified_since: Option<chrono::DateTime<chrono::Utc>>
) -> Result<Response> {
    // Check if content has been modified
    if let Some(since) = if_modified_since {
        if last_modified <= since {
            return Ok(ResponseBuilder::new()
                .status(StatusCode::NOT_MODIFIED)
                .header("Last-Modified", last_modified.format("%a, %d %b %Y %H:%M:%S GMT"))
                .header("Cache-Control", "public, max-age=3600")
                .build());
        }
    }

    // Return full content
    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Last-Modified", last_modified.format("%a, %d %b %Y %H:%M:%S GMT"))
        .header("Cache-Control", "public, max-age=3600")
        .header("ETag", format!(r#""{}""#, calculate_content_hash(content)))
        .body(content.to_string())
        .build())
}

// Cache control strategies
async fn cache_control_strategies(resource_type: &str) -> Result<Response> {
    let content = "Resource content here";
    let etag = format!(r#""{}""#, calculate_content_hash(content));

    let (cache_control, vary) = match resource_type {
        "static" => (
            "public, max-age=31536000, immutable", // 1 year for static assets
            "Accept-Encoding"
        ),
        "api" => (
            "private, max-age=300, must-revalidate", // 5 minutes for API data
            "Accept-Encoding, Authorization"
        ),
        "dynamic" => (
            "private, no-cache, must-revalidate", // Always revalidate
            "Accept-Encoding, Accept-Language, Authorization"
        ),
        "public" => (
            "public, max-age=3600, s-maxage=86400", // 1 hour client, 1 day CDN
            "Accept-Encoding, Accept-Language"
        ),
        _ => (
            "no-cache, no-store, must-revalidate",
            "Accept-Encoding"
        )
    };

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Cache-Control", cache_control)
        .header("ETag", &etag)
        .header("Vary", vary)
        .header("X-Cache-Strategy", resource_type)
        .body(content)
        .build())
}
```

## Performance Optimization

### Response Compression

```rust
// Automatic compression based on content type and size
async fn compression_optimization(content: &str, accept_encoding: &str) -> Result<Response> {
    let content_size = content.len();

    // Only compress if content is large enough and client supports it
    if content_size > 1024 && accept_encoding.contains("gzip") {
        // In practice, use a proper compression library
        let compressed = compress_gzip(content.as_bytes())?;

        Ok(ResponseBuilder::new()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Content-Encoding", "gzip")
            .header("Content-Length", compressed.len().to_string())
            .header("Vary", "Accept-Encoding")
            .header("X-Original-Size", content_size.to_string())
            .header("X-Compressed-Size", compressed.len().to_string())
            .header("X-Compression-Ratio", format!("{:.1}", content_size as f64 / compressed.len() as f64))
            .body(compressed)
            .build())
    } else {
        Ok(ResponseBuilder::new()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Content-Length", content_size.to_string())
            .body(content.to_string())
            .build())
    }
}

// Memory-efficient response building
async fn memory_efficient_response(large_data: Vec<serde_json::Value>) -> Result<Response> {
    // Stream JSON array instead of loading everything into memory
    let mut json_lines = Vec::new();
    json_lines.push("[".to_string());

    for (i, item) in large_data.into_iter().enumerate() {
        if i > 0 {
            json_lines.push(",".to_string());
        }
        json_lines.push(serde_json::to_string(&item)?);
    }

    json_lines.push("]".to_string());
    let response_body = json_lines.join("");

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("X-Memory-Optimized", "true")
        .body(response_body)
        .build())
}
```

### Response Time Optimization

```rust
// Response time monitoring
async fn timed_response<F, T>(handler: F) -> Result<Response>
where
    F: std::future::Future<Output = Result<T>> + Send,
    T: serde::Serialize,
{
    let start = std::time::Instant::now();
    let result = handler.await?;
    let duration = start.elapsed();

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("X-Response-Time", format!("{}ms", duration.as_millis()))
        .header("X-Processing-Time", format!("{:.2}ms", duration.as_secs_f64() * 1000.0))
        .json(&result)?
        .build())
}

// Async response optimization
async fn optimized_async_response(id: u64) -> Result<Response> {
    let start = std::time::Instant::now();

    // Parallel data fetching
    let (user_future, posts_future, stats_future) = tokio::join!(
        fetch_user(id),
        fetch_user_posts(id),
        fetch_user_stats(id)
    );

    let user = user_future?;
    let posts = posts_future?;
    let stats = stats_future?;

    let response_data = serde_json::json!({
        "user": user,
        "posts": posts,
        "stats": stats,
        "meta": {
            "total_fetch_time_ms": start.elapsed().as_millis(),
            "parallel_requests": 3
        }
    });

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("X-Parallel-Requests", "3")
        .header("X-Total-Time", format!("{}ms", start.elapsed().as_millis()))
        .json(&response_data)?
        .build())
}
```

## Security Best Practices

### Secure Response Headers

```rust
// Production security headers
async fn secure_response(content: serde_json::Value) -> Result<Response> {
    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)

        // Content security
        .header("Content-Type", "application/json; charset=utf-8")
        .header("X-Content-Type-Options", "nosniff")

        // Frame protection
        .header("X-Frame-Options", "DENY")
        .header("Content-Security-Policy", "frame-ancestors 'none'")

        // HTTPS enforcement
        .header("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload")

        // XSS protection
        .header("X-XSS-Protection", "1; mode=block")
        .header("Content-Security-Policy",
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'")

        // Privacy protection
        .header("Referrer-Policy", "strict-origin-when-cross-origin")
        .header("Permissions-Policy", "geolocation=(), microphone=(), camera=()")

        // Cache control for sensitive data
        .header("Cache-Control", "private, no-cache, no-store, must-revalidate")
        .header("Pragma", "no-cache")
        .header("Expires", "0")

        // Information disclosure prevention
        .header("Server", "") // Remove server information
        .header("X-Powered-By", "") // Remove technology information

        .json(&content)?
        .build())
}

// Sensitive data response
async fn sensitive_data_response(user_data: serde_json::Value) -> Result<Response> {
    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)

        // Strict caching for sensitive data
        .header("Cache-Control", "private, no-cache, no-store, max-age=0, must-revalidate")
        .header("Pragma", "no-cache")
        .header("Expires", "Thu, 01 Jan 1970 00:00:00 GMT")

        // Content protection
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "DENY")

        // Prevent sensitive data logging
        .header("X-Robots-Tag", "noindex, nofollow, nosnippet, noarchive")

        // HTTPS only
        .header("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload")

        .json(&user_data)?
        .build())
}
```

### Input Sanitization in Responses

```rust
// Sanitize user-generated content in responses
async fn sanitized_response(user_input: &str) -> Result<Response> {
    // Sanitize HTML and script tags
    let sanitized_content = user_input
        .replace("<script", "&lt;script")
        .replace("</script>", "&lt;/script&gt;")
        .replace("<iframe", "&lt;iframe")
        .replace("javascript:", "")
        .replace("on", ""); // Remove event handlers

    let response_data = serde_json::json!({
        "content": sanitized_content,
        "sanitized": true,
        "original_length": user_input.len(),
        "sanitized_length": sanitized_content.len()
    });

    Ok(ResponseBuilder::new()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("X-Content-Sanitized", "true")
        .header("X-XSS-Protection", "1; mode=block")
        .json(&response_data)?
        .build())
}
```

## Testing Responses

### Response Testing Utilities

```rust
#[cfg(test)]
mod response_tests {
    use super::*;
    use ignitia::test::TestResponse;

    #[tokio::test]
    async fn test_json_response() {
        let response = Response::json(serde_json::json!({
            "message": "test",
            "status": "success"
        })).unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.header("content-type").unwrap(), "application/json");

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["message"], "test");
    }

    #[tokio::test]
    async fn test_error_response() {
        let error = ApiError::new("TEST_ERROR", "This is a test error");
        let response = ResponseBuilder::new()
            .status(StatusCode::BAD_REQUEST)
            .json(&error)
            .unwrap()
            .build();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: ApiError = response.json().await.unwrap();
        assert_eq!(body.error.code, "TEST_ERROR");
    }

    #[tokio::test]
    async fn test_security_headers() {
        let response = secure_response(serde_json::json!({"data": "test"})).await.unwrap();

        assert!(response.header("x-content-type-options").is_some());
        assert!(response.header("x-frame-options").is_some());
        assert!(response.header("strict-transport-security").is_some());
    }

    #[tokio::test]
    async fn test_caching_headers() {
        let response = cache_control_strategies("static").await.unwrap();

        let cache_control = response.header("cache-control").unwrap();
        assert!(cache_control.contains("max-age=31536000"));
        assert!(cache_control.contains("immutable"));
    }

    #[tokio::test]
    async fn test_compression_optimization() {
        let large_content = "x".repeat(2048); // 2KB content
        let response = compression_optimization(&large_content, "gzip").await.unwrap();

        assert_eq!(response.header("content-encoding").unwrap(), "gzip");
        assert!(response.header("x-original-size").is_some());
    }
}
```

## Production Examples

### Complete API Response Handler

```rust
use ignitia::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ProductionApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
    meta: ResponseMeta,
}

#[derive(Serialize)]
struct ResponseMeta {
    timestamp: chrono::DateTime<chrono::Utc>,
    request_id: String,
    version: String,
    processing_time_ms: u128,
    server_id: String,
}

async fn production_api_handler<T, F>(
    request_id: String,
    handler: F
) -> Result<Response>
where
    T: serde::Serialize,
    F: std::future::Future<Output = Result<T>>,
{
    let start = std::time::Instant::now();

    let response_data = match handler.await {
        Ok(data) => ProductionApiResponse {
            success: true,
            data: Some(data),
            error: None,
            meta: ResponseMeta {
                timestamp: chrono::Utc::now(),
                request_id: request_id.clone(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                processing_time_ms: start.elapsed().as_millis(),
                server_id: std::env::var("SERVER_ID").unwrap_or_else(|_| "unknown".to_string()),
            },
        },
        Err(err) => {
            let api_error = match err {
                ignitia::Error::ValidationError(details) => ApiError::new("VALIDATION_ERROR", "Validation failed")
                    .with_details(serde_json::json!({ "validation_errors": details })),
                ignitia::Error::NotFound(resource) => ApiError::new("NOT_FOUND", format!("Resource not found: {}", resource)),
                ignitia::Error::Unauthorized => ApiError::new("UNAUTHORIZED", "Authentication required"),
                ignitia::Error::Forbidden(reason) => ApiError::new("FORBIDDEN", format!("Access denied: {}", reason)),
                _ => ApiError::new("INTERNAL_ERROR", "An unexpected error occurred"),
            };

            ProductionApiResponse {
                success: false,
                data: None,
                error: Some(api_error),
                meta: ResponseMeta {
                    timestamp: chrono::Utc::now(),
                    request_id: request_id.clone(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    processing_time_ms: start.elapsed().as_millis(),
                    server_id: std::env::var("SERVER_ID").unwrap_or_else(|_| "unknown".to_string()),
                },
            }
        }
    };

    let status = if response_data.success { StatusCode::OK } else { StatusCode::INTERNAL_SERVER_ERROR };

    Ok(ResponseBuilder::new()
        .status(status)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("X-Request-ID", &request_id)
        .header("X-Response-Time", format!("{}ms", start.elapsed().as_millis()))
        .header("X-API-Version", env!("CARGO_PKG_VERSION"))
        .header("X-Server-ID", std::env::var("SERVER_ID").unwrap_or_else(|_| "unknown".to_string()))
        .header("Cache-Control", "private, no-cache")
        .json(&response_data)?
        .build())
}

// Usage example
async fn get_user_endpoint(Path(id): Path<u64>, headers: Headers) -> Result<Response> {
    let request_id = headers.get("x-request-id")
        .map(|s| s.to_string())
        .unwrap_or_else(|| generate_request_id());

    production_api_handler(request_id, async move {
        // Your business logic here
        let user = fetch_user(id).await?;
        Ok(user)
    }).await
}
```

## Best Practices Summary

### 1. Status Codes
- Use appropriate HTTP status codes for different scenarios
- Be consistent across your API
- Provide meaningful error messages with proper status codes

### 2. Headers
- Always set `Content-Type` headers correctly
- Use security headers in production
- Implement proper caching strategies
- Include API versioning headers

### 3. Content Negotiation
- Support multiple content types when appropriate
- Respect client `Accept` headers
- Provide clear error messages for unsupported types

### 4. Error Handling
- Use structured error responses
- Include helpful error codes and messages
- Provide request IDs for debugging
- Be consistent with error formats

### 5. Performance
- Use compression for large responses
- Implement proper caching headers
- Consider streaming for large datasets
- Monitor response times

### 6. Security
- Set security headers by default
- Sanitize user-generated content
- Use HTTPS-only headers in production
- Implement proper CORS headers

### 7. Testing
- Test response formats and status codes
- Validate headers are set correctly
- Test error scenarios thoroughly
- Use type-safe response testing utilities

This comprehensive guide covers all aspects of HTTP response handling in Ignitia, from basic response creation to advanced production patterns. The framework provides powerful, flexible tools for creating any type of HTTP response your application needs.

---

**🔥 Ready to build blazing-fast responses with Ignitia? Start creating powerful APIs today!**
