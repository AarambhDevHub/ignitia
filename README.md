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

***

## ⚡ Why Ignitia?

Ignitia embodies the spirit of **Aarambh** (new beginnings) - the spark that ignites your web development journey. Built for developers who demand speed, simplicity, and power with modern protocol support.

- **🚀 Multi-Protocol**: HTTP/1.1, HTTP/2, and HTTPS with automatic protocol negotiation
- **⚡ Industry-Leading Performance**: **18,367+ RPS** - faster than Axum, matches Actix-web
- **🔒 TLS/HTTPS**: Built-in TLS support with ALPN and self-signed certificates for development
- **🪶 Lightweight**: Minimal overhead, maximum efficiency
- **🔥 Powerful**: Advanced routing, middleware, and WebSocket support
- **🎯 Developer-First**: Clean, intuitive, and productive APIs
- **🛡️ Secure**: Built-in security features and best practices
- **🍪 Cookie Management**: Full-featured cookie handling with security attributes
- **🌐 WebSocket Ready**: First-class WebSocket support with optimized performance
- **📊 Type-Safe Extractors**: Path, Query, JSON, Form, Multipart, State, and custom extractors
- **📁 Multipart Support**: Advanced file uploads with metadata extraction

***

## 📋 Table of Contents

- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Performance](#-performance--benchmarks)
- [Core Features](#-core-features)
- [HTTP/2 & HTTPS](#-http2--https)
- [WebSocket Support](#-websocket-support)
- [Extractors](#-type-safe-extractors)
- [State Management](#-state-management)
- [Form Handling](#-form-handling)
- [File Uploads & Multipart](#-file-uploads--multipart)
- [Routing](#-routing)
- [Middleware](#-middleware)
- [CORS Configuration](#-cors-configuration)
- [Cookie Management](#-cookie-management)
- [Authentication](#-authentication)
- [Examples](#-examples)
- [Documentation](#-documentation)
- [API Reference](#-api-reference)
- [Contributing](#-contributing)

***

## 📚 Documentation

Ignitia comes with comprehensive documentation to help you get started quickly and master advanced features:

```
doc/
├── 📄 README.md              # This file - overview and quick start
├── 📄 QUICK_START.md         # 5-minute setup guide
├── 📄 INSTALLATION.md        # Detailed installation instructions
├── 📄 ROUTING_GUIDE.md       # Advanced routing patterns
├── 📄 MIDDLEWARE_GUIDE.md    # Custom middleware development
├── 📄 ERROR_HANDLING.md      # Error handling patterns
├── 📄 EXTRACTORS.md          # Type-safe request extraction
├── 📄 WEB_SOCKETS.md         # WebSocket implementation guide
├── 📄 FILE_UPLOADS.md        # File upload and multipart handling
├── 📄 SERVER_CONFIG.md       # Server configuration options
├── 📄 SECURITY.md            # Security best practices
├── 📄 API_REFERENCE.md       # API reference documentation
├── 📄 MIGRATION.md           # Migration from other frameworks
├── 📄 CONTRIBUTING.md        # How to contribute
├── 📄 CHANGELOG.md           # Version history
└── 📄 EXAMPLES.md            # Comprehensive examples
```

**🚀 Quick Links:**
- 📖 [Quick Start Guide](doc/QUICK_START.md) - Get up and running in 5 minutes
- ⚡ [Performance Guide](doc/PERFORMANCE.md) - Optimize for maximum speed
- 🔒 [Security Guide](doc/SECURITY.md) - Secure your applications
- 🌐 [WebSocket Guide](doc/WEB_SOCKETS.md) - Real-time communication
- 📁 [File Upload Guide](doc/FILE_UPLOADS.md) - Handle file uploads
- 🛣️ [Routing Guide](doc/ROUTING_GUIDE.md) - Advanced routing patterns
- 🔧 [Middleware Guide](doc/MIDDLEWARE_GUIDE.md) - Custom middleware development

***

## 🛠️ Installation

Add Ignitia to your `Cargo.toml`:

```toml
[dependencies]
ignitia = { version = "0.2.0", features = ["tls", "websocket", "self-signed"] }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing-subscriber = "0.3"
```

### Feature Flags

- **`tls`**: Enables HTTPS/TLS support with certificate management and ALPN
- **`websocket`**: Enables WebSocket protocol support with connection management
- **`self-signed`**: Enables self-signed certificate generation (development only)

***

## 🚀 Quick Start

### Basic High-Performance Server

```rust
use ignitia::{
    Router, Server, Response, Result, State,
    handler::extractor::{Path, Query, Json, Form, Multipart},
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    name: String,
    version: String,
    upload_dir: String,
}

#[derive(Deserialize)]
struct UserForm {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();

    // Create shared application state
    let app_state = AppState {
        name: "My Ignitia App".to_string(),
        version: "1.0.0".to_string(),
        upload_dir: "./uploads".to_string(),
    };

    let router = Router::new()
        .get("/", home)
        .get("/users/:id", get_user)
        .post("/api/data", create_data)
        .post("/forms/user", handle_form)
        .post("/upload", handle_file_upload)  // File upload endpoint
        .post("/upload/profile", handle_profile_upload)  // Profile with file
        .get("/health", health_check)
        .state(app_state); // Add shared state

    // Start high-performance server
    Server::new("127.0.0.1:8080")
        .run(router)
        .await
}

// Handler with state access
async fn home(State(state): State<AppState>) -> Result<Response> {
    Response::html(format!(
        "<h1>🔥 Welcome to {}!</h1><p>Version: {}</p>",
        state.name, state.version
    ))
}

// Path parameter extraction
async fn get_user(Path(user_id): Path<String>) -> Result<Response> {
    Response::json(serde_json::json!({
        "user_id": user_id,
        "name": "John Doe",
        "framework": "Ignitia"
    }))
}

// JSON body handling
#[derive(Deserialize, Serialize)]
struct ApiData {
    message: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn create_data(Json(data): Json<ApiData>) -> Result<Response> {
    Response::json(serde_json::json!({
        "status": "created",
        "data": data,
        "received_at": chrono::Utc::now()
    }))
}

// Form data handling
async fn handle_form(Form(form): Form<UserForm>) -> Result<Response> {
    Response::json(serde_json::json!({
        "message": "Form received",
        "user": {
            "name": form.name,
            "email": form.email
        }
    }))
}

// File upload handling
async fn handle_file_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if let Some(filename) = field.file_name() {
            let content_type = field.content_type().unwrap_or("application/octet-stream");
            let data = field.bytes().await?;

            // Save file
            let file_path = format!("{}/{}", state.upload_dir, filename);
            tokio::fs::write(&file_path, data).await?;

            uploaded_files.push(serde_json::json!({
                "filename": filename,
                "size": data.len(),
                "content_type": content_type,
                "path": file_path
            }));
        }
    }

    Response::json(serde_json::json!({
        "message": "Files uploaded successfully",
        "files": uploaded_files
    }))
}

// Profile with file upload
#[derive(Deserialize)]
struct ProfileForm {
    name: String,
    bio: String,
    age: Option<u32>,
}

async fn handle_profile_upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response> {
    let mut profile_data = None;
    let mut avatar_file = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("unknown");

        if let Some(filename) = field.file_name() {
            // Handle file field
            let data = field.bytes().await?;
            let file_path = format!("{}/avatars/{}", state.upload_dir, filename);
            tokio::fs::write(&file_path, &data).await?;

            avatar_file = Some(serde_json::json!({
                "filename": filename,
                "size": data.len(),
                "path": file_path
            }));
        } else {
            // Handle text field
            let value = field.text().await?;
            match field_name {
                "name" => profile_data = Some(ProfileForm {
                    name: value,
                    bio: "".to_string(),
                    age: None
                }),
                _ => {} // Handle other fields
            }
        }
    }

    Response::json(serde_json::json!({
        "message": "Profile updated successfully",
        "profile": profile_data,
        "avatar": avatar_file
    }))
}

// Health check with state
async fn health_check(State(state): State<AppState>) -> Result<Response> {
    Response::json(serde_json::json!({
        "status": "healthy",
        "app": state.name,
        "version": state.version,
        "timestamp": chrono::Utc::now()
    }))
}
```

***

## 📊 Performance & Benchmarks

### **🏆 Industry-Leading Performance**

Ignitia delivers exceptional performance that outperforms popular frameworks:

```
📈 COMPREHENSIVE BENCHMARK RESULTS
--------------------------------------------------
🥇 Ignitia Framework
   Average RPS: 18,367.7    (+185% vs Axum)
   Peak RPS: 24,014.1       (Near Actix-web peak)
   Average Response Time: 13.34ms  (45% faster than Axum)
   Best Response Time: 0.14ms      (48% faster than Axum)
   Failed Requests: 0              (100% reliability)

🥈 Actix-web
   Average RPS: 17,792.7
   Peak RPS: 24,296.3
   Average Response Time: 14.06ms
   Best Response Time: 0.18ms
   Failed Requests: 0

🥉 Axum
   Average RPS: 6,437.3
   Peak RPS: 9,331.4
   Average Response Time: 24.42ms
   Best Response Time: 0.27ms
   Failed Requests: 0
```

### **⚡ Performance Features**

```rust
// Zero-copy request processing
async fn high_performance_handler(Body(body): Body) -> Result<Response> {
    // Process large payloads efficiently without copying
    let processed = process_large_data(&body).await?;
    Response::binary(processed) // Zero-copy response
}

// HTTP/2 multiplexing advantage
let config = ServerConfig {
    http2: Http2Config {
        enabled: true,
        max_concurrent_streams: Some(1000),
        initial_connection_window_size: Some(1024 * 1024), // 1MB
        adaptive_window: true,
        ..Default::default()
    },
    ..Default::default()
};
```

### **🔥 Why Ignitia is Faster**

1. **🔧 Compiled Router**: Zero-allocation route matching with `ArcSwap`
2. **⚡ Efficient Extractors**: Minimal overhead type-safe parameter extraction
3. **🚀 Smart Middleware**: Pipeline optimization without boxing overhead
4. **📦 Memory Optimized**: Careful use of `Arc<T>` and `RwLock` for shared state
5. **🎯 Direct Dispatch**: Minimal abstraction layers between request and handler

***

## 🔥 Core Features

### 🌐 HTTP/2 & HTTPS Support

Ignitia provides comprehensive support for modern HTTP protocols with automatic negotiation:

#### Production HTTPS Configuration
```rust
use ignitia::{Server, Router, TlsConfig, TlsVersion};

let router = Router::new()
    .get("/", || async { Response::text("Secure HTTPS with HTTP/2!") });

// Production TLS setup
let tls_config = TlsConfig::new("production.crt", "production.key")
    .with_alpn_protocols(vec!["h2", "http/1.1"]) // HTTP/2 priority
    .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
    .enable_client_cert_verification();

Server::new("0.0.0.0:443")
    .tls(tls_config)
    .run(router)
    .await
```

#### Development with Self-Signed Certificates
```rust
// Quick HTTPS setup for development
Server::new("127.0.0.1:8443")
    .self_signed_cert("localhost") // ⚠️ Development only!
    .run(router)
    .await
```

#### H2C (HTTP/2 Cleartext) Support
```rust
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

***

## 🌐 WebSocket Support

First-class WebSocket implementation with optimized performance:

```rust
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
    }));
```

***

## 🎯 Type-Safe Extractors

Ignitia provides powerful, type-safe extractors for handling various parts of HTTP requests:

### **Available Extractors**

```rust
use ignitia::{Path, Query, Json, Form, Body, Headers, Cookies, State, Multipart};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserParams {
    user_id: u32,
    post_id: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

// Multiple extractors in one handler
async fn advanced_handler(
    Path(params): Path<UserParams>,           // URL parameters
    Query(search): Query<SearchQuery>,        // Query string
    Json(user_data): Json<CreateUser>,        // JSON body
    Form(login): Form<LoginForm>,             // Form data
    Body(raw_body): Body,                     // Raw request body
    headers: Headers,                         // All headers
    cookies: Cookies,                         // All cookies
    State(app_state): State<AppState>,        // Shared state
    mut multipart: Multipart,                 // Multipart data
) -> Result<Response> {
    // Access all extracted data
    println!("User ID: {}", params.user_id);
    println!("Search query: {}", search.q);
    println!("New user: {}", user_data.name);
    println!("Login attempt: {}", login.username);

    // Process multipart if present
    while let Some(field) = multipart.next_field().await? {
        if let Some(filename) = field.file_name() {
            let data = field.bytes().await?;
            println!("Received file: {} ({} bytes)", filename, data.len());
        }
    }

    Response::json(serde_json::json!({
        "message": "All extractors working!",
        "user_agent": headers.get("user-agent")
    }))
}
```

***

## 🗄️ State Management

Ignitia provides powerful state management for sharing data across handlers:

### **Application State Setup**

```rust
use ignitia::{Router, State};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_pool: Arc<DatabasePool>,
    config: AppConfig,
    metrics: Arc<MetricsCollector>,
}

#[derive(Clone)]
struct AppConfig {
    api_url: String,
    max_connections: u32,
    debug: bool,
}

async fn main() -> Result<()> {
    let app_state = AppState {
        db_pool: Arc::new(create_database_pool().await?),
        config: AppConfig {
            api_url: "https://api.example.com".to_string(),
            max_connections: 100,
            debug: cfg!(debug_assertions),
        },
        metrics: Arc::new(MetricsCollector::new()),
    };

    let router = Router::new()
        .get("/users", list_users)
        .post("/users", create_user)
        .get("/metrics", get_metrics)
        .state(app_state);  // Add state to router

    Server::new("127.0.0.1:8080")
        .run(router)
        .await
}

// Handlers can access state
async fn list_users(State(state): State<AppState>) -> Result<Response> {
    let users = state.db_pool.get_all_users().await?;
    state.metrics.increment("users_listed");

    Response::json(users)
}

async fn create_user(
    State(state): State<AppState>,
    Json(user_data): Json<CreateUserRequest>,
) -> Result<Response> {
    let user = state.db_pool.create_user(user_data).await?;
    state.metrics.increment("users_created");

    Response::json(user).status(201)
}

// Multiple state types
async fn get_metrics(
    State(app_state): State<AppState>,
    State(cache): State<RedisCache>,  // Multiple state types
) -> Result<Response> {
    let metrics = app_state.metrics.get_all();
    cache.store("last_metrics", &metrics, 300).await?;

    Response::json(metrics)
}
```

***

## 🧪 Testing

```bash
# Run all tests
cargo test

# Test with features
cargo test --features "tls,websocket,multipart"

# Integration tests
cargo test --test integration

# Performance benchmarks
cargo bench
```

### Testing Your API

```bash
# HTTP/1.1
curl -v http://localhost:8080/api/users

# HTTP/2
curl -v --http2-prior-knowledge http://localhost:8080/api/users

# HTTPS/HTTP2
curl -v --http2 https://localhost:8443/api/users

# WebSocket
websocat ws://localhost:8080/ws/echo

# Form data
curl -X POST -d "name=John&email=john@example.com" http://localhost:8080/forms/user

# File upload
curl -X POST -F "file=@image.jpg" http://localhost:8080/upload

# Multiple file upload
curl -X POST -F "files=@file1.pdf" -F "files=@file2.jpg" http://localhost:8080/upload/multiple

# Mixed form with file
curl -X POST \
  -F "name=John Doe" \
  -F "email=john@example.com" \
  -F "avatar=@profile.jpg" \
  http://localhost:8080/users/123/profile
```

***

## 🤝 Contributing

We welcome contributions! Here's how you can help:

### Development Setup

```bash
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
cargo run --example file_upload
```

### Guidelines

1. **Code Quality**: Run `cargo fmt` and `cargo clippy`
2. **Tests**: Add tests for new features
3. **Documentation**: Update docs and examples
4. **Performance**: Benchmark performance-critical changes
5. **Compatibility**: Maintain backwards compatibility

***

## 📝 Changelog

### v0.2.0 - Performance Champion Release 🏆

#### 🚀 **Industry-Leading Performance**
- ✅ **18,367+ RPS**: Outperforms Axum by 185%, matches Actix-web
- ✅ **Sub-millisecond responses**: 0.14ms best response time
- ✅ **Zero failures**: 100% reliability across all benchmarks
- ✅ **HTTP/2 optimization**: Enhanced multiplexing and connection handling

#### 🆕 **New Features**
- ✅ **State Management**: Type-safe shared application state with `State<T>`
- ✅ **Form Handling**: Comprehensive form data extraction with `Form<T>`
- ✅ **Multipart Support**: Advanced file uploads with `Multipart` extractor
- ✅ **Enhanced Extractors**: More powerful parameter extraction
- ✅ **HTTP/2 Support**: Full HTTP/2 implementation with ALPN negotiation
- ✅ **TLS/HTTPS**: Comprehensive TLS support with certificate management
- ✅ **Advanced CORS**: Regex origin matching and credential support
- ✅ **Enhanced WebSocket**: Optimized WebSocket with batch processing

#### 🔒 **Security & Reliability**
- ✅ **TLS 1.2/1.3**: Modern TLS support with ALPN
- ✅ **Secure Cookies**: Full security attribute support
- ✅ **CORS Protection**: Advanced cross-origin controls
- ✅ **Input Validation**: Type-safe parameter extraction and validation
- ✅ **File Upload Security**: Content type validation and size limits

#### 🎯 **Developer Experience**
- ✅ **Better Documentation**: Comprehensive examples and guides
- ✅ **Error Messages**: Improved error handling and debugging
- ✅ **Type Safety**: Enhanced compile-time guarantees
- ✅ **Easy Setup**: Simplified configuration and deployment
- ✅ **File Handling**: Streaming uploads for large files

***

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## ☕ Support & Community

If you find Ignitia helpful, consider supporting the project:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/aarambhdevhub)

<div align="center">

[![Star History Chart](https://api.star-history.com/svg?repos=AarambhDevHub/ignitia&type=Date)](https://star-history.com/#AarambhDevHub/ignitia&Date)

## 🚀 Get Started Today

```bash
# Create new project
cargo new my-ignitia-app && cd my-ignitia-app

# Add Ignitia with all features
echo 'ignitia = { version = "0.2.0", features = ["tls", "websocket", "self-signed"] }' >> Cargo.toml

# Create your first high-performance app
cargo run
```

**Join thousands of developers building the future with Ignitia**

[![YouTube](https://img.shields.io/badge/YouTube-Aarambh%20Dev%20Hub-red?style=for-the-badge&logo=youtube)](https://youtube.com/@aarambhdevhub)
[![GitHub](https://img.shields.io/badge/GitHub-ignitia-black?style=for-the-badge&logo=github)](https://github.com/AarambhDevHub/ignitia)
[![Discord](https://img.shields.io/badge/Discord-Community-blue?style=for-the-badge&logo=discord)](https://discord.gg/HDth6PfCnp)

***

### 🔥 **Ignitia. Build Fast. Scale Faster.** 🔥

**Built with ❤️ by [Aarambh Dev Hub](https://youtube.com/@aarambhdevhub)**

*Where every line of code ignites possibilities.*

</div>
