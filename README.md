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

---

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
- [API Reference](#-api-reference)
- [Contributing](#-contributing)

---

## 🛠️ Installation

Add Ignitia to your `Cargo.toml`:

```
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

---

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

---

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

---

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

---

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

---

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

---

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

### **State Methods**

```rust
let router = Router::new()
    .state(database_pool)           // Add single state
    .state(redis_client)           // Add another state type
    .state_arc(shared_config)      // Add Arc<T> directly
    .state_factory(|| {            // Create state with factory
        AppConfig::from_env()
    });
```

---

## 📝 Form Handling

Ignitia provides comprehensive form handling for both URL-encoded and multipart forms:

### **URL-Encoded Forms**

```rust
use ignitia::{Form, Response};
use serde::Deserialize;

#[derive(Deserialize)]
struct ContactForm {
    name: String,
    email: String,
    message: String,
    newsletter: Option<bool>,
}

async fn handle_contact(Form(form): Form<ContactForm>) -> Result<Response> {
    println!("Contact from: {} <{}>", form.name, form.email);
    println!("Message: {}", form.message);
    println!("Newsletter: {:?}", form.newsletter);

    // Process form data...
    send_email(&form).await?;

    Response::html(r#"
        <h2>Thank you!</h2>
        <p>Your message has been received.</p>
        <a href="/">Back to home</a>
    "#)
}

// Form with validation
#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

async fn login(Form(form): Form<LoginForm>) -> Result<Response> {
    // Validate form data
    if form.username.is_empty() || form.password.is_empty() {
        return Response::bad_request()
            .text("Username and password are required");
    }

    // Authenticate user
    if authenticate_user(&form.username, &form.password).await? {
        let mut response = Response::json(serde_json::json!({
            "success": true,
            "user": form.username
        }))?;

        // Set session cookie if remember_me is checked
        if form.remember_me.unwrap_or(false) {
            response = response.add_cookie(
                Cookie::new("session", generate_session_token())
                    .max_age(86400 * 30) // 30 days
                    .http_only()
                    .secure()
            );
        }

        Ok(response)
    } else {
        Response::unauthorized().text("Invalid credentials")
    }
}
```

### **HTML Form Example**

```html
<!-- Contact form -->
<form action="/contact" method="post" enctype="application/x-www-form-urlencoded">
    <input type="text" name="name" placeholder="Your Name" required>
    <input type="email" name="email" placeholder="Your Email" required>
    <textarea name="message" placeholder="Your Message" required></textarea>
    <label>
        <input type="checkbox" name="newsletter" value="true">
        Subscribe to newsletter
    </label>
    <button type="submit">Send Message</button>
</form>

<!-- Login form -->
<form action="/login" method="post">
    <input type="text" name="username" placeholder="Username" required>
    <input type="password" name="password" placeholder="Password" required>
    <label>
        <input type="checkbox" name="remember_me" value="true">
        Remember me
    </label>
    <button type="submit">Login</button>
</form>
```

---

## 📁 File Uploads & Multipart

Ignitia provides comprehensive multipart/form-data support for file uploads and mixed content:

### **Basic File Upload**

```rust
use ignitia::{Multipart, Response, State};
use std::path::Path;

async fn upload_single_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response> {
    while let Some(field) = multipart.next_field().await? {
        if let Some(filename) = field.file_name() {
            let content_type = field.content_type().unwrap_or("application/octet-stream");
            let data = field.bytes().await?;

            // Validate file type
            if !is_allowed_content_type(content_type) {
                return Response::bad_request().text("File type not allowed");
            }

            // Validate file size (10MB limit)
            if data.len() > 10 * 1024 * 1024 {
                return Response::bad_request().text("File too large (max 10MB)");
            }

            // Generate safe filename
            let safe_filename = sanitize_filename(filename);
            let file_path = format!("{}/{}", state.upload_dir, safe_filename);

            // Save file
            tokio::fs::write(&file_path, &data).await?;

            return Response::json(serde_json::json!({
                "message": "File uploaded successfully",
                "filename": safe_filename,
                "size": data.len(),
                "content_type": content_type,
                "path": file_path
            }));
        }
    }

    Response::bad_request().text("No file provided")
}

fn is_allowed_content_type(content_type: &str) -> bool {
    matches!(content_type,
        "image/jpeg" | "image/png" | "image/gif" | "image/webp" |
        "application/pdf" | "text/plain" | "application/json"
    )
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(*c, '.' | '-' | '_'))
        .collect()
}
```

### **Multiple File Upload**

```rust
async fn upload_multiple_files(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response> {
    let mut uploaded_files = Vec::new();
    let mut total_size = 0;
    const MAX_FILES: usize = 5;
    const MAX_TOTAL_SIZE: usize = 50 * 1024 * 1024; // 50MB

    while let Some(field) = multipart.next_field().await? {
        if let Some(filename) = field.file_name() {
            if uploaded_files.len() >= MAX_FILES {
                return Response::bad_request()
                    .text(format!("Too many files (max {})", MAX_FILES));
            }

            let data = field.bytes().await?;
            total_size += data.len();

            if total_size > MAX_TOTAL_SIZE {
                return Response::bad_request()
                    .text("Total file size too large (max 50MB)");
            }

            let safe_filename = sanitize_filename(filename);
            let file_path = format!("{}/{}", state.upload_dir, safe_filename);

            tokio::fs::write(&file_path, &data).await?;

            uploaded_files.push(serde_json::json!({
                "filename": safe_filename,
                "size": data.len(),
                "content_type": field.content_type().unwrap_or("application/octet-stream"),
                "path": file_path
            }));
        }
    }

    Response::json(serde_json::json!({
        "message": "Files uploaded successfully",
        "files": uploaded_files,
        "total_files": uploaded_files.len(),
        "total_size": total_size
    }))
}
```

### **Mixed Form with Files and Data**

```rust
#[derive(Deserialize, Debug)]
struct UserProfile {
    name: String,
    email: String,
    bio: String,
    age: Option<u32>,
}

async fn update_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Response> {
    let mut profile_data: Option<UserProfile> = None;
    let mut avatar_path: Option<String> = None;
    let mut form_fields = std::collections::HashMap::new();

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("unknown").to_string();

        if let Some(filename) = field.file_name() {
            // Handle file uploads
            if field_name == "avatar" {
                let data = field.bytes().await?;

                // Validate image
                if !field.content_type().unwrap_or("").starts_with("image/") {
                    return Response::bad_request().text("Avatar must be an image");
                }

                let safe_filename = format!("{}_{}", user_id, sanitize_filename(filename));
                let file_path = format!("{}/avatars/{}", state.upload_dir, safe_filename);

                // Create directory if it doesn't exist
                if let Some(parent) = Path::new(&file_path).parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                tokio::fs::write(&file_path, &data).await?;
                avatar_path = Some(file_path);
            }
        } else {
            // Handle text fields
            let value = field.text().await?;
            form_fields.insert(field_name, value);
        }
    }

    // Parse form fields into struct
    if !form_fields.is_empty() {
        profile_data = Some(UserProfile {
            name: form_fields.get("name").cloned().unwrap_or_default(),
            email: form_fields.get("email").cloned().unwrap_or_default(),
            bio: form_fields.get("bio").cloned().unwrap_or_default(),
            age: form_fields.get("age")
                .and_then(|s| s.parse().ok()),
        });
    }

    // Update user profile in database
    if let Some(profile) = &profile_data {
        update_user_in_database(&user_id, profile, avatar_path.as_deref()).await?;
    }

    Response::json(serde_json::json!({
        "message": "Profile updated successfully",
        "user_id": user_id,
        "profile": profile_data,
        "avatar_uploaded": avatar_path.is_some(),
        "avatar_path": avatar_path
    }))
}
```

### **Image Processing with Multipart**

```rust
use image::{ImageFormat, DynamicImage};

async fn upload_and_process_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response> {
    while let Some(field) = multipart.next_field().await? {
        if let Some(filename) = field.file_name() {
            let content_type = field.content_type().unwrap_or("");

            if !content_type.starts_with("image/") {
                return Response::bad_request().text("Only image files allowed");
            }

            let data = field.bytes().await?;

            // Process image
            let image = image::load_from_memory(&data)
                .map_err(|_| ignitia::Error::BadRequest("Invalid image format".into()))?;

            // Create thumbnail (200x200)
            let thumbnail = image.resize(200, 200, image::imageops::FilterType::Lanczos3);

            // Save original
            let original_path = format!("{}/images/original_{}", state.upload_dir, filename);
            tokio::fs::write(&original_path, &data).await?;

            // Save thumbnail
            let thumb_path = format!("{}/images/thumb_{}", state.upload_dir, filename);
            let mut thumb_bytes = Vec::new();
            thumbnail.write_to(&mut std::io::Cursor::new(&mut thumb_bytes), ImageFormat::Jpeg)?;
            tokio::fs::write(&thumb_path, &thumb_bytes).await?;

            return Response::json(serde_json::json!({
                "message": "Image processed successfully",
                "original": {
                    "path": original_path,
                    "size": data.len(),
                    "dimensions": {
                        "width": image.width(),
                        "height": image.height()
                    }
                },
                "thumbnail": {
                    "path": thumb_path,
                    "size": thumb_bytes.len(),
                    "dimensions": {
                        "width": 200,
                        "height": 200
                    }
                }
            }));
        }
    }

    Response::bad_request().text("No image provided")
}
```

### **Streaming Large Files**

```rust
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

async fn upload_large_file_streaming(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response> {
    while let Some(field) = multipart.next_field().await? {
        if let Some(filename) = field.file_name() {
            let safe_filename = sanitize_filename(filename);
            let file_path = format!("{}/large_files/{}", state.upload_dir, safe_filename);

            // Create directory
            if let Some(parent) = Path::new(&file_path).parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // Stream file to disk
            let file = File::create(&file_path).await?;
            let mut writer = BufWriter::new(file);
            let mut total_size = 0;

            // Stream chunks
            while let Ok(Some(chunk)) = field.chunk().await {
                total_size += chunk.len();

                // Check size limit (1GB)
                if total_size > 1024 * 1024 * 1024 {
                    // Cleanup partial file
                    let _ = tokio::fs::remove_file(&file_path).await;
                    return Response::bad_request().text("File too large (max 1GB)");
                }

                writer.write_all(&chunk).await?;
            }

            writer.flush().await?;

            return Response::json(serde_json::json!({
                "message": "Large file uploaded successfully",
                "filename": safe_filename,
                "size": total_size,
                "path": file_path
            }));
        }
    }

    Response::bad_request().text("No file provided")
}
```

### **HTML File Upload Forms**

```html
<!-- Single file upload -->
<form action="/upload" method="post" enctype="multipart/form-data">
    <input type="file" name="file" required>
    <button type="submit">Upload File</button>
</form>

<!-- Multiple file upload -->
<form action="/upload/multiple" method="post" enctype="multipart/form-data">
    <input type="file" name="files" multiple accept="image/*,application/pdf">
    <button type="submit">Upload Files</button>
</form>

<!-- Profile with avatar -->
<form action="/users/123/profile" method="post" enctype="multipart/form-data">
    <input type="text" name="name" placeholder="Full Name" required>
    <input type="email" name="email" placeholder="Email" required>
    <textarea name="bio" placeholder="Bio"></textarea>
    <input type="number" name="age" placeholder="Age" min="1" max="120">
    <input type="file" name="avatar" accept="image/*">
    <button type="submit">Update Profile</button>
</form>

<!-- Image upload with preview -->
<form action="/upload/image" method="post" enctype="multipart/form-data" id="imageForm">
    <input type="file" name="image" accept="image/*" id="imageInput" required>
    <img id="preview" style="max-width: 300px; display: none;">
    <button type="submit">Process Image</button>
</form>

<script>
document.getElementById('imageInput').addEventListener('change', function(e) {
    const file = e.target.files;
    if (file) {
        const reader = new FileReader();
        reader.onload = function(e) {
            const preview = document.getElementById('preview');
            preview.src = e.target.result;
            preview.style.display = 'block';
        };
        reader.readAsDataURL(file);
    }
});
</script>
```

---

## 🛣️ Advanced Routing

### Path Parameter Extraction with Types

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPath {
    user_id: u32,
    post_id: String,
}

async fn get_user_post(Path(params): Path<UserPath>) -> Result<Response> {
    Response::json(serde_json::json!({
        "user_id": params.user_id,
        "post_id": params.post_id
    }))
}

// Supports: /users/123/posts/abc-def
let router = Router::new()
    .get("/users/:user_id/posts/:post_id", get_user_post);
```

### Wildcard Routes

```rust
// Wildcard routing for file serving
async fn serve_static(Path(path): Path<String>) -> Result<Response> {
    let safe_path = sanitize_path(&path)?;
    serve_file_from_directory("./static", &safe_path).await
}

let router = Router::new()
    .get("/static/*path", serve_static); // Matches /static/css/style.css, etc.
```

### Nested Routers

```rust
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

```rust
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

```rust
use ignitia::{Middleware, Request, Response, Result};

#[derive(Clone)]
struct RateLimitMiddleware {
    max_requests: usize,
    window_seconds: u64,
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let client_ip = req.header("x-forwarded-for")
            .or_else(|| req.header("x-real-ip"))
            .unwrap_or("unknown");

        // Check rate limit (implement your logic)
        if is_rate_limited(client_ip, self.max_requests, self.window_seconds).await? {
            return Err(ignitia::Error::TooManyRequests("Rate limit exceeded".into()));
        }

        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        res.header("X-Rate-Limit-Window", &self.window_seconds.to_string());
        Ok(())
    }
}
```

---

## 🛡️ Advanced CORS Configuration

```rust
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

---

## 🍪 Advanced Cookie Management

### Secure Session Management

```rust
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

        Response::json(serde_json::json!({
            "status": "success",
            "user": credentials.username
        }))?.add_cookie(session_cookie)
    } else {
        Err(ignitia::Error::Unauthorized)
    }
}

// Protected route using cookies
async fn profile(cookies: Cookies) -> Result<Response> {
    let session_id = cookies.get("session")
        .ok_or(ignitia::Error::Unauthorized)?;

    let user = get_user_by_session(session_id).await?
        .ok_or(ignitia::Error::Unauthorized)?;

    Response::json(serde_json::json!({
        "user": user,
        "session": session_id
    }))
}
```

---

## 📚 Comprehensive Examples

### 🏢 Production REST API

```rust
use ignitia::{Router, Server, Response, Json, Path, Query, State, CorsMiddleware, LoggerMiddleware};
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

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<HashMap<u32, User>>>,
    next_id: Arc<Mutex<u32>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();

    let app_state = AppState {
        users: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(Mutex::new(1)),
    };

    let router = Router::new()
        .middleware(LoggerMiddleware)
        .middleware(
            CorsMiddleware::secure_api(&["https://frontend.myapp.com"])
                .allow_credentials()
                .build()?
        )

        // API routes
        .get("/api/users", list_users)
        .post("/api/users", create_user)
        .get("/api/users/:id", get_user)
        .delete("/api/users/:id", delete_user)
        .state(app_state);

    Server::new("0.0.0.0:8080")
        .run(router)
        .await
}

async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserQuery>
) -> Result<Response> {
    let users = state.users.lock().unwrap();
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

    Response::json(serde_json::json!({
        "users": paginated_users,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": user_list.len(),
            "total_pages": (user_list.len() + per_page as usize - 1) / per_page as usize
        }
    }))
}

async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>
) -> Result<Response> {
    let mut users = state.users.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();

    let user = User {
        id: *next_id,
        name: req.name,
        email: req.email,
        created_at: chrono::Utc::now(),
    };

    users.insert(*next_id, user.clone());
    *next_id += 1;

    Response::json(user)?.status(201)
}

async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<u32>
) -> Result<Response> {
    let users = state.users.lock().unwrap();

    match users.get(&user_id) {
        Some(user) => Response::json(user),
        None => Err(ignitia::Error::NotFound(format!("User {} not found", user_id))),
    }
}

async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<u32>
) -> Result<Response> {
    let mut users = state.users.lock().unwrap();

    match users.remove(&user_id) {
        Some(_) => Response::json(serde_json::json!({"status": "deleted"})),
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
| `.state(state)` | Add shared state | `.state(app_state)` |

### Server Configuration

| Method | Description | Example |
|--------|-------------|---------|
| `Server::new(addr)` | Create server | `Server::new("127.0.0.1:8080")` |
| `.tls(tls_config)` | Enable HTTPS | `.tls(tls_config)` |
| `.self_signed_cert(domain)` | Dev HTTPS | `.self_signed_cert("localhost")` |
| `.run(router)` | Start server | `.run(router).await` |

### Extractors

| Extractor | Type | Example |
|-----------|------|---------|
| `Path<T>` | URL parameters | `Path(user_id): Path<String>` |
| `Query<T>` | Query parameters | `Query(params): Query<SearchParams>` |
| `Json<T>` | JSON body | `Json(user): Json<User>` |
| `Form<T>` | Form data | `Form(form): Form<LoginForm>` |
| `Multipart` | Multipart data | `multipart: Multipart` |
| `Body` | Raw body | `Body(body): Body` |
| `Headers` | Request headers | `headers: Headers` |
| `Cookies` | Request cookies | `cookies: Cookies` |
| `State<T>` | Shared state | `State(state): State<AppState>` |

---

## 🧪 Testing

```
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

```
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
cargo run --example file_upload
```

### Guidelines

1. **Code Quality**: Run `cargo fmt` and `cargo clippy`
2. **Tests**: Add tests for new features
3. **Documentation**: Update docs and examples
4. **Performance**: Benchmark performance-critical changes
5. **Compatibility**: Maintain backwards compatibility

---

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
echo 'ignitia = { version = "0.2.0", features = ["tls", "websocket", "self-signed"] }' >> Cargo.toml

# Create your first high-performance app
cargo run
```

**Join thousands of developers building the future with Ignitia**

[![YouTube](https://img.shields.io/badge/YouTube-Aarambh%20Dev%20Hub-red?style=for-the-badge&logo=youtube)](https://youtube.com/@aarambhdevhub)
[![GitHub](https://img.shields.io/badge/GitHub-ignitia-black?style=for-the-badge&logo=github)](https://github.com/AarambhDevHub/ignitia)
[![Discord](https://img.shields.io/badge/Discord-Community-blue?style=for-the-badge&logo=discord)](https://discord.gg/aarambhdevhub)

---

### 🔥 **Ignitia. Build Fast. Scale Faster.** 🔥

**Built with ❤️ by [Aarambh Dev Hub](https://youtube.com/@aarambhdevhub)**

*Where every line of code ignites possibilities.*

</div>
