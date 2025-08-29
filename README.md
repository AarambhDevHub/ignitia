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
ignitia = "0.1.0"
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing-subscriber = "0.3"
```

---

## 🚀 Quick Start

Create your first Ignitia application:

```
use ignitia::{Router, Server, Request, Response, Result, handler_fn};

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", handler_fn(hello_ignitia))
        .get("/users/:id", handler_fn(get_user))
        .post("/api/data", handler_fn(create_data));

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());

    println!("🔥 Igniting server...");
    server.ignitia().await.unwrap(); // Custom ignitia method!
    Ok(())
}

async fn hello_ignitia(_req: Request) -> Result<Response> {
    Ok(Response::html(r#"
        <h1>🔥 Welcome to ignitia!</h1>
        <p>Your web development journey starts here!</p>
    "#))
}

async fn get_user(req: Request) -> Result<Response> {
    let user_id = req.param("id").unwrap_or(&"unknown".to_string());
    Ok(Response::json(serde_json::json!({
        "message": "User ignitiad!",
        "user_id": user_id,
        "framework": "ignitia 🔥"
    }))?)
}

async fn create_data(req: Request) -> Result<Response> {
    let data: serde_json::Value = req.json()?;
    Ok(Response::json(serde_json::json!({
        "status": "success",
        "received": data,
        "ignitiad_at": std::time::SystemTime::now()
    }))?)
}
```

---

## 🔥 Core Features

### **🛣️ Advanced Routing**

```
let router = Router::new()
    // Basic routes
    .get("/", handler_fn(home))
    .post("/api/users", handler_fn(create_user))
    .put("/api/users/:id", handler_fn(update_user))
    .delete("/api/users/:id", handler_fn(delete_user))

    // Route parameters
    .get("/users/:id/posts/:post_id", handler_fn(get_post))

    // Wildcard routes for static files
    .get("/*path", handler_fn(serve_static))

    // Custom 404 handler
    .not_found(handler_fn(not_found));
```

### **🍪 Built-in Cookie Management**

Secure, easy-to-use cookie handling with all security attributes:

```
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
async fn protected_route(req: Request) -> Result<Response> {
    let username = req.cookie("session")
        .unwrap_or("anonymous".to_string());

    Ok(Response::text(format!("Welcome back, {}!", username)))
}

// Remove cookies
let response = Response::text("Logged out")
    .remove_cookie("session");
```

### **🛡️ Powerful Middleware System**

Composable middleware for authentication, logging, CORS, and more:

```
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

    .get("/api/admin/users", handler_fn(admin_users))
    .get("/dashboard", handler_fn(dashboard));
```

### **🔐 Authentication & Authorization**

Built-in session management and role-based access control:

```
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

## 🛣️ Routing

### **Parameter Extraction**

```
// Route: /users/:id/posts/:post_id
async fn get_post(req: Request) -> Result<Response> {
    let user_id = req.param("id").unwrap();
    let post_id = req.param("post_id").unwrap();

    Response::json(serde_json::json!({
        "user_id": user_id,
        "post_id": post_id
    }))
}
```

### **Query Parameters**

```
// URL: /search?q=rust&limit=10&page=1
async fn search(req: Request) -> Result<Response> {
    let query = req.query("q").unwrap_or(&"".to_string());
    let limit = req.query("limit")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(20);
    let page = req.query("page")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);

    // Perform search...
    Response::json(serde_json::json!({
        "query": query,
        "limit": limit,
        "page": page,
        "results": []
    }))
}
```

### **Wildcard Routes**

```
// Serve static files: /*path matches any path
.get("/*path", handler_fn(serve_static))

async fn serve_static(req: Request) -> Result<Response> {
    let path = req.param("path").unwrap();
    // Serve file from static directory with security checks
    serve_file_from_directory("./static", path).await
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

```
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

```
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

```
async fn handle_request(req: Request) -> Result<Response> {
    // Get all cookies
    let cookies = req.cookies();

    // Get specific cookie
    let session = req.cookie("session");

    // Check if cookie exists
    if cookies.contains("user_preferences") {
        // Handle with preferences
    }

    Response::json(cookies.all())
}
```

### **Cookie Security**

```
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

```
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

```
let router = Router::new()
    .middleware(AuthMiddleware::new()
        .protect_paths(vec!["/dashboard", "/admin", "/api/protected"]))

    // These routes are automatically protected
    .get("/dashboard", handler_fn(dashboard))
    .get("/admin", handler_fn(admin_panel))
    .get("/api/protected/data", handler_fn(protected_data));
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

| Method | Description |
|--------|-------------|
| `Router::new()` | Create a new router |
| `.get(path, handler)` | Add GET route |
| `.post(path, handler)` | Add POST route |
| `.put(path, handler)` | Add PUT route |
| `.delete(path, handler)` | Add DELETE route |
| `.middleware(middleware)` | Add middleware |
| `.not_found(handler)` | Set 404 handler |

### **Request**

| Method | Description | Example |
|--------|-------------|---------|
| `req.param(key)` | Get route parameter | `req.param("id")` |
| `req.query(key)` | Get query parameter | `req.query("limit")` |
| `req.header(key)` | Get header value | `req.header("User-Agent")` |
| `req.json<T>()` | Parse JSON body | `req.json::<User>()?` |
| `req.cookies()` | Get all cookies | `req.cookies().all()` |
| `req.cookie(key)` | Get specific cookie | `req.cookie("session")` |

### **Response**

| Method | Description | Example |
|--------|-------------|---------|
| `Response::text(content)` | Text response | `Response::text("Hello")` |
| `Response::html(content)` | HTML response | `Response::html("<h1>Hi</h1>")` |
| `Response::json(data)` | JSON response | `Response::json(user)?` |
| `Response::not_found()` | 404 response | `Response::not_found()` |
| `.add_cookie(cookie)` | Add cookie | `.add_cookie(session)` |
| `.remove_cookie(name)` | Remove cookie | `.remove_cookie("session")` |

### **Cookie Builder**

| Method | Description | Example |
|--------|-------------|---------|
| `Cookie::new(name, value)` | Create cookie | `Cookie::new("user", "john")` |
| `.path(path)` | Set cookie path | `.path("/")` |
| `.max_age(seconds)` | Set expiration | `.max_age(3600)` |
| `.secure()` | HTTPS only | `.secure()` |
| `.http_only()` | No JavaScript access | `.http_only()` |
| `.same_site(policy)` | CSRF protection | `.same_site(SameSite::Lax)` |

### **Middleware Trait**

```
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, req: &mut Request) -> Result<()> { Ok(()) }
    async fn after(&self, res: &mut Response) -> Result<()> { Ok(()) }
}
```

---

## 🧪 Testing

### **Run Tests**

```
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_test

# Run specific test
cargo test test_authentication
```

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

### **Example Test**

```
#[tokio::test]
async fn test_cookie_authentication() {
    let router = Router::new()
        .get("/protected", handler_fn(protected_route));

    // Test without cookie (should fail)
    let req = Request::new(Method::GET, "/protected".parse().unwrap(), /*...*/);
    let response = router.handle(req).await;
    assert!(matches!(response, Err(Error::Unauthorized)));

    // Test with valid cookie (should succeed)
    let mut req_with_cookie = Request::new(/*...*/);
    req_with_cookie.headers.insert(
        "Cookie",
        "session=valid_token".parse().unwrap()
    );
    let response = router.handle(req_with_cookie).await.unwrap();
    assert_eq!(response.status, StatusCode::OK);
}
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

```
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
├── src/
│   ├── lib.rs              # Main library exports
│   ├── router/             # Routing with wildcards
│   │   ├── mod.rs
│   │   ├── route.rs
│   │   └── method.rs
│   ├── middleware/         # Middleware implementations
│   │   ├── mod.rs
│   │   ├── logger.rs
│   │   ├── cors.rs
│   │   └── auth.rs
│   ├── request/            # Request handling with cookies
│   ├── response/           # Response building with cookies
│   ├── handler/            # Handler traits and functions
│   ├── server/             # HTTP server
│   ├── cookie/             # Built-in cookie management
│   ├── error/              # Error types
│   └── utils/              # Utility functions
├── examples/               # 8+ comprehensive examples
│   ├── basic_server.rs
│   ├── login_example.rs
│   ├── cookie_framework_example.rs
│   ├── custom_middleware_example.rs
│   ├── login_with_middleware_example.rs
│   ├── middleware_example.rs
│   ├── json_api.rs
│   └── file_server.rs
├── tests/                  # Integration tests
├── Cargo.toml
└── README.md
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

### **v0.1.0** - Initial Release 🎉

#### **Core Features**
- ✅ Advanced routing with parameters and wildcards
- ✅ Comprehensive middleware system
- ✅ Built-in cookie management with security features
- ✅ Authentication and authorization examples
- ✅ Static file serving with security protections
- ✅ JSON API support with serialization
- ✅ Request/response handling with proper error management

#### **Middleware**
- ✅ Authentication middleware with path protection
- ✅ CORS middleware with configurable origins
- ✅ Request logging middleware
- ✅ Rate limiting middleware
- ✅ Security headers middleware
- ✅ Custom middleware support

#### **Examples**
- ✅ Basic server with routing
- ✅ Complete authentication system
- ✅ Cookie management showcase
- ✅ Custom middleware implementations
- ✅ JSON API with CRUD operations
- ✅ Static file server
- ✅ Middleware integration examples

#### **Security**
- ✅ Secure cookie attributes
- ✅ Path traversal protection
- ✅ Session management
- ✅ CSRF protection
- ✅ XSS prevention

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
ignitia = "0.1.0"
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
