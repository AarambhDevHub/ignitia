# Ignitia Web Framework Documentation

## 🚨 Important Notice

**Ignitia is currently in development and is NOT ready for production use.** This framework is an experimental project designed for learning and exploration of web framework architecture in Rust. Use in production environments is strongly discouraged until a stable release is published.

## Overview

Ignitia is a high-performance, lightweight web framework for Rust that focuses on developer ergonomics and blazing-fast request processing. The framework takes its name from the Latin word for "spark" or "fire," reflecting its goal to ignite rapid development while maintaining exceptional performance.

## 🏗️ Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                       Ignitia Framework                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Server    │  │   Router    │  │      Middleware         │  │
│  │             │◄─┤             │◄─┤                         │  │
│  │ - TCP Listener │ - Route matching  │ - Logger              │  │
│  │ - TLS/HTTPS    │ - Handler dispatch│ - CORS               │  │
│  │ - HTTP/1.1+2   │ - WebSocket       │ - Authentication     │  │
│  │ - Connection   │   support         │ - Error handling     │  │
│  │   management   │ - Nested routers  │ - Custom middleware  │  │
│  │ - Protocol     │ - Parameter       │ - Rate limiting      │  │
│  │   detection    │   extraction      │                      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│         │                │                         │            │
│         ▼                ▼                         ▼            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Handler   │  │  Request    │  │       Response          │  │
│  │   System    │  │  Processing │  │       Builder           │  │
│  │             │─►│             │─►│                         │  │
│  │ - Extractors │  │ - Params    │  │ - JSON/Text/HTML       │  │
│  │ - Async      │  │ - Query     │  │ - Status codes         │  │
│  │   handlers   │  │ - Headers   │  │ - Headers              │  │
│  │ - Type-safe  │  │ - Body      │  │ - Cookies              │  │
│  │   routing    │  │   parsing   │  │ - Streaming            │  │
│  │ - Extensions │  │ - Cookies   │  │ - Error responses      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│         │                │                         │            │
│         └────────────────┴─────────────────────────┘            │
│                                 │                               │
│                                 ▼                               │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    WebSocket System                        ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  ││
│  │  │  Upgrade    │  │ Connection  │  │    Message          │  ││
│  │  │  Handler    │  │  Management │  │    Processing       │  ││
│  │  │             │─►│             │─►│                     │  ││
│  │  │ - WS handshake│ - Message     │  │ - Text/Binary       │  ││
│  │  │ - Protocol   │    routing     │  │ - Ping/Pong        │  ││
│  │  │   negotiation│ - Batch        │  │ - Close frames     │  ││
│  │  │ - Security   │    processing   │  │ - JSON serialization│  ││
│  │  │   validation │ - Heartbeat    │  │ - Custom handlers   │  ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  ││
│  └─────────────────────────────────────────────────────────────┘│
│                                 │                               │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                      TLS System                            ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  ││
│  │  │ Certificate │  │    ALPN     │  │    Protocol         │  ││
│  │  │ Management  │  │ Negotiation │  │    Detection        │  ││
│  │  │             │─►│             │─►│                     │  ││
│  │  │ - PEM format │  │ - HTTP/1.1  │  │ - Auto detection    │  ││
│  │  │ - Key loading│  │ - HTTP/2    │  │ - TLS handshake     │  ││
│  │  │ - Self-signed│  │ - WebSocket │  │ - Cipher suites     │  ││
│  │  │   generation │  │   support   │  │ - Version support   │  ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Performance Analysis

Based on benchmark comparisons with major Rust web frameworks:

### Request Processing Speed (RPS)
```
Ignitia:   19,845.9 RPS (average)
Actix-web: 17,882.6 RPS (average)
Axum:       6,545.1 RPS (average)
```

Ignitia demonstrates **~11% better performance** than Actix-web and **~203% better performance** than Axum in request processing throughput.

### Response Times
```
Framework  | Average  | Best    | Worst
-----------------------------------------
Ignitia    | 12.06ms  | 0.13ms  | 255.83ms
Actix-web  | 13.30ms  | 0.15ms  | 207.72ms
Axum       | 24.18ms  | 0.26ms  | 414.00ms
```

Ignitia shows the lowest average response time and excellent consistency across different request types.

### Key Performance Features

1. **ArcSwap for Atomic Router Updates**: Enables hot-reloading of routes without downtime
2. **Optimized Regex Matching**: Pre-compiled route patterns with specificity sorting
3. **Efficient Memory Management**: Minimal cloning and borrowing where possible
4. **Batch WebSocket Processing**: Reduced overhead for high-frequency messaging
5. **Selective Regex Escaping**: Faster route compilation without unnecessary escaping
6. **HTTP/2 Support**: Native HTTP/2 with H2C (cleartext) support
7. **TLS Optimization**: Efficient ALPN negotiation and protocol detection

## Core Components

### 1. Server (`src/server/`)
The server module handles TCP connections, HTTP/1.1 and HTTP/2 protocol processing, TLS/HTTPS, and WebSocket upgrades.

**Key Features:**
- HTTP/1.1 and HTTP/2 support
- TLS/HTTPS with ALPN negotiation
- Connection pooling and management
- Request size limiting (10MB max)
- WebSocket upgrade handling
- Graceful error handling
- Self-signed certificate generation (development)
- HTTP to HTTPS redirects

**TLS Features:**
- Certificate and private key loading (PEM format)
- ALPN protocol negotiation
- TLS v1.2 and v1.3 support
- Client certificate verification
- Self-signed certificate generation for development

### 2. Router (`src/router/`)
The router implements efficient route matching with parameter extraction and supports nested routers.

**Key Features:**
- Fast path parameter extraction
- Method-based routing
- Middleware support per route and globally
- WebSocket route registration
- Nested routers with prefix support
- Wildcard parameter support
- Route compilation and caching

### 3. Handler System (`src/handler/`)
Provides a type-safe way to define request handlers with automatic parameter extraction.

**Extractors Available:**
- `Path<T>` - URL path parameters
- `Query<T>` - Query string parameters
- `Json<T>` - JSON request bodies
- `Headers` - HTTP headers
- `Cookies` - Cookie data
- `Body` - Raw request body
- `Method` - HTTP method
- `Uri` - Request URI
- `Extension<T>` - Custom extensions

### 4. Middleware System (`src/middleware/`)
Middleware provides cross-cutting concerns for request processing.

**Built-in Middleware:**
- `LoggerMiddleware` - Request/response logging
- `CorsMiddleware` - CORS headers with extensive configuration
- `AuthMiddleware` - Bearer token authentication
- `ErrorHandlerMiddleware` - Custom error handling and logging

**CORS Features:**
- Origin validation (exact, regex, wildcard)
- Method and header filtering
- Credential support
- Preflight request handling
- Max-age configuration

### 5. WebSocket Support (`src/websocket/`)
Full WebSocket implementation with efficient message processing.

**Features:**
- RFC-compliant handshake protocol
- Message batching for high throughput
- Ping/Pong heartbeat support
- Type-safe message handling
- Connection management
- JSON message support
- Batch message processing
- Timeout handling

### 6. Error Handling (`src/error/`)
Comprehensive error handling with customizable error responses.

**Error Types:**
- HTTP status code mapping
- Custom error definitions
- Structured error responses
- Error conversion utilities
- TLS-specific errors
- WebSocket errors
- Validation errors

**Custom Error Support:**
- `define_error!` macro for easy custom error creation
- Error metadata and codes
- Timestamp inclusion
- Request context support

### 7. TLS/HTTPS Support (`src/server/tls.rs`)
Complete TLS implementation for secure connections.

**Features:**
- Certificate and key file loading
- ALPN protocol negotiation
- TLS version configuration
- Self-signed certificate generation
- Client certificate verification
- Error handling and validation

## Feature Flags

Ignitia uses Cargo features to enable optional functionality:

```toml
[features]
default = ["websocket", "self-signed"]
websocket = ["dep:tokio-tungstenite", "dep:tungstenite", "dep:sha1", "dep:base64", "dep:lazy_static"]
tls = ["dep:tokio-rustls", "dep:rustls", "dep:rustls-pemfile"]
self-signed = ["tls", "dep:rcgen"]
```

## Getting Started

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ignitia = { version = "0.1.9", features = ["websocket", "tls"] }
```

### Basic HTTP Example

```rust
use ignitia::{Router, Server, Response, Json, Path};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> ignitia::Result<Response> {
    let user = User {
        id: 1,
        name: payload.name,
        email: payload.email,
    };
    Response::json(user)
}

async fn get_user(Path(id): Path<u64>) -> ignitia::Result<Response> {
    let user = User {
        id,
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    Response::json(user)
}

#[tokio::main]
async fn main() -> ignitia::Result<()> {
    let router = Router::new()
        .post("/users", create_user)
        .get("/users/:id", get_user);

    let server = Server::new(router, "127.0.0.1:8080".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

### HTTPS Example

```rust
use ignitia::{Router, Server, Response};

async fn hello() -> ignitia::Result<Response> {
    Ok(Response::text("Hello HTTPS World!"))
}

#[tokio::main]
async fn main() -> ignitia::Result<()> {
    let router = Router::new()
        .get("/", hello);

    let server = Server::new(router, "127.0.0.1:8443".parse().unwrap())
        .enable_https("cert.pem", "key.pem")?;

    server.ignitia().await?;
    Ok(())
}
```

### Self-Signed Certificate Example (Development)

```rust
use ignitia::{Router, Server, Response};

async fn hello() -> ignitia::Result<Response> {
    Ok(Response::text("Hello Secure World!"))
}

#[tokio::main]
async fn main() -> ignitia::Result<()> {
    let router = Router::new()
        .get("/", hello);

    let server = Server::new(router, "127.0.0.1:8443".parse().unwrap())
        .with_self_signed_cert("localhost")?;

    server.ignitia().await?;
    Ok(())
}
```

### WebSocket Example

```rust
use ignitia::{Router, Server, websocket_handler};
use ignitia::websocket::{WebSocketConnection, Message};

async fn chat_handler(ws: WebSocketConnection) -> ignitia::Result<()> {
    while let Some(message) = ws.recv().await {
        match message {
            Message::Text(text) => {
                println!("Received: {}", text);
                ws.send_text(format!("Echo: {}", text)).await?;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ignitia::Result<()> {
    let router = Router::new()
        .websocket_fn("/chat", chat_handler);

    let server = Server::new(router, "127.0.0.1:8080".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

### Middleware Example

```rust
use ignitia::{Router, Server, Response};
use ignitia::middleware::{LoggerMiddleware, Cors};

async fn hello() -> ignitia::Result<Response> {
    Ok(Response::text("Hello World!"))
}

#[tokio::main]
async fn main() -> ignitia::Result<()> {
    let cors = Cors::new()
        .allowed_origins(&["https://example.com", "https://app.example.com"])
        .allowed_methods(&[ignitia::Method::GET, ignitia::Method::POST])
        .build()?;

    let router = Router::new()
        .middleware(LoggerMiddleware)
        .middleware(cors)
        .get("/", hello);

    let server = Server::new(router, "127.0.0.1:8080".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

## Performance Benchmarks

### Test Environment
- **CPU**: Intel Core i5-1135G7 (11th Gen, Quad Core)
- **RAM**: 8GB DDR4
- **OS**: Ubuntu (latest release)
- **Rust**: 1.75.0
- **Network**: Localhost testing

### Benchmark Results

| Test Case            | Ignitia (RPS) | Actix-web (RPS) | Axum (RPS) | Improvement vs Actix | Improvement vs Axum |
|----------------------|---------------|-----------------|------------|----------------------|---------------------|
| Hello World          | 25,274.7      | 25,122.8        | 9,244.7    | +0.6%                | +173.4%             |
| JSON Serialization   | 19,845.9      | 17,882.6        | 6,545.1    | +11.0%               | +203.2%             |
| Parameter Extraction | 18,342.1      | 16,789.3        | 5,892.4    | +9.3%                | +211.4%             |
| WebSocket Messages   | 23,156.2      | 21,845.7        | 8,934.1    | +6.0%                | +159.1%             |
| HTTPS Requests       | 18,945.3      | 17,234.6        | 6,123.8    | +9.9%                | +209.4%             |
| Slow Response        | 46.4          | 46.6            | 46.3       | -0.4%                | +0.2%               |

## Advanced Features

### Custom Error Handling

```rust
use ignitia::{define_error, Error, CustomError};
use http::StatusCode;

define_error! {
    UserError {
        InvalidCredentials(StatusCode::UNAUTHORIZED, "invalid_credentials", "AUTH_001"),
        AccountLocked(StatusCode::FORBIDDEN, "account_locked", "AUTH_002"),
        ProfileNotFound(StatusCode::NOT_FOUND, "profile_not_found"),
    }
}

async fn login() -> Result<Response, UserError> {
    // Your authentication logic here
    Err(UserError::InvalidCredentials("Invalid username or password".to_string()))
}
```

### Route Parameters and Wildcards

```rust
use ignitia::{Router, Path};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserParams {
    id: u64,
    section: String,
}

#[derive(Deserialize)]
struct FileParams {
    path: String, // This will capture the wildcard
}

async fn get_user_section(Path(params): Path<UserParams>) -> ignitia::Result<Response> {
    Ok(Response::text(format!("User {} section {}", params.id, params.section)))
}

async fn serve_file(Path(params): Path<FileParams>) -> ignitia::Result<Response> {
    Ok(Response::text(format!("Serving file: {}", params.path)))
}

let router = Router::new()
    .get("/users/:id/:section", get_user_section)
    .get("/files/*path", serve_file); // Wildcard route
```

### Extensions and Request Context

```rust
use ignitia::{Router, Request, Extension};
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    // Your database connection
}

async fn middleware_that_adds_db(req: &mut Request) -> ignitia::Result<()> {
    let db = Database { /* initialize */ };
    req.insert_extension(db);
    Ok(())
}

async fn handler_using_db(Extension(db): Extension<Database>) -> ignitia::Result<Response> {
    // Use the database
    Ok(Response::text("Data from database"))
}
```

## Limitations and Known Issues

1. **Not Production Ready**: Missing comprehensive testing, security audits, and production hardening
2. **Limited Documentation**: Some advanced features lack comprehensive documentation
3. **API Stability**: Breaking changes may occur between minor versions
4. **Testing Coverage**: Needs more comprehensive test suite
5. **Security Auditing**: TLS implementation needs security review

## Contributing

We welcome contributions! Please see our contributing guidelines for details on how to submit pull requests, report issues, and suggest features.

### Development Setup

```
git clone https://github.com/AarambhDevHub/ignitia
cd ignitia
cargo build
cargo test
```

### Running Examples

```
# Basic HTTP server
cargo run --example basic

# HTTPS server (requires certificates)
cargo run --example https --features tls

# WebSocket chat server
cargo run --example websocket --features websocket
```

## License

Ignitia is licensed under the MIT License. See LICENSE file for details.

## Support

For support and questions:
- GitHub Issues: https://github.com/AarambhDevHub/ignitia/issues
- Documentation: https://docs.rs/ignitia
- Discord: [Join our community](#)

---

**Remember**: Ignitia is currently experimental software. Use with caution and not in production environments until a stable release is available.
