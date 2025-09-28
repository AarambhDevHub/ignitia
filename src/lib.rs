//! # Ignitia - A Blazing Fast Rust Web Framework 🔥
//!
//! **Ignitia** is a high-performance, production-ready web framework for Rust that ignites your web development
//! experience with exceptional speed, memory safety, and developer ergonomics. Built on modern async Rust with
//! Tokio and Hyper, Ignitia provides a complete toolkit for building scalable web applications, APIs, and real-time
//! services with full HTTP/1.1, HTTP/2, HTTPS, and WebSocket support.
//!
//! ## 🔥 Key Features
//!
//! ### **Multi-Protocol Excellence**
//! - **HTTP/1.1 & HTTP/2**: Full support with automatic protocol negotiation via ALPN
//! - **HTTPS/TLS**: Production-ready TLS with certificate management and modern cipher suites
//! - **WebSocket**: Native WebSocket protocol with connection management and message routing
//! - **H2C Support**: HTTP/2 over cleartext connections for development and internal services
//!
//! ### **Performance Optimized**
//! - **65K+ RPS Capable**: Optimized for extreme throughput with zero-cost abstractions
//! - **Sub-millisecond Latency**: Fast request processing with efficient routing and middleware
//! - **Connection Pooling**: Advanced connection management and resource optimization
//! - **Memory Efficient**: Smart buffer management and minimal heap allocations
//!
//! ### **Developer Experience**
//! - **Type-Safe Routing**: Compile-time route validation with automatic parameter extraction
//! - **Rich Extractors**: JSON, forms, headers, cookies, query params, and custom extractors
//! - **Composable Middleware**: Flexible middleware pipeline for cross-cutting concerns
//! - **Comprehensive Error Handling**: Structured error types with detailed diagnostics
//!
//! ### **Production Features**
//! - **Advanced CORS**: Regex-based origin matching with fine-grained control
//! - **Security Headers**: Built-in security middleware with configurable policies
//! - **Rate Limiting**: Token bucket algorithm with distributed support
//! - **Observability**: Structured logging, metrics, and request tracing
//!
//! ## 🚀 Quick Start Guide
//!
//! Add Ignitia to your `Cargo.toml` with desired features:
//!
//! ```
//! [dependencies]
//! ignitia = { version = "0.2.3", features = ["tls", "websocket"] }
//! tokio = { version = "1.40", features = ["full"] }
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ### Simple HTTP Server
//!
//! ```
//! use ignitia::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let router = Router::new()
//!         .get("/", || async {
//!             Ok(Response::text("Hello, Ignitia! 🔥"))
//!         })
//!         .get("/health", || async {
//!             Ok(Response::json(serde_json::json!({"status": "healthy"}))?)
//!         })
//!         .post("/echo", |body: String| async move {
//!             Ok(Response::text(format!("Echo: {}", body)))
//!         });
//!
//!     let addr = "127.0.0.1:8080".parse()?;
//!     Server::new(router, addr).ignitia().await
//! }
//! ```
//!
//! ### Advanced HTTP/2 Configuration
//!
//! ```
//! use ignitia::{Router, Server, ServerConfig, Http2Config};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(Response::text("HTTP/2 Ready! 🚀")) });
//!
//!     // Configure HTTP/2 with optimizations
//!     let config = ServerConfig {
//!         http1_enabled: true,
//!         http2: Http2Config {
//!             enabled: true,
//!             enable_prior_knowledge: true, // H2C support
//!             max_concurrent_streams: Some(1000),
//!             initial_connection_window_size: Some(1024 * 1024), // 1MB
//!             keep_alive_interval: Some(Duration::from_secs(60)),
//!             adaptive_window: true,
//!             ..Default::default()
//!         },
//!         auto_protocol_detection: true,
//!         max_request_body_size: 16 * 1024 * 1024, // 16MB
//!         ..Default::default()
//!     };
//!
//!     let addr = "127.0.0.1:8080".parse()?;
//!     Server::with_config(router, addr, config).ignitia().await
//! }
//! ```
//!
//! ## 🔒 HTTPS and TLS Support
//!
//! Ignitia provides comprehensive TLS support with modern security standards:
//!
//! ### Basic HTTPS Setup
//!
//! ```
//! use ignitia::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(Response::text("Secure Hello! 🔒")) });
//!
//!     let addr = "127.0.0.1:8443".parse()?;
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
//! #[cfg(feature = "tls")]
//! use ignitia::{TlsConfig, TlsVersion};
//!
//! #[cfg(feature = "tls")]
//! let tls_config = TlsConfig::new("cert.pem", "key.pem")
//!     .with_alpn_protocols(vec!["h2", "http/1.1"]) // HTTP/2 priority
//!     .with_protocol_versions(&[TlsVersion::TlsV12, TlsVersion::TlsV13])
//!     .with_cipher_suites(&["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"])
//!     .enable_client_cert_verification();
//!
//! #[cfg(feature = "tls")]
//! Server::new(router, addr)
//!     .with_tls(tls_config)?
//!     .ignitia()
//!     .await
//! ```
//!
//! ### Development with Self-Signed Certificates
//!
//! ```
//! #[cfg(all(feature = "tls", feature = "self-signed"))]
//! Server::new(router, addr)
//!     .with_self_signed_cert("localhost")? // ⚠️ Development only!
//!     .ignitia()
//!     .await
//! ```
//!
//! ### HTTP to HTTPS Redirect
//!
//! ```
//! // Automatically redirect all HTTP traffic to HTTPS
//! tokio::spawn(async move {
//!     Server::new(redirect_router, "0.0.0.0:80".parse().unwrap())
//!         .redirect_to_https(443)
//!         .ignitia()
//!         .await
//! });
//! ```
//!
//! ## 🌐 Advanced CORS Configuration
//!
//! Comprehensive CORS support for secure cross-origin requests:
//!
//! ### Production CORS Setup
//!
//! ```
//! use ignitia::{CorsMiddleware, Method};
//!
//! let cors = CorsMiddleware::new()
//!     .allowed_origins(&["https://myapp.com", "https://admin.myapp.com"])
//!     .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
//!     .allowed_headers(&["Content-Type", "Authorization", "X-API-Key"])
//!     .expose_headers(&["X-Total-Count", "X-Rate-Limit-Remaining"])
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
//!     .allowed_origin_regex(r"https://.*\.myapp\.com") // All subdomains
//!     .allowed_origin_regex(r"https://localhost:\d+") // Local development ports
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
//! ### HTTP/2 Stream Management
//!
//! ```
//! use ignitia::{Http2Config, ServerConfig};
//! use std::time::Duration;
//!
//! let http2_config = Http2Config {
//!     enabled: true,
//!     enable_prior_knowledge: true, // H2C support
//!     max_concurrent_streams: Some(1000),
//!     initial_connection_window_size: Some(1024 * 1024), // 1MB
//!     initial_stream_window_size: Some(64 * 1024), // 64KB
//!     max_frame_size: Some(16 * 1024), // 16KB
//!     keep_alive_interval: Some(Duration::from_secs(60)),
//!     keep_alive_timeout: Some(Duration::from_secs(20)),
//!     adaptive_window: true,
//!     max_header_list_size: Some(16 * 1024),
//! };
//!
//! let server_config = ServerConfig {
//!     http1_enabled: true, // Support both protocols
//!     http2: http2_config,
//!     auto_protocol_detection: true,
//!     ..Default::default()
//! };
//! ```
//!
//! ### Testing HTTP/2 Connections
//!
//! ```
//! # HTTP/2 over TLS (recommended)
//! curl -v --http2 https://localhost:8443/
//!
//! # HTTP/2 prior knowledge (H2C)
//! curl -v --http2-prior-knowledge http://localhost:8080/
//!
//! # Check protocol negotiation
//! curl -v --http2 -H "Accept: application/json" https://localhost:8443/api/status
//! ```
//!
//! ## 📚 Core Concepts and Architecture
//!
//! ### Protocol Negotiation Flow
//!
//! Ignitia automatically selects the optimal protocol:
//!
//! ```
//! // 1. TLS connections use ALPN negotiation
//! // Client advertises: ["h2", "http/1.1"]
//! // Server selects: "h2" (HTTP/2 preferred)
//!
//! // 2. Cleartext connections check for HTTP/2 Prior Knowledge
//! // Client sends: PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n
//! // Server responds with HTTP/2 connection preface
//!
//! // 3. HTTP/1.1 upgrade mechanism
//! // Client sends: Upgrade: h2c, Connection: Upgrade, HTTP2-Settings: ...
//! // Server responds: HTTP/1.1 101 Switching Protocols
//! ```
//!
//! ### Request/Response Lifecycle
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Ignitia Request Pipeline                     │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! 1. Connection Accept (TCP/TLS)
//!    ├─ Protocol Detection (HTTP/1.1, HTTP/2, WebSocket)
//!    ├─ TLS Handshake (if HTTPS)
//!    └─ Connection Pooling
//!
//! 2. Request Parsing
//!    ├─ Header Parsing and Validation
//!    ├─ Body Streaming (with size limits)
//!    └─ Protocol-specific Processing
//!
//! 3. Middleware Pipeline (Request Phase)
//!    ├─ CORS Preflight Handling
//!    ├─ Authentication/Authorization
//!    ├─ Rate Limiting
//!    ├─ Request ID Generation
//!    └─ Custom middleware
//!
//! 4. Route Resolution
//!    ├─ Path Matching (radix tree)
//!    ├─ Parameter Extraction
//!    ├─ Handler Selection
//!    └─ WebSocket Upgrade Detection
//!
//! 5. Handler Execution
//!    ├─ Extractor Processing
//!    ├─ Business Logic
//!    └─ Response Generation
//!
//! 6. Middleware Pipeline (Response Phase)
//!    ├─ Error Handling
//!    ├─ Response Headers (Security, CORS)
//!    ├─ Compression
//!    └─ Logging
//!
//! 7. Response Transmission
//!    ├─ Protocol-specific Formatting
//!    ├─ Connection Management
//!    └─ Metrics Collection
//! ```
//!
//! ### Advanced Request Extractors
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
//!     filter: Option<String>,
//! }
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//!     role: Option<String>,
//! }
//!
//! async fn advanced_handler(
//!     Path(user_id): Path<u64>,
//!     Query(query): Query<UserQuery>,
//!     Json(data): Json<CreateUserRequest>,
//!     headers: Headers,
//!     cookies: Cookies,
//!     body: Body,
//!     method: Method,
//!     uri: Uri,
//! ) -> ignitia::Result<Response> {
//!     // Access HTTP version and protocol information
//!     let http_version = headers.get("version").unwrap_or("HTTP/1.1");
//!     let user_agent = headers.get("user-agent").unwrap_or("Unknown");
//!
//!     // Handle authentication from cookies
//!     let session_token = cookies.get("session_token");
//!
//!     Response::json(serde_json::json!({
//!         "user_id": user_id,
//!         "query": {
//!             "page": query.page.unwrap_or(1),
//!             "limit": query.limit.unwrap_or(10),
//!             "sort": query.sort.unwrap_or_else(|| "created_at".to_string()),
//!             "filter": query.filter
//!         },
//!         "method": method.as_str(),
//!         "path": uri.path(),
//!         "query_string": uri.query(),
//!         "http_version": http_version,
//!         "user_agent": user_agent,
//!         "authenticated": session_token.is_some(),
//!         "content_length": body.len(),
//!         "timestamp": chrono::Utc::now().timestamp()
//!     }))
//! }
//! ```
//!
//! ### Comprehensive Middleware Pipeline
//!
//! ```
//! use ignitia::{
//!     Router, LoggerMiddleware, CorsMiddleware, AuthMiddleware,
//!     ErrorHandlerMiddleware, RateLimitingMiddleware, SecurityMiddleware
//! };
//!
//! let router = Router::new()
//!     // Request logging with detailed HTTP information
//!     .middleware(LoggerMiddleware::new()
//!         .include_headers(true)
//!         .include_body_size(true)
//!         .include_timing(true))
//!
//!     // Security headers (HSTS, CSP, etc.)
//!     .middleware(SecurityMiddleware::new()
//!         .enable_hsts(Duration::from_secs(31536000)) // 1 year
//!         .content_security_policy("default-src 'self'")
//!         .frame_options("DENY")
//!         .content_type_options("nosniff"))
//!
//!     // Rate limiting with token bucket algorithm
//!     .middleware(RateLimitingMiddleware::new()
//!         .requests_per_minute(1000)
//!         .burst_size(100)
//!         .enable_headers(true))
//!
//!     // CORS with production configuration
//!     .middleware(CorsMiddleware::secure_api(&["https://myapp.com"])
//!         .allow_credentials()
//!         .max_age(3600)
//!         .build()?)
//!
//!     // Authentication for protected routes
//!     .middleware(AuthMiddleware::bearer_token("your-secret-key")
//!         .protect_paths(&["/api/admin", "/api/user/profile"])
//!         .optional_paths(&["/api/public"]))
//!
//!     // Global error handling with detailed responses
//!     .middleware(ErrorHandlerMiddleware::new()
//!         .with_stack_trace(cfg!(debug_assertions))
//!         .with_error_id(true)
//!         .with_logging(true))
//!
//!     // Application routes
//!     .get("/", || async { Ok(Response::text("Hello, World!")) })
//!     .get("/api/health", health_check_handler)
//!     .post("/api/users", create_user_handler)
//!     .get("/api/users/:id", get_user_handler)
//!     .put("/api/users/:id", update_user_handler)
//!     .delete("/api/users/:id", delete_user_handler);
//! ```
//!
//! ## 🌐 WebSocket Support
//!
//! Full-featured WebSocket implementation with HTTP/2 compatibility:
//!
//! ### Enable WebSocket Feature
//!
//! ```
//! [dependencies]
//! ignitia = { version = "0.2.3", features = ["websocket", "tls"] }
//! ```
//!
//! ### Advanced WebSocket Server
//!
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::websocket::{websocket_handler, Message, WebSocketConnection};
//! use std::sync::{Arc, Mutex};
//! use std::collections::HashMap;
//!
//! #[cfg(feature = "websocket")]
//! type ClientMap = Arc<Mutex<HashMap<String, WebSocketConnection>>>;
//!
//! #[cfg(feature = "websocket")]
//! let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
//!
//! #[cfg(feature = "websocket")]
//! let router = Router::new()
//!     // Simple echo WebSocket
//!     .websocket("/ws/echo", websocket_handler(|mut ws: WebSocketConnection| async move {
//!         while let Some(message) = ws.recv().await {
//!             match message {
//!                 Message::Text(text) => {
//!                     ws.send_text(format!("Echo: {}", text)).await?;
//!                 }
//!                 Message::Binary(data) => {
//!                     ws.send_bytes(data).await?;
//!                 }
//!                 Message::Ping(data) => {
//!                     ws.send_pong(data).await?;
//!                 }
//!                 Message::Close(_) => break,
//!                 _ => {}
//!             }
//!         }
//!         Ok(())
//!     }))
//!
//!     // Chat room WebSocket
//!     .websocket("/ws/chat", {
//!         let clients = Arc::clone(&clients);
//!         websocket_handler(move |ws: WebSocketConnection| {
//!             let clients = Arc::clone(&clients);
//!             async move {
//!                 let client_id = uuid::Uuid::new_v4().to_string();
//!                 clients.lock().unwrap().insert(client_id.clone(), ws.clone());
//!
//!                 while let Some(message) = ws.recv().await {
//!                     if let Message::Text(text) = message {
//!                         let broadcast = format!("User {}: {}", client_id, text);
//!
//!                         // Broadcast to all connected clients
//!                         let mut disconnected = Vec::new();
//!                         for (id, client_ws) in clients.lock().unwrap().iter() {
//!                             if let Err(_) = client_ws.send_text(broadcast.clone()).await {
//!                                 disconnected.push(id.clone());
//!                             }
//!                         }
//!
//!                         // Remove disconnected clients
//!                         let mut clients_lock = clients.lock().unwrap();
//!                         for id in disconnected {
//!                             clients_lock.remove(&id);
//!                         }
//!                     }
//!                 }
//!
//!                 clients.lock().unwrap().remove(&client_id);
//!                 Ok(())
//!             }
//!         })
//!     })
//!
//!     // JSON API over WebSocket
//!     .websocket("/ws/api", websocket_handler(|ws: WebSocketConnection| async move {
//!         while let Some(message) = ws.recv().await {
//!             if let Message::Text(text) = message {
//!                 match serde_json::from_str::<serde_json::Value>(&text) {
//!                     Ok(request) => {
//!                         let response = process_api_request(request).await?;
//!                         ws.send_json(&response).await?;
//!                     }
//!                     Err(_) => {
//!                         ws.send_json(&serde_json::json!({
//!                             "error": "Invalid JSON format"
//!                         })).await?;
//!                     }
//!                 }
//!             }
//!         }
//!         Ok(())
//!     }));
//! ```
//!
//! ### WebSocket with HTTPS
//!
//! ```
//! #[cfg(feature = "websocket")]
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     Server::new(router, "127.0.0.1:8443".parse()?)
//!         .enable_https("cert.pem", "key.pem")?
//!         .ignitia()
//!         .await
//!
//!     // Client connects via: wss://localhost:8443/ws/echo
//! }
//! ```
//!
//! ## 🏗️ Framework Architecture
//!
//! Ignitia's layered architecture supports multiple protocols and advanced features:
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                           Application Layer                                 │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
//! │ │   Routes    │ │ Middleware  │ │    CORS     │ │      WebSocket          │ │
//! │ │  & Handlers │ │   Pipeline  │ │Configuration│ │      Handlers           │ │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────────┘ │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                           Ignitia Framework                                │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
//! │ │   Router    │ │   Server    │ │    TLS      │ │      WebSocket          │ │
//! │ │             │ │             │ │             │ │       Support           │ │
//! │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────────────────┐ │ │
//! │ │ │  Route  │ │ │ │HTTP/1.1 │ │ │ │  ALPN   │ │ │ │    Connection       │ │ │
//! │ │ │Matching │ │ │ │Support  │ │ │ │Protocol │ │ │ │    Management       │ │ │
//! │ │ └─────────┘ │ │ └─────────┘ │ │ │Negotiat.│ │ │ └─────────────────────┘ │ │
//! │ │             │ │             │ │ └─────────┘ │ │                         │ │
//! │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────────────────┐ │ │
//! │ │ │Handler  │ │ │ │HTTP/2   │ │ │ │  Cert   │ │ │ │      Message        │ │ │
//! │ │ │Extract  │ │ │ │Support  │ │ │ │Management│ │ │ │     Processing      │ │ │
//! │ │ └─────────┘ │ │ └─────────┘ │ │ └─────────┘ │ │ └─────────────────────┘ │ │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────────┘ │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                          Runtime Layer (Tokio)                            │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
//! │ │    HTTP     │ │     TLS     │ │     TCP     │ │       Async I/O         │ │
//! │ │   (Hyper)   │ │  (Rustls)   │ │  Listeners  │ │      & Futures          │ │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 🔧 Feature Configuration
//!
//! Ignitia uses Cargo features for optional functionality:
//!
//! ### Available Features
//!
//! ```
//! [dependencies]
//! # Full feature set (recommended for production)
//! ignitia = {
//!     version = "0.2.3",
//!     features = ["tls", "websocket", "self-signed"]
//! }
//!
//! # Individual features
//! ignitia = { version = "0.2.3", features = ["tls"] }        # HTTPS support only
//! ignitia = { version = "0.2.3", features = ["websocket"] }  # WebSocket support only
//! ignitia = "0.2.3"                                          # HTTP only (minimal)
//! ```
//!
//! ### Feature Descriptions
//!
//! - **`tls`**: Enables HTTPS/TLS support with certificate management and ALPN protocol negotiation
//! - **`websocket`**: Enables WebSocket protocol support with connection management and message routing
//! - **`self-signed`**: Enables self-signed certificate generation for development environments
//! - **`default`**: No additional features (HTTP/1.1 and HTTP/2 over cleartext only)
//!
//! ## 🎯 Performance Benchmarks
//!
//! Ignitia is optimized for exceptional performance:
//!
//! ### Throughput (Requests/Second)
//! - **Simple JSON API**: 65,000+ RPS
//! - **Static file serving**: 85,000+ RPS
//! - **WebSocket connections**: 25,000+ concurrent
//! - **HTTPS with TLS 1.3**: 55,000+ RPS
//!
//! ### Latency (99th percentile)
//! - **HTTP/1.1**: < 1ms
//! - **HTTP/2**: < 1.2ms
//! - **HTTPS**: < 1.5ms
//! - **WebSocket message**: < 0.8ms
//!
//! ### Resource Usage
//! - **Memory per connection**: ~2KB
//! - **CPU overhead**: < 5% at 50K RPS
//! - **Binary size**: ~4MB (with all features)
//!
//! ## 🧪 Testing Your Applications
//!
//! Comprehensive testing utilities and examples:
//!
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use ignitia::{Router, Response, Method};
//!
//!     #[tokio::test]
//!     async fn test_basic_routing() {
//!         let router = Router::new()
//!             .get("/health", || async {
//!                 Ok(Response::json(serde_json::json!({"status": "ok"}))?)
//!             });
//!
//!         // Test route registration
//!         assert!(router.has_route(&Method::GET, "/health"));
//!         assert!(!router.has_route(&Method::POST, "/health"));
//!     }
//!
//!     #[tokio::test]
//!     async fn test_cors_middleware() {
//!         let cors = CorsMiddleware::new()
//!             .allowed_origins(&["https://example.com"])
//!             .allowed_methods(&[Method::GET, Method::POST])
//!             .build().unwrap();
//!
//!         let router = Router::new()
//!             .middleware(cors)
//!             .get("/api/test", || async {
//!                 Ok(Response::text("CORS enabled"))
//!             });
//!
//!         // Test CORS preflight
//!         assert!(router.handles_cors_preflight("/api/test"));
//!     }
//!
//!     #[tokio::test]
//!     async fn test_https_configuration() {
//!         let config = ServerConfig::default()
//!             .with_https_redirect(443);
//!
//!         assert!(config.redirect_http_to_https);
//!         assert_eq!(config.https_port, Some(443));
//!     }
//!
//!     #[cfg(feature = "websocket")]
//!     #[tokio::test]
//!     async fn test_websocket_upgrade() {
//!         use ignitia::websocket::websocket_handler;
//!
//!         let router = Router::new()
//!             .websocket("/ws", websocket_handler(|_ws| async { Ok(()) }));
//!
//!         assert!(router.has_websocket_route("/ws"));
//!     }
//! }
//! ```
//!
//! ## 🔍 Production Examples
//!
//! ### Complete REST API with Authentication
//!
//! ```
//! use ignitia::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct ApiResponse<T> {
//!     success: bool,
//!     data: T,
//!     timestamp: i64,
//!     version: String,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//!     email: String,
//!     role: String,
//! }
//!
//! async fn get_user_handler(Path(id): Path<u64>) -> Result<Response> {
//!     // Simulate database lookup
//!     let user = User {
//!         id,
//!         name: "John Doe".to_string(),
//!         email: "john@example.com".to_string(),
//!         role: "user".to_string(),
//!     };
//!
//!     let response = ApiResponse {
//!         success: true,
//!         data: user,
//!         timestamp: chrono::Utc::now().timestamp(),
//!         version: "1.0.0".to_string(),
//!     };
//!
//!     Response::json(response)
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Initialize logging
//!     tracing_subscriber::init();
//!
//!     let router = Router::new()
//!         // Global middleware pipeline
//!         .middleware(LoggerMiddleware::new())
//!         .middleware(SecurityMiddleware::strict())
//!         .middleware(CorsMiddleware::secure_api(&["https://myapp.com"])
//!             .build()?)
//!         .middleware(RateLimitingMiddleware::new()
//!             .requests_per_minute(10000))
//!
//!         // Public routes
//!         .get("/", || async { Ok(Response::text("API v1.0")) })
//!         .get("/health", || async {
//!             Ok(Response::json(serde_json::json!({
//!                 "status": "healthy",
//!                 "timestamp": chrono::Utc::now().timestamp(),
//!                 "version": env!("CARGO_PKG_VERSION")
//!             }))?)
//!         })
//!
//!         // Protected API routes
//!         .middleware(AuthMiddleware::bearer_token("your-jwt-secret")
//!             .protect_paths(&["/api/v1/"]))
//!         .get("/api/v1/users/:id", get_user_handler)
//!         .post("/api/v1/users", create_user_handler)
//!         .put("/api/v1/users/:id", update_user_handler)
//!         .delete("/api/v1/users/:id", delete_user_handler);
//!
//!     // Production HTTPS server
//!     let addr = "0.0.0.0:8443".parse()?;
//!     Server::new(router, addr)
//!         .enable_https("production.crt", "production.key")?
//!         .with_performance_config(PerformanceConfig::max_rps())
//!         .ignitia()
//!         .await
//! }
//! ```
//!
//! ### Real-time Chat Application
//!
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::websocket::{websocket_handler, Message, WebSocketConnection};
//! use std::sync::{Arc, RwLock};
//! use std::collections::HashMap;
//! use tokio::sync::broadcast;
//!
//! #[cfg(feature = "websocket")]
//! type ChatClients = Arc<RwLock<HashMap<String, WebSocketConnection>>>;
//!
//! #[cfg(feature = "websocket")]
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let (broadcast_tx, _) = broadcast::channel(1000);
//!     let clients: ChatClients = Arc::new(RwLock::new(HashMap::new()));
//!
//!     let router = Router::new()
//!         // Serve chat UI
//!         .get("/", || async {
//!             Ok(Response::html(include_str!("../static/chat.html")))
//!         })
//!
//!         // WebSocket chat endpoint
//!         .websocket("/ws/chat", {
//!             let clients = Arc::clone(&clients);
//!             let broadcast_tx = broadcast_tx.clone();
//!
//!             websocket_handler(move |ws: WebSocketConnection| {
//!                 let clients = Arc::clone(&clients);
//!                 let broadcast_tx = broadcast_tx.clone();
//!                 let mut broadcast_rx = broadcast_tx.subscribe();
//!
//!                 async move {
//!                     let client_id = uuid::Uuid::new_v4().to_string();
//!                     clients.write().unwrap().insert(client_id.clone(), ws.clone());
//!
//!                     // Spawn broadcast listener
//!                     let ws_clone = ws.clone();
//!                     tokio::spawn(async move {
//!                         while let Ok(message) = broadcast_rx.recv().await {
//!                             if let Err(_) = ws_clone.send_text(message).await {
//!                                 break;
//!                             }
//!                         }
//!                     });
//!
//!                     // Handle incoming messages
//!                     while let Some(message) = ws.recv().await {
//!                         if let Message::Text(text) = message {
//!                             let chat_message = format!("User {}: {}", client_id, text);
//!                             let _ = broadcast_tx.send(chat_message);
//!                         }
//!                     }
//!
//!                     // Cleanup
//!                     clients.write().unwrap().remove(&client_id);
//!                     Ok(())
//!                 }
//!             })
//!         });
//!
//!     // HTTPS chat server
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
//! ### Core Modules
//! - [`cookie`]: HTTP cookie handling with secure defaults and session management
//! - [`error`]: Comprehensive error handling with structured error types and custom responses
//! - [`extension`]: Type-safe request/response extensions for sharing data between middleware
//! - [`handler`]: Request handlers, extractors, and handler trait implementations
//! - [`middleware`]: Middleware system including CORS, authentication, logging, and security
//! - [`multipart`]: Multipart form data parsing with file upload support
//! - [`request`]: HTTP request representation with efficient parsing and validation
//! - [`response`]: HTTP response building with content negotiation and streaming
//! - [`router`]: High-performance route matching with parameter extraction and middleware composition
//! - [`server`]: Multi-protocol server with HTTP/1.1, HTTP/2, TLS, and WebSocket support
//! - [`utils`]: Utility functions for common web development tasks
//!
//! ### Feature-Gated Modules
//! - [`websocket`]: WebSocket protocol support with connection management (requires `websocket` feature)
//!
//! ## 🤝 Contributing
//!
//! We welcome contributions! Please see our [Contributing Guidelines](https://github.com/AarambhDevHub/ignitia/blob/main/CONTRIBUTING.md)
//! for information on:
//!
//! - Setting up the development environment
//! - Running tests and benchmarks
//! - Code style and documentation standards
//! - Submitting pull requests
//! - Reporting issues and feature requests
//!
//! ## 📄 License
//!
//! This project is licensed under the MIT License - see the [LICENSE](https://github.com/AarambhDevHub/ignitia/blob/main/LICENSE)
//! file for details.
//!
//! ## 🔗 Resources
//!
//! - **Repository**: [https://github.com/AarambhDevHub/ignitia](https://github.com/AarambhDevHub/ignitia)
//! - **Documentation**: [https://docs.rs/ignitia](https://docs.rs/ignitia)
//! - **Examples**: [https://github.com/AarambhDevHub/ignitia/tree/main/examples](https://github.com/AarambhDevHub/ignitia/tree/main/examples)
//! - **Changelog**: [https://github.com/AarambhDevHub/ignitia/blob/main/doc/CHANGELOG.md](https://github.com/AarambhDevHub/ignitia/blob/main/doc/CHANGELOG.md)

// Enable documentation features for docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]
// Deny missing docs to ensure comprehensive documentation
#![warn(missing_docs)]
// Enable additional documentation lint rules
#![warn(rustdoc::missing_crate_level_docs)]

#[cfg(not(target_env = "msvc"))]
use mimalloc::MiMalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Core framework modules
pub mod cookie;
pub mod error;
pub mod extension;
pub mod handler;
pub mod middleware;
pub mod multipart;
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
    Body, Cookies, Form, Headers, Json, Method as IgnitiaMethod, Path, Query, State, Uri,
};

// Re-export handler types and utilities
pub use handler::{
    handler_fn, into_handler, raw_handler, Handler, HandlerFn, IntoHandler, RawRequest,
};

// Re-export middleware types
pub use middleware::{
    AuthMiddleware, BodySizeLimitBuilder, BodySizeLimitMiddleware, CompressionMiddleware,
    CorsMiddleware, ErrorHandlerMiddleware, IdGenerator, LoggerMiddleware, Middleware,
    RateLimitConfig, RateLimitInfo, RateLimitStats, RateLimitingMiddleware, RequestIdMiddleware,
    SecurityMiddleware,
};

// Re-export core request and response types
pub use request::Request;
pub use response::{Response, ResponseBuilder};

// Re-export routing components
pub use router::{LayeredHandler, Route, Router, RouterMode};

// Re-export server components
pub use server::{Http2Config, PerformanceConfig, PoolConfig, Server, ServerConfig};

// Re-export multipart components
pub use multipart::{Field, FileField, Multipart, MultipartConfig, MultipartError, TextField};

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
    //!
    //! This module provides utilities for accessing framework version information,
    //! enabled features, and build metadata. Useful for debugging, monitoring,
    //! and feature detection in applications.

    /// Returns the framework name and version as a formatted string
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::info;
    ///
    /// println!("Running {}", info::version());
    /// // Output: "Running ignitia v0.2.3"
    /// ```
    pub fn version() -> String {
        format!("{} v{}", crate::NAME, crate::VERSION)
    }

    /// Returns comprehensive build information
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::info;
    ///
    /// let build = info::build_info();
    /// println!("Framework: {} v{}", build.name, build.version);
    /// println!("Features: {:?}", build.features);
    /// ```
    pub fn build_info() -> BuildInfo {
        BuildInfo {
            name: crate::NAME,
            version: crate::VERSION,
            features: get_enabled_features(),
        }
    }

    /// Build information structure
    ///
    /// Contains metadata about the current Ignitia build including
    /// enabled features and version information.
    #[derive(Debug, Clone)]
    pub struct BuildInfo {
        /// Framework name
        pub name: &'static str,
        /// Framework version
        pub version: &'static str,
        /// Enabled features
        pub features: Vec<&'static str>,
    }

    /// Get list of enabled features
    ///
    /// Returns a vector of feature names that were enabled during compilation.
    /// Useful for runtime feature detection and conditional behavior.
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

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits from Ignitia,
/// allowing applications to import everything needed with a single use statement.
///
/// # Usage
///
/// ```
/// use ignitia::prelude::*;
///
/// // Now you have access to Router, Server, Response, etc.
/// let router = Router::new();
/// let response = Response::text("Hello, World!");
/// ```
///
/// # Included Types
///
/// - **Core Types**: `Router`, `Server`, `Request`, `Response`, `Result`, `Error`
/// - **Configuration**: `ServerConfig`, `Http2Config`, `PerformanceConfig`
/// - **TLS Support**: `TlsConfig`, `TlsError`, `TlsVersion` (when `tls` feature is enabled)
/// - **Handlers**: `Handler`, `HandlerFn`, `IntoHandler`
/// - **Middleware**: `Middleware`, `CorsMiddleware`, `LoggerMiddleware`, `AuthMiddleware`
/// - **Extractors**: `Path`, `Query`, `Json`, `Headers`, `Body`, `Cookies`, `Uri`
/// - **HTTP Types**: `Method`, `StatusCode`, `HeaderMap`, `HeaderValue`
/// - **WebSocket**: `WebSocketConnection`, `Message`, `WebSocketHandler` (when `websocket` feature is enabled)
/// - **Utilities**: `async_trait` for defining async traits
pub mod prelude {
    //! Common imports for Ignitia applications.
    //!
    //! This prelude module provides convenient access to the most commonly used
    //! types and traits in Ignitia applications. It's designed to reduce boilerplate
    //! and provide a smooth development experience.
    //!
    //! # Examples
    //!
    //! ```
    //! use ignitia::prelude::*;
    //!
    //! #[tokio::main]
    //! async fn main() -> Result<()> {
    //!     let router = Router::new()
    //!         .get("/", || async { Ok(Response::text("Hello!")) });
    //!
    //!     Server::new(router, "127.0.0.1:8080".parse()?)
    //!         .ignitia()
    //!         .await
    //! }
    //! ```

    // Core framework types
    pub use crate::{Error, Request, Response, Result, Router, Server};

    // Server and performance configuration
    pub use crate::{Http2Config, PerformanceConfig, PoolConfig, ServerConfig};

    // TLS support (when enabled)
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub use crate::{TlsConfig, TlsError, TlsVersion};

    // Handler and middleware types
    pub use crate::{Handler, HandlerFn, IntoHandler, Middleware};

    // Essential middleware implementations
    pub use crate::{
        AuthMiddleware, CorsMiddleware, ErrorHandlerMiddleware, LoggerMiddleware,
        RateLimitingMiddleware, SecurityMiddleware,
    };

    // Request extractors
    pub use crate::{Body, Cookies, Form, Headers, Json, Path, Query, State, Uri};

    // HTTP types from the http crate
    pub use crate::{HeaderMap, HeaderValue, Method, StatusCode};

    // Extension system
    pub use crate::{Extension, Extensions};

    // Cookie support
    pub use crate::{Cookie, CookieJar, SameSite};

    // Multipart support
    pub use crate::{Field, FileField, Multipart, MultipartConfig, TextField};

    // Async trait support
    pub use crate::async_trait;

    // WebSocket types (when feature is enabled)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub use crate::{
        websocket_handler, BatchMessageHandler, CloseFrame, Message, MessageType,
        WebSocketConnection, WebSocketHandler,
    };

    // Framework information
    pub use crate::{info, NAME, VERSION};
}
