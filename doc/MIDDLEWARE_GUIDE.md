# Middleware Guide

Middleware in Ignitia provides a powerful way to process HTTP requests and responses in a composable, reusable manner. This guide covers everything you need to know about using and creating middleware in your Ignitia applications.

## Table of Contents

- [What is Middleware?](#what-is-middleware)
- [Built-in Middleware](#built-in-middleware)
- [Using Middleware](#using-middleware)
- [Creating Custom Middleware](#creating-custom-middleware)
- [Middleware Order](#middleware-order)
- [Best Practices](#best-practices)
- [Examples](#examples)

## What is Middleware?

Middleware functions are executed in sequence for each request, allowing you to:
- Modify requests before they reach handlers
- Process responses before they're sent to clients
- Implement cross-cutting concerns like logging, authentication, and CORS
- Handle errors in a centralized way

In Ignitia, middleware implements the `Middleware` trait:

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

## Built-in Middleware

Ignitia provides several built-in middleware components for common use cases:

### 1. Logger Middleware

Logs incoming requests and outgoing responses.

```rust
use ignitia::{Router, middleware::LoggerMiddleware};

let router = Router::new()
    .middleware(LoggerMiddleware)
    .get("/", || async { Ok(Response::text("Hello World")) });
```

### 2. CORS Middleware

Handles Cross-Origin Resource Sharing (CORS) headers.

```rust
use ignitia::middleware::{Cors, CorsMiddleware};

// Permissive CORS (development)
let cors = Cors::permissive().build()?;
let router = Router::new()
    .middleware(cors)
    .get("/api/data", get_data);

// Secure CORS (production)
let cors = Cors::secure_api(&["https://myapp.com", "https://api.myapp.com"])
    .build()?;
let router = Router::new().middleware(cors);
```

### 3. Rate Limiting Middleware

Protects your API from abuse with configurable rate limiting.

```rust
use ignitia::middleware::RateLimitingMiddleware;

// 100 requests per minute
let rate_limiter = RateLimitingMiddleware::per_minute(100);

// Custom configuration
let rate_limiter = RateLimitingMiddleware::new(
    RateLimitConfig::new(1000, Duration::from_secs(3600)) // 1000 requests per hour
        .with_key_extractor(|req| {
            // Custom rate limiting key (e.g., by user ID)
            req.header("x-user-id").unwrap_or("anonymous").to_string()
        })
        .with_burst(1.5) // Allow 50% burst capacity
);

let router = Router::new()
    .middleware(rate_limiter)
    .get("/api/*", api_handler);
```

### 4. Authentication Middleware

Protects routes with token-based authentication.

```rust
use ignitia::middleware::AuthMiddleware;

let auth = AuthMiddleware::new("your-secret-token")
    .protect_path("/admin")
    .protect_paths(vec!["/api/private", "/dashboard"]);

let router = Router::new()
    .middleware(auth)
    .get("/public", public_handler)
    .get("/admin/users", admin_users_handler);
```

### 5. Compression Middleware

Automatically compresses responses using Gzip or Brotli.

```rust
use ignitia::middleware::CompressionMiddleware;

// Default compression
let compression = CompressionMiddleware::new();

// Custom configuration
let compression = CompressionMiddleware::new()
    .with_threshold(1024) // Only compress responses > 1KB
    .with_level(CompressionLevel::Best)
    .with_compressible_types(vec![
        "application/json",
        "text/html",
        "text/css",
        "application/javascript"
    ]);

let router = Router::new()
    .middleware(compression)
    .get("/", html_handler);
```

### 6. Security Middleware

Adds security headers and provides basic protection against common attacks.

```rust
use ignitia::middleware::SecurityMiddleware;

// Default security
let security = SecurityMiddleware::new();

// High security configuration
let security = SecurityMiddleware::high_security();

// Custom configuration
let security = SecurityMiddleware::new()
    .with_hsts_config(31536000, true, true) // 1 year HSTS with subdomains and preload
    .with_rate_limit(500, Duration::from_secs(60))
    .with_csp(CspConfig {
        default_src: vec!["'self'".to_string()],
        script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
        // ... other CSP directives
        ..Default::default()
    });

let router = Router::new()
    .middleware(security)
    .get("/", secure_handler);
```

### 7. Request ID Middleware

Adds unique request IDs for tracing and debugging.

```rust
use ignitia::middleware::RequestIdMiddleware;

let request_id = RequestIdMiddleware::new()
    .with_generator(IdGenerator::NanoId { length: 16 })
    .with_header_name("x-trace-id");

let router = Router::new()
    .middleware(request_id)
    .get("/", traced_handler);
```

### 8. Error Handler Middleware

Centralizes error handling and formatting.

```rust
use ignitia::middleware::ErrorHandlerMiddleware;

let error_handler = ErrorHandlerMiddleware::new()
    .with_details(true) // Include detailed error info in responses
    .with_logging(true)
    .with_json_format(ErrorFormat::Detailed);

let router = Router::new()
    .middleware(error_handler)
    .get("/", fallible_handler);
```

## Using Middleware

### Global Middleware

Apply middleware to all routes:

```rust
let router = Router::new()
    .middleware(LoggerMiddleware)
    .middleware(SecurityMiddleware::new())
    .get("/", home_handler)
    .post("/api/data", create_data);
```

### Route-Specific Middleware

Apply middleware to specific routes using `LayeredHandler`:

```rust
use ignitia::LayeredHandler;

let protected_handler = LayeredHandler::new(admin_handler)
    .layer(AuthMiddleware::new("admin-token"));

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
    .middleware(LoggerMiddleware)
    .nest("/api/v1", api_router)
    .get("/", home_handler);
```

## Creating Custom Middleware

### Basic Custom Middleware

```rust
use ignitia::{Middleware, Request, Response, Result};

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

#[async_trait::async_trait]
impl Middleware for CustomHeaderMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Process request before handler
        tracing::info!("Processing request to: {}", req.uri.path());
        Ok(())
    }

    async fn after(&self, _req: &Request, res: &mut Response) -> Result<()> {
        // Process response after handler
        res.headers.insert(
            "X-Custom-Header",
            self.header_value.parse().unwrap()
        );
        Ok(())
    }
}
```

### Advanced Custom Middleware with Configuration

```rust
use std::time::Duration;
use tokio::time::Instant;

pub struct TimingMiddleware {
    log_slow_requests: bool,
    slow_threshold: Duration,
}

impl TimingMiddleware {
    pub fn new() -> Self {
        Self {
            log_slow_requests: true,
            slow_threshold: Duration::from_millis(500),
        }
    }

    pub fn with_threshold(mut self, threshold: Duration) -> Self {
        self.slow_threshold = threshold;
        self
    }
}

#[async_trait::async_trait]
impl Middleware for TimingMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Store start time in request extensions
        req.insert_extension(Instant::now());
        Ok(())
    }

    async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        if let Some(start_time) = req.get_extension::<Instant>() {
            let duration = start_time.elapsed();

            // Add timing header
            res.headers.insert(
                "X-Response-Time",
                format!("{}ms", duration.as_millis()).parse().unwrap()
            );

            // Log slow requests
            if self.log_slow_requests && duration > self.slow_threshold {
                tracing::warn!(
                    "Slow request: {} {} took {}ms",
                    req.method,
                    req.uri.path(),
                    duration.as_millis()
                );
            }
        }
        Ok(())
    }
}
```

### Middleware with State

```rust
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

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

#[async_trait::async_trait]
impl Middleware for RequestCounterMiddleware {
    async fn before(&self, _req: &mut Request) -> Result<()> {
        let count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        tracing::info!("Request #{}", count);
        Ok(())
    }
}
```

## Middleware Order

Middleware execution follows a specific order:

1. **Before Phase**: Executed in the order middleware was added
2. **Handler**: Your route handler executes
3. **After Phase**: Executed in reverse order

```rust
let router = Router::new()
    .middleware(MiddlewareA) // Before: 1st, After: 3rd
    .middleware(MiddlewareB) // Before: 2nd, After: 2nd
    .middleware(MiddlewareC) // Before: 3rd, After: 1st
    .get("/", handler);
```

**Best Practices for Ordering:**

1. **Authentication/Authorization** - Early in the chain
2. **Request ID/Tracing** - Very early for full request tracing
3. **Rate Limiting** - Before expensive operations
4. **Compression** - Last in the chain for response processing
5. **Error Handling** - Can be anywhere, but often early or last

```rust
let router = Router::new()
    .middleware(RequestIdMiddleware::new())     // 1. Request tracing
    .middleware(LoggerMiddleware)              // 2. Logging
    .middleware(RateLimitingMiddleware::per_minute(1000)) // 3. Rate limiting
    .middleware(AuthMiddleware::new("token"))   // 4. Authentication
    .middleware(SecurityMiddleware::new())      // 5. Security headers
    .middleware(CorsMiddleware::default())      // 6. CORS
    .middleware(CompressionMiddleware::new())   // 7. Response compression
    .get("/", handler);
```

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

### 2. Use Configuration Builders

Provide fluent APIs for middleware configuration:

```rust
let middleware = MyMiddleware::new()
    .with_timeout(Duration::from_secs(30))
    .with_retry_count(3)
    .enable_caching();
```

### 3. Handle Errors Gracefully

```rust
#[async_trait::async_trait]
impl Middleware for MyMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Handle potential failures
        match self.validate_request(req) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("Request validation failed: {}", e);
                Err(Error::BadRequest("Invalid request".into()))
            }
        }
    }
}
```

### 4. Use Extensions for Data Sharing

Share data between middleware and handlers using extensions:

```rust
// In middleware
req.insert_extension(RequestMetadata {
    start_time: Instant::now(),
    user_id: extract_user_id(req)?,
});

// In handler
fn my_handler(Extension(metadata): Extension<RequestMetadata>) -> Result<Response> {
    // Use the metadata
    Ok(Response::text(format!("Hello user {}", metadata.user_id)))
}
```

### 5. Performance Considerations

- Minimize allocations in hot paths
- Use `Arc` for shared state
- Consider middleware overhead for high-traffic applications
- Profile your middleware stack

```rust
// Efficient: Reuse parsed headers
lazy_static! {
    static ref CORS_HEADERS: HeaderValue = "application/json".parse().unwrap();
}

// Less efficient: Parse on every request
let content_type = "application/json".parse().unwrap();
```

## Examples

### Complete Application with Middleware Stack

```rust
use ignitia::{
    Router, Server, Response,
    middleware::{
        LoggerMiddleware, SecurityMiddleware, CorsMiddleware,
        CompressionMiddleware, RateLimitingMiddleware
    }
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create comprehensive middleware stack
    let router = Router::new()
        // Request tracing and logging
        .middleware(RequestIdMiddleware::for_microservices())
        .middleware(LoggerMiddleware)

        // Security and rate limiting
        .middleware(RateLimitingMiddleware::per_minute(1000))
        .middleware(SecurityMiddleware::for_api())

        // CORS for API access
        .middleware(
            Cors::secure_api(&["https://myapp.com"])
                .allow_credentials()
                .build()?
        )

        // Response optimization
        .middleware(CompressionMiddleware::for_api())

        // Routes
        .get("/", home_handler)
        .get("/health", health_check)
        .nest("/api", api_routes());

    Server::new(router, "127.0.0.1:3000".parse()?)
        .ignitia()
        .await?;

    Ok(())
}

async fn home_handler() -> ignitia::Result<Response> {
    Ok(Response::json(serde_json::json!({
        "message": "Welcome to Ignitia!",
        "version": "1.0.0"
    }))?)
}

async fn health_check() -> ignitia::Result<Response> {
    Ok(Response::json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))?)
}

fn api_routes() -> Router {
    Router::new()
        .middleware(AuthMiddleware::new("api-key").protect_paths(vec!["/users"]))
        .get("/users", list_users)
        .post("/users", create_user)
}
```

This comprehensive guide covers all aspects of middleware in Ignitia. The framework provides a powerful and flexible middleware system that enables you to build robust, secure, and performant web applications with clean separation of concerns.
