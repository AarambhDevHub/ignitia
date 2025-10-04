# 🔥 Ignitia

<div align="center">

<img src="https://raw.githubusercontent.com/AarambhDevHub/ignitia/main/assets/ignitia-logo.png" alt="Ignitia Logo" width="200">

**A blazing fast, lightweight web framework for Rust**

[![Crates.io](https://img.shields.io/crates/v/ignitia.svg)](https://crates.io/crates/ignitia)
[![Documentation](https://docs.rs/ignitia/badge.svg)](https://docs.rs/ignitia)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

*Ignite your development journey with performance and elegance*

[Getting Started](#-quick-start) • [Documentation](#-documentation) • [Examples](#-examples) • [Benchmarks](#-performance)

<img src="https://raw.githubusercontent.com/AarambhDevHub/ignitia/main/assets/ignitia-banner.png" alt="Ignitia Banner" width="100%">

</div>

---

## ✨ What's New in v0.2.4

🚀 **Major Performance Improvements**
- **94% Latency Reduction**: Middleware chain optimization (30ms → 1.9ms)
- **40% Memory Reduction**: Optimized body handling with direct `Bytes` usage
- **51,574 req/sec**: Production-ready throughput competing with Axum

🎯 **Breaking Changes**
- Radix-only routing (removed Base router mode)
- Unified `Json<T>` type for both extraction and response
- Optimized body types (`Arc<Bytes>` → `Bytes`)

🎨 **Enhanced Developer Experience**
- New `IntoResponse` trait for ergonomic responses
- Support for returning primitives (bool, numbers, Option, Result)
- Comprehensive documentation with examples

See [CHANGELOG.md](CHANGELOG.md) for complete details and migration guide.

---

## 📋 Table of Contents

- [Features](#-features)
- [Quick Start](#-quick-start)
- [Core Concepts](#-core-concepts)
- [Performance](#-performance)
- [Documentation](#-documentation)
- [Examples](#-examples)
- [Middleware](#-middleware)
- [State Management](#-state-management)
- [Error Handling](#-error-handling)
- [Feature Flags](#-feature-flags)
- [Migration from v0.2.3](#-migration-from-v023)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🎯 Features

### Core Features
- 🚀 **Blazing Fast**: High-performance Radix tree routing with O(log n) lookup
- 🔥 **Zero-Cost Abstractions**: Compile-time optimizations with minimal runtime overhead
- 🎨 **Ergonomic API**: Intuitive builder pattern inspired by Axum and Actix
- 🛡️ **Type-Safe**: Leverage Rust's type system for compile-time guarantees
- 🔌 **Middleware**: Composable middleware system with `from_fn` helper
- 📦 **State Management**: Built-in support for shared application state
- 🔒 **Secure by Default**: HTTPS/TLS, CORS, security headers, and rate limiting
- 🌐 **WebSocket Support**: Full-featured WebSocket implementation
- 📝 **Rich Extractors**: Path, Query, Json, Form, Headers, Cookies, State

### v0.2.4 Enhancements
- ✨ **IntoResponse Trait**: Return any type that implements `IntoResponse`
- 🎯 **Unified Json**: Single `Json<T>` type for both input and output
- 🚄 **Performance**: 94% latency reduction through middleware optimization
- 📚 **Documentation**: Comprehensive rustdoc coverage with examples
- 🔧 **Simplified API**: Radix-only routing, infallible response builders

---

## 🚀 Quick Start

### Installation

Add Ignitia to your `Cargo.toml`:

```toml
[dependencies]
ignitia = "0.2.4"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Hello World

```rust
use ignitia::{Router, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple router
    let router = Router::new()
        .get("/", || async { "Hello, World! 🔥" });

    // Start the server
    let addr = SocketAddr::from((, 3000));
    println!("🔥 Server running on http://{}", addr);

    Server::new(router, addr).ignitia().await?;
    Ok(())
}
```

### RESTful API Example

```rust
use ignitia::{Router, Json, Path, Query, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    page: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        // List users with pagination
        .get("/users", |Query(params): Query<SearchParams>| async move {
            let page = params.page.unwrap_or(1);
            Json(vec![
                User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
                User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
            ])
        })
        // Get user by ID
        .get("/users/{id}", |Path(id): Path<u32>| async move {
            Json(User {
                id,
                name: "Alice".into(),
                email: "alice@example.com".into(),
            })
        })
        // Create new user
        .post("/users", |Json(user): Json<CreateUser>| async move {
            Json(User {
                id: 1,
                name: user.name,
                email: user.email,
            })
        })
        // Update user
        .put("/users/{id}", |Path(id): Path<u32>, Json(user): Json<CreateUser>| async move {
            Json(User {
                id,
                name: user.name,
                email: user.email,
            })
        })
        // Delete user
        .delete("/users/{id}", |Path(id): Path<u32>| async move {
            format!("Deleted user {}", id)
        });

    let addr = SocketAddr::from((, 3000));
    println!("🔥 API running on http://{}", addr);

    Server::new(router, addr).ignitia().await?;
    Ok(())
}
```

---

## 🧠 Core Concepts

### IntoResponse - New in v0.2.4

Return any type that implements `IntoResponse`:

```rust
use ignitia::{Router, Json, StatusCode};
use http::StatusCode;

let router = Router::new()
    // Simple string
    .get("/", || async { "Hello!" })

    // With status code
    .post("/users", || async {
        (StatusCode::CREATED, "User created")
    })

    // JSON response
    .get("/api/data", || async {
        Json(serde_json::json!({ "status": "ok" }))
    })

    // Boolean (returns JSON true/false)
    .get("/health", || async { true })

    // Numbers (returns as text/plain)
    .get("/count", || async { 42 })

    // Option (auto 404 if None)
    .get("/find/{id}", |Path(id): Path<u32>| async move {
        database.find(id).await.map(Json)
    })

    // Result with automatic error handling
    .get("/users", || async {
        let users = fetch_users().await?;
        Ok::<_, Error>(Json(users))
    });
```

### Radix Tree Routing

**v0.2.4: Exclusively uses high-performance Radix tree routing**

```rust
use ignitia::Router;

let router = Router::new()
    // Static routes
    .get("/", home)
    .get("/about", about)

    // Path parameters
    .get("/users/{id}", get_user)
    .get("/posts/{slug}", get_post)

    // Multiple parameters
    .get("/users/{user_id}/posts/{post_id}", get_user_post)

    // Wildcard routes
    .get("/files/{*path}", serve_file);
```

**Performance**: O(log n) lookup, zero allocations, 51,574 req/sec

### Extractors

Type-safe request data extraction:

```rust
use ignitia::{Path, Query, Json, State, Headers};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserParams {
    id: u32,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    page: u32,
}

#[derive(Deserialize)]
struct CreateData {
    name: String,
}

// Multiple extractors (up to 16)
async fn handler(
    Path(params): Path<UserParams>,
    Query(search): Query<SearchQuery>,
    Json(data): Json<CreateData>,
    State(db): State<Database>,
    headers: Headers,
) -> Json<Response> {
    // Use extracted data
    Json(response)
}
```

### Nested Routers

Organize routes by feature:

```rust
// API v1 routes
let api_v1 = Router::new()
    .get("/users", list_users_v1)
    .post("/users", create_user_v1);

// API v2 routes
let api_v2 = Router::new()
    .get("/users", list_users_v2)
    .post("/users", create_user_v2);

// Main router
let app = Router::new()
    .get("/", home)
    .nest("/api/v1", api_v1)
    .nest("/api/v2", api_v2);
```

---

## ⚡ Performance

### Benchmarks (v0.2.4)

**Test Setup**: `wrk -c100 -d30s http://localhost:3000/`

```
Throughput:    51,574 req/sec
Latency:       1.90ms avg, 10.60ms max
Transfer:      7.97 MB/sec
Consistency:   70.54% within 1 std dev
```

### Performance Improvements (v0.2.4)

| Metric | v0.2.3 | v0.2.4 | Improvement |
|--------|--------|--------|-------------|
| Avg Latency | 30ms | 1.9ms | **94%** ↓ |
| Allocations | baseline | -40% | **40%** ↓ |
| Throughput | ~45K RPS | 51.5K RPS | **14.5%** ↑ |

### Optimizations

- **Middleware Chain**: Pre-compiled at startup, lock-free runtime access
- **Body Handling**: Direct `Bytes` usage, zero-copy for static content
- **Route Matching**: Radix tree with O(log n) lookup, no regex compilation
- **Response Building**: Infallible builders, inline optimizations

### Comparison with Other Frameworks

| Framework | Throughput | Latency (avg) |
|-----------|------------|---------------|
| **Ignitia v0.2.4** | 51,574 RPS | 1.90ms |
| Axum | ~50-70K RPS | ~2-3ms |
| Actix Web | ~60-80K RPS | ~1-2ms |
| Rocket | ~40-50K RPS | ~3-4ms |

*Competitive with industry-leading frameworks while maintaining ease of use.*

---

## 📚 Documentation

### Comprehensive Guides

- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Routing Guide](docs/ROUTING_GUIDE.md)** - Advanced routing patterns
- **[Response Guide](docs/RESPONSES.md)** - Response types and IntoResponse
- **[Middleware Guide](docs/MIDDLEWARE_GUIDE.md)** - Custom middleware creation
- **[State Management](docs/STATE_GUIDE.md)** - Sharing state across handlers
- **[Error Handling](docs/ERROR_GUIDE.md)** - Error types and handling
- **[WebSocket Guide](docs/WEBSOCKET_GUIDE.md)** - Real-time communication
- **[TLS/HTTPS Guide](docs/TLS_GUIDE.md)** - Secure connections

### Online Documentation

- [docs.rs](https://docs.rs/ignitia) - Comprehensive API documentation
- [Examples](examples/) - Working code examples
- [CHANGELOG.md](CHANGELOG.md) - Version history and migration guides

---

## 💡 Examples

### With Middleware

```rust
use ignitia::{Router, middleware::{LoggerMiddleware, CorsMiddleware}};

let router = Router::new()
    .middleware(LoggerMiddleware::new())
    .middleware(CorsMiddleware::new())
    .get("/api/users", list_users);
```

### Custom Middleware with `from_fn`

```rust
use ignitia::middleware::from_fn;

let auth = from_fn(|req, next| async move {
    if let Some(token) = req.headers.get("Authorization") {
        if verify_token(token).await {
            return next.run(req).await;
        }
    }
    Response::new(StatusCode::UNAUTHORIZED)
});

let router = Router::new()
    .middleware(auth)
    .get("/protected", secret_handler);
```

### State Management

```rust
use std::sync::Arc;
use ignitia::{Router, State};

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

async fn get_user(
    Path(id): Path<u32>,
    State(state): State<AppState>
) -> Json<User> {
    let user = state.db.find_user(id).await;
    Json(user)
}

let state = AppState {
    db: Arc::new(Database::connect().await),
};

let router = Router::new()
    .state(state)
    .get("/users/{id}", get_user);
```

### WebSocket Support

```rust
#[cfg(feature = "websocket")]
use ignitia::websocket::{WebSocketConnection, Message};

#[cfg(feature = "websocket")]
let router = Router::new()
    .websocket("/ws", |mut ws: WebSocketConnection| async move {
        while let Some(msg) = ws.recv().await {
            match msg {
                Message::Text(text) => {
                    ws.send_text(format!("Echo: {}", text)).await?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    });
```

### TLS/HTTPS

```rust
#[cfg(feature = "tls")]
use ignitia::Server;

let router = Router::new()
    .get("/", || async { "Secure!" });

Server::new(router, addr)
    .enable_https("cert.pem", "key.pem")?
    .ignitia()
    .await?;
```

---

## 🔧 Middleware

### Built-in Middleware

```rust
use ignitia::{
    Router,
    middleware::{
        LoggerMiddleware,
        CorsMiddleware,
        RateLimitingMiddleware,
        SecurityMiddleware,
        RequestIdMiddleware,
        CompressionMiddleware,
        BodySizeLimitMiddleware,
    }
};

let router = Router::new()
    .middleware(LoggerMiddleware::new())
    .middleware(CorsMiddleware::new())
    .middleware(RateLimitingMiddleware::per_minute(100))
    .middleware(SecurityMiddleware::high_security())
    .middleware(RequestIdMiddleware::new())
    .middleware(CompressionMiddleware::new())
    .middleware(BodySizeLimitMiddleware::new(10 * 1024 * 1024)); // 10MB
```

### Custom Middleware

```rust
use ignitia::{middleware::from_fn, Request, Response, Next};

let custom = from_fn(|req: Request, next: Next| async move {
    println!("Before handler");
    let response = next.run(req).await;
    println!("After handler");
    response
});
```

---

## 🗄️ State Management

Share state across handlers:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use ignitia::{Router, State};

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

let state = AppState {
    db: Arc::new(Database::new()),
    cache: Arc::new(RwLock::new(HashMap::new())),
};

let router = Router::new()
    .state(state)
    .get("/users", |State(state): State<AppState>| async move {
        // Use state.db or state.cache
        Json(users)
    });
```

---

## ❌ Error Handling

### Automatic Error Conversion

```rust
use ignitia::{Error, Result};

async fn handler() -> Result<Json<User>> {
    let user = database.find_user(1).await?;  // ? works!
    Ok(Json(user))
}
```

### Custom Error Types

```rust
use ignitia::IntoResponse;

#[derive(Debug)]
struct MyError(String);

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        Response::new(StatusCode::BAD_REQUEST)
            .with_body(self.0)
    }
}
```

---

## 🎛️ Feature Flags

Enable optional features in your `Cargo.toml`:

```toml
[dependencies]
ignitia = {
    version = "0.2.4",
    features = ["tls", "websocket", "self-signed"]
}
```

### Available Features

- **`tls`** - HTTPS/TLS support with rustls
- **`websocket`** - WebSocket protocol support
- **`self-signed`** - Generate self-signed certificates for development

---

## 🔄 Migration from v0.2.3

### Router Mode Removal

```rust
// ❌ v0.2.3
let router = Router::new()
    .with_mode(RouterMode::Radix);

// ✅ v0.2.4
let router = Router::new();  // Always uses Radix
```

### Body Type Changes

```rust
// ❌ v0.2.3
response.body = Arc::new(Bytes::from(data));

// ✅ v0.2.4
response.body = Bytes::from(data);
```

### Response Builders

```rust
// ❌ v0.2.3
async fn handler() -> Result<Response> {
    Response::json(data)?  // Double Result
}

// ✅ v0.2.4
async fn handler() -> impl IntoResponse {
    Json(data)  // Clean and simple
}
```

### Json Type

```rust
// ❌ v0.2.3
use ignitia::handler::extractor::Json as JsonExtractor;
use ignitia::response::Json as JsonResponse;

// ✅ v0.2.4
use ignitia::Json;  // One type for both!

async fn handler(Json(data): Json<Input>) -> Json<Output> {
    Json(output)
}
```

See [CHANGELOG.md](CHANGELOG.md) for complete migration guide.

---


## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```
# Clone the repository
git clone https://github.com/AarambhDevHub/ignitia.git
cd ignitia

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings
```

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## ☕ Support & Community

If you find Ignitia helpful, consider supporting the project:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/aarambhdevhub)

---

## 🙏 Acknowledgments

Design patterns and inspiration from:
- **[Axum](https://github.com/tokio-rs/axum)** - Middleware patterns, extractors, IntoResponse
- **[Actix Web](https://github.com/actix/actix-web)** - Performance optimizations
- **[Rocket](https://github.com/SergioBenitez/Rocket)** - Response type ergonomics

Built with ❤️ using:
- [Tokio](https://tokio.rs/) - Async runtime
- [Hyper](https://hyper.rs/) - HTTP implementation
- [Serde](https://serde.rs/) - Serialization
- [Tower](https://github.com/tower-rs/tower) - Service abstractions

---

<div align="center">

**🔥 Ignite your development journey with Ignitia! 🔥**

[⭐ Star us on GitHub](https://github.com/AarambhDevHub/ignitia) • [📖 Read the Docs](https://docs.rs/ignitia) • [💬 Join Discussion](https://github.com/AarambhDevHub/ignitia/discussions) • [💬 Discord](https://discord.gg/HDth6PfCnp) • [☕ Support](https://buymeacoffee.com/aarambhdevhub)

</div>
