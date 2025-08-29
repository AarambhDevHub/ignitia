# 🚀 Mini Web Framework

A lightweight, fast, and feature-rich web framework for Rust built on top of Hyper and Tokio. This framework provides all the essential features needed to build modern web applications with minimal overhead, maximum performance, and developer-friendly APIs.

Built on Hyper and Tokio for maximum async performance
- **🎯 Advanced Routing**: Parameter extraction, wildcard patterns, and method-specific routes
- **🔧 Powerful Middleware**: Composable middleware architecture with built-in auth, CORS, logging, and rate limiting
- **🍪 Built-in Cookies**: Full cookie management with security attributes and session handling
- **📄 Static File Serving**: Built-in static file server with security features and MIME type detection
- **🔒 Authentication & Authorization**: Session-based auth with role-based access control
- **📊 JSON Support**: First-class JSON serialization and deserialization
- **⚡ Fast Compilation**: Minimal dependencies for quick build times
- **🛡️ Type Safety**: Full Rust type safety throughout the framework
- **📝 Comprehensive Examples**: Multiple real-world usage examples included

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

## 🛠️ Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mini-web-framework = "0.1.0"
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing-subscriber = "0.3"
```

## 🚀 Quick Start

Create a simple web server in just a few lines:

```rust
use mini_web_framework::{Router, Server, Request, Response, Result, handler_fn};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", handler_fn(hello_world))
        .get("/users/:id", handler_fn(get_user))
        .post("/users", handler_fn(create_user));

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🚀 Server running on http://{}", addr);
    server.run().await.unwrap();
    Ok(())
}

async fn hello_world(_req: Request) -> Result<Response> {
    Ok(Response::text("Hello, World! 🌍"))
}

async fn get_user(req: Request) -> Result<Response> {
    let user_id = req.param("id").unwrap_or(&"unknown".to_string());
    Ok(Response::text(format!("User ID: {}", user_id)))
}

async fn create_user(req: Request) -> Result<Response> {
    let user_data: serde_json::Value = req.json()?;
    Response::json(user_data)
}
```

## 🧭 Core Features

### Request & Response Handling

```rust
// Access request data
let method = req.method;
let path = req.uri.path();
let user_agent = req.header("User-Agent");
let json_data: MyStruct = req.json()?;
let cookies = req.cookies();
let user_id = req.param("id");
let search = req.query("q");

// Create responses
Response::text("Plain text")
Response::html("<h1>HTML content</h1>")
Response::json(my_data)
Response::not_found()
    .add_cookie(Cookie::new("session", "abc123"))
```

### Built-in Error Handling

```rust
use mini_web_framework::{Error, Result};

async fn handler(req: Request) -> Result<Response> {
    let data = req.json::<MyData>()
        .map_err(|_| Error::BadRequest("Invalid JSON".into()))?;

    Ok(Response::json(data)?)
}
```

## 🛣️ Routing

### Basic Routes with HTTP Methods

```rust
let router = Router::new()
    .get("/", handler_fn(home))
    .post("/users", handler_fn(create_user))
    .put("/users/:id", handler_fn(update_user))
    .delete("/users/:id", handler_fn(delete_user))
    .not_found(handler_fn(not_found));
```

### Advanced Parameter Handling

```rust
// Route parameters: /users/:id/posts/:post_id
async fn get_post(req: Request) -> Result<Response> {
    let user_id = req.param("id").unwrap();
    let post_id = req.param("post_id").unwrap();
    // Handle the request...
}

// Query parameters: /search?q=rust&limit=10
async fn search(req: Request) -> Result<Response> {
    let query = req.query("q").unwrap_or(&"".to_string());
    let limit = req.query("limit")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(20);
    // Handle search...
}
```

### Wildcard Routes for Static Files

```rust
let router = Router::new()
    .get("/*path", handler_fn(serve_static_files));

async fn serve_static_files(req: Request) -> Result<Response> {
    let path = req.param("path").unwrap();
    // Serve file from static directory
}
```

## 🔧 Middleware

### Built-in Middleware

```rust
use mini_web_framework::middleware::{LoggerMiddleware, CorsMiddleware, AuthMiddleware};

let router = Router::new()
    .middleware(LoggerMiddleware)
    .middleware(CorsMiddleware::new().allow_origin("https://example.com"))
    .middleware(AuthMiddleware::new("secret-token")
        .protect_path("/api/")
        .protect_path("/admin/"));
```

### Custom Middleware

```rust
use mini_web_framework::{Middleware, async_trait};

struct RateLimitMiddleware {
    max_requests: usize,
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Check rate limits
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // Add rate limit headers
        res.headers.insert("X-RateLimit-Remaining", "99".parse().unwrap());
        Ok(())
    }
}
```

## 🍪 Cookie Management

Built-in cookie support with security features:

```rust
use mini_web_framework::{Cookie, SameSite};

// Set cookies
let session_cookie = Cookie::new("session", "user123")
    .path("/")
    .max_age(3600) // 1 hour
    .http_only()
    .secure()
    .same_site(SameSite::Lax);

let response = Response::text("Login successful")
    .add_cookie(session_cookie);

// Read cookies
async fn protected_route(req: Request) -> Result<Response> {
    let session = req.cookie("session")
        .ok_or_else(|| Error::Unauthorized)?;

    Ok(Response::text(format!("Welcome back, {}!", session)))
}

// Remove cookies
let response = Response::text("Logged out")
    .remove_cookie("session");
```

## 🔐 Authentication

Complete authentication system with middleware:

```rust
#[derive(Clone)]
struct AuthMiddleware {
    protected_paths: Vec<String>,
}

impl AuthMiddleware {
    fn protect_paths(mut self, paths: Vec<&str>) -> Self {
        self.protected_paths = paths.into_iter().map(String::from).collect();
        self
    }
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

// Usage
let router = Router::new()
    .middleware(AuthMiddleware::new()
        .protect_paths(vec!["/admin", "/dashboard"]))
    .get("/admin", handler_fn(admin_panel))
    .get("/dashboard", handler_fn(dashboard));
```

## 📚 Examples

### 1. Basic Server
```bash
cargo run --example basic_server
```
Demonstrates basic routing, JSON responses, and parameter extraction.

### 2. Middleware Demo
```bash
cargo run --example middleware_example
```
Shows authentication, CORS, and logging middleware in action.

### 3. Cookie Framework
```bash
cargo run --example cookie_framework_example
```
Built-in cookie functionality with security features.

### 4. Login System
```bash
cargo run --example login_example
```
Complete authentication system with session management.

### 5. Login with Middleware
```bash
cargo run --example login_with_middleware_example
```
Advanced authentication using middleware for cleaner code.

### 6. Custom Middleware
```bash
cargo run --example custom_middleware_example
```
Rate limiting, request validation, and security headers.

### 7. JSON API
```bash
cargo run --example json_api
```
RESTful API for managing todos with in-memory storage.

### 8. File Server
```bash
cargo run --example file_server
```
Static file server with security features and MIME type detection.

## 🔌 API Reference

### Router Methods

| Method | Description |
|--------|-------------|
| `Router::new()` | Create a new router |
| `.get(path, handler)` | Add GET route |
| `.post(path, handler)` | Add POST route |
| `.put(path, handler)` | Add PUT route |
| `.delete(path, handler)` | Add DELETE route |
| `.middleware(middleware)` | Add middleware |
| `.not_found(handler)` | Set 404 handler |

### Request Methods

| Method | Description |
|--------|-------------|
| `req.param(key)` | Get route parameter |
| `req.query(key)` | Get query parameter |
| `req.header(key)` | Get header value |
| `req.json<T>()` | Parse JSON body |
| `req.cookies()` | Get all cookies |
| `req.cookie(key)` | Get specific cookie |

### Response Methods

| Method | Description |
|--------|-------------|
| `Response::text(content)` | Create text response |
| `Response::html(content)` | Create HTML response |
| `Response::json(data)` | Create JSON response |
| `Response::not_found()` | Create 404 response |
| `.add_cookie(cookie)` | Add cookie to response |
| `.remove_cookie(name)` | Remove cookie |

### Cookie Builder

| Method | Description |
|--------|-------------|
| `Cookie::new(name, value)` | Create cookie |
| `.path(path)` | Set cookie path |
| `.max_age(seconds)` | Set expiration |
| `.secure()` | HTTPS only |
| `.http_only()` | No JavaScript access |
| `.same_site(policy)` | CSRF protection |

### Middleware Trait

```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, req: &mut Request) -> Result<()> { Ok(()) }
    async fn after(&self, res: &mut Response) -> Result<()> { Ok(()) }
}
```

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test file
cargo test --test integration_test

# Test a specific example
cargo run --example basic_server &
curl http://127.0.0.1:3000/
```

### Example Test

```rust
#[tokio::test]
async fn test_cookie_authentication() {
    let router = Router::new()
        .get("/protected", handler_fn(protected_route));

    // Test without cookie
    let req = Request::new(Method::GET, "/protected".parse().unwrap(), /* ... */);
    let response = router.handle(req).await;
    assert!(response.is_err()); // Should be unauthorized

    // Test with valid cookie
    let mut req_with_cookie = Request::new(/* ... */);
    req_with_cookie.headers.insert("Cookie", "session=valid_session".parse().unwrap());
    let response = router.handle(req_with_cookie).await.unwrap();
    assert_eq!(response.status, StatusCode::OK);
}
```

## 🎯 Performance

- **Zero-copy**: Efficient request/response handling with minimal allocations
- **Async**: Built on Tokio for excellent concurrency
- **Lightweight**: Minimal overhead compared to larger frameworks
- **Fast compilation**: Small dependency tree for quick builds
- **Memory efficient**: Smart use of Arc and shared state

## 🔒 Security Features

- **Path traversal protection** in static file serving
- **CORS middleware** for cross-origin requests
- **Authentication middleware** with configurable paths
- **Security headers middleware** (CSP, XSS protection, etc.)
- **Rate limiting middleware** to prevent abuse
- **Secure cookie attributes** (HttpOnly, Secure, SameSite)
- **Session management** with proper invalidation

## 🛠️ Development

### Project Structure

```
mini-web-framework/
├── src/
│   ├── lib.rs              # Main library exports
│   ├── router/             # Routing system with wildcards
│   ├── middleware/         # Middleware implementations
│   ├── request/            # Request handling with cookies
│   ├── response/           # Response building with cookies
│   ├── handler/            # Handler traits and functions
│   ├── server/             # HTTP server
│   ├── cookie/             # Built-in cookie management
│   ├── error/              # Error types
│   └── utils/              # Utility functions
├── examples/               # 8+ comprehensive examples
│   ├── basic_server.rs
│   ├── middleware_example.rs
│   ├── cookie_framework_example.rs
│   ├── login_example.rs
│   ├── login_with_middleware_example.rs
│   ├── custom_middleware_example.rs
│   ├── json_api.rs
│   └── file_server.rs
├── tests/                  # Integration tests
└── README.md
```

### Building and Testing

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run all examples
find examples -name "*.rs" -exec basename {} .rs \; | xargs -I {} cargo run --example {}

# Run tests with coverage
cargo test

# Check code style
cargo fmt
cargo clippy

# Generate documentation
cargo doc --open
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Guidelines

1. **Code Quality**: Run `cargo fmt` and `cargo clippy` before submitting
2. **Tests**: Add tests for new features
3. **Documentation**: Update README and add doc comments
4. **Examples**: Provide examples for new features

### Development Setup

```bash
git clone https://github.com/AarambhDevHub/mini-web-framework.git
cd mini-web-framework
cargo build
cargo test --all
```

## 📝 Changelog

### v0.1.0 - Current Release
- ✅ Basic routing with parameters and wildcards
- ✅ Comprehensive middleware system
- ✅ Built-in cookie management with security features
- ✅ Authentication and authorization examples
- ✅ Static file serving with security
- ✅ JSON API support
- ✅ 8+ comprehensive examples
- ✅ Rate limiting and custom middleware
- ✅ Session-based authentication
- ✅ Role-based access control
- ✅ Request logging and security headers

## 🙏 Acknowledgments

- Built on [Hyper](https://hyper.rs/) for HTTP handling
- Uses [Tokio](https://tokio.rs/) for async runtime
- Inspired by modern web frameworks like Express.js and Actix-web

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ☕ Support the Project
If you find this project helpful, consider buying me a coffee!
[Buy Me a Coffee](https://buymeacoffee.com/aarambhdevhub)


## 🚀 Getting Started

Ready to build something awesome? Check out our comprehensive examples:

```bash
git clone https://github.com/AarambhDevHub/mini-web-framework.git
cd mini-web-framework

# Try the basic server
cargo run --example basic_server

# Test authentication system
cargo run --example login_with_middleware_example

# Explore cookie functionality
cargo run --example cookie_framework_example
```

Visit the running servers and start building! 🎉

### Quick Links
- 🏠 **Home**: `http://127.0.0.1:3000/`
- 🍪 **Cookies**: `http://127.0.0.1:3006/`
- 🔐 **Login**: `http://127.0.0.1:3008/`
- 🛡️ **Middleware**: `http://127.0.0.1:3009/`

***

**Built with ❤️ in Rust** - A powerful, secure, and developer-friendly web framework for modern applications.
