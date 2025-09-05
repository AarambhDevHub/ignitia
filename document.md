# Ignitia Web Framework Documentation

**Version:** 0.1.2
**Built by:** Aarambh Dev Hub
**Last Updated:** Sep 5, 2025

***

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

***

## Introduction

Ignitia is a blazing fast, lightweight web framework for Rust that embodies the spirit of **Aarambh** (new beginnings). Built on top of Hyper and Tokio, Ignitia provides developers with a powerful yet simple toolkit for building modern web applications.

### Philosophy

Ignitia follows these core principles:

- **Performance First**: Zero-copy operations and minimal allocations
- **Developer Experience**: Intuitive APIs and comprehensive error messages
- **Type Safety**: Leveraging Rust's type system for compile-time correctness
- **Security**: Built-in protection against common web vulnerabilities
- **Extensibility**: Composable middleware and modular architecture with an extension system inspired by Axum

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

***

## Installation & Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

### Adding Ignitia to Your Project

```toml
[dependencies]
ignitia = "0.1.2"
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

***

## Core Concepts

### Server

The `Server` struct is the entry point of your Ignitia application:

```rust
use ignitia::{Server, Router};
use std::net::SocketAddr;

let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
let router = Router::new();
// register routes, middleware, etc

let server = Server::new(router, addr);
server.ignitia().await?;
```

### Router

The `Router` handles request routing and middleware application. Now it provides a **generic handler registration API** that supports **automatic extraction of typed parameters** from the HTTP request, including path parameters, query parameters, JSON bodies, headers, cookies, and extensions — similar to Axum's extractor pattern.

Example usage without the old `handler_fn` pattern (which is now deprecated):

```rust
use ignitia::{Router, Path, Query, Json, Response, Result};
use http::Method;

async fn get_user(Path(user_id): Path<u32>, Query(params): Query<HashMap<String, String>>) -> Result<Response> {
    // Use extracted parameters directly.
    Ok(Response::text(format!("User ID: {}, Params: {:?}", user_id, params)))
}

let router = Router::new()
    .get("/user/:user_id", get_user);
```

### Handlers & Extractors

Handlers are async functions or closures that take zero or more extractor arguments (e.g., `Path<T>`, `Query<T>`, `Json<T>`, `Headers`, `Cookies`, `Extension<T>`) and return a `Result<Response>`. This automatic extraction provides a clean interface:

```rust
async fn create_user(Json(payload): Json<CreateUser>) -> Result<Response> {
    // payload is deserialized JSON body of type CreateUser
    Ok(Response::json(payload)?)
}
```

The framework supports automatic deserialization and validation errors are propagated as route errors.

### Middleware

Middleware provides cross-cutting functionality like logging, authorization, and CORS. Implement the `Middleware` trait with async `before()` and `after()` hooks that modify the request and response.

```rust
#[async_trait::async_trait]
impl Middleware for CustomMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // preprocess request
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // postprocess response
        Ok(())
    }
}
```

***

## API Reference

### Server

#### `Server::new(router: Router, addr: SocketAddr) -> Server`

Creates a new server instance binding to the specified address.

#### `async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>>`

Starts the HTTP server and listens for incoming connections.

***

### Router

#### Construction and Route Registration

```rust
pub fn new() -> Self
```

Creates an empty router.

You register routes with type-safe handlers directly:

```rust
pub fn get<H, T>(self, path: &str, handler: H) -> Self
where
    H: IntoHandler<T>,
```

Similarly for `.post()`, `.put()`, `.delete()`, `.patch()`.

Handlers can have from 0 to 8 extractor arguments, supported by the built-in `IntoHandler` trait implementations.

#### Middleware Registration

```rust
pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self
```

Add middleware to the router, which runs before and after handlers.

#### 404 Handler

```rust
pub fn not_found<H, T>(mut self, handler: H) -> Self
where
    H: IntoHandler<T>,
```

Assign a custom handler for unmatched routes.

***

### Request

Properties:

```rust
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub extensions: Extensions,
}
```

Methods:

- `param(&self, key: &str) -> Option<&String>` — get route path parameter
- `query(&self, key: &str) -> Option<&String>` — get query parameter
- `header(&self, key: &str) -> Option<&str>` — get header value
- `json<T: DeserializeOwned>(&self) -> Result<T>` — parse JSON body
- `cookies(&self) -> CookieJar` — get cookies
- `cookie(&self, name: &str) -> Option<String>` — get specific cookie
- Extension APIs:
  - `insert_extension<T: Send + Sync + 'static>(&mut self, value: T)`
  - `get_extension<T: Send + Sync + Clone + 'static>(&self) -> Option<Arc<T>>`
  - `remove_extension<T: Send + Sync + 'static>(&mut self) -> Option<T>`
  - `has_extension<T: Send + Sync + 'static>(&self) -> bool`

***

### Response

### Constructors

- `Response::text(content: impl Into<String>) -> Response`
- `Response::html(content: impl Into<String>) -> Response`
- `Response::json<T: Serialize>(data: T) -> Result<Response>`

The `ResponseBuilder` offers a fluent API:

```rust
ResponseBuilder::new()
    .status(StatusCode::OK)
    .header("X-Custom", "value")
    .json(&data)?
    .build();
```

### Cookie Support

- `add_cookie(self, cookie: Cookie) -> Self`
- `add_cookies(self, cookies: Vec<Cookie>) -> Self`
- `remove_cookie(self, name: impl Into<String>) -> Self`

***

### Cookie

Construct cookies with builder pattern and detailed attributes:

```rust
let cookie = Cookie::new("session_id", "abc123")
    .path("/")
    .http_only()
    .secure()
    .same_site(SameSite::Strict);
```

Remove cookies by expiration:

```rust
Response::new().remove_cookie("session_id");
```

***

## Routing System

### Route Patterns

- **Static**: `/home`, `/about`
- **Path parameters**: `/users/:id`
- **Wildcards**: `/static/*filepath`

### Route Matching Priority

1. Exact static paths
2. Path parameters
3. Wildcards

### Parameter Extraction Example

```rust
async fn user_handler(Path(user_id): Path<u32>, Query(params): Query<HashMap<String, String>>) -> Result<Response> {
    Ok(Response::text(format!("User: {}, Query: {:?}", user_id, params)))
}
```

***

## Middleware Architecture

Middleware hooks run:

- In registration order before handlers (`before`)
- In reverse order after handlers (`after`)

Common middleware:

- `LoggerMiddleware`
- `CorsMiddleware`
- `AuthMiddleware` (token-based guard)

Custom middleware can be easily authored with `before` and `after` async hooks.

***

## Cookie Management

Ignitia exposes a typed `Cookie` and `CookieJar` for managing HTTP cookies, including parsing from requests and adding to responses with attributes like `HttpOnly`, `Secure`, and `SameSite`.

***

## Request & Response Handling

The request abstracts multiple parts of HTTP requests, offering typed extraction. Response supports flexible content types with builder APIs.

***

## Authentication & Authorization

Path-based authentication is implemented in `AuthMiddleware`. Other auth schemes can be layered with custom middleware and extensions.

Example:

```rust
.middleware(AuthMiddleware::new("secret-token").protect_path("/admin"))
```

***

## Error Handling

Unified `Error` enum with status code mapping. Custom error handlers and middleware can format error responses.

***

## Testing

Provides asynchronous unit and integration testing support, including middleware tests.

***

## Advanced Topics

- Custom extensions to share data in request lifecycle.
- Database connection middleware.
- WebSocket upgrade handling outline.
- Background task queue pattern.
- Structured logging integration.
- Health check endpoints.

***

## Best Practices

- Use typed extractors for clean handlers.
- Structure middleware logically.
- Validate input strictly and early.
- Use security headers and CSRF protections.
- Profile and compress responses depending on size and client support.

***

## Troubleshooting

Tips on addressing common server errors, debugging request/response, and mitigating memory leaks.

***

## Migration Guide

### From Older Versions (pre-0.1.1) and from Other Frameworks

- Deprecated `handler_fn` usage replaced by generic handler registration supporting typed extractors like `Path<T>`, `Query<T>`, `Json<T>`, `Extension<T>`.
- Example migration from old `handler_fn`:

```rust
// Old
router.get("/", handler_fn(home));

// New
router.get("/", home);  // where `home` accepts typed extractor args
```

- Migration examples from Actix and Rocket included.

***

For code examples, API docs, support, and community:

- **GitHub:** https://github.com/AarambhDevHub/ignitia
- **Docs.rs:** https://docs.rs/ignitia
- **YouTube:** https://youtube.com/@AarambhDevHub
- **Issues:** https://github.com/AarambhDevHub/ignitia/issues

***

**Built with ❤️ by Aarambh Dev Hub — where every line of code ignitias possibilities.**
