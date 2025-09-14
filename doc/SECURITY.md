# Security Guide

This document provides comprehensive security guidance for applications built with the Ignitia web framework. Ignitia is designed with security-first principles, leveraging Rust's memory safety guarantees while providing additional security features for web applications.

## Table of Contents

- [Security Philosophy](#security-philosophy)
- [Built-in Security Features](#built-in-security-features)
- [Security Middleware](#security-middleware)
- [TLS/HTTPS Configuration](#tlshttps-configuration)
- [Request Security](#request-security)
- [Authentication & Authorization](#authentication--authorization)
- [Common Vulnerabilities & Mitigations](#common-vulnerabilities--mitigations)
- [Security Best Practices](#security-best-practices)
- [Production Security Checklist](#production-security-checklist)
- [Security Testing](#security-testing)
- [Reporting Security Issues](#reporting-security-issues)

## Security Philosophy

Ignitia follows a **"secure by default"** philosophy:

- **Memory Safety**: Built on Rust, eliminating entire classes of vulnerabilities like buffer overflows, use-after-free, and data races
- **Type Safety**: Strong typing prevents many runtime errors and injection attacks
- **Explicit Error Handling**: No silent failures that could mask security issues
- **Minimal Attack Surface**: Only essential features are enabled by default
- **Defense in Depth**: Multiple layers of security controls

## Built-in Security Features

### Memory Safety
Rust's ownership model provides automatic memory safety without garbage collection:
- No buffer overflows
- No use-after-free vulnerabilities
- No data races in concurrent code
- No null pointer dereferences

### Request Size Limits
```rust
// Automatic request body size limiting (default: 10MB)
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

// Configurable in middleware
let app = Router::new()
    .middleware(BodyLimitMiddleware::new(5 * 1024 * 1024)) // 5MB limit
    .get("/", handler);
```

### Input Validation
Built-in extractors provide automatic validation:
```rust
use ignitia::{Json, Path, Query, Form};

// JSON validation
async fn create_user(Json(user): Json<CreateUserRequest>) -> Result<Response> {
    // Automatic deserialization with validation
    Ok(Response::json(&user)?)
}

// Path parameter validation
async fn get_user(Path(user_id): Path<u32>) -> Result<Response> {
    // Type-safe parameter extraction
    Ok(Response::text(format!("User: {}", user_id)))
}
```

### Error Information Disclosure Protection
```rust
impl From<Error> for Response {
    fn from(err: Error) -> Self {
        // Sanitized error responses in production
        let error_response = err.to_response(cfg!(debug_assertions));
        // Detailed errors only in debug mode
    }
}
```

## Security Middleware

### Security Headers Middleware
```rust
use ignitia::middleware::SecurityMiddleware;

let app = Router::new()
    .middleware(SecurityMiddleware::new()
        .with_hsts_config(31536000, true, true) // HSTS with preload
        .with_csp(CspConfig {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            object_src: vec!["'none'".to_string()],
            ..Default::default()
        }))
    .get("/", handler);
```

**Headers Added:**
- `Strict-Transport-Security`: Forces HTTPS connections
- `Content-Security-Policy`: Prevents XSS attacks
- `X-Frame-Options`: Prevents clickjacking
- `X-Content-Type-Options`: Prevents MIME sniffing
- `Referrer-Policy`: Controls referrer information
- `Permissions-Policy`: Restricts browser features

### CORS Middleware
```rust
use ignitia::middleware::Cors;

let cors = Cors::new()
    .allowed_origins(&["https://example.com", "https://app.example.com"])
    .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allowed_headers(&["Content-Type", "Authorization"])
    .allow_credentials()
    .build()?;

let app = Router::new()
    .middleware(cors)
    .get("/api/data", handler);
```

### Rate Limiting
```rust
use ignitia::middleware::RateLimitingMiddleware;

let rate_limiter = RateLimitingMiddleware::per_minute(100) // 100 requests per minute
    .with_key_extractor(|req| {
        // Custom rate limiting key (e.g., by user ID)
        req.header("authorization")
            .and_then(|auth| extract_user_id(auth))
            .unwrap_or_else(|| format!("ip:{}", get_client_ip(req)))
    })
    .with_burst(1.5); // Allow 50% burst capacity

let app = Router::new()
    .middleware(rate_limiter)
    .get("/api/data", handler);
```

### Request ID Middleware
```rust
use ignitia::middleware::RequestIdMiddleware;

let app = Router::new()
    .middleware(RequestIdMiddleware::new()
        .with_generator(IdGenerator::Uuid)
        .with_logging(true))
    .get("/", handler);
```

### Compression Security
```rust
use ignitia::middleware::CompressionMiddleware;

// Secure compression configuration
let compression = CompressionMiddleware::new()
    .with_threshold(1024) // Only compress responses > 1KB
    .with_compressible_types(vec![
        "application/json",
        "text/html",
        "text/css",
        "application/javascript"
    ]);

let app = Router::new()
    .middleware(compression)
    .get("/", handler);
```

## TLS/HTTPS Configuration

### Basic HTTPS Setup
```rust
use ignitia::{Server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .get("/", || async { Ok(Response::text("Secure Hello World!")) });

    let server = Server::new(app, "127.0.0.1:8443".parse()?)
        .enable_https("cert.pem", "key.pem")?;

    server.ignitia().await?;
    Ok(())
}
```

### Advanced TLS Configuration
```rust
use ignitia::server::{TlsConfig, TlsVersion};

let tls_config = TlsConfig::new("cert.pem", "key.pem")
    .with_alpn_protocols(vec!["h2", "http/1.1"])
    .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
    .enable_client_cert_verification();

let server = Server::new(app, "0.0.0.0:443".parse()?)
    .with_tls(tls_config)?;
```

### HTTP to HTTPS Redirect
```rust
let server = Server::new(app, "127.0.0.1:8443".parse()?)
    .enable_https("cert.pem", "key.pem")?
    .redirect_to_https(8443);

// HTTP server will automatically redirect to HTTPS
let http_server = Server::new(redirect_router, "127.0.0.1:8080".parse()?)
    .redirect_to_https(8443);
```

### Self-Signed Certificates (Development Only)
```rust
#[cfg(feature = "self-signed")]
let server = Server::new(app, "127.0.0.1:8443".parse()?)
    .with_self_signed_cert("localhost")?;
```

## Request Security

### Input Sanitization
```rust
use ignitia::{Json, Query, Form};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreatePost {
    #[serde(deserialize_with = "sanitize_html")]
    title: String,
    #[serde(deserialize_with = "sanitize_html")]
    content: String,
}

fn sanitize_html<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // Implement HTML sanitization logic
    Ok(sanitize(&s))
}
```

### File Upload Security
```rust
use ignitia::{Multipart, MultipartConfig};

async fn upload_handler(mut multipart: Multipart) -> Result<Response> {
    let config = MultipartConfig {
        max_file_size: 10 * 1024 * 1024, // 10MB max
        max_files: 5,
        allowed_extensions: vec!["jpg", "png", "pdf"],
        ..Default::default()
    };

    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            // Validate file type
            if let Some(filename) = field.file_name() {
                validate_filename(filename)?;
            }

            // Validate content type
            if let Some(content_type) = field.content_type() {
                validate_content_type(content_type)?;
            }

            // Save file securely
            let file = field.save_to_file(generate_secure_filename()).await?;
        }
    }

    Ok(Response::json(UploadResponse { success: true })?)
}

fn validate_filename(filename: &str) -> Result<()> {
    // Check for directory traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(Error::BadRequest("Invalid filename".into()));
    }

    // Check file extension
    let allowed_extensions = ["jpg", "jpeg", "png", "gif", "pdf", "txt"];
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();

    if !allowed_extensions.contains(&extension.as_str()) {
        return Err(Error::BadRequest("File type not allowed".into()));
    }

    Ok(())
}
```

### Cookie Security
```rust
use ignitia::{Cookie, SameSite, Response};

fn set_secure_cookie(mut response: Response) -> Response {
    let cookie = Cookie::new("session_id", generate_session_id())
        .path("/")
        .secure() // HTTPS only
        .http_only() // Not accessible via JavaScript
        .same_site(SameSite::Strict) // CSRF protection
        .max_age(3600); // 1 hour expiration

    response.add_cookie(cookie)
}
```

## Authentication & Authorization

### JWT Authentication Middleware
```rust
use ignitia::middleware::AuthMiddleware;

let auth = AuthMiddleware::new("your-secret-key")
    .protect_path("/api/admin")
    .protect_paths(vec!["/api/user", "/api/protected"]);

let app = Router::new()
    .middleware(auth)
    .get("/api/admin/users", admin_handler)
    .get("/api/user/profile", user_handler)
    .get("/public", public_handler);
```

### Custom Authentication
```rust
use ignitia::{Request, Response, Result, State};

#[derive(Clone)]
struct AuthState {
    secret: String,
}

async fn auth_middleware(req: &mut Request) -> Result<()> {
    let auth_header = req.header("authorization")
        .ok_or(Error::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(Error::Unauthorized);
    }

    let token = &auth_header[7..];
    let user = validate_jwt_token(token)?;

    // Store user in request extensions
    req.insert_extension(user);

    Ok(())
}

async fn protected_handler(user: Extension<User>) -> Result<Response> {
    Ok(Response::json(&*user)?)
}
```

### Session Management
```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone)]
struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    async fn create_session(&self, user_id: u32) -> String {
        let session_id = generate_secure_session_id();
        let session = Session {
            user_id,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
        };

        self.sessions.write().await.insert(session_id.clone(), session);
        session_id
    }

    async fn validate_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id)
            .filter(|session| session.expires_at > Utc::now())
            .cloned()
    }
}
```

## Common Vulnerabilities & Mitigations

### Cross-Site Scripting (XSS)
**Prevention:**
- Use Content Security Policy headers
- Sanitize all user input
- Use type-safe templating
- Escape output properly

```rust
// CSP headers automatically added by SecurityMiddleware
let security = SecurityMiddleware::new()
    .with_csp(CspConfig {
        default_src: vec!["'self'".to_string()],
        script_src: vec!["'self'".to_string()],
        ..Default::default()
    });
```

### Cross-Site Request Forgery (CSRF)
**Prevention:**
- Use SameSite cookies
- Implement CSRF tokens
- Validate Origin/Referer headers

```rust
// SameSite cookies
let cookie = Cookie::new("csrf_token", generate_csrf_token())
    .same_site(SameSite::Strict);

// CORS configuration
let cors = Cors::new()
    .allowed_origins(&["https://yourdomain.com"])
    .allow_credentials();
```

### SQL Injection
**Prevention:**
- Use parameterized queries
- Validate input types
- Use ORM/query builders

```rust
// Using sqlx with parameterized queries
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE id = $1",
    user_id
).fetch_one(&pool).await?;
```

### Path Traversal
**Prevention:**
- Validate file paths
- Use safe path manipulation
- Restrict file access

```rust
fn validate_path(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);

    // Check for directory traversal
    if path.components().any(|comp| comp == Component::ParentDir) {
        return Err(Error::BadRequest("Invalid path".into()));
    }

    // Ensure path is within allowed directory
    let canonical = path.canonicalize()
        .map_err(|_| Error::BadRequest("Invalid path".into()))?;

    if !canonical.starts_with("/allowed/uploads/") {
        return Err(Error::Forbidden);
    }

    Ok(canonical)
}
```

### Denial of Service (DoS)
**Prevention:**
- Rate limiting
- Request size limits
- Timeout configurations
- Connection limits

```rust
use ignitia::middleware::{RateLimitingMiddleware, BodyLimitMiddleware};

let app = Router::new()
    .middleware(RateLimitingMiddleware::per_minute(100))
    .middleware(BodyLimitMiddleware::new(1024 * 1024)) // 1MB limit
    .get("/", handler);
```

### Information Disclosure
**Prevention:**
- Sanitize error messages
- Remove debug information in production
- Secure logging practices

```rust
// Error handling that doesn't leak information
impl From<DatabaseError> for Error {
    fn from(_err: DatabaseError) -> Self {
        if cfg!(debug_assertions) {
            Error::Database(_err.to_string())
        } else {
            Error::Internal("Database operation failed".into())
        }
    }
}
```

## Security Best Practices

### 1. Secure Configuration
```rust
use ignitia::{Server, ServerConfig};

let config = ServerConfig::default()
    .with_tls(tls_config)
    .redirect_http_to_https(443);

let server = Server::new(app, addr)
    .with_config(config);
```

### 2. Environment-Based Security
```rust
fn create_security_middleware() -> SecurityMiddleware {
    if cfg!(debug_assertions) {
        SecurityMiddleware::for_development()
    } else {
        SecurityMiddleware::high_security()
    }
}
```

### 3. Secure Headers
```rust
let app = Router::new()
    .middleware(SecurityMiddleware::new()
        .with_hsts(true)
        .with_csp(csp_config)
        .with_security_headers(true))
    .get("/", handler);
```

### 4. Input Validation
```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, max = 50))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    user.validate()
        .map_err(|e| Error::Validation(e.to_string()))?;

    // Process validated user
    Ok(Response::json(&user)?)
}
```

### 5. Secure Logging
```rust
use tracing::{info, warn, error};

// Don't log sensitive information
info!("User login attempt for: {}", sanitize_username(&username));

// Use structured logging
info!(
    user_id = %user.id,
    action = "login",
    ip_address = %get_client_ip(&req),
    "User authentication successful"
);
```

### 6. WebSocket Security
```rust
use ignitia::websocket::{WebSocketConnection, Message};

async fn websocket_handler(ws: WebSocketConnection) -> Result<()> {
    // Authenticate WebSocket connection
    if !authenticate_websocket(&ws).await? {
        return ws.close_with_reason(1008, "Unauthorized".to_string()).await;
    }

    // Rate limit messages
    let mut message_count = 0;
    let mut last_reset = Instant::now();

    while let Some(message) = ws.recv().await {
        // Reset counter every minute
        if last_reset.elapsed() > Duration::from_secs(60) {
            message_count = 0;
            last_reset = Instant::now();
        }

        // Rate limit: 60 messages per minute
        message_count += 1;
        if message_count > 60 {
            ws.close_with_reason(1008, "Rate limit exceeded".to_string()).await?;
            break;
        }

        // Validate message size
        let message_size = match &message {
            Message::Text(text) => text.len(),
            Message::Binary(data) => data.len(),
            _ => 0,
        };

        if message_size > 1024 * 1024 { // 1MB limit
            ws.close_with_reason(1009, "Message too large".to_string()).await?;
            break;
        }

        // Process message
        handle_websocket_message(&ws, message).await?;
    }

    Ok(())
}
```

## Production Security Checklist

### Before Deployment

- [ ] **TLS/HTTPS Configuration**
  - [ ] Valid SSL/TLS certificates installed
  - [ ] Strong cipher suites configured
  - [ ] HTTP to HTTPS redirect enabled
  - [ ] HSTS headers configured

- [ ] **Security Headers**
  - [ ] Content Security Policy configured
  - [ ] X-Frame-Options set
  - [ ] X-Content-Type-Options set
  - [ ] Referrer-Policy configured

- [ ] **Authentication & Authorization**
  - [ ] Strong authentication mechanisms
  - [ ] Secure session management
  - [ ] Proper authorization checks
  - [ ] Secure password policies

- [ ] **Input Validation**
  - [ ] All input validated and sanitized
  - [ ] File upload restrictions
  - [ ] Request size limits
  - [ ] SQL injection protection

- [ ] **Rate Limiting**
  - [ ] API rate limits configured
  - [ ] DDoS protection enabled
  - [ ] Connection limits set

- [ ] **Logging & Monitoring**
  - [ ] Security events logged
  - [ ] Sensitive data not logged
  - [ ] Log analysis configured
  - [ ] Alerting configured

### Environment Configuration

```rust
// Production configuration
fn production_config() -> Router {
    Router::new()
        .middleware(SecurityMiddleware::high_security())
        .middleware(RateLimitingMiddleware::per_minute(60))
        .middleware(CorsMiddleware::secure_api(&["https://yourdomain.com"]))
        .middleware(RequestIdMiddleware::new())
        .middleware(ErrorHandlerMiddleware::new()
            .with_details(false) // Hide error details in production
            .with_stack_trace(false))
}
```

## Security Testing

### Unit Tests
```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[tokio::test]
    async fn test_xss_prevention() {
        let malicious_input = "<script>alert('xss')</script>";
        let sanitized = sanitize_html(malicious_input);
        assert!(!sanitized.contains("<script>"));
    }

    #[tokio::test]
    async fn test_path_traversal_prevention() {
        let malicious_path = "../../../etc/passwd";
        let result = validate_path(malicious_path);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        // Test rate limiting functionality
        let rate_limiter = RateLimitingMiddleware::per_minute(5);
        // ... test implementation
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_security_headers() {
    let app = create_test_app();
    let response = app.get("/").send().await;

    assert!(response.headers().contains_key("strict-transport-security"));
    assert!(response.headers().contains_key("content-security-policy"));
    assert!(response.headers().contains_key("x-frame-options"));
}
```

### Security Scanning
```bash
# Use cargo-audit to check for known vulnerabilities
cargo install cargo-audit
cargo audit

# Use clippy for security lints
cargo clippy -- -W clippy::all

# Use cargo-deny for license and security checks
cargo install cargo-deny
cargo deny check
```

## Reporting Security Issues

### Security Contact
<!--For security-related issues, please contact:
- **Email**: security@aaramabhdevhub.com
- **PGP Key**: [Link to PGP key]-->

### Reporting Guidelines
1. **Do not** create public GitHub issues for security vulnerabilities
2. Provide detailed information about the vulnerability
3. Include steps to reproduce the issue
4. Allow reasonable time for response (typically 90 days)

### What to Include
- Description of the vulnerability
- Affected versions
- Steps to reproduce
- Potential impact
- Suggested mitigation (if known)

### Response Process
1. **Acknowledgment**: Within 24 hours
2. **Initial Assessment**: Within 7 days
3. **Fix Development**: Timeline depends on severity
4. **Release**: Coordinated disclosure
5. **Public Disclosure**: After fix is available

---

> **Note**: Security is an ongoing process, not a one-time configuration. Regularly update dependencies, review security practices, and stay informed about new threats and best practices in web application security.
