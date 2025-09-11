//! # Ignitia - A Blazing Fast Rust Web Framework
//!
//! **Ignitia** is a high-performance, lightweight web framework for Rust that ignites your web development
//! experience with speed, safety, and simplicity. Built on top of modern async Rust with Tokio and Hyper,
//! Ignitia provides everything you need to build fast, reliable web applications and APIs with full
//! HTTP/1.1, HTTP/2, and HTTPS support.
//!
//! ## 🔥 Key Features
//!
//! - **Multi-Protocol Support**: HTTP/1.1, HTTP/2, and HTTPS with automatic protocol negotiation
//! - **TLS/HTTPS**: Built-in TLS support with automatic certificate management and ALPN
//! - **Blazing Fast Performance**: Zero-cost abstractions with optimized connection handling
//! - **Type-Safe Routing**: Compile-time route validation with automatic parameter extraction
//! - **Advanced CORS**: Comprehensive CORS middleware with regex origin matching
//! - **Flexible Middleware**: Composable middleware system for cross-cutting concerns
//! - **WebSocket Support**: Built-in WebSocket support with multiple handler patterns
//! - **Production Ready**: Comprehensive error handling, logging, and observability
//!
//! ## 🚀 Quick Start
//!
//! Add Ignitia to your `Cargo.toml`:
//!
//! ```
//! [dependencies]
//! ignitia = { version = "0.1.8", features = ["tls", "websocket"] }
//! tokio = { version = "1.40", features = ["full"] }
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! Create your first Ignitia application with HTTP/2 support:
//!
//! ```
//! use ignitia::{Router, Server, Response, Http2Config, ServerConfig};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(Response::text("Hello, Ignitia! 🔥")) })
//!         .get("/health", || async { Ok(Response::json("OK")?) })
//!         .post("/echo", |body: String| async move {
//!             Ok(Response::text(format!("Echo: {}", body)))
//!         });
//!
//!     // Configure HTTP/2 with optimized settings
//!     let config = ServerConfig {
//!         http1_enabled: true,
//!         http2: Http2Config {
//!             enabled: true,
//!             max_concurrent_streams: Some(1000),
//!             initial_connection_window_size: Some(1024 * 1024), // 1MB
//!             keep_alive_interval: Some(std::time::Duration::from_secs(60)),
//!             ..Default::default()
//!         },
//!         auto_protocol_detection: true,
//!         ..Default::default()
//!     };
//!
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()?;
//!     Server::new(router, addr)
//!         .with_config(config)
//!         .ignitia()
//!         .await
//! }
//! ```
//!
//! ## 🔒 HTTPS and TLS Support
//!
//! Ignitia provides comprehensive TLS support with automatic protocol negotiation:
//!
//! ### Basic HTTPS Setup
//!
//! ```
//! use ignitia::{Router, Server, TlsConfig};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(Response::text("Secure Hello! 🔒")) });
//!
//!     let addr: SocketAddr = "127.0.0.1:8443".parse()?;
//!
//!     Server::new(router, addr)
//!         .enable_https("server.crt", "server.key")?
//!         .ignitia()
//!         .await
//! }
//! ```
//!
//! ### Advanced TLS Configuration
//!
//! ```
//! use ignitia::{TlsConfig, TlsVersion};
//!
//! let tls_config = TlsConfig::new("cert.pem", "key.pem")
//!     .with_alpn_protocols(vec!["h2", "http/1.1"]) // HTTP/2 priority
//!     .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
//!     .enable_client_cert_verification();
//!
//! Server::new(router, addr)
//!     .with_tls(tls_config)?
//!     .ignitia()
//!     .await
//! ```
//!
//! ### Development with Self-Signed Certificates
//!
//! ```
//! #[cfg(feature = "self-signed")]
//! Server::new(router, addr)
//!     .with_self_signed_cert("localhost")?  // ⚠️ Development only!
//!     .ignitia()
//!     .await
//! ```
//!
//! ### HTTP to HTTPS Redirect
//!
//! ```
//! // Automatically redirect all HTTP traffic to HTTPS
//! Server::new(router, "127.0.0.1:80".parse()?)
//!     .redirect_to_https(443)
//!     .ignitia()
//!     .await
//! ```
//!
//! ## 🌐 Advanced CORS Configuration
//!
//! Ignitia provides a comprehensive CORS middleware with flexible configuration options:
//!
//! ### Basic CORS Setup
//!
//! ```
//! use ignitia::{Router, CorsMiddleware};
//!
//! let router = Router::new()
//!     .middleware(CorsMiddleware::new().allow_any_origin())
//!     .get("/api/data", || async { Ok(Response::json("API data")?) });
//! ```
//!
//! ### Production CORS Configuration
//!
//! ```
//! use ignitia::{CorsMiddleware, Method};
//!
//! let cors = CorsMiddleware::new()
//!     .allowed_origins(&["https://myapp.com", "https://admin.myapp.com"])
//!     .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
//!     .allowed_headers(&["Content-Type", "Authorization", "X-API-Key"])
//!     .expose_headers(&["X-Total-Count", "X-Page-Count"])
//!     .allow_credentials()
//!     .max_age(86400) // 24 hours
//!     .build()?;
//!
//! let router = Router::new()
//!     .middleware(cors)
//!     .get("/api/users", || async { Ok(Response::json("users")?) });
//! ```
//!
//! ### Regex-Based Origin Matching
//!
//! ```
//! let cors = CorsMiddleware::new()
//!     .allowed_origin_regex(r"https://.*\.myapp\.com")  // All subdomains
//!     .build()?;
//! ```
//!
//! ### Convenience Configurations
//!
//! ```
//! // Development (permissive)
//! let dev_cors = CorsMiddleware::permissive();
//!
//! // Production API
//! let api_cors = CorsMiddleware::default_api();
//!
//! // Secure API with specific origins
//! let secure_cors = CorsMiddleware::secure_api(&[
//!     "https://app.example.com",
//!     "https://admin.example.com"
//! ]);
//! ```
//!
//! ## 📡 HTTP/2 Features and Optimization
//!
//! Leverage HTTP/2's advanced features for maximum performance:
//!
//! ### HTTP/2 Configuration
//!
//! ```
//! use ignitia::{Http2Config, ServerConfig};
//! use std::time::Duration;
//!
//! let http2_config = Http2Config {
//!     enabled: true,
//!     enable_prior_knowledge: true,  // H2C support
//!     max_concurrent_streams: Some(1000),
//!     initial_connection_window_size: Some(1024 * 1024), // 1MB
//!     initial_stream_window_size: Some(64 * 1024),       // 64KB
//!     max_frame_size: Some(16 * 1024),                   // 16KB
//!     keep_alive_interval: Some(Duration::from_secs(60)),
//!     keep_alive_timeout: Some(Duration::from_secs(20)),
//!     adaptive_window: true,
//!     max_header_list_size: Some(16 * 1024),
//! };
//!
//! let server_config = ServerConfig {
//!     http1_enabled: true,  // Support both protocols
//!     http2: http2_config,
//!     auto_protocol_detection: true,
//!     ..Default::default()
//! };
//! ```
//!
//! ### Testing HTTP/2 with curl
//!
//! ```
//! # HTTP/2 over TLS (default)
//! curl -v --http2 https://localhost:8443/
//!
//! # HTTP/2 prior knowledge (H2C)
//! curl -v --http2-prior-knowledge http://localhost:8080/
//! ```
//!
//! ## 📚 Core Concepts
//!
//! ### Protocol Negotiation
//!
//! Ignitia automatically negotiates the best protocol based on client capabilities:
//!
//! ```
//! // Server supports both HTTP/1.1 and HTTP/2
//! // Protocol is negotiated via:
//! // 1. ALPN for TLS connections (h2, http/1.1)
//! // 2. HTTP/2 Prior Knowledge for cleartext
//! // 3. Connection Upgrade for HTTP/1.1 -> HTTP/2
//!
//! let config = ServerConfig {
//!     http1_enabled: true,      // HTTP/1.1 support
//!     http2: Http2Config {
//!         enabled: true,        // HTTP/2 support
//!         enable_prior_knowledge: true,  // H2C support
//!         ..Default::default()
//!     },
//!     auto_protocol_detection: true,
//!     ..Default::default()
//! };
//! ```
//!
//! ### Request Extractors with Enhanced Types
//!
//! ```
//! use ignitia::{Path, Query, Json, Headers, Cookies, Body, Method, Uri};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserQuery {
//!     page: Option<u32>,
//!     limit: Option<u32>,
//!     sort: Option<String>,
//! }
//!
//! async fn advanced_handler(
//!     Path(user_id): Path<u32>,
//!     Query(query): Query<UserQuery>,
//!     Json(data): Json<serde_json::Value>,
//!     headers: Headers,
//!     cookies: Cookies,
//!     body: Body,
//!     method: Method,
//!     uri: Uri,
//! ) -> ignitia::Result<Response> {
//!     println!("HTTP Version: {:?}", headers.get("version"));
//!     println!("Protocol: {:?}", headers.get("protocol"));
//!
//!     Response::json(serde_json::json!({
//!         "user_id": user_id,
//!         "query": {
//!             "page": query.page.unwrap_or(1),
//!             "limit": query.limit.unwrap_or(10),
//!             "sort": query.sort.unwrap_or_else(|| "created_at".to_string())
//!         },
//!         "method": method.as_str(),
//!         "path": uri.path(),
//!         "headers_count": headers.len(),
//!         "cookies_count": cookies.len(),
//!         "body_size": body.len()
//!     }))
//! }
//! ```
//!
//! ### Enhanced Middleware Pipeline
//!
//! ```
//! use ignitia::{LoggerMiddleware, CorsMiddleware, AuthMiddleware, ErrorHandlerMiddleware};
//!
//! let router = Router::new()
//!     // Request logging with HTTP version info
//!     .middleware(LoggerMiddleware)
//!
//!     // Advanced CORS with credentials
//!     .middleware(
//!         CorsMiddleware::secure_api(&["https://myapp.com"])
//!             .allow_credentials()
//!             .build()?
//!     )
//!
//!     // Authentication for protected routes
//!     .middleware(
//!         AuthMiddleware::new("secret-token")
//!             .protect_paths(vec!["/admin", "/api/private"])
//!     )
//!
//!     // Custom error handling
//!     .middleware(
//!         ErrorHandlerMiddleware::new()
//!             .with_details(cfg!(debug_assertions))
//!             .with_logging(true)
//!     )
//!
//!     .get("/", || async { Ok(Response::text("Hello, World!")) })
//!     .get("/admin", || async { Ok(Response::text("Admin Panel")) });
//! ```
//!
//! ## 🌐 Enhanced WebSocket Support
//!
//! Full-featured WebSocket implementation with HTTP/2 compatibility:
//!
//! ### Enable WebSocket Support
//!
//! ```
//! [dependencies]
//! ignitia = { version = "0.1.8", features = ["websocket", "tls"] }
//! ```
//!
//! ### Advanced WebSocket Server
//!
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::websocket::{websocket_handler, websocket_message_handler, Message, WebSocketConnection};
//!
//! #[cfg(feature = "websocket")]
//! let router = Router::new()
//!     // Simple echo server
//!     .websocket("/ws/echo", websocket_handler(|ws: WebSocketConnection| async move {
//!         while let Some(message) = ws.recv().await {
//!             match message {
//!                 Message::Text(text) => {
//!                     ws.send_text(format!("Echo: {}", text)).await?;
//!                 }
//!                 Message::Binary(data) => {
//!                     ws.send_bytes(data).await?;
//!                 }
//!                 Message::Close(_) => break,
//!                 _ => {}
//!             }
//!         }
//!         Ok(())
//!     }))
//!
//!     // JSON API over WebSocket
//!     .websocket("/ws/api", websocket_message_handler(|ws, message| async move {
//!         if let Message::Text(text) = message {
//!             if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
//!                 let response = process_api_request(request).await?;
//!                 ws.send_json(&response).await?;
//!             }
//!         }
//!         Ok(())
//!     }))
//!
//!     // Batch processing
//!     .websocket("/ws/batch", ignitia::websocket::websocket_batch_handler(
//!         |ws, messages| async move {
//!             let processed = process_message_batch(messages).await?;
//!             ws.send_batch(processed).await?;
//!             Ok(())
//!         },
//!         100,  // batch size
//!         1000, // timeout ms
//!     ));
//! ```
//!
//! ### WebSocket with HTTPS
//!
//! ```
//! #[cfg(feature = "websocket")]
//! Server::new(router, "127.0.0.1:8443".parse()?)
//!     .enable_https("cert.pem", "key.pem")?
//!     .ignitia()
//!     .await
//!
//! // Client connects via: wss://localhost:8443/ws/echo
//! ```
//!
//! ## 🏗️ Enhanced Architecture
//!
//! Ignitia's modern architecture supports multiple protocols and advanced features:
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                          Application Layer                                  │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
//! │ │   Routes    │ │ Middleware  │ │    CORS     │ │    WebSocket        │   │
//! │ │             │ │             │ │ Configuration│ │    Handlers         │   │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────┘   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                         Ignitia Framework                                   │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
//! │ │   Router    │ │   Server    │ │     TLS     │ │     WebSocket       │   │
//! │ │             │ │             │ │             │ │     Support         │   │
//! │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────────────┐ │   │
//! │ │ │  Route  │ │ │ │HTTP/1.1 │ │ │ │  ALPN   │ │ │ │   Connection    │ │   │
//! │ │ │Matching │ │ │ │Support  │ │ │ │Protocol │ │ │ │   Management    │ │   │
//! │ │ └─────────┘ │ │ └─────────┘ │ │ │Negotiat.│ │ │ └─────────────────┘ │   │
//! │ │             │ │             │ │ └─────────┘ │ │                     │   │
//! │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────────────┐ │   │
//! │ │ │Handler  │ │ │ │HTTP/2   │ │ │ │  Cert   │ │ │ │    Message      │ │   │
//! │ │ │Extract  │ │ │ │Support  │ │ │ │ Management│ │ │   Processing    │ │   │
//! │ │ └─────────┘ │ │ └─────────┘ │ │ └─────────┘ │ │ └─────────────────┘ │   │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────┘   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                        Runtime Layer (Tokio)                               │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
//! │ │    HTTP     │ │     TLS     │ │     TCP     │ │     Async I/O       │   │
//! │ │   (Hyper)   │ │  (Rustls)   │ │ Listeners   │ │                     │   │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 🔧 Feature Flags
//!
//! Ignitia uses feature flags to provide optional functionality:
//!
//! ```
//! [dependencies]
//! # Full feature set (recommended for production)
//! ignitia = { version = "0.1.8", features = ["tls", "websocket", "self-signed"] }
//!
//! # TLS/HTTPS support only
//! ignitia = { version = "0.1.8", features = ["tls"] }
//!
//! # WebSocket support only
//! ignitia = { version = "0.1.8", features = ["websocket"] }
//!
//! # Minimal installation (HTTP only)
//! ignitia = "0.1.8"
//! ```
//!
//! ### Available Features:
//!
//! - **`tls`**: Enables HTTPS/TLS support with certificate management and ALPN
//! - **`websocket`**: Enables WebSocket protocol support with connection management
//! - **`self-signed`**: Enables self-signed certificate generation (development only)
//!
//! ## 🎯 Performance Optimizations
//!
//! Ignitia is designed for maximum performance:
//!
//! - **Multi-Protocol Efficiency**: Automatic protocol selection for optimal performance
//! - **Zero-Cost Abstractions**: Compile-time optimizations with no runtime overhead
//! - **Connection Pooling**: Efficient connection reuse and management
//! - **HTTP/2 Multiplexing**: Multiple streams over single connections
//! - **Optimized Parsing**: Fast HTTP header and body parsing
//! - **Memory Management**: Minimal allocations with smart buffer management
//! - **Async-First Design**: Built on Tokio for excellent concurrency
//!
//! ## 🧪 Testing Your Application
//!
//! Test your multi-protocol applications:
//!
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use ignitia::{Router, Response, Method};
//!
//!     #[tokio::test]
//!     async fn test_cors_endpoint() {
//!         let router = Router::new()
//!             .middleware(CorsMiddleware::permissive())
//!             .get("/api/test", || async {
//!                 Ok(Response::json(serde_json::json!({"status": "ok"}))?)
//!             });
//!
//!         // Test route matching
//!         assert!(router.matches(&Method::GET, "/api/test"));
//!         assert!(router.matches(&Method::OPTIONS, "/api/test")); // CORS preflight
//!     }
//!
//!     #[tokio::test]
//!     async fn test_https_redirect() {
//!         // Test HTTP to HTTPS redirect logic
//!         let config = ServerConfig::default().redirect_to_https(443);
//!         assert!(config.redirect_http_to_https);
//!         assert_eq!(config.https_port, Some(443));
//!     }
//! }
//! ```
//!
//! ## 🔍 Production Examples
//!
//! ### Full-Featured REST API with HTTPS
//!
//! ```
//! use ignitia::{Router, Server, Response, Json, Path, CorsMiddleware, LoggerMiddleware};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct ApiResponse<T> {
//!     success: bool,
//!     data: T,
//!     version: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         // Global middleware
//!         .middleware(LoggerMiddleware)
//!         .middleware(
//!             CorsMiddleware::secure_api(&["https://myapp.com"])
//!                 .allow_credentials()
//!                 .max_age(3600)
//!                 .build()?
//!         )
//!
//!         // API routes
//!         .get("/api/v1/health", || async {
//!             Ok(Response::json(ApiResponse {
//!                 success: true,
//!                 data: "healthy",
//!                 version: "1.0.0".to_string(),
//!             })?)
//!         })
//!
//!         .get("/api/v1/users/:id", |Path(id): Path<u32>| async move {
//!             // Simulate user lookup
//!             let user = serde_json::json!({
//!                 "id": id,
//!                 "name": "John Doe",
//!                 "email": "john@example.com"
//!             });
//!
//!             Ok(Response::json(ApiResponse {
//!                 success: true,
//!                 data: user,
//!                 version: "1.0.0".to_string(),
//!             })?)
//!         });
//!
//!     // Production HTTPS server
//!     let addr = "0.0.0.0:8443".parse()?;
//!     Server::new(router, addr)
//!         .enable_https("production.crt", "production.key")?
//!         .ignitia()
//!         .await
//! }
//! ```
//!
//! ### WebSocket Chat Server with TLS
//!
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::websocket::{websocket_handler, Message, WebSocketConnection};
//! use std::sync::{Arc, Mutex};
//! use std::collections::HashMap;
//!
//! #[cfg(feature = "websocket")]
//! type Clients = Arc<Mutex<HashMap<String, WebSocketConnection>>>;
//!
//! #[cfg(feature = "websocket")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
//!
//!     let router = Router::new()
//!         .get("/", || async {
//!             Ok(Response::html(include_str!("chat.html")))
//!         })
//!         .websocket("/ws/chat", {
//!             let clients = Arc::clone(&clients);
//!             websocket_handler(move |ws: WebSocketConnection| {
//!                 let clients = Arc::clone(&clients);
//!                 async move {
//!                     let client_id = uuid::Uuid::new_v4().to_string();
//!                     clients.lock().unwrap().insert(client_id.clone(), ws.clone());
//!
//!                     while let Some(message) = ws.recv().await {
//!                         match message {
//!                             Message::Text(text) => {
//!                                 // Broadcast to all clients
//!                                 let broadcast_msg = format!("User {}: {}", client_id, text);
//!                                 for client_ws in clients.lock().unwrap().values() {
//!                                     let _ = client_ws.send_text(broadcast_msg.clone()).await;
//!                                 }
//!                             }
//!                             Message::Close(_) => break,
//!                             _ => {}
//!                         }
//!                     }
//!
//!                     clients.lock().unwrap().remove(&client_id);
//!                     Ok(())
//!                 }
//!             })
//!         });
//!
//!     let addr = "127.0.0.1:8443".parse()?;
//!     Server::new(router, addr)
//!         .enable_https("chat.crt", "chat.key")?
//!         .ignitia()
//!         .await
//! }
//! ```
//!
//! ## 📖 Module Documentation
//!
//! - [`cookie`]: HTTP cookie handling and management
//! - [`error`]: Error types, custom error handling, and error response generation
//! - [`extension`]: Type-safe request/response extensions for sharing data
//! - [`handler`]: Request handlers, extractors, and handler trait implementations
//! - [`middleware`]: Middleware system including CORS, auth, and logging
//! - [`request`]: HTTP request representation and utilities
//! - [`response`]: HTTP response building and utilities
//! - [`router`]: Route matching, parameter extraction, and request routing
//! - [`server`]: Multi-protocol server with HTTP/1.1, HTTP/2, and TLS support
//! - [`utils`]: Utility functions for common web development tasks
//! - [`websocket`]: WebSocket protocol support and connection management (feature-gated)
//!
//! ## 🤝 Contributing
//!
//! We welcome contributions! Please see our [contributing guidelines](https://github.com/AarambhDevHub/ignitia/blob/main/CONTRIBUTING.md)
//! for more information.
//!
//! ## 📄 License
//!
//! This project is licensed under the MIT License - see the [LICENSE](https://github.com/AarambhDevHub/ignitia/blob/main/LICENSE)
//! file for details.
//!
//! ## 🔗 Links
//!
//! - [Repository](https://github.com/AarambhDevHub/ignitia)
//! - [Documentation](https://docs.rs/ignitia)
//! - [Examples](https://github.com/AarambhDevHub/ignitia/tree/main/examples)
//! - [Changelog](https://github.com/AarambhDevHub/ignitia/blob/main/CHANGELOG.md)

// Enable documentation features for docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]
// Deny missing docs to ensure comprehensive documentation
#![warn(missing_docs)]
// Enable additional documentation lint rules
#![warn(rustdoc::missing_crate_level_docs)]

// Core framework modules
pub mod cookie;
pub mod error;
pub mod extension;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod utils;

// WebSocket support (feature-gated)
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod websocket;

// Re-export cookie types for easy access
pub use cookie::{Cookie, CookieJar, SameSite};

// Re-export error types and utilities
pub use error::{
    CustomError, Error, ErrorExt, ErrorHandler, ErrorHandlerType, ErrorHandlerWithRequest,
    ErrorResponse, Result,
};

// Re-export extension system
pub use extension::{Extension, Extensions};

// Re-export handler extractors with aliases to avoid naming conflicts
pub use handler::extractor::{
    Body, Cookies, Headers, Json, Method as IgnitiaMethod, Path, Query, Uri,
};

// Re-export handler types and utilities
pub use handler::{
    handler_fn, into_handler, raw_handler, Handler, HandlerFn, IntoHandler, RawRequest,
};

// Re-export middleware types
pub use middleware::{
    AuthMiddleware, BodySizeLimitBuilder, BodySizeLimitMiddleware, CompressionMiddleware,
    CorsMiddleware, ErrorHandlerMiddleware, IdGenerator, LoggerMiddleware, Middleware,
    RequestIdMiddleware, SecurityMiddleware,
};

// Re-export core request and response types
pub use request::Request;
pub use response::{Response, ResponseBuilder};

// Re-export routing components
pub use router::{LayeredHandler, Route, Router};

// Re-export server components
pub use server::{Http2Config, Server, ServerConfig};

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
pub use server::tls::{TlsConfig, TlsError, TlsVersion};

// Re-export WebSocket functionality when feature is enabled
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use websocket::{
    handle_websocket_upgrade, is_websocket_request, upgrade_connection, websocket_batch_handler,
    websocket_handler, websocket_message_handler, BatchMessageHandler, CloseFrame, Message,
    MessageType, OptimizedMessageHandler, WebSocketConnection, WebSocketHandler,
};

// Re-export commonly used external types for convenience
/// Async trait support for defining async traits
pub use async_trait::async_trait;

/// HTTP types from the `http` crate
pub use http::{HeaderMap, HeaderValue, Method, StatusCode};

// Version information
/// The current version of the Ignitia framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the Ignitia framework
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Framework information and build details
pub mod info {
    //! Framework information and build metadata.

    /// Returns the framework name and version as a formatted string
    pub fn version() -> String {
        format!("{} v{}", crate::NAME, crate::VERSION)
    }

    /// Returns build information
    pub fn build_info() -> BuildInfo {
        BuildInfo {
            name: crate::NAME,
            version: crate::VERSION,
            features: get_enabled_features(),
        }
    }

    /// Build information structure
    #[derive(Debug, Clone)]
    pub struct BuildInfo {
        /// Framework name
        pub name: &'static str,
        /// Framework version
        pub version: &'static str,
        /// Enabled features
        pub features: Vec<&'static str>,
    }

    fn get_enabled_features() -> Vec<&'static str> {
        let mut features = Vec::new();

        #[cfg(feature = "tls")]
        features.push("tls");

        #[cfg(feature = "websocket")]
        features.push("websocket");

        #[cfg(feature = "self-signed")]
        features.push("self-signed");

        if features.is_empty() {
            features.push("default");
        }

        features
    }
}

/// Prelude module for common imports
pub mod prelude {
    //! Common imports for Ignitia applications.
    //!
    //! This module provides a convenient way to import the most commonly used
    //! types and traits from Ignitia. Instead of importing each type individually,
    //! you can use:
    //!
    //! ```
    //! use ignitia::prelude::*;
    //! ```

    // Core framework types
    pub use crate::{Error, Request, Response, Result, Router, Server};

    // Server configuration
    pub use crate::{Http2Config, ServerConfig};

    // TLS support (when enabled)
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub use crate::{TlsConfig, TlsError, TlsVersion};

    // Handler and middleware types
    pub use crate::{Handler, HandlerFn, Middleware};

    // Middleware implementations
    pub use crate::{AuthMiddleware, CorsMiddleware, LoggerMiddleware};

    // Common extractors
    pub use crate::{Body, Cookies, Headers, Json, Path, Query, Uri};

    // HTTP types
    pub use crate::{HeaderMap, HeaderValue, Method, StatusCode};

    // Async trait
    pub use crate::async_trait;

    // WebSocket types (when feature is enabled)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub use crate::{Message, MessageType, WebSocketConnection, WebSocketHandler};
}
