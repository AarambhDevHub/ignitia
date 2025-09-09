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
│  │ - Connection   │ - Handler dispatch│ - CORS               │  │
│  │ management     │ - WebSocket       │ - Authentication     │  │
│  │ - HTTP/WS      │   support         │ - Error handling     │  │
│  │   processing   │ - Nested routers  │ - Custom middleware  │  │
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
│  │   routing    │  │ parsing     │  │ - Streaming            │  │
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
│  │  │             │    processing   │  │ - JSON serialization│  ││
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

## Core Components

### 1. Server (`src/server/`)
The server module handles TCP connections, HTTP/1.1 protocol processing, and WebSocket upgrades. It uses Tokio for async I/O and Hyper for HTTP implementation.

**Key Features:**
- Connection pooling and management
- Request size limiting (10MB max)
- WebSocket upgrade handling
- Graceful error handling

### 2. Router (`src/router/`)
The router implements efficient route matching with parameter extraction and supports nested routers.

**Key Features:**
- Fast path parameter extraction
- Method-based routing
- Middleware support
- WebSocket route registration
- Nested routers with prefix support

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

### 4. Middleware System (`src/middleware/`)
Middleware provides cross-cutting concerns for request processing.

**Built-in Middleware:**
- `LoggerMiddleware` - Request/response logging
- `CorsMiddleware` - CORS headers
- `AuthMiddleware` - Authentication
- `ErrorHandlerMiddleware` - Custom error handling

### 5. WebSocket Support (`src/websocket/`)
Full WebSocket implementation with efficient message processing.

**Features:**
- RFC-compliant handshake protocol
- Message batching for high throughput
- Ping/Pong heartbeat support
- Type-safe message handling
- Connection management

### 6. Error Handling (`src/error/`)
Comprehensive error handling with customizable error responses.

**Error Types:**
- HTTP status code mapping
- Custom error definitions
- Structured error responses
- Error conversion utilities

## Getting Started

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ignitia = { version = "0.1.7", features = ["websocket"] }
```

### Basic Example

```rust
use ignitia::{Router, Server, get, Json};
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

async fn create_user(Json(payload): Json<CreateUser>) -> ignitia::Result<Json<User>> {
    let user = User {
        id: 1,
        name: payload.name,
        email: payload.email,
    };
    Ok(Json(user))
}

async fn get_user(Path(id): Path<u64>) -> ignitia::Result<Json<User>> {
    let user = User {
        id,
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    Ok(Json(user))
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .post("/users", create_user)
        .get("/users/:id", get_user);

    let server = Server::new(router, "127.0.0.1:8080".parse().unwrap());
    server.ignitia().await.unwrap();
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
async fn main() {
    let router = Router::new()
        .websocket_fn("/chat", chat_handler);

    let server = Server::new(router, "127.0.0.1:8080".parse().unwrap());
    server.ignitia().await.unwrap();
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

| Test Case          | Ignitia (RPS) | Actix-web (RPS) | Axum (RPS) | Improvement vs Actix | Improvement vs Axum |
|--------------------|---------------|-----------------|------------|----------------------|---------------------|
| Hello World        | 25,274.7      | 25,122.8        | 9,244.7    | +0.6%                | +173.4%             |
| JSON Serialization | 19,845.9      | 17,882.6        | 6,545.1    | +11.0%               | +203.2%             |
| Parameter Extraction | 18,342.1    | 16,789.3        | 5,892.4    | +9.3%                | +211.4%             |
| Slow Response      | 46.4          | 46.6            | 46.3       | -0.4%                | +0.2%               |

## Limitations and Known Issues

1. **Not Production Ready**: Missing comprehensive testing, security audits, and production hardening
2. **Limited HTTP/2 Support**: Currently only supports HTTP/1.1
3. **No HTTPS Built-in**: TLS termination must be handled externally
4. **Limited Middleware**: Basic middleware compared to mature frameworks
5. **Documentation Gaps**: Some advanced features lack comprehensive documentation
6. **API Stability**: Breaking changes may occur between minor versions

## Contributing

We welcome contributions! Please see our contributing guidelines for details on how to submit pull requests, report issues, and suggest features.

## License

Ignitia is licensed under the MIT License. See LICENSE file for details.

## Support

For support and questions:
- GitHub Issues: https://github.com/AarambhDevHub/ignitia/issues
- Documentation: https://docs.rs/ignitia

---

**Remember**: Ignitia is currently experimental software. Use with caution and not in production environments until a stable release is available.
