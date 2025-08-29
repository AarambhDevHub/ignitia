# 🚀 Mini Web Framework

A lightweight, fast, and extensible web framework for Rust built on top of Hyper and Tokio. This framework provides all the essential features needed to build modern web applications with minimal overhead and maximum performance.

[![License: MIT](https://img.shields.io/badge/Licenseance**: Built on Hyper and Tokio for maximum async performance
- **🎯 Simple Routing**: Intuitive route definition with parameter and wildcard support
- **🔧 Middleware System**: Composable middleware architecture for cross-cutting concerns
- **📄 Static File Serving**: Built-in static file server with security features
- **🔒 Security**: CORS, authentication, rate limiting, and security headers
- **📊 JSON Support**: First-class JSON serialization and deserialization
- **⚡ Fast Compilation**: Minimal dependencies for quick build times
- **🛡️ Type Safety**: Full Rust type safety throughout the framework
- **📝 Comprehensive Examples**: Multiple real-world usage examples included

## 📋 Table of Contents

- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Core Concepts](#-core-concepts)
- [Routing](#-routing)
- [Middleware](#-middleware)
- [Examples](#-examples)
- [API Reference](#-api-reference)
- [Testing](#-testing)
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
        .get("/users/:id", handler_fn(get_user));

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
```

## 🧭 Core Concepts

### Request & Response

The framework provides simple `Request` and `Response` types:

```rust
// Access request data
let method = req.method;
let path = req.uri.path();
let user_agent = req.header("User-Agent");
let json_data: MyStruct = req.json()?;

// Create responses
Response::text("Plain text")
Response::html("<h1>HTML</h1>")
Response::json(my_data)
Response::not_found()
```

### Error Handling

Built-in error types for common HTTP scenarios:

```rust
use mini_web_framework::{Error, Result};

async fn handler(req: Request) -> Result<Response> {
    let data = req.json::<MyData>()
        .map_err(|_| Error::BadRequest("Invalid JSON".into()))?;

    Ok(Response::json(data)?)
}
```

## 🛣️ Routing

### Basic Routes

```rust
let router = Router::new()
    .get("/", handler_fn(home))
    .post("/users", handler_fn(create_user))
    .put("/users/:id", handler_fn(update_user))
    .delete("/users/:id", handler_fn(delete_user));
```

### Parameters

```rust
// Route: /users/:id/posts/:post_id
async fn get_post(req: Request) -> Result<Response> {
    let user_id = req.param("id").unwrap();
    let post_id = req.param("post_id").unwrap();
    // ...
}
```

### Query Parameters

```rust
// URL: /search?q=rust&limit=10
async fn search(req: Request) -> Result<Response> {
    let query = req.query("q").unwrap_or(&"".to_string());
    let limit = req.query("limit")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(20);
    // ...
}
```

### Wildcard Routes

```rust
let router = Router::new()
    .get("/*path", handler_fn(serve_static_files));

async fn serve_static_files(req: Request) -> Result<Response> {
    let path = req.param("path").unwrap();
    // Serve file from path
}
```

## 🔧 Middleware

### Built-in Middleware

```rust
use mini_web_framework::middleware::{LoggerMiddleware, CorsMiddleware, AuthMiddleware};

let router = Router::new()
    .middleware(LoggerMiddleware)
    .middleware(CorsMiddleware::new())
    .middleware(AuthMiddleware::new("secret-token")
        .protect_path("/api/")
        .protect_path("/admin/"));
```

### Custom Middleware

```rust
use mini_web_framework::{Middleware, async_trait};

struct TimingMiddleware;

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Log request start time
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // Add timing header
        res.headers.insert("X-Response-Time", "42ms".parse().unwrap());
        Ok(())
    }
}
```

## 📚 Examples

### Basic Server
```bash
cargo run --example basic_server
```
A simple HTTP server demonstrating basic routing and JSON responses.

### Middleware Demo
```bash
cargo run --example middleware_example
```
Shows authentication, CORS, and logging middleware in action.

### JSON API
```bash
cargo run --example json_api
```
A RESTful API for managing todos with in-memory storage.

### File Server
```bash
cargo run --example file_server
```
Static file server with security features and MIME type detection.

### Custom Middleware
```bash
cargo run --example custom_middleware_example
```
Advanced middleware examples including rate limiting, request validation, and security headers.

## 🔌 API Reference

### Router Methods

| Method | Description |
|--------|-------------|
| `new()` | Create a new router |
| `get(path, handler)` | Add GET route |
| `post(path, handler)` | Add POST route |
| `put(path, handler)` | Add PUT route |
| `delete(path, handler)` | Add DELETE route |
| `middleware(middleware)` | Add middleware |
| `not_found(handler)` | Set 404 handler |

### Request Methods

| Method | Description |
|--------|-------------|
| `param(key)` | Get route parameter |
| `query(key)` | Get query parameter |
| `header(key)` | Get header value |
| `json<T>()` | Parse JSON body |

### Response Methods

| Method | Description |
|--------|-------------|
| `text(content)` | Create text response |
| `html(content)` | Create HTML response |
| `json(data)` | Create JSON response |
| `not_found()` | Create 404 response |

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
```

### Example Test

```rust
#[tokio::test]
async fn test_json_response() {
    let router = Router::new()
        .get("/user", handler_fn(|_| async {
            Response::json(User { id: 1, name: "Alice".to_string() })
        }));

    let req = Request::new(/* ... */);
    let response = router.handle(req).await.unwrap();

    assert_eq!(response.status, StatusCode::OK);
    let user: User = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(user.name, "Alice");
}
```

## 🎯 Performance

- **Zero-copy**: Efficient request/response handling with minimal allocations
- **Async**: Built on Tokio for excellent concurrency
- **Lightweight**: Minimal overhead compared to larger frameworks
- **Fast compilation**: Small dependency tree for quick builds

## 🔒 Security Features

- **Path traversal protection** in static file serving
- **CORS middleware** for cross-origin requests
- **Authentication middleware** with configurable paths
- **Security headers middleware** (CSP, XSS protection, etc.)
- **Rate limiting middleware** to prevent abuse

## 🛠️ Development

### Project Structure

```
mini-web-framework/
├── src/
│   ├── lib.rs              # Main library exports
│   ├── router/             # Routing system
│   ├── middleware/         # Middleware implementations
│   ├── request/            # Request handling
│   ├── response/           # Response building
│   ├── handler/            # Handler traits and functions
│   ├── server/             # HTTP server
│   ├── error/              # Error types
│   └── utils/              # Utility functions
├── examples/               # Usage examples
├── tests/                  # Integration tests
└── README.md
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run examples
cargo run --example basic_server

# Run tests
cargo test

# Check code style
cargo fmt
cargo clippy
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Guidelines

1. **Code Quality**: Run `cargo fmt` and `cargo clippy` before submitting
2. **Tests**: Add tests for new features
3. **Documentation**: Update README and add doc comments
4. **Examples**: Provide examples for new features

### Development Setup

```bash
git clone https://github.com/yourusername/mini-web-framework.git
cd mini-web-framework
cargo build
cargo test
```

## 📝 Changelog

### v0.1.0
- Initial release
- Basic routing with parameters and wildcards
- Middleware system
- JSON support
- Static file serving
- Comprehensive examples
- Security features

## 🙏 Acknowledgments

- Built on [Hyper](https://hyper.rs/) for HTTP handling
- Uses [Tokio](https://tokio.rs/) for async runtime
- Inspired by modern web frameworks

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🚀 Getting Started

Ready to build something awesome? Check out our [examples](examples/) directory and start with the [basic server example](examples/basic_server.rs)!

```bash
git clone https://github.com/yourusername/mini-web-framework.git
cd mini-web-framework
cargo run --example basic_server
```

Visit `http://127.0.0.1:3000` and start building! 🎉

***

**Built with ❤️ in Rust**
