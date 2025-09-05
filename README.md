# 🔥 Ignitia

<div align="center">

<img src="https://raw.githubusercontent.com/AarambhDevHub/ignitia/main/assets/ignitia-logo.png" alt="Ignitia Logo" width="200">

**A blazing fast, lightweight web framework for Rust that ignitias your development journey.**

*Embodies the spirit of **Aarambh** (new beginnings) - the spark that ignitias your web development journey*

**Built with ❤️ by [Aarambh Dev Hub](https://youtube.com/@aarambhdevhub)**

[![Crates.io](https://img.shields.io/crates/v/ignitia.svg?style=for-the-badge&logo=rust&color=orange&labelColor=black)](https://crates.io/crates/ignitia)
[![Downloads](https://img.shields.io/crates/d/ignitia.svg?style=for-the-badge&color=red&labelColor=black)](https://crates.io/crates/ignitia)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge&labelColor=black)](https://opensource.org/licenses/MIT)

[![Build Status](https://img.shields.io/github/actions/workflow/status/AarambhDevHub/ignitia/ci.yml?branch=main&style=for-the-badge&logo=github&labelColor=black)](https://github.com/AarambhDevHub/ignitia/actions)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=for-the-badge&logo=rust&labelColor=black)](https://www.rust-lang.org/)
[![Docs](https://img.shields.io/docsrs/ignitia?style=for-the-badge&logo=rust&color=blue&labelColor=black)](https://docs.rs/ignitia)

[![GitHub Stars](https://img.shields.io/github/stars/AarambhDevHub/ignitia?style=social&logo=github)](https://github.com/AarambhDevHub/ignitia/stargazers)
[![YouTube](https://img.shields.io/youtube/channel/subscribers/UCm5U5uQiZA_mQY5wQ6WfUVA?style=social&logo=youtube&label=Aarambh%20Dev%20Hub)](https://youtube.com/@aarambhdevhub)

<img src="https://raw.githubusercontent.com/AarambhDevHub/ignitia/main/assets/ignitia-banner.png" alt="Ignitia Banner" width="100%">

</div>

---

## ⚡ Why Ignitia?

Ignitia embodies the spirit of **Aarambh** (new beginnings) - the spark that ignitias your web development journey. Built for developers who demand speed, simplicity, and power.

- **🚀 Blazing Fast**: Built on Hyper and Tokio for maximum async performance
- **🪶 Lightweight**: Minimal overhead, maximum efficiency
- **🔥 Powerful**: Advanced routing, middleware, and built-in features
- **⚡ Energetic**: Modern APIs that energize your development
- **🎯 Developer-First**: Clean, intuitive, and productive
- **🛡️ Secure**: Built-in security features and best practices
- **🍪 Cookie Management**: Full-featured cookie handling with security attributes
- **🔧 Middleware**: Composable middleware architecture for cross-cutting concerns

---

## 📋 Table of Contents

- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Core Features](#-core-features)
- [Routing](#-routing)
- [Middleware](#-middleware)
- [Cookie Management](#-cookie-management)
- [Authentication](#-authentication)
- [Examples](#-examples)
- [API Reference](#-api-reference)
- [Testing](#-testing)
- [Performance](#-performance)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🛠️ Installation

Add Ignitia to your `Cargo.toml`:

```
[dependencies]
ignitia = "0.1.1"
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing-subscriber = "0.3"
```

---

## 🚀 Quick Start

Create your first Ignitia application:

```rust
use ignitia::{
    Router, Server, Response, Result,
    handler_fn,
    handler::extractor::{Path, Query, Json, Body, Extension},
};
use serde::Deserialize;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let shared_state = Arc::new(AppState { app_name: "IgnitiaApp".to_string() });

    let router = Router::new()
        // Simple route with no params
        .get("/", handler_fn(root_handler))
        // Route with path params
        .get("/users/:id", get_user)
        // Route with query params extractor
        .get("/search", search_handler)
        // Route to create user with JSON body
        .post("/users", create_user)
        // Route using raw Body extractor
        .post("/upload", upload_handler)
        // Route using extension for shared state
        .get("/info", with_state)
    ;

    let router = router.middleware(StateExtensionMiddleware::new(shared_state.clone()));

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());

    println!("🔥 Igniting server at http://127.0.0.1:3000");
    server.ignitia().await?;

    Ok(())
}

async fn root_handler() -> Result<Response> {
    Ok(Response::html("<h1>🔥 Welcome to Ignitia!</h1>"))
}

async fn get_user(Path(params): Path<(String,)>) -> Result<Response> {
    let user_id = &params.0;
    Ok(Response::json(serde_json::json!({ "user_id": user_id }))?)
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    limit: Option<u32>,
}

async fn search_handler(Query(params): Query<SearchParams>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "query": params.q,
        "limit": params.limit.unwrap_or(10)
    }))?)
}

#[derive(Deserialize)]
struct NewUser {
    name: String,
    email: String,
}

async fn create_user(Json(user): Json<NewUser>) -> Result<Response> {
    // Simulate saving user
    Ok(Response::json(serde_json::json!({
        "status": "created",
        "user": user
    }))?)
}

async fn upload_handler(Body(body): Body) -> Result<Response> {
    Ok(Response::text(format!("Uploaded {} bytes", body.len())))
}

struct AppState {
    app_name: String,
}

async fn with_state(Extension(state): Extension<Arc<AppState>>) -> Result<Response> {
    Ok(Response::text(format!("App name: {}", state.app_name)))
}

// Middleware to inject state into request extensions (see below)
struct StateExtensionMiddleware(Arc<AppState>);
impl StateExtensionMiddleware {
    fn new(state: Arc<AppState>) -> Self {
        Self(state)
    }
}

#[async_trait::async_trait]
impl ignitia::middleware::Middleware for StateExtensionMiddleware {
    async fn before(&self, req: &mut ignitia::Request) -> ignitia::Result<()> {
        req.insert_extension(self.0.clone());
        Ok(())
    }
}
```

---

## 🔥 Core Features

### 🛣️ Routing Examples

#### Path Parameter Extraction

```rust
async fn get_post(Path((user_id, post_id)): Path<(String, String)>) -> Result<Response> {
    Ok(Response::text(format!("User: {}, Post: {}", user_id, post_id)))
}

let router = Router::new()
    .get("/users/:user_id/posts/:post_id", handler_fn(get_post));
```

#### Query Parameter Extraction

```rust
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

async fn list_items(Query(pagination): Query<Pagination>) -> Result<Response> {
    Ok(Response::json(&pagination)?)
}
```

#### JSON Body Parsing

```rust
#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(Json(form): Json<LoginForm>) -> Result<Response> {
    // Authenticate
    Ok(Response::text(format!("Welcome, {}!", form.username)))
}
```

#### Raw Body Usage

```rust
async fn raw_upload(Body(body): Body) -> Result<Response> {
    Ok(Response::text(format!("Received {} bytes", body.len())))
}
```

#### Using Shared State via Extensions

```rust
use std::sync::Arc;

struct AppConfig {
    version: String,
}

async fn version_info(Extension(config): Extension<Arc<AppConfig>>) -> Result<Response> {
    Ok(Response::text(format!("App version: {}", config.version)))
}
```

### **Wildcard Routes**

```rust
// Serve static files: /*path matches any path
.get("/*path", serve_static)

async fn serve_static() -> Result<Response> {
    let path = req.param("path").unwrap();
    // Serve file from static directory with security checks
    serve_file_from_directory("./static", path).await
}
```


### **🍪 Built-in Cookie Management**

Secure, easy-to-use cookie handling with all security attributes:

```rust
use ignitia::{Cookie, SameSite};

// Set secure cookies
let session = Cookie::new("session", "user123")
    .path("/")
    .max_age(3600) // 1 hour
    .http_only()
    .secure()
    .same_site(SameSite::Lax);

let response = Response::text("Session ignitiad!")
    .add_cookie(session);

// Read cookies
async fn protected_route(cookies: Cookies) -> Result<Response> {
    let username = cookies.get("session_user")
        .unwrap_or("anonymous".to_string());

    Ok(Response::text(format!("Welcome back, {}!", username)))
}

// Remove cookies
let response = Response::text("Logged out")
    .remove_cookie("session");
```

### **🛡️ Powerful Middleware System**

Composable middleware for authentication, logging, CORS, and more:

```rust
use ignitia::middleware::{AuthMiddleware, CorsMiddleware, LoggerMiddleware};

let router = Router::new()
    // Global middleware
    .middleware(LoggerMiddleware)
    .middleware(CorsMiddleware::new()
        .allow_origin("https://example.com"))

    // Protected routes
    .middleware(AuthMiddleware::new("secret-token")
        .protect_path("/api/admin")
        .protect_path("/dashboard"))

    .get("/api/admin/users", admin_users)
    .get("/dashboard", dashboard);
```

### **🔐 Authentication & Authorization**

Built-in session management and role-based access control:

```rust
// Custom authentication middleware
#[derive(Clone)]
struct AuthMiddleware {
    protected_paths: Vec<String>,
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let path = req.uri.path();

        if self.protected_paths.iter().any(|p| path.starts_with(p)) {
            let _session = req.cookie("session")
                .ok_or_else(|| Error::Unauthorized)?;
            // Validate session...
        }

        Ok(())
    }
}
```

---

## 🔧 Middleware

### **Built-in Middleware**

| Middleware | Purpose | Usage |
|------------|---------|-------|
| `LoggerMiddleware` | Request/response logging | `.middleware(LoggerMiddleware)` |
| `CorsMiddleware` | Cross-origin resource sharing | `.middleware(CorsMiddleware::new())` |
| `AuthMiddleware` | Authentication | `.middleware(AuthMiddleware::new("token"))` |

### **Custom Middleware**

Create your own middleware by implementing the `Middleware` trait:

```rust
use ignitia::{Middleware, async_trait};

struct RateLimitMiddleware {
    max_requests: usize,
    window: Duration,
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Rate limiting logic
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // Add rate limit headers
        res.headers.insert(
            "X-RateLimit-Remaining",
            "99".parse().unwrap()
        );
        Ok(())
    }
}
```

---

## 🍪 Cookie Management

### **Setting Cookies**

```rust
use ignitia::{Cookie, SameSite};

// Session cookie
let session = Cookie::new("session", "abc123")
    .path("/")
    .max_age(3600)
    .http_only()
    .same_site(SameSite::Lax);

// Persistent cookie
let preferences = Cookie::new("theme", "dark")
    .path("/")
    .max_age(86400 * 30) // 30 days
    .same_site(SameSite::Strict);

// Secure cookie
let secure_token = Cookie::new("csrf_token", "xyz789")
    .path("/")
    .secure()
    .http_only()
    .same_site(SameSite::Strict);

Response::text("Cookies set!")
    .add_cookie(session)
    .add_cookie(preferences)
    .add_cookie(secure_token)
```

### **Reading Cookies**

```rust
async fn handle_request(cookies: Cookies) -> Result<Response> {
    // Get all cookies
    let cookies = cookies;

    // Get specific cookie
    let session = cookies.get("session");

    // Check if cookie exists
    if cookies.contains("user_preferences") {
        // Handle with preferences
    }

    Response::json(cookies.all())
}
```

### **Cookie Security**

```rust
// Production-ready secure cookie
let secure_session = Cookie::new("session", session_id)
    .path("/")
    .max_age(3600)
    .http_only()        // Prevent XSS
    .secure()           // HTTPS only
    .same_site(SameSite::Strict); // CSRF protection
```

---

## 🔐 Authentication

### **Session-Based Authentication**

```rust
async fn login(req: Request) -> Result<Response> {
    let credentials: LoginForm = req.json()?;

    if validate_credentials(&credentials).await? {
        let session_token = generate_session_token();

        let session_cookie = Cookie::new("session", session_token)
            .path("/")
            .max_age(3600)
            .http_only()
            .same_site(SameSite::Lax);

        Ok(Response::json(serde_json::json!({
            "status": "success",
            "message": "Login successful"
        }))?.add_cookie(session_cookie))
    } else {
        Err(Error::Unauthorized)
    }
}

async fn logout(_req: Request) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "status": "success",
        "message": "Logged out"
    }))?.remove_cookie("session"))
}
```

### **Protected Routes with Middleware**

```rust
let router = Router::new()
    .middleware(AuthMiddleware::new()
        .protect_paths(vec!["/dashboard", "/admin", "/api/protected"]))

    // These routes are automatically protected
    .get("/dashboard", dashboard)
    .get("/admin", admin_panel)
    .get("/api/protected/data", protected_data);
```

---

## 📚 Examples

Ignitia comes with comprehensive examples to get you started:

### **🏠 Basic Server**
```
cargo run --example basic_server
# http://127.0.0.1:3000
```
Demonstrates basic routing, JSON responses, and parameter extraction.

### **🔐 Authentication System**
```
cargo run --example login_example
# http://127.0.0.1:3008
```
Complete login/logout system with session management and protected routes.

### **🛡️ Middleware Showcase**
```
cargo run --example login_with_middleware_example
# http://127.0.0.1:3009
```
Advanced authentication using middleware with role-based access control.

### **🍪 Cookie Management**
```
cargo run --example cookie_framework_example
# http://127.0.0.1:3006
```
Comprehensive cookie handling with security features and session management.

### **⚡ Custom Middleware**
```
cargo run --example custom_middleware_example
# http://127.0.0.1:3004
```
Rate limiting, request validation, security headers, and custom middleware examples.

### **📊 JSON API**
```
cargo run --example json_api
# http://127.0.0.1:3002
```
RESTful API for managing todos with in-memory storage and CRUD operations.

### **📁 File Server**
```
cargo run --example file_server
# http://127.0.0.1:3003
```
Static file server with security features, MIME type detection, and directory traversal protection.

### **🎭 Middleware Demo**
```
cargo run --example middleware_example
# http://127.0.0.1:3001
```
CORS, authentication, and logging middleware examples.

---

## 🔌 API Reference

### **Router**

| Method              | Description              |
|---------------------|--------------------------|
| `Router::new()`     | Create a new router      |
| `.get(path, handler)` | Add GET route            |
| `.post(path, handler)` | Add POST route           |
| `.put(path, handler)`  | Add PUT route            |
| `.delete(path, handler)` | Add DELETE route         |
| `.patch(path, handler)` | Add PATCH route          |
| `.middleware(middleware)` | Add middleware          |
| `.not_found(handler)`     | Set 404 handler          |

***

### **Request**

| Method                 | Description                   | Example                          |
|------------------------|-------------------------------|---------------------------------|
| `req.param(key)`       | Get route parameter            | `req.param("id")`               |
| `req.query(key)`       | Get query parameter            | `req.query("limit")`            |
| `req.header(key)`      | Get header value               | `req.header("User-Agent")`      |
| `req.json::<T>()`      | Parse JSON body into type `T` | `req.json::<User>()?`            |
| `req.cookies()`        | Get all cookies                | `req.cookies().all()`           |
| `req.cookie(key)`      | Get specific cookie            | `req.cookie("session")`         |

***

### **Extractors for Handler Functions**

| Extractor             | Description                                    | Example Usage                                   |
|-----------------------|------------------------------------------------|------------------------------------------------|
| `Path<T>`             | Extract typed route parameters                  | `async fn handler(Path(params): Path<(String, u32)>) { ... }` |
| `Query<T>`            | Extract typed query parameters                  | `async fn handler(Query(filter): Query<Filter>) { ... }`    |
| `Json<T>`             | Parse JSON request body into type `T`          | `async fn handler(Json(user): Json<User>) { ... }`           |
| `Body`                | Access raw request body as bytes                 | `async fn handler(Body(body): Body) { ... }`                 |
| `Extension<T>`        | Inject shared application state or dependencies | `async fn handler(Extension(state): Extension<AppState>) { ... }` |

***

### **Response**

| Method                   | Description                | Example                              |
|--------------------------|----------------------------|------------------------------------|
| `Response::text(content)` | Return plain text response | `Response::text("Hello")`           |
| `Response::html(content)` | Return HTML response       | `Response::html("<h1>Hi</h1>")`    |
| `Response::json(data)`    | Return JSON response       | `Response::json(user)?`              |
| `Response::not_found()`   | Return 404 Not Found       | `Response::not_found()`              |
| `.add_cookie(cookie)`     | Add cookie to response     | `.add_cookie(session)`               |
| `.remove_cookie(name)`    | Remove cookie from client  | `.remove_cookie("session")`         |

***

### **Cookie Builder**

| Method              | Description                     | Example                         |
|---------------------|---------------------------------|--------------------------------|
| `Cookie::new(name, value)` | Create a new cookie             | `Cookie::new("user", "john")`  |
| `.path(path)`       | Set cookie path                 | `.path("/")`                   |
| `.domain(domain)`   | Set cookie domain               | `.domain("example.com")`       |
| `.max_age(seconds)` | Set max age in seconds          | `.max_age(3600)`               |
| `.expires(time)`    | Set expiration time             | `.expires(SystemTime::now())`  |
| `.secure()`         | Mark cookie secure (HTTPS only) | `.secure()`                    |
| `.http_only()`      | Disallow JavaScript access      | `.http_only()`                 |
| `.same_site(policy)`| Set SameSite attribute          | `.same_site(SameSite::Lax)`   |

### **Middleware Trait**

```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, req: &mut Request) -> Result<()> { Ok(()) }
    async fn after(&self, res: &mut Response) -> Result<()> { Ok(()) }
}
```

---


### **Test Your Application**

```
# Start server
cargo run --example basic_server &

# Test endpoints
curl http://127.0.0.1:3000/
curl http://127.0.0.1:3000/users/123
curl -X POST -H "Content-Type: application/json" \
     -d '{"name":"John"}' \
     http://127.0.0.1:3000/api/data
```

---

## 🎯 Performance

### **Benchmarks**

- **Zero-copy**: Efficient request/response handling with minimal allocations
- **Async**: Built on Tokio for excellent concurrency
- **Lightweight**: ~50KB binary overhead
- **Fast compilation**: Small dependency tree for quick builds
- **Memory efficient**: Smart use of Arc and shared state

### **Performance Tips**

```
// Use connection pooling
let router = Router::new()
    .middleware(ConnectionPoolMiddleware::new())

    // Cache static responses
    .middleware(CacheMiddleware::new())

    // Compress responses
    .middleware(CompressionMiddleware::new());
```

---

## 🔒 Security Features

### **Built-in Security**

- ✅ **Path traversal protection** in static file serving
- ✅ **CORS middleware** for cross-origin requests
- ✅ **Authentication middleware** with configurable paths
- ✅ **Security headers middleware** (CSP, XSS protection, etc.)
- ✅ **Rate limiting middleware** to prevent abuse
- ✅ **Secure cookie attributes** (HttpOnly, Secure, SameSite)
- ✅ **Session management** with proper invalidation

### **Security Best Practices**

```rust
// Secure cookie configuration
let secure_cookie = Cookie::new("session", session_token)
    .path("/")
    .max_age(3600)
    .http_only()     // Prevent XSS
    .secure()        // HTTPS only in production
    .same_site(SameSite::Strict); // CSRF protection

// Security headers
let router = Router::new()
    .middleware(SecurityHeadersMiddleware::new()
        .csp("default-src 'self'")
        .hsts(31536000)
        .frame_deny());
```

---

## 🛠️ Development

### **Project Structure**

```
ignitia/
├── Cargo.toml
├── README.md
├── examples/
│   ├── basic_server.rs
│   ├── login_example.rs
│   ├── cookie_framework.rs
│   ├── custom_middleware.rs
│   ├── login_with_middleware.rs
│   ├── middleware_example.rs
│   ├── json_api.rs
│   └── file_server.rs
└── src/
    ├── lib.rs                  # Main library exports and re-exports
    ├── router/                 # Routing modules
    │   ├── mod.rs
    │   ├── route.rs
    │   └── method.rs
    ├── middleware/             # Middleware implementations
    │   ├── mod.rs
    │   ├── logger.rs
    │   ├── cors.rs
    │   └── auth.rs
    ├── request/                # Request handling components
    │   ├── mod.rs
    │   ├── body.rs
    │   └── params.rs
    ├── response/               # Response creation and builders
    │   ├── mod.rs
    │   ├── builder.rs
    │   └── status.rs
    ├── handler/                # Handler traits and extractors
    │   ├── mod.rs
    │   └── extractor.rs
    ├── server/                 # HTTP server implementation
    │   ├── mod.rs
    │   └── connection.rs
    ├── cookie/                 # Cookie management utilities
    │   └── mod.rs
    ├── extension/              # Extension types and utilities
    │   └── mod.rs
    ├── error/                  # Error types and handling
    │   └── mod.rs
    └── utils/                  # Helper utilities
        └── mod.rs
```

### **Building**

```
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run all examples
cargo run --example basic_server
cargo run --example login_example
cargo run --example cookie_framework_example

# Check code quality
cargo fmt
cargo clippy
cargo test

# Generate documentation
cargo doc --open
```

---

## 🤝 Contributing

We welcome contributions! Here's how you can help make Ignitia even better:

### **Ways to Contribute**

- 🐛 **Bug Reports**: Found a bug? Open an issue!
- ✨ **Feature Requests**: Have an idea? We'd love to hear it!
- 📖 **Documentation**: Help improve our docs
- 🧪 **Tests**: Add more test coverage
- 🔧 **Code**: Submit pull requests

### **Development Setup**

```
# Clone the repository
git clone https://github.com/AarambhDevHub/ignitia.git
cd ignitia

# Install dependencies and build
cargo build

# Run tests
cargo test --all

# Run examples
cargo run --example basic_server
```

### **Guidelines**

1. **Code Quality**: Run `cargo fmt` and `cargo clippy` before submitting
2. **Tests**: Add tests for new features
3. **Documentation**: Update README and add doc comments
4. **Examples**: Provide examples for new features
5. **Compatibility**: Ensure backward compatibility when possible

### **Pull Request Process**

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📝 Changelog

### **v0.1.1** - Initial Release 🎉

#### **Core Features**
- ✅ Advanced routing with support for typed path parameters and wildcard routes
- ✅ Comprehensive middleware system with support for common and custom middleware
- ✅ Built-in secure cookie management with full attribute controls (HttpOnly, Secure, SameSite)
- ✅ Authentication and authorization middleware with path-based protection
- ✅ Static file serving with path traversal security protections
- ✅ JSON API support with automatic request body deserialization and response serialization
- ✅ Robust request/response handling with detailed error management and status codes

#### **Middleware**
- ✅ Authentication middleware supporting token and session validation with configurable protected paths
- ✅ CORS middleware with customizable allowed origins, methods, and headers
- ✅ Request logging middleware with HTTP version and status codes logging
- ✅ Rate limiting middleware for abuse prevention (custom middleware support)
- ✅ Security headers middleware for CSP, HSTS, frame options, and more (custom middleware support)
- ✅ Full support for user-defined custom middleware implementations via the Middleware trait

#### **Examples**
- ✅ Basic server demonstrating core routing and responses
- ✅ Complete authentication system with session management and role-based access control
- ✅ Cookie management showcase with secure cookie creation, reading, and removal
- ✅ Custom middleware examples including rate limiting and security headers
- ✅ JSON API example with CRUD operations and typed data handling
- ✅ Static file server example with MIME detection and security checks
- ✅ Middleware integration examples covering logging, CORS, and authentication

#### **Security**
- ✅ Secure cookie configurations (HttpOnly, Secure, SameSite)
- ✅ Path traversal protection in static file serving
- ✅ Session management with cookie-based authentication
- ✅ CSRF protection via SameSite cookie policies and middleware
- ✅ XSS prevention through secure cookie flags and content headers

---

## 🙏 Acknowledgments

### **Built on Strong Foundations**

- **[Hyper](https://hyper.rs/)** - High-performance HTTP implementation
- **[Tokio](https://tokio.rs/)** - Asynchronous runtime for Rust
- **Community** - Inspired by frameworks like Actix-web, Warp, and Axum

### **Special Thanks**

- **Rust Community** - For creating an amazing ecosystem
- **Contributors** - Everyone who helps make ignitia better
- **Early Adopters** - Thanks for trying ignitia and providing feedback!

---


## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ☕ Support the Project
If you find this project helpful, consider buying me a coffee!
[Buy Me a Coffee](https://buymeacoffee.com/aarambhdevhub)


---

## 🚀 Getting Started

Ready to ignitia your web development? Let's get started:

```
# Create a new project
cargo new my-ignitia-app
cd my-ignitia-app

# Add ignitia to Cargo.toml
echo '[dependencies]
ignitia = "0.1.1"
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"' >> Cargo.toml

# Create your first ignitia app
echo 'use ignitia::{Router, Server, Response, handler_fn};

#[tokio::main]
async fn main() -> ignitia::Result<()> {
    let router = Router::new()
        .get("/", handler_fn(|_| async {
            Ok(Response::html("<h1>🔥 Welcome to ignitia!</h1>"))
        }));

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Server igniting on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}' > src/main.rs

# Run your app
cargo run
```

### **Next Steps**

1. 📖 **Explore Examples**: Check out our comprehensive examples
2. 🛠️ **Build Something**: Create your first API or web app
3. 🤝 **Join Community**: Connect with other ignitia developers
4. 📺 **Learn More**: Subscribe to [Aarambh Dev Hub](https://youtube.com/@aarambhdevhub)

---

<div align="center">

## 🔥 **Ignitia. Build. Deploy.** 🔥

**Built with ❤️ by [Aarambh Dev Hub](https://youtube.com/@aarambhdevhub)**

*Where every line of code ignitias possibilities.*

[![YouTube](https://img.shields.io/badge/YouTube-Aarambh%20Dev%20Hub-red?style=for-the-badge&logo=youtube)](https://youtube.com/@aarambhdevhub)
[![GitHub](https://img.shields.io/badge/GitHub-ignitia-black?style=for-the-badge&logo=github)](https://github.com/AarambhDevHub/ignitia)
[![Crates.io](https://img.shields.io/badge/Crates.io-ignitia-orange?style=for-the-badge&logo=rust)](https://crates.io/crates/ignitia)

</div>
