# Ignite Web Framework Documentation

**Version:** 0.1.0
**Built by:** Aarambh Dev Hub
**Last Updated:** August 29, 2025

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation & Setup](#installation--setup)
3. [Core Concepts](#core-concepts)
4. [API Reference](#api-reference)
5. [Routing System](#routing-system)
6. [Middleware Architecture](#middleware-architecture)
7. [Cookie Management](#cookie-management)
8. [Request & Response Handling](#request--response-handling)
9. [Authentication & Authorization](#authentication--authorization)
10. [Error Handling](#error-handling)
11. [Testing](#testing)
12. [Advanced Topics](#advanced-topics)
13. [Best Practices](#best-practices)
14. [Troubleshooting](#troubleshooting)
15. [Migration Guide](#migration-guide)

---

## Introduction

Ignite is a blazing fast, lightweight web framework for Rust that embodies the spirit of **Aarambh** (new beginnings). Built on top of Hyper and Tokio, Ignite provides developers with a powerful yet simple toolkit for building modern web applications.

### Philosophy

Ignite follows these core principles:

- **Performance First**: Zero-copy operations and minimal allocations
- **Developer Experience**: Intuitive APIs and comprehensive error messages
- **Type Safety**: Leveraging Rust's type system for compile-time correctness
- **Security**: Built-in protection against common web vulnerabilities
- **Extensibility**: Composable middleware and modular architecture

### Architecture Overview

```
┌─────────────────┐
│   Application   │
├─────────────────┤
│   Middleware    │
├─────────────────┤
│     Router      │
├─────────────────┤
│     Server      │
├─────────────────┤
│ Hyper + Tokio   │
└─────────────────┘
```

---

## Installation & Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

### Adding Ignite to Your Project

```
[dependencies]
ignite = "0.1.0"
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing-subscriber = "0.3"

# Optional dependencies for specific features
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Project Structure

```
your-project/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   └── auth.rs
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── custom.rs
│   ├── models/
│   │   ├── mod.rs
│   │   └── user.rs
│   └── utils/
│       ├── mod.rs
│       └── validation.rs
├── static/
│   ├── css/
│   ├── js/
│   └── images/
└── templates/
    └── index.html
```

---

## Core Concepts

### Server

The `Server` struct is the entry point of your Ignite application:

```
use ignite::{Server, Router};

let server = Server::new(router, addr);
server.ignite().await?; // Start the server
```

### Router

The `Router` handles request routing and middleware application:

```
let router = Router::new()
    .middleware(LoggerMiddleware)
    .get("/", handler_fn(home))
    .post("/api/users", handler_fn(create_user));
```

### Handlers

Handlers are async functions that process requests and return responses:

```
async fn home(req: Request) -> Result<Response> {
    Ok(Response::html("<h1>Welcome to Ignite!</h1>"))
}
```

### Middleware

Middleware provides cross-cutting functionality like logging, authentication, and CORS:

```
#[async_trait]
impl Middleware for CustomMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Process request
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // Process response
        Ok(())
    }
}
```

---

## API Reference

### Server

#### `Server::new(router: Router, addr: SocketAddr) -> Server`

Creates a new server instance.

**Parameters:**
- `router`: The router containing routes and middleware
- `addr`: The socket address to bind to

**Example:**
```
let addr = "127.0.0.1:3000".parse().unwrap();
let server = Server::new(router, addr);
```

#### `async fn ignite(self) -> Result<(), Box<dyn std::error::Error>>`

Starts the server and begins listening for connections.

### Router

#### `Router::new() -> Router`

Creates a new router instance.

#### HTTP Method Handlers

- `get(path: &str, handler: HandlerFn) -> Router`
- `post(path: &str, handler: HandlerFn) -> Router`
- `put(path: &str, handler: HandlerFn) -> Router`
- `delete(path: &str, handler: HandlerFn) -> Router`
- `patch(path: &str, handler: HandlerFn) -> Router`
- `head(path: &str, handler: HandlerFn) -> Router`
- `options(path: &str, handler: HandlerFn) -> Router`

#### `middleware<M: Middleware + 'static>(mut self, middleware: M) -> Router`

Adds middleware to the router.

#### `not_found(handler: HandlerFn) -> Router`

Sets a custom 404 handler.

### Request

#### Properties

```
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub body: Bytes,
    pub params: HashMap<String, String>,
}
```

#### Methods

- `param(&self, key: &str) -> Option<&String>` - Get route parameter
- `query(&self, key: &str) -> Option<&String>` - Get query parameter
- `header(&self, key: &str) -> Option<&str>` - Get header value
- `json<T: DeserializeOwned>(&self) -> Result<T>` - Parse JSON body
- `cookies(&self) -> CookieJar` - Get all cookies
- `cookie(&self, key: &str) -> Option<String>` - Get specific cookie

### Response

#### Constructors

- `Response::text(content: impl Into<String>) -> Response`
- `Response::html(content: impl Into<String>) -> Response`
- `Response::json<T: Serialize>(data: T) -> Result<Response>`
- `Response::not_found() -> Response`
- `Response::redirect(location: impl Into<String>) -> Response`

#### Methods

- `add_cookie(self, cookie: Cookie) -> Self`
- `add_cookies(self, cookies: Vec<Cookie>) -> Self`
- `remove_cookie(self, name: impl Into<String>) -> Self`
- `status(mut self, status: StatusCode) -> Self`
- `header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self`

### Cookie

#### Constructor

```
Cookie::new(name: impl Into<String>, value: impl Into<String>) -> Cookie
```

#### Builder Methods

- `path(mut self, path: impl Into<String>) -> Self`
- `domain(mut self, domain: impl Into<String>) -> Self`
- `max_age(mut self, seconds: u64) -> Self`
- `expires(mut self, time: SystemTime) -> Self`
- `secure(mut self) -> Self`
- `http_only(mut self) -> Self`
- `same_site(mut self, same_site: SameSite) -> Self`

#### Static Methods

- `Cookie::removal(name: impl Into<String>) -> Cookie`

---

## Routing System

### Route Patterns

#### Static Routes

```
.get("/", handler_fn(home))
.get("/about", handler_fn(about))
.get("/contact", handler_fn(contact))
```

#### Parameter Routes

```
.get("/users/:id", handler_fn(get_user))
.get("/posts/:post_id/comments/:comment_id", handler_fn(get_comment))
```

#### Wildcard Routes

```
.get("/*path", handler_fn(serve_static))
.get("/api/v1/*endpoint", handler_fn(api_handler))
```

### Route Priority

Routes are matched in the order they are defined:

1. Exact static matches
2. Parameter matches
3. Wildcard matches

### Parameter Extraction

```
async fn get_user(req: Request) -> Result<Response> {
    let user_id: u32 = req.param("id")
        .ok_or_else(|| Error::BadRequest("Missing user ID".into()))?
        .parse()
        .map_err(|_| Error::BadRequest("Invalid user ID".into()))?;

    // Fetch user...
    Ok(Response::json(user)?)
}
```

### Query Parameters

```
async fn search(req: Request) -> Result<Response> {
    let query = req.query("q").cloned().unwrap_or_default();
    let page: usize = req.query("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);
    let limit: usize = req.query("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(10);

    // Perform search...
    Ok(Response::json(results)?)
}
```

---

## Middleware Architecture

### Built-in Middleware

#### LoggerMiddleware

Logs all incoming requests and outgoing responses.

```
.middleware(LoggerMiddleware)
```

Output format:
```
2025-08-29T10:30:00Z INFO GET /api/users 200 45ms
```

#### CorsMiddleware

Handles Cross-Origin Resource Sharing (CORS).

```
.middleware(CorsMiddleware::new()
    .allow_origin("https://example.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_headers(vec!["Content-Type", "Authorization"])
    .allow_credentials(true))
```

#### AuthMiddleware

Provides path-based authentication.

```
.middleware(AuthMiddleware::new("secret-token")
    .protect_path("/admin")
    .protect_path("/api/protected"))
```

### Custom Middleware

#### Basic Middleware

```
use ignite::{Middleware, async_trait, Request, Response, Result};

struct TimingMiddleware;

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let start_time = std::time::Instant::now();
        req.extensions.insert(start_time);
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // Add timing header
        res.headers.insert(
            "X-Response-Time",
            "42ms".parse().unwrap()
        );
        Ok(())
    }
}
```

#### Conditional Middleware

```
struct ConditionalMiddleware<F> {
    condition: F,
    middleware: Box<dyn Middleware>,
}

impl<F> ConditionalMiddleware<F>
where
    F: Fn(&Request) -> bool + Send + Sync,
{
    fn new<M: Middleware + 'static>(condition: F, middleware: M) -> Self {
        Self {
            condition,
            middleware: Box::new(middleware),
        }
    }
}

#[async_trait]
impl<F> Middleware for ConditionalMiddleware<F>
where
    F: Fn(&Request) -> bool + Send + Sync,
{
    async fn before(&self, req: &mut Request) -> Result<()> {
        if (self.condition)(req) {
            self.middleware.before(req).await
        } else {
            Ok(())
        }
    }
}
```

#### Rate Limiting Middleware

```
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct RateLimitMiddleware {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimitMiddleware {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    fn is_rate_limited(&self, client_ip: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        let client_requests = requests.entry(client_ip.to_string()).or_default();
        client_requests.retain(|&timestamp| now.duration_since(timestamp) < self.window);

        if client_requests.len() >= self.max_requests {
            return true;
        }

        client_requests.push(now);
        false
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let client_ip = "127.0.0.1"; // Extract from headers in real implementation

        if self.is_rate_limited(client_ip) {
            return Err(Error::TooManyRequests);
        }

        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        res.headers.insert(
            "X-RateLimit-Limit",
            self.max_requests.to_string().parse().unwrap()
        );
        Ok(())
    }
}
```

---

## Cookie Management

### Setting Cookies

#### Session Cookie

```
let session_cookie = Cookie::new("session_id", session_id)
    .path("/")
    .http_only();

Response::text("Session started").add_cookie(session_cookie)
```

#### Persistent Cookie

```
let remember_cookie = Cookie::new("remember_token", token)
    .path("/")
    .max_age(86400 * 30) // 30 days
    .http_only()
    .secure()
    .same_site(SameSite::Strict);

Response::text("Remember me set").add_cookie(remember_cookie)
```

#### Preference Cookie

```
let theme_cookie = Cookie::new("theme", "dark")
    .path("/")
    .max_age(86400 * 365); // 1 year

Response::text("Theme saved").add_cookie(theme_cookie)
```

### Reading Cookies

```
async fn dashboard(req: Request) -> Result<Response> {
    let cookies = req.cookies();

    // Check for session
    let session_id = cookies.get("session_id")
        .ok_or_else(|| Error::Unauthorized)?;

    // Get theme preference
    let theme = cookies.get("theme").unwrap_or(&"light".to_string());

    // Validate session
    if !is_valid_session(session_id) {
        return Err(Error::Unauthorized);
    }

    Ok(Response::html(format!(
        "<h1>Dashboard</h1><p>Theme: {}</p>",
        theme
    )))
}
```

### Cookie Security

#### Secure Cookie Configuration

```
// Production-ready secure cookie
let secure_cookie = Cookie::new("auth_token", token)
    .path("/")
    .max_age(3600)
    .http_only()          // Prevent XSS
    .secure()             // HTTPS only
    .same_site(SameSite::Strict); // CSRF protection
```

#### Cookie Validation

```
fn validate_cookie_value(value: &str) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::BadRequest("Empty cookie value".into()));
    }

    if value.len() > 4096 {
        return Err(Error::BadRequest("Cookie value too long".into()));
    }

    // Additional validation logic
    Ok(())
}
```

---

## Request & Response Handling

### Request Body Processing

#### JSON Parsing

```
#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    age: Option<u32>,
}

async fn create_user(req: Request) -> Result<Response> {
    let user_data: CreateUserRequest = req.json()
        .map_err(|_| Error::BadRequest("Invalid JSON".into()))?;

    // Validate data
    if user_data.name.is_empty() {
        return Err(Error::BadRequest("Name is required".into()));
    }

    // Create user...
    Ok(Response::json(created_user)?)
}
```

#### Form Data Processing

```
use url::form_urlencoded;

async fn handle_form(req: Request) -> Result<Response> {
    let body = String::from_utf8(req.body.to_vec())
        .map_err(|_| Error::BadRequest("Invalid UTF-8".into()))?;

    let form_data: HashMap<String, String> = form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();

    let username = form_data.get("username")
        .ok_or_else(|| Error::BadRequest("Username required".into()))?;

    // Process form data...
    Ok(Response::text("Form processed"))
}
```

### Response Generation

#### JSON Responses

```
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: u64,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

async fn api_handler(req: Request) -> Result<Response> {
    match process_request(req).await {
        Ok(data) => Response::json(ApiResponse::success(data)),
        Err(e) => Response::json(ApiResponse::<()>::error(e.to_string())),
    }
}
```

#### File Downloads

```
async fn download_file(req: Request) -> Result<Response> {
    let filename = req.param("filename")
        .ok_or_else(|| Error::NotFound)?;

    let file_path = PathBuf::from("uploads").join(filename);

    // Security check
    if !file_path.starts_with("uploads/") {
        return Err(Error::BadRequest("Invalid file path".into()));
    }

    let content = tokio::fs::read(&file_path).await
        .map_err(|_| Error::NotFound)?;

    let mime_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream();

    Ok(Response::new()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type.to_string())
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .body(content))
}
```

---

## Authentication & Authorization

### Session-Based Authentication

#### Session Manager

```
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

#[derive(Clone)]
struct Session {
    user_id: String,
    created_at: SystemTime,
    expires_at: SystemTime,
    data: HashMap<String, String>,
}

#[derive(Clone)]
struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    session_timeout: Duration,
}

impl SessionManager {
    fn new(session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout,
        }
    }

    fn create_session(&self, user_id: String) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now();

        let session = Session {
            user_id,
            created_at: now,
            expires_at: now + self.session_timeout,
            data: HashMap::new(),
        };

        self.sessions.write().unwrap().insert(session_id.clone(), session);
        session_id
    }

    fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().unwrap();
        let session = sessions.get(session_id)?;

        if SystemTime::now() > session.expires_at {
            drop(sessions);
            self.remove_session(session_id);
            return None;
        }

        Some(session.clone())
    }

    fn remove_session(&self, session_id: &str) {
        self.sessions.write().unwrap().remove(session_id);
    }

    fn cleanup_expired(&self) {
        let now = SystemTime::now();
        self.sessions.write().unwrap().retain(|_, session| {
            session.expires_at > now
        });
    }
}
```

#### Authentication Middleware

```
#[derive(Clone)]
struct AuthenticationMiddleware {
    session_manager: SessionManager,
    protected_paths: Vec<String>,
}

impl AuthenticationMiddleware {
    fn new(session_manager: SessionManager) -> Self {
        Self {
            session_manager,
            protected_paths: Vec::new(),
        }
    }

    fn protect_path(mut self, path: impl Into<String>) -> Self {
        self.protected_paths.push(path.into());
        self
    }

    fn requires_auth(&self, path: &str) -> bool {
        self.protected_paths.iter().any(|p| path.starts_with(p))
    }
}

#[async_trait]
impl Middleware for AuthenticationMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        if !self.requires_auth(req.uri.path()) {
            return Ok(());
        }

        let session_id = req.cookie("session_id")
            .ok_or_else(|| Error::Unauthorized)?;

        let session = self.session_manager.get_session(&session_id)
            .ok_or_else(|| Error::Unauthorized)?;

        // Add user info to request
        req.extensions.insert(session.user_id);

        Ok(())
    }
}
```

### Role-Based Authorization

```
#[derive(Clone, PartialEq)]
enum Role {
    Admin,
    User,
    Guest,
}

#[derive(Clone)]
struct User {
    id: String,
    username: String,
    role: Role,
}

struct AuthorizationMiddleware {
    required_role: Role,
}

impl AuthorizationMiddleware {
    fn new(required_role: Role) -> Self {
        Self { required_role }
    }
}

#[async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let user_id = req.extensions.get::<String>()
            .ok_or_else(|| Error::Unauthorized)?;

        let user = get_user_by_id(user_id).await
            .ok_or_else(|| Error::Unauthorized)?;

        match (self.required_role.clone(), user.role) {
            (Role::Admin, Role::Admin) => Ok(()),
            (Role::User, Role::Admin) | (Role::User, Role::User) => Ok(()),
            _ => Err(Error::Forbidden),
        }
    }
}
```

---

## Error Handling

### Error Types

```
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not Found")]
    NotFound,

    #[error("Method Not Allowed")]
    MethodNotAllowed,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Too Many Requests")]
    TooManyRequests,

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Service Unavailable")]
    ServiceUnavailable,
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Error::Conflict(_) => StatusCode::CONFLICT,
            Error::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Error::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
```

### Error Handling Middleware

```
struct ErrorHandlingMiddleware;

#[async_trait]
impl Middleware for ErrorHandlingMiddleware {
    async fn after(&self, res: &mut Response) -> Result<()> {
        if res.status.is_client_error() || res.status.is_server_error() {
            let error_response = serde_json::json!({
                "error": {
                    "code": res.status.as_u16(),
                    "message": res.status.canonical_reason().unwrap_or("Unknown error"),
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }
            });

            *res = Response::json(error_response)?
                .status(res.status);
        }

        Ok(())
    }
}
```

### Custom Error Pages

```
async fn custom_404(_req: Request) -> Result<Response> {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>404 - Page Not Found</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    text-align: center;
                    padding: 50px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                }
                .container {
                    max-width: 600px;
                    margin: 0 auto;
                    background: rgba(255,255,255,0.1);
                    padding: 40px;
                    border-radius: 10px;
                    backdrop-filter: blur(10px);
                }
                h1 { font-size: 72px; margin: 0; }
                p { font-size: 18px; margin: 20px 0; }
                a {
                    color: #ffd700;
                    text-decoration: none;
                    font-weight: bold;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>404</h1>
                <p>🔥 The page you're looking for couldn't be ignited!</p>
                <p><a href="/">← Return to Home</a></p>
            </div>
        </body>
        </html>
    "#;

    Ok(Response::html(html).status(StatusCode::NOT_FOUND))
}
```

---

## Testing

### Unit Testing

```
#[cfg(test)]
mod tests {
    use super::*;
    use ignite::test::TestRequest;

    #[tokio::test]
    async fn test_home_handler() {
        let req = TestRequest::get("/").build();
        let response = home(req).await.unwrap();

        assert_eq!(response.status, StatusCode::OK);
        assert!(response.body.contains("Welcome"));
    }

    #[tokio::test]
    async fn test_json_parsing() {
        let json_body = r#"{"name": "John", "email": "john@example.com"}"#;
        let req = TestRequest::post("/users")
            .json(json_body)
            .build();

        let response = create_user(req).await.unwrap();
        assert_eq!(response.status, StatusCode::CREATED);
    }
}
```

### Integration Testing

```
use ignite::test::TestServer;

#[tokio::test]
async fn test_full_app() {
    let router = create_app_router();
    let server = TestServer::new(router);

    // Test login
    let login_response = server
        .post("/login")
        .json(serde_json::json!({
            "username": "testuser",
            "password": "password123"
        }))
        .send()
        .await;

    assert_eq!(login_response.status(), 200);

    let cookies = login_response.cookies();

    // Test protected route
    let protected_response = server
        .get("/dashboard")
        .cookies(cookies)
        .send()
        .await;

    assert_eq!(protected_response.status(), 200);
}
```

### Testing Middleware

```
#[tokio::test]
async fn test_auth_middleware() {
    let middleware = AuthMiddleware::new("test-token")
        .protect_path("/protected");

    // Test unprotected route
    let mut req = TestRequest::get("/public").build();
    assert!(middleware.before(&mut req).await.is_ok());

    // Test protected route without token
    let mut req = TestRequest::get("/protected").build();
    assert!(middleware.before(&mut req).await.is_err());

    // Test protected route with token
    let mut req = TestRequest::get("/protected")
        .header("Authorization", "Bearer test-token")
        .build();
    assert!(middleware.before(&mut req).await.is_ok());
}
```

---

## Advanced Topics

### Custom Extensions

```
// Define custom request extensions
#[derive(Clone)]
struct RequestId(String);

#[derive(Clone)]
struct UserContext {
    user_id: String,
    permissions: Vec<String>,
}

// Middleware to add request ID
struct RequestIdMiddleware;

#[async_trait]
impl Middleware for RequestIdMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let request_id = uuid::Uuid::new_v4().to_string();
        req.extensions.insert(RequestId(request_id));
        Ok(())
    }
}

// Use in handlers
async fn handler(req: Request) -> Result<Response> {
    let request_id = req.extensions.get::<RequestId>()
        .map(|id| &id.0)
        .unwrap_or("unknown");

    tracing::info!("Processing request {}", request_id);

    Ok(Response::text("OK"))
}
```

### Database Integration

```
use sqlx::{PgPool, Row};

#[derive(Clone)]
struct DatabaseMiddleware {
    pool: PgPool,
}

impl DatabaseMiddleware {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Middleware for DatabaseMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        req.extensions.insert(self.pool.clone());
        Ok(())
    }
}

// Use in handlers
async fn get_users(req: Request) -> Result<Response> {
    let pool = req.extensions.get::<PgPool>().unwrap();

    let users = sqlx::query("SELECT id, name, email FROM users")
        .fetch_all(pool)
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    let user_list: Vec<serde_json::Value> = users
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "name": row.get::<String, _>("name"),
                "email": row.get::<String, _>("email")
            })
        })
        .collect();

    Response::json(user_list)
}
```

### WebSocket Support

```
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

async fn websocket_handler(req: Request) -> Result<Response> {
    // Check if it's a WebSocket upgrade request
    if req.header("Upgrade") != Some("websocket") {
        return Err(Error::BadRequest("Not a WebSocket request".into()));
    }

    // This would need to be implemented in the server layer
    // For now, return an upgrade response
    Ok(Response::new()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Accept", "calculated-accept-key"))
}
```

### Background Tasks

```
use tokio::sync::mpsc;
use std::sync::Arc;

#[derive(Clone)]
struct TaskQueue {
    sender: mpsc::UnboundedSender<Task>,
}

enum Task {
    SendEmail { to: String, subject: String, body: String },
    ProcessImage { path: String },
    GenerateReport { user_id: String },
}

impl TaskQueue {
    fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();

        // Spawn background worker
        tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                match task {
                    Task::SendEmail { to, subject, body } => {
                        // Send email
                        tracing::info!("Sending email to {}: {}", to, subject);
                    }
                    Task::ProcessImage { path } => {
                        // Process image
                        tracing::info!("Processing image at {}", path);
                    }
                    Task::GenerateReport { user_id } => {
                        // Generate report
                        tracing::info!("Generating report for user {}", user_id);
                    }
                }
            }
        });

        Self { sender }
    }

    fn enqueue(&self, task: Task) -> Result<(), Error> {
        self.sender.send(task)
            .map_err(|_| Error::InternalServerError("Failed to enqueue task".into()))
    }
}

// Use in handlers
async fn upload_image(req: Request) -> Result<Response> {
    let task_queue = req.extensions.get::<TaskQueue>().unwrap();

    // Save uploaded file
    let file_path = save_uploaded_file(req).await?;

    // Queue image processing
    task_queue.enqueue(Task::ProcessImage {
        path: file_path.clone()
    })?;

    Response::json(serde_json::json!({
        "message": "Image uploaded successfully",
        "path": file_path,
        "status": "processing"
    }))
}
```

---

## Best Practices

### Security

#### Input Validation

```
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 13, max = 120))]
    age: u32,

    #[validate(custom = "validate_password")]
    password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("Password must be at least 8 characters"));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("Password must contain uppercase letter"));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(ValidationError::new("Password must contain a number"));
    }

    Ok(())
}

async fn create_user(req: Request) -> Result<Response> {
    let user_data: CreateUserRequest = req.json()?;

    // Validate input
    user_data.validate()
        .map_err(|e| Error::BadRequest(format!("Validation error: {}", e)))?;

    // Hash password before storing
    let password_hash = hash_password(&user_data.password)?;

    // Create user...
    Ok(Response::json(created_user)?)
}
```

#### SQL Injection Prevention

```
// Good: Using parameterized queries
async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, Error> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, name, email FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| Error::InternalServerError(e.to_string()))?;

    Ok(user)
}

// Bad: String concatenation (vulnerable to SQL injection)
// let query = format!("SELECT * FROM users WHERE email = '{}'", email);
```

#### CSRF Protection

```
use ring::rand::{SystemRandom, SecureRandom};

struct CsrfMiddleware {
    secret: Vec<u8>,
}

impl CsrfMiddleware {
    fn new() -> Self {
        let mut secret = vec![0u8; 32];
        SystemRandom::new().fill(&mut secret).unwrap();
        Self { secret }
    }

    fn generate_token(&self) -> String {
        let mut token = vec![0u8; 32];
        SystemRandom::new().fill(&mut token).unwrap();
        base64::encode(token)
    }

    fn verify_token(&self, token: &str, session_token: &str) -> bool {
        // Implement CSRF token verification
        token == session_token
    }
}

#[async_trait]
impl Middleware for CsrfMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        if matches!(req.method, Method::POST | Method::PUT | Method::DELETE) {
            let csrf_token = req.header("X-CSRF-Token")
                .or_else(|| req.query("_csrf_token"))
                .ok_or_else(|| Error::BadRequest("Missing CSRF token".into()))?;

            let session_token = req.cookie("csrf_token")
                .ok_or_else(|| Error::BadRequest("Missing CSRF session token".into()))?;

            if !self.verify_token(csrf_token, &session_token) {
                return Err(Error::BadRequest("Invalid CSRF token".into()));
            }
        }

        Ok(())
    }
}
```

### Performance

#### Response Compression

```
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

struct CompressionMiddleware {
    min_size: usize,
}

impl CompressionMiddleware {
    fn new(min_size: usize) -> Self {
        Self { min_size }
    }

    fn should_compress(&self, req: &Request, body: &[u8]) -> bool {
        if body.len() < self.min_size {
            return false;
        }

        if let Some(accept_encoding) = req.header("Accept-Encoding") {
            accept_encoding.contains("gzip")
        } else {
            false
        }
    }

    fn compress_body(&self, body: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(body)?;
        encoder.finish()
    }
}

#[async_trait]
impl Middleware for CompressionMiddleware {
    async fn after(&self, res: &mut Response) -> Result<()> {
        if let Some(request) = res.extensions.get::<Request>() {
            if self.should_compress(request, &res.body) {
                match self.compress_body(&res.body) {
                    Ok(compressed) => {
                        res.body = compressed.into();
                        res.headers.insert("Content-Encoding", "gzip".parse().unwrap());
                        res.headers.insert("Content-Length", res.body.len().to_string().parse().unwrap());
                    }
                    Err(e) => {
                        tracing::warn!("Compression failed: {}", e);
                    }
                }
            }
        }

        Ok(())
    }
}
```

#### Connection Pooling

```
use deadpool_postgres::{Config, Pool, Runtime};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_pool: Arc<Pool>,
    redis_pool: Arc<deadpool_redis::Pool>,
}

async fn create_app_state() -> Result<AppState, Box<dyn std::error::Error>> {
    // Database pool
    let mut db_config = Config::new();
    db_config.host = Some("localhost".to_string());
    db_config.user = Some("postgres".to_string());
    db_config.password = Some("password".to_string());
    db_config.dbname = Some("myapp".to_string());

    let db_pool = Arc::new(db_config.create_pool(Some(Runtime::Tokio1))?);

    // Redis pool
    let redis_config = deadpool_redis::Config::from_url("redis://127.0.0.1/");
    let redis_pool = Arc::new(redis_config.create_pool(Some(Runtime::Tokio1))?);

    Ok(AppState {
        db_pool,
        redis_pool,
    })
}
```

### Logging and Monitoring

#### Structured Logging

```
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

#[instrument(skip(pool))]
async fn get_user(pool: &PgPool, user_id: i32) -> Result<User, Error> {
    info!("Fetching user with ID: {}", user_id);

    match sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(user)) => {
            info!("User found: {}", user.email);
            Ok(user)
        }
        Ok(None) => {
            warn!("User not found: {}", user_id);
            Err(Error::NotFound)
        }
        Err(e) => {
            error!("Database error: {}", e);
            Err(Error::InternalServerError("Database error".into()))
        }
    }
}
```

#### Health Checks

```
#[derive(Serialize)]
struct HealthStatus {
    status: String,
    timestamp: u64,
    version: String,
    checks: HashMap<String, CheckResult>,
}

#[derive(Serialize)]
struct CheckResult {
    status: String,
    message: Option<String>,
    duration_ms: u64,
}

async fn health_check(req: Request) -> Result<Response> {
    let start_time = std::time::Instant::now();
    let mut checks = HashMap::new();

    // Database check
    let db_start = std::time::Instant::now();
    let db_status = match check_database().await {
        Ok(_) => CheckResult {
            status: "healthy".to_string(),
            message: None,
            duration_ms: db_start.elapsed().as_millis() as u64,
        },
        Err(e) => CheckResult {
            status: "unhealthy".to_string(),
            message: Some(e.to_string()),
            duration_ms: db_start.elapsed().as_millis() as u64,
        },
    };
    checks.insert("database".to_string(), db_status);

    // Redis check
    let redis_start = std::time::Instant::now();
    let redis_status = match check_redis().await {
        Ok(_) => CheckResult {
            status: "healthy".to_string(),
            message: None,
            duration_ms: redis_start.elapsed().as_millis() as u64,
        },
        Err(e) => CheckResult {
            status: "unhealthy".to_string(),
            message: Some(e.to_string()),
            duration_ms: redis_start.elapsed().as_millis() as u64,
        },
    };
    checks.insert("redis".to_string(), redis_status);

    let overall_status = if checks.values().all(|c| c.status == "healthy") {
        "healthy"
    } else {
        "unhealthy"
    };

    let health = HealthStatus {
        status: overall_status.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    };

    let status_code = if overall_status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Response::json(health)?.status(status_code)
}
```

---

## Troubleshooting

### Common Issues

#### Port Already in Use

```
use std::net::TcpListener;

fn find_available_port(start_port: u16) -> Option<u16> {
    (start_port..start_port + 100).find(|&port| {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    })
}

async fn start_server_with_fallback(router: Router) -> Result<()> {
    let desired_port = 3000;
    let port = find_available_port(desired_port).unwrap_or(desired_port);

    if port != desired_port {
        println!("Port {} is in use, using port {} instead", desired_port, port);
    }

    let addr = format!("127.0.0.1:{}", port).parse().unwrap();
    let server = Server::new(router, addr);

    server.ignite().await
}
```

#### Memory Leaks in Long-Running Servers

```
// Clean up expired sessions periodically
async fn cleanup_task(session_manager: Arc<SessionManager>) {
    let mut interval = tokio::time::interval(Duration::from_minutes(5));

    loop {
        interval.tick().await;
        session_manager.cleanup_expired();

        // Log memory usage
        let memory_usage = get_memory_usage();
        if memory_usage > 1024 * 1024 * 100 { // 100MB
            tracing::warn!("High memory usage: {} MB", memory_usage / 1024 / 1024);
        }
    }
}

fn get_memory_usage() -> usize {
    // Platform-specific memory usage detection
    // This is a simplified example
    0
}
```

#### Debug Request/Response Flow

```
struct DebugMiddleware;

#[async_trait]
impl Middleware for DebugMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        println!("=== REQUEST DEBUG ===");
        println!("Method: {}", req.method);
        println!("URI: {}", req.uri);
        println!("Headers: {:#?}", req.headers);

        if !req.body.is_empty() {
            if let Ok(body_str) = String::from_utf8(req.body.to_vec()) {
                println!("Body: {}", body_str);
            }
        }

        println!("==================");
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        println!("=== RESPONSE DEBUG ===");
        println!("Status: {}", res.status);
        println!("Headers: {:#?}", res.headers);

        if !res.body.is_empty() {
            if let Ok(body_str) = String::from_utf8(res.body.to_vec()) {
                println!("Body: {}", body_str);
            }
        }

        println!("====================");
        Ok(())
    }
}
```

### Performance Debugging

#### Request Timing

```
use std::time::Instant;

struct TimingMiddleware;

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        req.extensions.insert(Instant::now());
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        if let Some(start_time) = res.extensions.get::<Instant>() {
            let duration = start_time.elapsed();
            let duration_ms = duration.as_millis();

            res.headers.insert(
                "X-Response-Time",
                format!("{}ms", duration_ms).parse().unwrap()
            );

            if duration_ms > 1000 {
                tracing::warn!("Slow request: {}ms", duration_ms);
            }
        }

        Ok(())
    }
}
```

#### Memory Profiling

```
#[cfg(feature = "profiling")]
use jemallocator::Jemalloc;

#[cfg(feature = "profiling")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

async fn memory_stats(_req: Request) -> Result<Response> {
    #[cfg(feature = "profiling")]
    {
        let stats = jemalloc_ctl::stats::allocated::read().unwrap();
        Response::json(serde_json::json!({
            "allocated_bytes": stats,
            "allocated_mb": stats / 1024 / 1024
        }))
    }

    #[cfg(not(feature = "profiling"))]
    {
        Response::json(serde_json::json!({
            "message": "Profiling not enabled"
        }))
    }
}
```

---

## Migration Guide

### From Other Frameworks

#### From Actix-Web

```
// Actix-Web
use actix_web::{web, App, HttpServer, HttpResponse, Result};

async fn hello() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().route("/hello", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// Ignite equivalent
use ignite::{Router, Server, Request, Response, Result, handler_fn};

async fn hello(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello world!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/hello", handler_fn(hello));

    let server = Server::new(router, "127.0.0.1:8080".parse().unwrap());
    server.ignite().await.unwrap();
    Ok(())
}
```

#### From Rocket

```
// Rocket
#[macro_use] extern crate rocket;

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![hello])
}

// Ignite equivalent
use ignite::{Router, Server, Request, Response, Result, handler_fn};

async fn hello(req: Request) -> Result<Response> {
    let name = req.param("name").unwrap_or(&"World".to_string());
    Ok(Response::text(format!("Hello, {}!", name)))
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/hello/:name", handler_fn(hello));

    let server = Server::new(router, "127.0.0.1:8000".parse().unwrap());
    server.ignite().await.unwrap();
    Ok(())
}
```

### Version Upgrades

#### Breaking Changes in v0.2.0 (Planned)

- `Router::middleware()` will return `Result<Router, MiddlewareError>`
- `Cookie::expires()` will use `chrono::DateTime` instead of `SystemTime`
- New required parameter for `Server::new()` to specify configuration

#### Migration Script

```
// v0.1.0
let router = Router::new()
    .middleware(LoggerMiddleware);

// v0.2.0 (planned)
let router = Router::new()
    .middleware(LoggerMiddleware)?;
```

---

This documentation provides a comprehensive guide to using the Ignite web framework. For additional help, examples, and community support:

- **GitHub Repository**: https://github.com/AarambhDevHub/ignite
- **API Documentation**: https://docs.rs/ignite
- **YouTube Channel**: [Aarambh Dev Hub](https://youtube.com/@AarambhDevHub)
- **Issues & Support**: https://github.com/AarambhDevHub/ignite/issues

**Built with ❤️ by Aarambh Dev Hub - Where every line of code ignites possibilities.**
