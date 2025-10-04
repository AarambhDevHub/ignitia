# Middleware Guide

Middleware in Ignitia provides a powerful way to process HTTP requests and responses in a composable, reusable manner. This guide covers everything you need to know about using and creating middleware in your Ignitia applications.

## Table of Contents

- [What is Middleware?](#what-is-middleware)
- [New Middleware System (v0.2.4+)](#new-middleware-system-v024)
- [Built-in Middleware](#built-in-middleware)
- [Using Middleware](#using-middleware)
- [Creating Custom Middleware](#creating-custom-middleware)
- [The from_fn Helper](#the-from_fn-helper)
- [Middleware Order](#middleware-order)
- [Best Practices](#best-practices)
- [Migration Guide](#migration-guide)
- [Examples](#examples)

## What is Middleware?

Middleware functions are executed in sequence for each request, allowing you to:
- Process requests before they reach handlers
- Modify responses before they're sent to clients
- Implement cross-cutting concerns like logging, authentication, and CORS
- Handle errors in a centralized way
- Short-circuit request processing when needed

## New Middleware System (v0.2.4+)

Starting with version 0.2.4, Ignitia adopts a new middleware pattern inspired by Axum's proven design. This provides better composability, clearer control flow, and improved performance.

### The New Middleware Trait

```rust
pub trait Middleware: Send + Sync {
    async fn handle(&self, req: Request, next: Next) -> Response;
}
```

**Key Changes from Previous Versions:**
- Single `handle` method instead of separate `before` and `after` methods
- Takes ownership of the `Request` instead of borrowing
- Returns a `Response` directly instead of a `Result`
- Explicit control flow through the `Next` type
- Inspired by Axum's middleware pattern for better ergonomics

### The Next Type

The `Next` type represents the rest of the middleware chain:

```rust
impl Next {
    /// Execute the rest of the middleware chain and the handler
    pub async fn run(self, req: Request) -> Response {
        // Executes remaining middleware and handler
    }
}
```

### Why This Design?

This pattern, inspired by Axum, provides several advantages:

1. **Explicit Control Flow**: You decide when to call `next.run()`
2. **Request Ownership**: Full control over request modification
3. **Better Error Handling**: Return responses directly for errors
4. **Easier Composition**: Middleware naturally composes
5. **Familiar Pattern**: Similar to Express.js and Axum middleware

## Built-in Middleware

Ignitia provides several built-in middleware components (all updated for the new API):

### 1. Logger Middleware

Logs incoming requests and outgoing responses.

```rust
use ignitia::{Router, middleware::LoggerMiddleware};

let router = Router::new()
    .middleware(LoggerMiddleware::new())
    .get("/", || async { "Hello World" });
```

**Output:**
```
[INFO] → GET /api/users
[INFO] ← 200 GET /api/users (12ms)
```

### 2. CORS Middleware

Handles Cross-Origin Resource Sharing (CORS) headers.

```rust
use ignitia::middleware::CorsMiddleware;
use ignitia::Method;

// Permissive CORS (development)
let cors = CorsMiddleware::new()
    .allowed_origins(&["*"])
    .allowed_methods(&[Method::GET, Method::POST])
    .build()?;

let router = Router::new()
    .middleware(cors)
    .get("/api/data", get_data);

// Secure CORS (production)
let cors = CorsMiddleware::new()
    .allowed_origins(&["https://myapp.com", "https://api.myapp.com"])
    .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allowed_headers(&["Content-Type", "Authorization"])
    .allow_credentials()
    .max_age(3600)
    .build()?;

let router = Router::new().middleware(cors);
```

### 3. Rate Limiting Middleware

Protects your API from abuse with configurable rate limiting.

```rust
use ignitia::middleware::RateLimitingMiddleware;
use std::time::Duration;

// 100 requests per minute
let rate_limiter = RateLimitingMiddleware::per_minute(100);

// Custom configuration
let rate_limiter = RateLimitingMiddleware::new(
    RateLimitConfig::new(1000, Duration::from_secs(3600)) // 1000 requests per hour
        .with_key_extractor(|req| {
            // Custom rate limiting key (e.g., by user ID)
            req.headers.get("x-user-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("anonymous")
                .to_string()
        })
        .with_burst(1.5) // Allow 50% burst capacity
);

let router = Router::new()
    .middleware(rate_limiter)
    .get("/api/{*}", api_handler);
```

### 4. Compression Middleware

Automatically compresses responses using Gzip or Brotli.

```rust
use ignitia::middleware::CompressionMiddleware;

// Default compression
let compression = CompressionMiddleware::new();

// Custom configuration
let compression = CompressionMiddleware::new()
    .with_threshold(1024) // Only compress responses > 1KB
    .with_level(CompressionLevel::Best)
    .with_content_types(&[
        "application/json",
        "text/html",
        "text/css",
        "application/javascript"
    ]);

let router = Router::new()
    .middleware(compression)
    .get("/", html_handler);
```

### 5. Security Middleware

Adds security headers and provides basic protection against common attacks.

```rust
use ignitia::middleware::SecurityMiddleware;

// Default security
let security = SecurityMiddleware::new();

// High security configuration
let security = SecurityMiddleware::high_security();

// Custom configuration
let security = SecurityMiddleware::new()
    .with_hsts_config(31536000, true, true) // 1 year HSTS
    .with_frame_options("DENY")
    .with_content_type_nosniff()
    .with_xss_protection();

let router = Router::new()
    .middleware(security)
    .get("/", secure_handler);
```

### 6. Request ID Middleware

Adds unique request IDs for tracing and debugging.

```rust
use ignitia::middleware::RequestIdMiddleware;

let request_id = RequestIdMiddleware::new()
    .with_generator(IdGenerator::NanoId { length: 16 })
    .with_header_name("x-request-id");

let router = Router::new()
    .middleware(request_id)
    .get("/", traced_handler);
```

### 7. Body Size Limit Middleware

Limits the size of request bodies.

```rust
use ignitia::middleware::BodySizeLimitMiddleware;

// 10MB limit
let body_limit = BodySizeLimitMiddleware::new(10 * 1024 * 1024);

let router = Router::new()
    .middleware(body_limit)
    .post("/upload", file_upload_handler);
```

## Using Middleware

### Global Middleware

Apply middleware to all routes:

```rust
let router = Router::new()
    .middleware(LoggerMiddleware::new())
    .middleware(SecurityMiddleware::new())
    .get("/", home_handler)
    .post("/api/data", create_data);
```

### Route-Specific Middleware

Apply middleware to specific routes using `LayeredHandler`:

```rust
use ignitia::LayeredHandler;

let protected_handler = LayeredHandler::new(admin_handler)
    .layer(auth_middleware);

let router = Router::new()
    .route_with_layered("/admin", Method::GET, protected_handler)
    .get("/public", public_handler);
```

### Nested Router Middleware

Middleware applied to nested routers affects all routes within that router:

```rust
let api_router = Router::new()
    .middleware(RateLimitingMiddleware::per_minute(1000))
    .middleware(CorsMiddleware::default())
    .get("/users", get_users)
    .post("/users", create_user);

let main_router = Router::new()
    .middleware(LoggerMiddleware::new())
    .nest("/api/v1", api_router)
    .get("/", home_handler);
```

## Creating Custom Middleware

### Basic Custom Middleware

```rust
use ignitia::{Middleware, Request, Response, Next};

pub struct CustomHeaderMiddleware {
    header_value: String,
}

impl CustomHeaderMiddleware {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            header_value: value.into(),
        }
    }
}

impl Middleware for CustomHeaderMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Process request
        tracing::info!("Processing request to: {}", req.uri.path());

        // Call the next middleware/handler
        let mut response = next.run(req).await;

        // Modify response
        response.headers.insert(
            "X-Custom-Header",
            self.header_value.parse().unwrap()
        );

        response
    }
}
```

### Middleware That Short-Circuits

```rust
use ignitia::{Middleware, Request, Response, Next, StatusCode};

pub struct ApiKeyMiddleware {
    api_key: String,
}

impl Middleware for ApiKeyMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Check for API key
        let key = req.headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok());

        match key {
            Some(k) if k == self.api_key => {
                // Valid key, continue
                next.run(req).await
            }
            _ => {
                // Invalid or missing key, return error
                Response::new()
                    .with_status(StatusCode::UNAUTHORIZED)
                    .with_body("Invalid API key")
            }
        }
    }
}
```

### Advanced Custom Middleware with Timing

```rust
use std::time::Instant;
use ignitia::{Middleware, Request, Response, Next};

pub struct TimingMiddleware {
    log_slow_requests: bool,
    slow_threshold_ms: u64,
}

impl TimingMiddleware {
    pub fn new() -> Self {
        Self {
            log_slow_requests: true,
            slow_threshold_ms: 500,
        }
    }

    pub fn with_threshold(mut self, ms: u64) -> Self {
        self.slow_threshold_ms = ms;
        self
    }
}

impl Middleware for TimingMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let start = Instant::now();
        let method = req.method.clone();
        let path = req.uri.path().to_string();

        // Execute the rest of the chain
        let mut response = next.run(req).await;

        let duration = start.elapsed();
        let duration_ms = duration.as_millis();

        // Add timing header
        response.headers.insert(
            "X-Response-Time",
            format!("{}ms", duration_ms).parse().unwrap()
        );

        // Log slow requests
        if self.log_slow_requests && duration_ms > self.slow_threshold_ms as u128 {
            tracing::warn!(
                "Slow request: {} {} took {}ms",
                method,
                path,
                duration_ms
            );
        }

        response
    }
}
```

### Middleware with State Sharing

```rust
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use ignitia::{Middleware, Request, Response, Next};

#[derive(Clone)]
pub struct RequestCounterMiddleware {
    counter: Arc<AtomicUsize>,
}

impl RequestCounterMiddleware {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_count(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

impl Middleware for RequestCounterMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        tracing::info!("Request #{}", count);

        let mut response = next.run(req).await;

        // Add count to response header
        response.headers.insert(
            "X-Request-Count",
            count.to_string().parse().unwrap()
        );

        response
    }
}
```

## The from_fn Helper

The `from_fn` helper provides the most ergonomic way to create middleware from closures (inspired by Axum):

### Basic Usage

```rust
use ignitia::middleware::from_fn;

let logger = from_fn(|req, next| async move {
    println!("Request: {} {}", req.method, req.uri.path());
    next.run(req).await
});

let router = Router::new()
    .middleware(logger)
    .get("/", || async { "Hello" });
```

### Request and Response Processing

```rust
let timing = from_fn(|req, next| async move {
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();

    println!("Request took {:?}", duration);
    response
});
```

### Short-Circuit on Condition

```rust
let auth = from_fn(|req, next| async move {
    if let Some(token) = req.headers.get("Authorization") {
        if verify_token(token) {
            return next.run(req).await;
        }
    }

    Response::new().with_status(StatusCode::UNAUTHORIZED)
});
```

### With State Capture

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

let rate_limiter = Arc::new(Mutex::new(HashMap::new()));

let rate_limit = from_fn(move |req, next| {
    let limiter = rate_limiter.clone();
    async move {
        let ip = req.remote_addr.ip().to_string();
        let mut map = limiter.lock().await;

        let count = map.entry(ip).or_insert(0);
        *count += 1;

        if *count > 100 {
            return Response::new()
                .with_status(StatusCode::TOO_MANY_REQUESTS)
                .with_body("Rate limit exceeded");
        }

        drop(map);
        next.run(req).await
    }
});

router.middleware(rate_limit)
```

### Modifying Requests

```rust
let add_header = from_fn(|mut req, next| async move {
    req.headers.insert(
        "X-Custom-Header",
        "custom-value".parse().unwrap()
    );
    next.run(req).await
});
```

### Conditional Middleware

```rust
let conditional = from_fn(|req, next| async move {
    if req.uri.path().starts_with("/api") {
        // Apply special processing for API routes
        println!("API request");
    }
    next.run(req).await
});
```

## Middleware Order

Middleware execution follows a wrapper pattern (onion model):

```rust
let router = Router::new()
    .middleware(MiddlewareA) // Outer layer
    .middleware(MiddlewareB) // Middle layer
    .middleware(MiddlewareC) // Inner layer (closest to handler)
    .get("/", handler);
```

**Execution Flow:**
```rust
Request → A → B → C → Handler → C → B → A → Response
```

Each middleware can:
1. Process the request before calling `next.run()`
2. Execute the rest of the chain by calling `next.run()`
3. Process the response after `next.run()` returns

### Best Practices for Ordering

```rust
let router = Router::new()
    .middleware(RequestIdMiddleware::new())           // 1. Request tracing
    .middleware(LoggerMiddleware::new())              // 2. Logging
    .middleware(RateLimitingMiddleware::per_minute(1000)) // 3. Rate limiting
    .middleware(SecurityMiddleware::new())            // 4. Security headers
    .middleware(CorsMiddleware::default())            // 5. CORS
    .middleware(CompressionMiddleware::new())         // 6. Response compression
    .get("/", handler);
```

**Recommended Order:**
1. **Request ID/Tracing** - First, so all logs have request IDs
2. **Logging** - Early, to log all requests (even rejected ones)
3. **Rate Limiting** - Before expensive operations
4. **Authentication** - Early, to reject unauthorized requests
5. **Security Headers** - Can be anywhere
6. **CORS** - Handle preflight requests
7. **Compression** - Last, to compress final responses

## Best Practices

### 1. Keep Middleware Focused

Each middleware should have a single responsibility:

```rust
// Good: Focused middleware
pub struct RequestIdMiddleware { /* ... */ }
pub struct LoggingMiddleware { /* ... */ }

// Avoid: Kitchen-sink middleware
pub struct EverythingMiddleware { /* ... */ } // Does logging, auth, compression, etc.
```

### 2. Use from_fn for Simple Cases

For simple middleware, `from_fn` is more concise:

```rust
// Simple case - use from_fn
let simple = from_fn(|req, next| async move {
    println!("Request: {}", req.uri);
    next.run(req).await
});

// Complex case - implement Middleware trait
pub struct ComplexMiddleware {
    config: Arc<Config>,
    state: Arc<State>,
}
impl Middleware for ComplexMiddleware { /* ... */ }
```

### 3. Handle Errors Gracefully

```rust
let error_handler = from_fn(|req, next| async move {
    let response = next.run(req).await;

    if response.status.is_server_error() {
        tracing::error!("Server error: {}", response.status);
    }

    response
});
```

### 4. Use Extensions for Data Sharing

Share data between middleware and handlers:

```rust
#[derive(Clone)]
struct RequestMetadata {
    start_time: Instant,
    request_id: String,
}

// In middleware
let metadata_mw = from_fn(|mut req, next| async move {
    req.extensions.insert(RequestMetadata {
        start_time: Instant::now(),
        request_id: generate_request_id(),
    });
    next.run(req).await
});

// In handler
async fn handler(Extension(metadata): Extension<RequestMetadata>) -> impl IntoResponse {
    format!("Request ID: {}", metadata.request_id)
}
```

### 5. Performance Considerations

```rust
// Efficient: Share expensive resources
#[derive(Clone)]
struct MyMiddleware {
    config: Arc<Config>,  // Use Arc for shared state
}

// Avoid: Creating expensive resources per request
let bad = from_fn(|req, next| async move {
    let expensive_thing = ExpensiveResource::new(); // Don't do this!
    next.run(req).await
});
```

## Migration Guide

### From v0.2.3 and Earlier

**Old Middleware API:**
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
```

**New Middleware API (v0.2.4+):**
```rust
pub trait Middleware: Send + Sync {
    async fn handle(&self, req: Request, next: Next) -> Response;
}
```

### Migration Examples

**Example 1: Simple Logging**

Old:
```rust
#[async_trait::async_trait]
impl Middleware for LoggerMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        println!("Request: {} {}", req.method, req.uri);
        Ok(())
    }

    async fn after(&self, _req: &Request, res: &mut Response) -> Result<()> {
        println!("Response: {}", res.status);
        Ok(())
    }
}
```

New:
```rust
impl Middleware for LoggerMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        println!("Request: {} {}", req.method, req.uri);
        let response = next.run(req).await;
        println!("Response: {}", response.status);
        response
    }
}
```

**Example 2: Authentication**

Old:
```rust
#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        if !self.is_authenticated(req) {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}
```

New:
```rust
impl Middleware for AuthMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        if !self.is_authenticated(&req) {
            return Response::new()
                .with_status(StatusCode::UNAUTHORIZED);
        }
        next.run(req).await
    }
}
```

**Example 3: Response Modification**

Old:
```rust
#[async_trait::async_trait]
impl Middleware for HeaderMiddleware {
    async fn after(&self, _req: &Request, res: &mut Response) -> Result<()> {
        res.headers.insert("X-Custom", "value".parse().unwrap());
        Ok(())
    }
}
```

New:
```rust
impl Middleware for HeaderMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let mut response = next.run(req).await;
        response.headers.insert("X-Custom", "value".parse().unwrap());
        response
    }
}
```

## Examples

### Complete Application with Middleware Stack

```rust
use ignitia::prelude::*;
use ignitia::middleware::from_fn;

#[tokio::main]
async fn main() -> Result<()> {
    // Create comprehensive middleware stack
    let router = Router::new()
        // Request tracing
        .middleware(RequestIdMiddleware::new())

        // Logging
        .middleware(from_fn(|req, next| async move {
            let method = req.method.clone();
            let path = req.uri.path().to_string();
            let start = std::time::Instant::now();

            println!("→ {} {}", method, path);
            let response = next.run(req).await;

            let duration = start.elapsed();
            println!("← {} {} {} ({:?})", response.status, method, path, duration);
            response
        }))

        // Rate limiting
        .middleware(RateLimitingMiddleware::per_minute(1000))

        // Security headers
        .middleware(SecurityMiddleware::new())

        // CORS
        .middleware(
            CorsMiddleware::new()
                .allowed_origins(&["https://myapp.com"])
                .allow_credentials()
                .build()?
        )

        // Compression
        .middleware(CompressionMiddleware::new())

        // Routes
        .get("/", home_handler)
        .get("/health", health_check)
        .nest("/api", api_routes());

    let addr = "127.0.0.1:3000".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn home_handler() -> impl IntoResponse {
    Response::json(serde_json::json!({
        "message": "Welcome to Ignitia!",
        "version": "0.2.4"
    }))
}

async fn health_check() -> &'static str {
    "OK"
}

fn api_routes() -> Router {
    Router::new()
        .middleware(from_fn(|req, next| async move {
            // API-specific middleware
            if !req.headers.contains_key("X-API-Key") {
                return Response::new()
                    .with_status(StatusCode::UNAUTHORIZED);
            }
            next.run(req).await
        }))
        .get("/users", list_users)
        .post("/users", create_user)
}

async fn list_users() -> impl IntoResponse {
    Response::json(vec!["user1", "user2"])
}

async fn create_user(Json(user): Json<User>) -> impl IntoResponse {
    Response::json(user).with_status(StatusCode::CREATED)
}
```

### Custom Authentication Middleware

```rust
use ignitia::{Middleware, Request, Response, Next, StatusCode};
use jsonwebtoken::{decode, DecodingKey, Validation};

pub struct JwtAuthMiddleware {
    secret: String,
    exclude_paths: Vec<String>,
}

impl JwtAuthMiddleware {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            exclude_paths: Vec::new(),
        }
    }

    pub fn exclude_path(mut self, path: impl Into<String>) -> Self {
        self.exclude_paths.push(path.into());
        self
    }

    fn is_excluded(&self, path: &str) -> bool {
        self.exclude_paths.iter().any(|p| path.starts_with(p))
    }
}

impl Middleware for JwtAuthMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        // Skip authentication for excluded paths
        if self.is_excluded(req.uri.path()) {
            return next.run(req).await;
        }

        // Extract token from Authorization header
        let token = match req.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
        {
            Some(t) => t,
            None => {
                return Response::json(serde_json::json!({
                    "error": "Missing authorization token"
                }))
                .with_status(StatusCode::UNAUTHORIZED);
            }
        };

        // Verify token
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default()
        ) {
            Ok(token_data) => {
                // Add claims to request extensions
                req.extensions.insert(token_data.claims);
                next.run(req).await
            }
            Err(_) => {
                Response::json(serde_json::json!({
                    "error": "Invalid token"
                }))
                .with_status(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
```

---

This comprehensive guide covers the new Axum-inspired middleware system in Ignitia v0.2.4+. The framework provides a powerful, flexible, and ergonomic middleware system that enables you to build robust, secure, and performant web applications with clean separation of concerns.
