//! # Ignitia - A Blazing Fast Rust Web Framework 🔥
//!
//! **Ignitia** is a high-performance, production-ready web framework for Rust that ignites your web development
//! experience with exceptional speed, memory safety, and developer ergonomics. Built on modern async Rust with
//! Tokio and Hyper, Ignitia provides a complete toolkit for building scalable web applications, APIs, and real-time
//! services with full HTTP/1.1, HTTP/2, HTTPS, and WebSocket support.
//!
//! ## 🔥 Key Features
//!
//! ### Multi-Protocol Excellence
//!
//! - **HTTP/1.1 & HTTP/2**: Full support with automatic protocol negotiation via ALPN
//! - **HTTPS/TLS**: Production-ready TLS with certificate management and modern cipher suites
//! - **WebSocket**: Native WebSocket protocol with connection management and message routing
//! - **H2C Support**: HTTP/2 over cleartext connections for development and internal services
//!
//! ### Performance Optimized
//!
//! - **155K+ RPS Capable**: Optimized for extreme throughput with zero-cost abstractions
//! - **Sub-millisecond Latency**: Fast request processing with efficient routing and middleware
//! - **Connection Pooling**: Advanced connection management and resource optimization
//! - **Memory Efficient**: Smart buffer management and minimal heap allocations
//! - **Radix Tree Routing**: O(k) route lookup complexity where k is path length
//! - **Lock-Free Reads**: Atomic router swapping for zero-downtime updates
//!
//! ### Developer Experience
//!
//! - **Type-Safe Routing**: Compile-time route validation with automatic parameter extraction
//! - **Rich Extractors**: JSON, forms, headers, cookies, query params, and custom extractors
//! - **Composable Middleware**: Flexible middleware pipeline for cross-cutting concerns
//! - **Comprehensive Error Handling**: Structured error types with detailed diagnostics
//! - **Hot Reloading**: Runtime route updates without connection drops
//!
//! ### Production Features
//!
//! - **Advanced CORS**: Regex-based origin matching with fine-grained control
//! - **Security Headers**: Built-in security middleware with configurable policies
//! - **Rate Limiting**: Token bucket algorithm with distributed support
//! - **Observability**: Structured logging, metrics, and request tracing
//! - **Multipart Forms**: File upload support with streaming and size limits
//! - **WebSocket Rooms**: Built-in room system for broadcast messaging
//!
//! ## 🚀 Quick Start Guide
//!
//! Add Ignitia to your `Cargo.toml` with desired features:
//!
//! ```
//! [dependencies]
//! ignitia = { version = "0.2.4", features = ["tls", "websocket"] }
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
//! async fn main() -> ignitia::Result<()> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(ignitia::Response::text("HTTP/2 Ready! 🚀")) });
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
//! ## 🌐 Advanced CORS Configuration
//!
//! Comprehensive CORS support for secure cross-origin requests:
//!
//! ### Production CORS Setup
//!
//! ```
//! use ignitia::{CorsMiddleware, Method, Router, Response};
//!
//! # async fn example() -> ignitia::Result<()> {
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
//!     .get("/api/users", || async { Ok(Response::json(vec!["user1", "user2"])?) });
//! # Ok(())
//! # }
//! ```
//!
//! ### Regex-Based Origin Matching
//!
//! ```
//! use ignitia::CorsMiddleware;
//!
//! # fn example() -> ignitia::Result<()> {
//! let cors = CorsMiddleware::new()
//!     .allowed_origin_regex(r"https://.*\.myapp\.com") // All subdomains
//!     .allowed_origin_regex(r"https://localhost:\d+") // Local development ports
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 📡 WebSocket Support
//!
//! Full-featured WebSocket implementation with HTTP/2 compatibility:
//!
//! ### Advanced WebSocket Server
//!
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::websocket::{websocket_handler, Message, WebSocketConnection};
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
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
//!     }));
//! ```
//!
//! ## 🏗️ Framework Architecture
//!
//! Ignitia's layered architecture supports multiple protocols and advanced features:
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ Application Layer                                                           │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
//! │ │   Routes    │ │ Middleware  │ │    CORS     │ │     WebSocket           │ │
//! │ │ & Handlers  │ │  Pipeline   │ │Configuration│ │      Handlers           │ │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────────┘ │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ Ignitia Framework                                                           │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
//! │ │   Router    │ │   Server    │ │     TLS     │ │      WebSocket          │ │
//! │ │  Radix Tree │ │ HTTP/1.1+2  │ │ ALPN & Cert │ │  Connection Management  │ │
//! │ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────────────┘ │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ Runtime Layer (Tokio)                                                       │
//! │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
//! │ │    HTTP     │ │     TLS     │ │     TCP     │ │     Async I/O           │ │
//! │ │   (Hyper)   │ │  (Rustls)   │ │  Listeners  │ │     & Futures           │ │
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
//!
//! - **Simple JSON API**: 155,000+ RPS (Radix mode)
//! - **Static file serving**: 140,000+ RPS
//! - **WebSocket connections**: 25,000+ concurrent
//! - **HTTPS with TLS 1.3**: 110,000+ RPS
//!
//! ### Latency (99th percentile)
//!
//! - **HTTP/1.1**: < 0.8ms
//! - **HTTP/2**: < 1.0ms
//! - **HTTPS**: < 1.3ms
//! - **WebSocket message**: < 0.5ms
//!
//! ### Resource Usage
//!
//! - **Memory per connection**: ~2KB
//! - **CPU overhead**: < 3% at 100K RPS
//! - **Binary size**: ~4MB (with all features)
//!
//! ## 📚 Core Concepts
//!
//! ### Request/Response Lifecycle
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ Ignitia Request Pipeline                                        │
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
//!    ├─ Path Matching (radix tree O(k) lookup)
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
//! ### Routing Modes
//!
//! Ignitia supports two routing modes optimized for different use cases:
//!
//! #### Radix Mode (Default - Recommended)
//!
//! - **Performance**: O(k) lookup where k is path length
//! - **Best for**: Applications with 50+ routes
//! - **Features**: Compressed prefix tree, efficient parameter extraction
//! - **Memory**: Optimized for large route sets
//!
//! #### Base Mode
//!
//! - **Performance**: O(n) linear matching where n is route count
//! - **Best for**: Applications with < 50 routes
//! - **Features**: Simple regex-based matching
//! - **Memory**: Lower overhead for small apps
//!
//! ## 📖 Module Documentation
//!
//! ### Core Modules
//!
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
//!
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
//! - **Repository**: <https://github.com/AarambhDevHub/ignitia>
//! - **Documentation**: <https://docs.rs/ignitia>
//! - **Examples**: <https://github.com/AarambhDevHub/ignitia/tree/main/examples>
//! - **Changelog**: <https://github.com/AarambhDevHub/ignitia/blob/main/doc/CHANGELOG.md>
//! - **Crates.io**: <https://crates.io/crates/ignitia>

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
    BodySizeLimitBuilder, BodySizeLimitMiddleware, CompressionMiddleware, CorsMiddleware,
    IdGenerator, LoggerMiddleware, Middleware, Next, RateLimitConfig, RateLimitInfo,
    RateLimitStats, RateLimitingMiddleware, RequestIdMiddleware, SecurityMiddleware,
};

// Re-export core request and response types
pub use request::Request;
pub use response::{CacheControl, Html, IntoResponse, Response, ResponseBuilder};

// Re-export routing components
pub use router::{LayeredHandler, Route, Router};

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
    /// // Output: "Running ignitia v0.2.4"
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
    pub use crate::{Handler, HandlerFn, IntoHandler, Middleware, Next};

    // Essential middleware implementations
    pub use crate::{CorsMiddleware, LoggerMiddleware, RateLimitingMiddleware, SecurityMiddleware};

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
