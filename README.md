# 🔥 Ignitia

<div align="center">

<img src="https://raw.githubusercontent.com/AarambhDevHub/ignitia/main/assets/ignitia-logo.png" alt="Ignitia Logo" width="200">

**A blazing fast, lightweight web framework for Rust that ignites your development journey.**

*Embodies the spirit of **Aarambh** (new beginnings) - the spark that ignites your web development journey*

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

Ignitia embodies the spirit of **Aarambh** (new beginnings) - the spark that ignites your web development journey. Built for developers who demand speed, simplicity, and power with modern protocol support.

- **🚀 Multi-Protocol**: HTTP/1.1, HTTP/2, and HTTPS with automatic protocol negotiation
- **🔒 TLS/HTTPS**: Built-in TLS support with ALPN and self-signed certificates for development
- **🪶 Lightweight**: Minimal overhead, maximum efficiency
- **🔥 Powerful**: Advanced routing, middleware, and WebSocket support
- **⚡ Energetic**: Modern APIs that energize your development
- **🎯 Developer-First**: Clean, intuitive, and productive
- **🛡️ Secure**: Built-in security features and best practices
- **🍪 Cookie Management**: Full-featured cookie handling with security attributes
- **🌐 WebSocket Ready**: First-class WebSocket support with optimized performance

---

## 📋 Table of Contents

- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Core Features](#-core-features)
- [HTTP/2 & HTTPS](#-http2--https)
- [WebSocket Support](#-websocket-support)
- [Routing](#-routing)
- [Middleware](#-middleware)
- [CORS Configuration](#-cors-configuration)
- [Cookie Management](#-cookie-management)
- [Authentication](#-authentication)
- [Examples](#-examples)
- [Performance](#-performance)
- [API Reference](#-api-reference)
- [Contributing](#-contributing)

---

## 🛠️ Installation

Add Ignitia to your `Cargo.toml`:

```
[dependencies]
ignitia = { version = "0.1.8", features = ["tls", "websocket", "self-signed"] }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing-subscriber = "0.3"
```

### Feature Flags

- **`tls`**: Enables HTTPS/TLS support with certificate management and ALPN
- **`websocket`**: Enables WebSocket protocol support with connection management
- **`self-signed`**: Enables self-signed certificate generation (development only)

---

## 🚀 Quick Start

### Basic HTTP/2 + HTTPS Server

```
use ignitia::{
    Router, Server, Response, Result, Http2Config, ServerConfig, TlsConfig,
    handler::extractor::{Path, Query, Json},
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();

    let router = Router::new()
        .get("/", || async {
            Ok(Response::html("<h1>🔥 Welcome to Ignitia with HTTP/2!</h1>"))
        })
        .get("/users/:id", get_user)
        .post("/api/data", create_data)
        .get("/health", || async {
            Ok(Response::json(serde_json::json!({
                "status": "healthy",
                "protocol": "HTTP/2",
                "timestamp": chrono::Utc::now()
            }))?)
        });

    // Configure HTTP/2 with optimized settings
    let config = ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            enable_prior_knowledge: true, // H2C support
            max_concurrent_streams: Some(1000),
            initial_connection_window_size: Some(1024 * 1024), // 1MB
            adaptive_window: true,
            ..Default::default()
        },
        auto_protocol_detection: true,
        ..Default::default()
    };

    // HTTPS with automatic certificate (development)
    Server::new(router, "127.0.0.1:8443".parse()?)
        .with_config(config)
        .with_self_signed_cert("localhost")? // ⚠️ Development only!
        .ignitia()
        .await
}

async fn get_user(Path(user_id): Path<String>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "name": "John Doe",
        "protocol": "HTTP/2"
    }))?)
}

#[derive(Deserialize, Serialize)]
struct ApiData {
    message: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn create_data(Json(data): Json<ApiData>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "status": "created",
        "data": data,
        "received_at": chrono::Utc::now()
    }))?)
}
```

---

## 🔥 Core Features

### 🌐 HTTP/2 & HTTPS Support

Ignitia provides comprehensive support for modern HTTP protocols with automatic negotiation:

#### Production HTTPS Configuration
```
use ignitia::{Server, Router, TlsConfig, TlsVersion};

let router = Router::new()
    .get("/", || async { Ok(Response::text("Secure HTTPS with HTTP/2!")) });

// Production TLS setup
let tls_config = TlsConfig::new("production.crt", "production.key")
    .with_alpn_protocols(vec!["h2", "http/1.1"]) // HTTP/2 priority
    .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
    .enable_client_cert_verification();

Server::new(router, "0.0.0.0:443".parse()?)
    .with_tls(tls_config)?
    .ignitia()
    .await
```

#### HTTP to HTTPS Redirect
```
// Automatic redirect from HTTP to HTTPS
Server::new(router, "0.0.0.0:80".parse()?)
    .redirect_to_https(443)
    .ignitia()
    .await
```

#### H2C (HTTP/2 Cleartext) Support
```
let config = ServerConfig {
    http2: Http2Config {
        enabled: true,
        enable_prior_knowledge: true, // Enables H2C
        ..Default::default()
    },
    ..Default::default()
};

// Test with: curl --http2-prior-knowledge http://localhost:8080/
```

### 🌐 Advanced WebSocket Support

First-class WebSocket implementation with optimized performance:

```
use ignitia::{websocket_handler, websocket_message_handler, Message, WebSocketConnection};
use serde::{Deserialize, Serialize};

let router = Router::new()
    // Simple echo server
    .websocket("/ws/echo", websocket_handler(|ws: WebSocketConnection| async move {
        while let Some(message) = ws.recv().await {
            match message {
                Message::Text(text) => {
                    ws.send_text(format!("Echo: {}", text)).await?;
                }
                Message::Binary(data) => {
                    ws.send_bytes(data).await?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    }))

    // Advanced JSON chat
    .websocket("/ws/chat", websocket_message_handler(|ws, message| async move {
        if let Message::Text(text) = message {
            #[derive(Deserialize, Serialize)]
            struct ChatMessage {
                user: String,
                message: String,
                room: String,
            }

            if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                let response = serde_json::json!({
                    "user": chat_msg.user,
                    "message": chat_msg.message,
                    "room": chat_msg.room,
                    "timestamp": chrono::Utc::now(),
                    "server": "ignitia"
                });
                ws.send_json(&response).await?;
            }
        }
        Ok(())
    }))

    // Batch message processing
    .websocket("/ws/batch", ignitia::websocket_batch_handler(
        |ws, messages| async move {
            let processed_count = messages.len();
            let batch_response = serde_json::json!({
                "processed": processed_count,
                "timestamp": chrono::Utc::now()
            });
            ws.send_json(&batch_response).await?;
            Ok(())
        },
        100, // batch size
        500, // timeout ms
    ));
```

### 🛡️ Advanced CORS Configuration

Comprehensive CORS middleware with flexible origin matching:

```
use ignitia::{CorsMiddleware, Method};

// Development CORS (permissive)
let dev_cors = CorsMiddleware::permissive();

// Production CORS with specific origins
let prod_cors = CorsMiddleware::new()
    .allowed_origins(&["https://myapp.com", "https://api.myapp.com"])
    .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allowed_headers(&["Content-Type", "Authorization", "X-API-Key"])
    .expose_headers(&["X-Total-Count", "X-Rate-Limit"])
    .allow_credentials()
    .max_age(86400) // 24 hours
    .build()?;

// Regex-based origin matching
let regex_cors = CorsMiddleware::new()
    .allowed_origin_regex(r"https://.*\.myapp\.com$") // All subdomains
    .build()?;

let router = Router::new()
    .middleware(prod_cors)
    .get("/api/data", api_handler);
```

### 🚨 Enhanced Error Handling

Comprehensive error handling with custom error types and responses:

```
use ignitia::{define_error, ErrorHandlerMiddleware, ErrorFormat};

// Define domain-specific errors
define_error! {
    ApiError {
        UserNotFound(StatusCode::NOT_FOUND, "user_not_found", "USER_404"),
        ValidationFailed(StatusCode::BAD_REQUEST, "validation_failed", "VALIDATION_ERROR"),
        RateLimited(StatusCode::TOO_MANY_REQUESTS, "rate_limited", "RATE_LIMIT")
    }
}

// Custom error handler middleware
let error_middleware = ErrorHandlerMiddleware::new()
    .with_details(cfg!(debug_assertions)) // Detailed errors in debug mode
    .with_json_format(ErrorFormat::Detailed)
    .with_logging(true)
    .with_error_log_threshold(500); // Log 5xx as errors, 4xx as warnings

let router = Router::new()
    .middleware(error_middleware)
    .get("/users/:id", get_user_with_validation);

async fn get_user_with_validation(Path(user_id): Path<String>) -> Result<Response> {
    if user_id.is_empty() {
        return Err(ApiError::ValidationFailed("User ID cannot be empty".into()).into());
    }

    if user_id == "404" {
        return Err(ApiError::UserNotFound(format!("User {} not found", user_id)).into());
    }

    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "name": "John Doe"
    }))?)
}
```

---

## 🛣️ Advanced Routing

### Path Parameter Extraction with Types

```
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPath {
    user_id: u32,
    post_id: String,
}

async fn get_user_post(Path(params): Path<UserPath>) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": params.user_id,
        "post_id": params.post_id
    }))?)
}

// Supports: /users/123/posts/abc-def
let router = Router::new()
    .get("/users/:user_id/posts/:post_id", get_user_post);
```

### Wildcard Routes

```
// Wildcard routing for file serving
async fn serve_static(Path(path): Path<String>) -> Result<Response> {
    let safe_path = sanitize_path(&path)?;
    serve_file_from_directory("./static", &safe_path).await
}

let router = Router::new()
    .get("/static/*path", serve_static); // Matches /static/css/style.css, etc.
```

### Nested Routers

```
// API v1 routes
let api_v1 = Router::new()
    .get("/users", list_users)
    .post("/users", create_user)
    .get("/users/:id", get_user);

// API v2 routes
let api_v2 = Router::new()
    .get("/users", list_users_v2)
    .post("/users", create_user_v2);

// Main router with nested subrouters
let router = Router::new()
    .get("/", home)
    .nest("/api/v1", api_v1)
    .nest("/api/v2", api_v2);
```

---

## 🔧 Comprehensive Middleware

### Built-in Middleware Stack

```
use ignitia::{LoggerMiddleware, CorsMiddleware, AuthMiddleware, ErrorHandlerMiddleware};

let router = Router::new()
    // Request logging with HTTP version info
    .middleware(LoggerMiddleware)

    // Advanced CORS configuration
    .middleware(
        CorsMiddleware::secure_api(&["https://myapp.com"])
            .allow_credentials()
            .max_age(3600)
            .build()?
    )

    // Path-based authentication
    .middleware(
        AuthMiddleware::new("your-secret-token")
            .protect_paths(vec!["/admin", "/api/protected"])
    )

    // Enhanced error handling
    .middleware(
        ErrorHandlerMiddleware::new()
            .with_details(cfg!(debug_assertions))
            .with_logging(true)
    )

    .get("/", public_handler)
    .get("/admin", admin_handler)       // Protected
    .get("/api/protected", api_handler); // Protected
```

### Custom Middleware Implementation

```
use ignitia::{Middleware, Request, Response, Result};

struct RateLimitMiddleware {
    max_requests: usize,
    window_seconds: u64,
    // In production, use Redis or similar
    request_counts: Arc<Mutex<HashMap<String, (u64, std::time::Instant)>>>,
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let client_ip = req.header("x-forwarded-for")
            .or_else(|| req.header("x-real-ip"))
            .unwrap_or("unknown")
            .to_string();

        let mut counts = self.request_counts.lock().await;
        let now = std::time::Instant::now();

        if let Some((count, last_reset)) = counts.get_mut(&client_ip) {
            if now.duration_since(*last_reset).as_secs() > self.window_seconds {
                *count = 1;
                *last_reset = now;
            } else {
                *count += 1;
                if *count > self.max_requests {
                    return Err(ignitia::Error::BadRequest(
                        "Rate limit exceeded".into()
                    ));
                }
            }
        } else {
            counts.insert(client_ip, (1, now));
        }

        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        res.headers.insert(
            "X-Rate-Limit-Window",
            self.window_seconds.to_string().parse().unwrap()
        );
        Ok(())
    }
}
```

---

## 🍪 Advanced Cookie Management

### Secure Session Management

```
use ignitia::{Cookie, SameSite};

// Production-ready session cookie
let create_session_cookie = |session_id: &str| {
    Cookie::new("session", session_id)
        .path("/")
        .max_age(3600) // 1 hour
        .http_only()   // Prevent XSS
        .secure()      // HTTPS only
        .same_site(SameSite::Lax) // CSRF protection
};

async fn login(Json(credentials): Json<LoginForm>) -> Result<Response> {
    if validate_credentials(&credentials).await? {
        let session_id = generate_secure_session_id();

        let session_cookie = create_session_cookie(&session_id);

        // Store session in your preferred store (Redis, database, etc.)
        store_session(&session_id, &credentials.username).await?;

        Ok(Response::json(serde_json::json!({
            "status": "success",
            "user": credentials.username
        }))?
        .add_cookie(session_cookie))
    } else {
        Err(ignitia::Error::Unauthorized)
    }
}

async fn logout() -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "status": "logged_out"
    }))?
    .remove_cookie("session"))
}

// Protected route using cookies
async fn profile(cookies: Cookies) -> Result<Response> {
    let session_id = cookies.get("session")
        .ok_or(ignitia::Error::Unauthorized)?;

    let user = get_user_by_session(session_id).await?
        .ok_or(ignitia::Error::Unauthorized)?;

    Ok(Response::json(serde_json::json!({
        "user": user,
        "session": session_id
    }))?)
}
```

---

## ⚡ Performance & Benchmarks

### Real-World Performance Results

```
Framework Comparison (Higher is Better)
┌─────────────┬───────────────┬─────────────┬──────────────┐
│ Framework   │ Avg RPS       │ Peak RPS    │ Avg Latency  │
├─────────────┼───────────────┼─────────────┼──────────────┤
│ 🔥 Ignitia  │ 19,285.4      │ 25,050.5    │ 12.96ms      │
│ Actix Web   │ 18,816.8      │ 24,981.7    │ 13.26ms      │
│ Axum        │ 6,797.1       │ 9,753.9     │ 23.85ms      │
└─────────────┴───────────────┴─────────────┴──────────────┘

HTTP/2 vs HTTP/1.1 Performance (Ignitia)
┌──────────────┬───────────────┬──────────────┐
│ Protocol     │ Avg RPS       │ Concurrent   │
├──────────────┼───────────────┼──────────────┤
│ HTTP/2       │ 21,450.2      │ 1000         │
│ HTTP/1.1     │ 19,285.4      │ 1000         │
│ H2C          │ 20,892.1      │ 1000         │
└──────────────┴───────────────┴──────────────┘
```

### Performance Features

```
// Zero-copy request processing
async fn high_performance_handler(Body(body): Body) -> Result<Response> {
    // Process large payloads efficiently without copying
    let processed = process_large_data(&body).await?;
    Ok(Response::binary(processed)) // Zero-copy response
}

// HTTP/2 server push (when supported by client)
async fn optimized_page() -> Result<Response> {
    let html = include_str!("page.html");
    Ok(Response::html(html)
        .with_header("Link", "</style.css>; rel=preload; as=style")
        .with_header("Link", "</script.js>; rel=preload; as=script"))
}
```

---

## 📚 Comprehensive Examples

### 🏢 Production REST API

```
use ignitia::{Router, Server, Response, Json, Path, Query, CorsMiddleware, LoggerMiddleware};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct UserQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
}

type UserStore = Arc<Mutex<HashMap<u32, User>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();

    let store: UserStore = Arc::new(Mutex::new(HashMap::new()));

    let router = Router::new()
        .middleware(LoggerMiddleware)
        .middleware(
            CorsMiddleware::secure_api(&["https://frontend.myapp.com"])
                .allow_credentials()
                .build()?
        )

        // API routes
        .get("/api/users", {
            let store = Arc::clone(&store);
            move |query: Query<UserQuery>| {
                let store = Arc::clone(&store);
                async move { list_users(store, query).await }
            }
        })
        .post("/api/users", {
            let store = Arc::clone(&store);
            move |json: Json<CreateUserRequest>| {
                let store = Arc::clone(&store);
                async move { create_user(store, json).await }
            }
        })
        .get("/api/users/:id", {
            let store = Arc::clone(&store);
            move |path: Path<u32>| {
                let store = Arc::clone(&store);
                async move { get_user(store, path).await }
            }
        })
        .delete("/api/users/:id", {
            let store = Arc::clone(&store);
            move |path: Path<u32>| {
                let store = Arc::clone(&store);
                async move { delete_user(store, path).await }
            }
        });

    Server::new(router, "0.0.0.0:8080".parse()?)
        .ignitia()
        .await
}

async fn list_users(store: UserStore, Query(query): Query<UserQuery>) -> Result<Response> {
    let users = store.lock().unwrap();
    let mut user_list: Vec<&User> = users.values().collect();

    // Apply search filter
    if let Some(search) = &query.search {
        user_list.retain(|user| {
            user.name.to_lowercase().contains(&search.to_lowercase()) ||
            user.email.to_lowercase().contains(&search.to_lowercase())
        });
    }

    // Apply pagination
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(user_list.len());

    let paginated_users: Vec<&User> = user_list[start..end].to_vec();

    Ok(Response::json(serde_json::json!({
        "users": paginated_users,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": user_list.len(),
            "total_pages": (user_list.len() + per_page as usize - 1) / per_page as usize
        }
    }))?)
}

async fn create_user(store: UserStore, Json(req): Json<CreateUserRequest>) -> Result<Response> {
    let mut users = store.lock().unwrap();
    let id = users.len() as u32 + 1;

    let user = User {
        id,
        name: req.name,
        email: req.email,
        created_at: chrono::Utc::now(),
    };

    users.insert(id, user.clone());

    Ok(Response::json(user)?.with_status_code(201))
}

async fn get_user(store: UserStore, Path(user_id): Path<u32>) -> Result<Response> {
    let users = store.lock().unwrap();

    match users.get(&user_id) {
        Some(user) => Ok(Response::json(user)?),
        None => Err(ignitia::Error::NotFound(format!("User {} not found", user_id))),
    }
}

async fn delete_user(store: UserStore, Path(user_id): Path<u32>) -> Result<Response> {
    let mut users = store.lock().unwrap();

    match users.remove(&user_id) {
        Some(_) => Ok(Response::json(serde_json::json!({"status": "deleted"}))?),
        None => Err(ignitia::Error::NotFound(format!("User {} not found", user_id))),
    }
}
```

---

## 🔌 API Reference

### Router Methods

| Method | Description | Example |
|--------|-------------|---------|
| `Router::new()` | Create new router | `Router::new()` |
| `.get(path, handler)` | Add GET route | `.get("/users/:id", get_user)` |
| `.post(path, handler)` | Add POST route | `.post("/users", create_user)` |
| `.put(path, handler)` | Add PUT route | `.put("/users/:id", update_user)` |
| `.delete(path, handler)` | Add DELETE route | `.delete("/users/:id", delete_user)` |
| `.websocket(path, handler)` | Add WebSocket route | `.websocket("/ws", ws_handler)` |
| `.middleware(middleware)` | Add middleware | `.middleware(LoggerMiddleware)` |
| `.nest(path, router)` | Nest router | `.nest("/api/v1", api_router)` |

### Server Configuration

| Method | Description | Example |
|--------|-------------|---------|
| `Server::new(router, addr)` | Create server | `Server::new(router, addr)` |
| `.with_config(config)` | Set server config | `.with_config(server_config)` |
| `.with_tls(tls_config)` | Enable HTTPS | `.with_tls(tls_config)` |
| `.redirect_to_https(port)` | HTTP redirect | `.redirect_to_https(443)` |
| `.ignitia()` | Start server | `.ignitia().await` |

### Extractors

| Extractor | Type | Example |
|-----------|------|---------|
| `Path<T>` | URL parameters | `Path(user_id): Path<String>` |
| `Query<T>` | Query parameters | `Query(params): Query<SearchParams>` |
| `Json<T>` | JSON body | `Json(user): Json<User>` |
| `Body` | Raw body | `Body(body): Body` |
| `Headers` | Request headers | `headers: Headers` |
| `Cookies` | Request cookies | `cookies: Cookies` |
| `Extension<T>` | Shared state | `Extension(state): Extension<AppState>` |

---

## 🧪 Testing

```
# Run all tests
cargo test

# Test with features
cargo test --features "tls,websocket"

# Integration tests
cargo test --test integration

# Performance benchmarks
cargo bench
```

### Testing Your API

```
# HTTP/1.1
curl -v http://localhost:8080/api/users

# HTTP/2
curl -v --http2-prior-knowledge http://localhost:8080/api/users

# HTTPS/HTTP2
curl -v --http2 https://localhost:8443/api/users

# WebSocket
websocat ws://localhost:8080/ws/echo
```

---

## 🤝 Contributing

We welcome contributions! Here's how you can help:

### Development Setup

```
# Clone repository
git clone https://github.com/AarambhDevHub/ignitia.git
cd ignitia

# Install dependencies
cargo build

# Run tests
cargo test --all-features

# Run examples
cargo run --example basic_server
cargo run --example websocket_chat
```

### Guidelines

1. **Code Quality**: Run `cargo fmt` and `cargo clippy`
2. **Tests**: Add tests for new features
3. **Documentation**: Update docs and examples
4. **Performance**: Benchmark performance-critical changes

---

## 📝 Changelog

### v0.1.8 - Protocol Master Release 🚀

#### 🆕 New Features
- ✅ **HTTP/2 Support**: Full HTTP/2 implementation with ALPN negotiation
- ✅ **TLS/HTTPS**: Comprehensive TLS support with certificate management
- ✅ **H2C Support**: HTTP/2 cleartext for development and testing
- ✅ **Self-Signed Certificates**: Automatic cert generation for development
- ✅ **Advanced CORS**: Regex origin matching and credential support
- ✅ **Enhanced WebSocket**: Optimized WebSocket with batch processing
- ✅ **Custom Error Handling**: Detailed error types and responses
- ✅ **Protocol Detection**: Automatic HTTP version negotiation

#### 🚀 Performance
- ✅ **19,285+ RPS**: Leading performance in Rust ecosystem
- ✅ **Zero-Copy**: Efficient request/response handling
- ✅ **Connection Pooling**: Optimized connection management
- ✅ **HTTP/2 Multiplexing**: Multiple streams over single connection

#### 🔒 Security
- ✅ **TLS 1.2/1.3**: Modern TLS support with ALPN
- ✅ **Secure Cookies**: Full security attribute support
- ✅ **CORS Protection**: Advanced cross-origin controls
- ✅ **Path Traversal Protection**: Secure file serving

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## ☕ Support

If you find Ignitia helpful, consider:

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Support-orange?style=for-the-badge&logo=buy-me-a-coffee)](https://buymeacoffee.com/aarambhdevhub)

---

<div align="center">

## 🚀 Get Started Today

```
# Create new project
cargo new my-ignitia-app && cd my-ignitia-app

# Add Ignitia with all features
echo 'ignitia = { version = "0.1.8", features = ["tls", "websocket", "self-signed"] }' >> Cargo.toml

# Create your first HTTP/2 + WebSocket app
cargo run
```

**Join thousands of developers building the future with Ignitia**

[![YouTube](https://img.shields.io/badge/YouTube-Aarambh%20Dev%20Hub-red?style=for-the-badge&logo=youtube)](https://youtube.com/@aarambhdevhub)
[![GitHub](https://img.shields.io/badge/GitHub-ignitia-black?style=for-the-badge&logo=github)](https://github.com/AarambhDevHub/ignitia)
[![Discord](https://img.shields.io/badge/Discord-Community-blue?style=for-the-badge&logo=discord)](https://discord.gg/aarambhdevhub)

---

### 🔥 **Ignitia. Build Fast. Scale Further.** 🔥

**Built with ❤️ by [Aarambh Dev Hub](https://youtube.com/@aarambhdevhub)**

*Where every line of code ignites possibilities.*

</div>
