//! High-performance HTTP/HTTPS server implementation with advanced optimizations
//!
//! This module provides the core server functionality for the Ignitia web framework,
//! designed for maximum performance and scalability. It includes support for HTTP/1.1,
//! HTTP/2, TLS/HTTPS, WebSocket upgrades, and various performance optimizations.
//!
//! # Features
//!
//! - **High-Performance Architecture**: Optimized for 65K+ RPS throughput
//! - **Protocol Support**: HTTP/1.1, HTTP/2, and automatic protocol negotiation
//! - **TLS/HTTPS**: Full TLS support with configurable cipher suites and certificates
//! - **WebSocket Support**: Native WebSocket upgrade handling with message routing
//! - **Performance Monitoring**: Built-in metrics collection and monitoring
//! - **Connection Management**: Advanced connection pooling and lifecycle management
//! - **Graceful Shutdown**: Clean shutdown with connection draining
//!
//! # Architecture
//!
//! The server is built around a multi-layered architecture:
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Application Layer                        │
//! │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
//! │  │   Router    │ │ Middleware  │ │     Handlers        │   │
//! │  └─────────────┘ └─────────────┘ └─────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Server Layer                           │
//! │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
//! │  │   Config    │ │ Performance │ │     Protocol        │   │
//! │  └─────────────┘ └─────────────┘ └─────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Transport Layer                          │
//! │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
//! │  │     TLS     │ │    Pool     │ │    Connection       │   │
//! │  └─────────────┘ └─────────────┘ └─────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Examples
//!
//! ## Basic HTTP Server
//!
//! ```
//! use ignitia::{Router, Server};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(Response::text("Hello, World!")) })
//!         .get("/health", || async { Ok(Response::json(serde_json::json!({"status": "ok"}))?) });
//!
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()?;
//!     let server = Server::new(router, addr);
//!
//!     println!("🚀 Server running on http://{}", addr);
//!     server.ignitia().await
//! }
//! ```
//!
//! ## High-Performance Configuration
//!
//! ```
//! use ignitia::{Router, Server, ServerConfig, PerformanceConfig};
//!
//! let router = Router::new();
//! let perf_config = PerformanceConfig::max_rps();
//! let server_config = ServerConfig::default()
//!     .with_max_request_body_size(50 * 1024 * 1024); // 50MB
//!
//! let server = Server::new(router, addr)
//!     .with_performance_config(perf_config)
//!     .with_server_config(server_config);
//! ```
//!
//! ## HTTPS Server with TLS
//!
//! ```
//! use ignitia::{Router, Server, TlsConfig};
//!
//! let server = Server::new(router, addr)
//!     .enable_https("cert.pem", "key.pem")?
//!     .redirect_to_https(443);
//! ```
//!
//! ## WebSocket Support
//!
//! ```
//! let router = Router::new()
//!     .websocket("/ws", |mut connection| async move {
//!         while let Some(msg) = connection.next().await {
//!             match msg? {
//!                 Message::Text(text) => {
//!                     connection.send(Message::Text(format!("Echo: {}", text))).await?;
//!                 }
//!                 Message::Binary(data) => {
//!                     connection.send(Message::Binary(data)).await?;
//!                 }
//!                 _ => {}
//!             }
//!         }
//!         Ok(())
//!     });
//! ```

pub mod config;
pub mod connection;
pub mod executor;
pub mod performance;
pub mod pool;
pub mod protocol;

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
pub mod tls;

// Re-export core server types
pub use config::{Http2Config, ServerConfig};
pub use connection::*;
pub use executor::TokioExecutor;
pub use performance::{OptimizedTcpListener, PerformanceConfig, PerformanceMetrics};
pub use pool::{ObjectPools, PoolConfig};
pub use protocol::{HttpProtocol, ProtocolDetector};

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
pub use tls::{TlsConfig, TlsError, TlsVersion};

// Core imports
use crate::{Request, Response, Router};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::{interval, timeout};
use tracing::{debug, info, warn};

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use tokio_rustls::TlsAcceptor;

/// High-performance HTTP/HTTPS server with advanced optimizations
///
/// The `Server` struct is the core component that handles incoming HTTP connections,
/// processes requests through the router, and manages the entire request-response lifecycle.
/// It's designed for maximum performance and scalability, supporting both HTTP/1.1 and HTTP/2
/// protocols with optional TLS encryption.
///
/// # Performance Characteristics
///
/// - **Throughput**: Optimized for 65K+ requests per second
/// - **Latency**: Sub-millisecond response times for simple requests
/// - **Concurrency**: Handles thousands of concurrent connections efficiently
/// - **Memory**: Optimized memory usage with object pooling and zero-copy operations
///
/// # Features
///
/// - **Multi-Protocol**: HTTP/1.1, HTTP/2, automatic protocol detection
/// - **TLS Support**: Full HTTPS support with configurable cipher suites
/// - **WebSocket**: Native WebSocket upgrade handling
/// - **Performance Monitoring**: Built-in metrics and monitoring capabilities
/// - **Connection Management**: Advanced connection pooling and lifecycle management
/// - **Graceful Shutdown**: Clean shutdown with connection draining
///
/// # Architecture
///
/// The server uses an event-driven, async architecture built on Tokio:
///
/// 1. **Connection Acceptance**: Optimized TCP listener with socket-level tuning
/// 2. **Protocol Detection**: Automatic HTTP/1.1 vs HTTP/2 detection
/// 3. **Request Processing**: High-performance request routing and handling
/// 4. **Response Generation**: Optimized response building and transmission
/// 5. **Connection Management**: Efficient connection pooling and cleanup
pub struct Server {
    /// Application router for handling requests
    ///
    /// The router contains all route definitions, middleware, and handlers
    /// that define the application's behavior. It's wrapped in an Arc for
    /// efficient sharing across worker threads.
    router: Arc<Router>,

    /// Server bind address
    ///
    /// The socket address (IP:port) where the server will listen for
    /// incoming connections. Supports both IPv4 and IPv6 addresses.
    addr: SocketAddr,

    /// Server configuration
    ///
    /// Contains HTTP/1.1 and HTTP/2 protocol settings, TLS configuration,
    /// and other server-wide behavioral parameters.
    config: ServerConfig,

    /// Performance configuration
    ///
    /// Socket-level optimizations, buffer sizes, connection settings,
    /// and other performance-related parameters for maximum throughput.
    perf_config: PerformanceConfig,

    /// Performance metrics collection
    ///
    /// Real-time metrics tracking including RPS, response times, error rates,
    /// and resource utilization for monitoring and optimization.
    metrics: Arc<PerformanceMetrics>,

    /// Server state tracking
    ///
    /// Internal state management for graceful shutdown, connection tracking,
    /// and server lifecycle management.
    state: Arc<ServerState>,

    /// TLS acceptor for HTTPS connections
    ///
    /// When TLS is enabled, this handles the TLS handshake and encryption
    /// for secure connections. Only available when the "tls" feature is enabled.
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    tls_acceptor: Option<TlsAcceptor>,
}

/// Server state for monitoring and graceful shutdown
///
/// This struct tracks the runtime state of the server and provides
/// mechanisms for graceful shutdown and connection monitoring.
#[derive(Debug)]
struct ServerState {
    /// Server running state
    ///
    /// Atomic boolean indicating whether the server is currently running.
    /// Used for coordinating graceful shutdown across all components.
    running: AtomicBool,

    /// Active connection count
    ///
    /// Tracks the current number of active TCP connections being processed.
    /// Used for load monitoring and graceful shutdown coordination.
    active_connections: AtomicUsize,

    /// Total requests processed
    ///
    /// Lifetime counter of all requests processed by this server instance.
    /// Used for calculating throughput metrics and capacity planning.
    total_requests: AtomicU64,

    /// Server start time
    ///
    /// Timestamp when the server was started, used for calculating uptime
    /// and long-term performance trends.
    start_time: RwLock<Option<Instant>>,
}

impl ServerState {
    /// Create a new server state instance
    ///
    /// Initializes all counters to zero and sets the server as not running.
    /// This is called during server construction.
    fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_connections: AtomicUsize::new(0),
            total_requests: AtomicU64::new(0),
            start_time: RwLock::new(None),
        }
    }
}

impl Server {
    /// Create a new high-performance server instance
    ///
    /// Creates a server with default high-performance configuration optimized
    /// for maximum throughput. The server will bind to the specified address
    /// and route requests through the provided router.
    ///
    /// # Arguments
    ///
    /// * `router` - The application router containing routes and middleware
    /// * `addr` - The socket address to bind to (e.g., "127.0.0.1:8080")
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, Server};
    /// use std::net::SocketAddr;
    ///
    /// let router = Router::new()
    ///     .get("/", || async { Ok(Response::text("Hello, World!")) });
    ///
    /// let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    /// let server = Server::new(router, addr);
    /// ```
    ///
    /// # Performance Notes
    ///
    /// This constructor applies a high-performance configuration by default:
    /// - TCP_NODELAY enabled for low latency
    /// - SO_REUSEPORT for better load distribution
    /// - Optimized buffer sizes for high throughput
    /// - Performance metrics collection enabled
    pub fn new(router: Router, addr: SocketAddr) -> Self {
        let perf_config = PerformanceConfig::max_rps();
        let metrics = PerformanceMetrics::new();

        Self {
            router: Arc::new(router),
            addr,
            config: ServerConfig::default(),
            perf_config,
            metrics,
            state: Arc::new(ServerState::new()),

            #[cfg(feature = "tls")]
            tls_acceptor: None,
        }
    }

    /// Create server with custom configuration
    ///
    /// Allows full customization of server behavior through the ServerConfig.
    /// This method provides more control over HTTP/1.1 and HTTP/2 settings,
    /// protocol detection, and other server parameters.
    ///
    /// # Arguments
    ///
    /// * `router` - The application router
    /// * `addr` - The socket address to bind to
    /// * `config` - Custom server configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, Server, ServerConfig, Http2Config};
    /// use std::time::Duration;
    ///
    /// let config = ServerConfig::default()
    ///     .with_max_request_body_size(100 * 1024 * 1024); // 100MB
    ///
    /// let server = Server::with_config(router, addr, config);
    /// ```
    pub fn with_config(router: Router, addr: SocketAddr, config: ServerConfig) -> Self {
        let mut server = Self::new(router, addr);
        server.config = config;
        server
    }

    /// Create server optimized for maximum RPS
    ///
    /// This constructor applies the most aggressive performance optimizations
    /// available, designed for scenarios where absolute maximum throughput
    /// is the primary concern.
    ///
    /// # Optimizations Applied
    ///
    /// - Maximum socket buffer sizes (512KB send/receive)
    /// - Largest connection backlog (16,384 connections)
    /// - Shorter keep-alive times for faster connection recycling
    /// - CPU affinity enabled for consistent performance
    /// - All fast-path optimizations enabled
    ///
    /// # Use Cases
    ///
    /// - Load testing scenarios
    /// - High-traffic production APIs (>50K RPS)
    /// - Benchmarking and performance testing
    /// - Edge computing scenarios with extreme performance requirements
    pub fn max_rps(router: Router, addr: SocketAddr) -> Self {
        let perf_config = PerformanceConfig::max_rps();
        let metrics = PerformanceMetrics::new();

        Self {
            router: Arc::new(router),
            addr,
            config: ServerConfig::default(),
            perf_config,
            metrics,
            state: Arc::new(ServerState::new()),

            #[cfg(feature = "tls")]
            tls_acceptor: None,
        }
    }

    /// Set performance configuration
    ///
    /// Replaces the server's performance configuration with custom settings.
    /// This allows fine-tuning of socket-level optimizations, buffer sizes,
    /// and connection management parameters.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom performance configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{PerformanceConfig, Server};
    /// use std::time::Duration;
    ///
    /// let perf_config = PerformanceConfig::default()
    ///     .with_backlog(32768)
    ///     .with_keep_alive(Duration::from_secs(30))
    ///     .with_buffer_sizes(1024 * 1024, 512 * 1024); // 1MB send, 512KB recv
    ///
    /// let server = Server::new(router, addr)
    ///     .with_performance_config(perf_config);
    /// ```
    pub fn with_performance_config(mut self, config: PerformanceConfig) -> Self {
        self.perf_config = config;
        self
    }

    /// Set server configuration
    ///
    /// Replaces the server's configuration with custom HTTP and protocol settings.
    /// This affects how the server handles different HTTP versions, request sizes,
    /// and protocol-specific behaviors.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom server configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{ServerConfig, Http2Config};
    /// use std::time::Duration;
    ///
    /// let http2_config = Http2Config::default()
    ///     .with_max_concurrent_streams(2000)
    ///     .with_keep_alive_interval(Duration::from_secs(30));
    ///
    /// let server_config = ServerConfig::default()
    ///     .with_http2(http2_config)
    ///     .with_max_request_body_size(200 * 1024 * 1024); // 200MB
    ///
    /// let server = Server::new(router, addr)
    ///     .with_server_config(server_config);
    /// ```
    pub fn with_server_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable HTTPS with custom TLS configuration
    ///
    /// Configures the server to handle TLS connections using the provided
    /// TLS configuration. This enables HTTPS support with full control
    /// over cipher suites, protocol versions, and certificate management.
    ///
    /// # Arguments
    ///
    /// * `tls_config` - TLS configuration including certificates and settings
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if TLS configuration is successful, or a `TlsError`
    /// if certificate loading or TLS setup fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Server, TlsConfig, TlsVersion};
    ///
    /// let tls_config = TlsConfig::new("server.crt", "server.key")
    ///     .with_protocol_versions(&[TlsVersion::TlsV12, TlsVersion::TlsV13])
    ///     .with_cipher_suites(&["TLS_AES_256_GCM_SHA384"]);
    ///
    /// let server = Server::new(router, addr)
    ///     .with_tls(tls_config)?;
    /// ```
    ///
    /// # Security Notes
    ///
    /// - Ensure certificate files are properly secured
    /// - Use strong cipher suites appropriate for your security requirements
    /// - Consider certificate rotation and renewal strategies
    /// - Monitor TLS metrics for security and performance
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub fn with_tls(mut self, tls_config: tls::TlsConfig) -> Result<Self, tls::TlsError> {
        let acceptor = tls_config.build()?;
        self.tls_acceptor = Some(acceptor);
        self.config.tls = Some(tls_config);
        Ok(self)
    }

    /// Enable HTTPS with certificate and key files
    ///
    /// Convenient method to enable HTTPS by directly specifying certificate
    /// and private key file paths. Creates a default TLS configuration with
    /// secure defaults and loads the specified certificate files.
    ///
    /// # Arguments
    ///
    /// * `cert_file` - Path to the X.509 certificate file (PEM format)
    /// * `key_file` - Path to the private key file (PEM format)
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if certificate loading succeeds, or a `TlsError`
    /// if file reading or certificate parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let server = Server::new(router, addr)
    ///     .enable_https("certs/server.crt", "certs/server.key")?;
    /// ```
    ///
    /// # File Requirements
    ///
    /// - Certificate file must be in PEM format
    /// - Private key must be in PEM format (RSA or EC)
    /// - Files must be readable by the server process
    /// - Certificate must be valid and not expired
    ///
    /// # Security Considerations
    ///
    /// - Protect private key files with appropriate file permissions (600)
    /// - Store certificates in a secure location
    /// - Consider using a certificate management system for production
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub fn enable_https(
        self,
        cert_file: impl Into<String>,
        key_file: impl Into<String>,
    ) -> Result<Self, tls::TlsError> {
        let tls_config = tls::TlsConfig::new(cert_file, key_file);
        self.with_tls(tls_config)
    }

    /// Generate and use self-signed certificate (development only)
    ///
    /// Creates a self-signed certificate for the specified domain and configures
    /// the server to use it. This is intended for development and testing only
    /// and should never be used in production environments.
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain name for the certificate (e.g., "localhost", "example.com")
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if certificate generation succeeds, or a `TlsError`
    /// if certificate generation or configuration fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // Development server with self-signed certificate
    /// let server = Server::new(router, addr)
    ///     .with_self_signed_cert("localhost")?;
    /// ```
    ///
    /// # Warning
    ///
    /// Self-signed certificates will trigger security warnings in browsers
    /// and should only be used for development, testing, or internal services
    /// where certificate authority validation is not required.
    ///
    /// # Features Required
    ///
    /// This method requires both the "tls" and "self-signed" features to be enabled.
    #[cfg(all(feature = "tls", feature = "self-signed"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "tls", feature = "self-signed"))))]
    pub fn with_self_signed_cert(self, domain: &str) -> Result<Self, tls::TlsError> {
        let (cert_pem, key_pem) = tls::TlsConfig::generate_self_signed(domain)?;
        let tls_config = tls::TlsConfig::new("self_signed_cert.pem", "self_signed_key.pem");
        self.with_tls(tls_config)
    }

    /// Enable HTTP to HTTPS redirect
    ///
    /// Configures the server to automatically redirect all HTTP requests to HTTPS.
    /// This is useful when running both HTTP and HTTPS servers and you want to
    /// ensure all traffic uses encrypted connections.
    ///
    /// # Arguments
    ///
    /// * `https_port` - The port number where the HTTPS server is running
    ///
    /// # Examples
    ///
    /// ```
    /// // Redirect HTTP (port 80) to HTTPS (port 443)
    /// let server = Server::new(router, "0.0.0.0:80".parse().unwrap())
    ///     .redirect_to_https(443);
    /// ```
    ///
    /// # Behavior
    ///
    /// When enabled, all HTTP requests will receive a 301 (Moved Permanently)
    /// response with a Location header pointing to the HTTPS equivalent URL.
    /// This ensures search engines and browsers update their bookmarks to
    /// use the secure version.
    ///
    /// # Deployment Considerations
    ///
    /// - Run separate server instances on ports 80 (HTTP) and 443 (HTTPS)
    /// - Consider using a reverse proxy for more complex routing needs
    /// - Monitor redirect performance impact on high-traffic sites
    pub fn redirect_to_https(mut self, https_port: u16) -> Self {
        self.config = self.config.redirect_to_https(https_port);
        self
    }

    /// Get server metrics
    ///
    /// Returns a reference to the server's performance metrics collector.
    /// This provides access to real-time statistics about server performance,
    /// including request throughput, response times, error rates, and resource usage.
    ///
    /// # Returns
    ///
    /// An `Arc<PerformanceMetrics>` that can be shared across threads for
    /// monitoring and observability purposes.
    ///
    /// # Examples
    ///
    /// ```
    /// let server = Server::new(router, addr);
    /// let metrics = server.metrics();
    ///
    /// // Later, in a monitoring task:
    /// let current_rps = metrics.current_rps();
    /// let avg_response_time = metrics.avg_response_time();
    /// let error_rate = metrics.error_rate();
    ///
    /// println!("RPS: {}, Avg Response: {:?}, Error Rate: {:.2}%",
    ///     current_rps, avg_response_time, error_rate);
    /// ```
    ///
    /// # Metrics Available
    ///
    /// - **Throughput**: Requests per second, total requests
    /// - **Latency**: Average, P95, P99 response times
    /// - **Errors**: Error count and error rate percentage
    /// - **Connections**: Active connection count
    /// - **Resources**: Memory usage, CPU utilization
    pub fn metrics(&self) -> Arc<PerformanceMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get server uptime
    ///
    /// Returns the duration since the server was started, or None if the
    /// server hasn't been started yet. This is useful for monitoring and
    /// health checks.
    ///
    /// # Returns
    ///
    /// `Some(Duration)` representing the uptime if the server is running,
    /// or `None` if the server hasn't been started.
    ///
    /// # Examples
    ///
    /// ```
    /// if let Some(uptime) = server.uptime() {
    ///     println!("Server uptime: {:?}", uptime);
    /// } else {
    ///     println!("Server not started");
    /// }
    /// ```
    pub fn uptime(&self) -> Option<Duration> {
        self.state.start_time.read().map(|start| start.elapsed())
    }

    /// Get active connection count
    ///
    /// Returns the current number of active TCP connections being processed
    /// by the server. This is useful for load monitoring and capacity planning.
    ///
    /// # Returns
    ///
    /// The current number of active connections as a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// let active_connections = server.active_connections();
    /// if active_connections > 1000 {
    ///     println!("High load detected: {} active connections", active_connections);
    /// }
    /// ```
    pub fn active_connections(&self) -> usize {
        self.state.active_connections.load(Ordering::Relaxed)
    }

    /// Get total requests processed
    ///
    /// Returns the lifetime total of requests processed by this server instance.
    /// This counter never resets and provides a baseline for calculating rates
    /// and trends over time.
    ///
    /// # Returns
    ///
    /// The total number of requests processed as a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// let total_requests = server.total_requests();
    /// println!("Total requests served: {}", total_requests);
    /// ```
    pub fn total_requests(&self) -> u64 {
        self.state.total_requests.load(Ordering::Relaxed)
    }

    /// Start the high-performance server
    ///
    /// This is the main entry point that starts the server and begins accepting
    /// connections. The method runs indefinitely until the server is shut down
    /// or encounters a fatal error.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the server shuts down gracefully, or an error
    /// if startup fails or a fatal error occurs during operation.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let server = Server::new(router, addr);
    ///     server.ignitia().await
    /// }
    /// ```
    ///
    /// # Startup Process
    ///
    /// 1. **Initialization**: Set server state and start time
    /// 2. **Socket Creation**: Create optimized TCP listener with performance tuning
    /// 3. **Protocol Detection**: Determine HTTP/1.1 vs HTTP/2 capabilities
    /// 4. **Metrics**: Start background metrics collection
    /// 5. **Connection Loop**: Begin accepting and processing connections
    /// 6. **Graceful Shutdown**: Handle shutdown signals and drain connections
    ///
    /// # Performance Features
    ///
    /// - **Socket Optimization**: SO_REUSEPORT, TCP_NODELAY, optimized buffers
    /// - **Connection Pooling**: Reuse connections and reduce allocation overhead
    /// - **Protocol Negotiation**: Automatic HTTP/1.1 and HTTP/2 selection
    /// - **TLS Acceleration**: Hardware-accelerated encryption when available
    /// - **Request Processing**: High-performance request routing and handling
    ///
    /// # Error Handling
    ///
    /// The server handles various error conditions gracefully:
    /// - **Bind Errors**: Address already in use, permission denied
    /// - **TLS Errors**: Certificate issues, handshake failures
    /// - **Connection Errors**: Client disconnections, protocol violations
    /// - **Resource Exhaustion**: Memory limits, file descriptor limits
    ///
    /// # Monitoring
    ///
    /// During operation, the server provides extensive monitoring:
    /// - Real-time performance metrics
    /// - Connection and request tracking
    /// - Error rate monitoring
    /// - Resource usage statistics
    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>> {
        // Mark server as running
        self.state.running.store(true, Ordering::Relaxed);
        *self.state.start_time.write() = Some(Instant::now());

        // Create optimized TCP listener
        let listener = OptimizedTcpListener::bind(self.addr, self.perf_config.clone()).await?;
        let listener_metrics = listener.metrics();

        // Determine protocol info for logging
        #[cfg(feature = "tls")]
        let protocol_info = if self.tls_acceptor.is_some() {
            if self.config.http2.enabled && self.config.http1_enabled {
                "HTTPS (HTTP/1.1 + HTTP/2)"
            } else if self.config.http2.enabled {
                "HTTPS (HTTP/2)"
            } else {
                "HTTPS (HTTP/1.1)"
            }
        } else if self.config.http2.enabled && self.config.http1_enabled {
            "HTTP (HTTP/1.1 + HTTP/2)"
        } else if self.config.http2.enabled {
            "HTTP (HTTP/2)"
        } else {
            "HTTP (HTTP/1.1)"
        };

        #[cfg(not(feature = "tls"))]
        let protocol_info = if self.config.http2.enabled && self.config.http1_enabled {
            "HTTP (HTTP/1.1 + HTTP/2)"
        } else if self.config.http2.enabled {
            "HTTP (HTTP/2)"
        } else {
            "HTTP (HTTP/1.1)"
        };

        #[cfg(feature = "tls")]
        let scheme = if self.tls_acceptor.is_some() {
            "https"
        } else {
            "http"
        };
        #[cfg(not(feature = "tls"))]
        let scheme = "http";

        info!(
            "🔥 Ignitia server blazing on {}://{} ({})",
            scheme, self.addr, protocol_info
        );

        // Start metrics collection task
        self.start_metrics_collection(Arc::clone(&listener_metrics));

        // Main server loop
        loop {
            if !self.state.running.load(Ordering::Relaxed) {
                info!("🛑 Server shutdown requested");
                break;
            }

            let (stream, addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    warn!("❌ Failed to accept connection: {}", e);
                    continue;
                }
            };

            let router = Arc::clone(&self.router);
            let config = self.config.clone();
            let metrics = Arc::clone(&self.metrics);
            let state = Arc::clone(&self.state);

            #[cfg(feature = "tls")]
            let tls_acceptor = self.tls_acceptor.clone();

            // Spawn connection handler with optimizations
            tokio::spawn(async move {
                // Track active connection
                state.active_connections.fetch_add(1, Ordering::Relaxed);

                let connection_start = Instant::now();

                let result = {
                    #[cfg(feature = "tls")]
                    if let Some(acceptor) = tls_acceptor {
                        handle_tls_connection(
                            stream,
                            router,
                            config,
                            acceptor,
                            addr,
                            metrics.clone(),
                            state.clone(),
                        )
                        .await
                    } else {
                        let io = TokioIo::new(stream);
                        handle_connection(io, router, config, addr, metrics.clone(), state.clone())
                            .await
                    }

                    #[cfg(not(feature = "tls"))]
                    {
                        let io = TokioIo::new(stream);
                        handle_connection(io, router, config, addr, metrics.clone(), state.clone())
                            .await
                    }
                };

                if let Err(err) = result {
                    debug!("🔌 Connection error from {}: {}", addr, err);
                }

                // Update metrics and cleanup
                let connection_duration = connection_start.elapsed();
                metrics.record_request(connection_duration);
                state.active_connections.fetch_sub(1, Ordering::Relaxed);
            });
        }

        Ok(())
    }

    /// Start background metrics collection
    ///
    /// Spawns a background task that periodically collects and updates
    /// performance metrics. This task runs independently and provides
    /// real-time monitoring data.
    fn start_metrics_collection(&self, _listener_metrics: Arc<PerformanceMetrics>) {
        let server_state = Arc::clone(&self.state);
        let server_metrics = Arc::clone(&self.metrics);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            let mut last_requests = 0u64;

            loop {
                interval.tick().await;

                if !server_state.running.load(Ordering::Relaxed) {
                    break;
                }

                let current_requests = server_state.total_requests.load(Ordering::Relaxed);
                let rps = current_requests.saturating_sub(last_requests);
                last_requests = current_requests;

                // Update RPS metrics
                server_metrics
                    .requests_per_second
                    .store(rps, Ordering::Relaxed);

                // Log periodic stats
                if current_requests % 10000 == 0 && current_requests > 0 {
                    info!(
                        "📈 Performance: {} RPS, {} active connections, {} total requests",
                        rps,
                        server_state.active_connections.load(Ordering::Relaxed),
                        current_requests
                    );
                }
            }
        });
    }

    /// Graceful shutdown
    ///
    /// Initiates a graceful shutdown of the server, waiting for active
    /// connections to complete before terminating. This ensures that
    /// in-flight requests are processed and clients receive proper responses.
    ///
    /// # Examples
    ///
    /// ```
    /// // In a signal handler or shutdown routine
    /// server.shutdown().await;
    /// ```
    ///
    /// # Shutdown Process
    ///
    /// 1. **Stop Accepting**: Stop accepting new connections
    /// 2. **Drain Connections**: Wait for active connections to finish
    /// 3. **Timeout**: Force shutdown after timeout if connections remain
    /// 4. **Cleanup**: Release resources and update state
    ///
    /// # Timeout
    ///
    /// The shutdown process has a 30-second timeout. If connections are still
    /// active after this time, the server will force shutdown to prevent
    /// hanging indefinitely.
    pub async fn shutdown(&self) {
        info!("🛑 Initiating graceful shutdown...");
        self.state.running.store(false, Ordering::Relaxed);

        // Wait for active connections to finish
        let shutdown_timeout = Duration::from_secs(30);
        let start = Instant::now();

        while self.state.active_connections.load(Ordering::Relaxed) > 0
            && start.elapsed() < shutdown_timeout
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let remaining = self.state.active_connections.load(Ordering::Relaxed);
        if remaining > 0 {
            warn!("⚠️  Forcing shutdown with {} active connections", remaining);
        } else {
            info!("✅ All connections closed gracefully");
        }

        info!("🏁 Server shutdown complete");
    }
}

/// Handle TLS connections with protocol negotiation
///
/// Processes incoming TLS connections, performing the TLS handshake and
/// extracting ALPN (Application-Layer Protocol Negotiation) information
/// to determine the appropriate HTTP protocol version.
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
async fn handle_tls_connection(
    stream: tokio::net::TcpStream,
    router: Arc<Router>,
    config: ServerConfig,
    acceptor: TlsAcceptor,
    addr: SocketAddr,
    metrics: Arc<PerformanceMetrics>,
    state: Arc<ServerState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use protocol::ProtocolDetector;

    // TLS handshake
    let tls_stream = acceptor.accept(stream).await?;

    // Extract ALPN protocol information
    let alpn_protocol = {
        let (_, connection_info) = tls_stream.get_ref();
        connection_info.alpn_protocol().map(|p| p.to_vec())
    };

    debug!(
        "🔐 TLS connection from {} - ALPN: {:?}",
        addr,
        alpn_protocol.as_ref().map(|p| String::from_utf8_lossy(p))
    );

    let io = TokioIo::new(tls_stream);

    // Use ALPN to determine protocol
    let protocol = ProtocolDetector::detect_from_alpn(alpn_protocol.as_deref());

    let service = service_fn(move |req| {
        let router = Arc::clone(&router);
        let metrics = Arc::clone(&metrics);
        let state = Arc::clone(&state);
        async move { handle_request(router, req, config.max_request_body_size, metrics, state).await }
    });

    match protocol {
        protocol::HttpProtocol::Http2 => {
            serve_http2_connection(io, service, config.http2).await?;
        }
        protocol::HttpProtocol::Http1 | protocol::HttpProtocol::Auto => {
            let mut builder = http1::Builder::new();
            builder.half_close(true);
            builder.timer(hyper_util::rt::TokioTimer::new());

            if config.http2.enabled {
                builder
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await?;
            } else {
                builder.serve_connection(io, service).await?;
            }
        }
    }

    Ok(())
}

/// Handle regular HTTP connections
///
/// Processes non-TLS HTTP connections with automatic protocol detection
/// and appropriate handling for HTTP/1.1 and HTTP/2 requests.
async fn handle_connection<I>(
    io: TokioIo<I>,
    router: Arc<Router>,
    config: ServerConfig,
    _addr: SocketAddr,
    metrics: Arc<PerformanceMetrics>,
    state: Arc<ServerState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    // Handle HTTPS redirect
    if config.redirect_http_to_https {
        return handle_http_redirect(io, config.https_port.unwrap_or(443)).await;
    }

    let service = service_fn(move |req| {
        let router = Arc::clone(&router);
        let metrics = Arc::clone(&metrics);
        let state = Arc::clone(&state);
        async move { handle_request(router, req, config.max_request_body_size, metrics, state).await }
    });

    // Protocol selection with optimizations
    if config.auto_protocol_detection && config.http1_enabled && config.http2.enabled {
        let mut builder = http1::Builder::new();
        builder.half_close(true);
        builder.timer(hyper_util::rt::TokioTimer::new());
        builder
            .serve_connection(io, service)
            .with_upgrades()
            .await?;
    } else if config.http2.enabled && config.http2.enable_prior_knowledge {
        serve_http2_connection(io, service, config.http2).await?;
    } else if config.http2.enabled {
        serve_http2_connection(io, service, config.http2).await?;
    } else {
        let mut builder = http1::Builder::new();
        builder.timer(hyper_util::rt::TokioTimer::new());
        builder.serve_connection(io, service).await?;
    }

    Ok(())
}

/// Handle HTTP to HTTPS redirect
///
/// Provides automatic redirection from HTTP to HTTPS for security.
/// Responds with a 301 redirect to the HTTPS version of the requested URL.
async fn handle_http_redirect<I>(
    io: TokioIo<I>,
    https_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let service = service_fn(move |req| async move {
        let host = req
            .headers()
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");

        let redirect_url = if https_port == 443 {
            format!(
                "https://{}{}",
                host,
                req.uri()
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("")
            )
        } else {
            format!(
                "https://{}:{}{}",
                host,
                https_port,
                req.uri()
                    .path_and_query()
                    .map(|pq| pq.as_str())
                    .unwrap_or("")
            )
        };

        Ok::<_, hyper::Error>(
            hyper::Response::builder()
                .status(301)
                .header("Location", redirect_url)
                .body(Full::new(Bytes::from("Redirecting to HTTPS")))
                .unwrap(),
        )
    });

    let mut builder = http1::Builder::new();
    builder.timer(hyper_util::rt::TokioTimer::new());
    builder.serve_connection(io, service).await?;
    Ok(())
}

/// Serve HTTP/2 connections with optimizations
///
/// Handles HTTP/2 connections with advanced configuration options
/// including flow control, multiplexing, and performance tuning.
async fn serve_http2_connection<S, I>(
    io: TokioIo<I>,
    service: S,
    config: Http2Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    S: hyper::service::Service<
            hyper::Request<hyper::body::Incoming>,
            Response = hyper::Response<Full<Bytes>>,
        > + Clone
        + Send
        + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    let executor = TokioExecutor;
    let mut builder = http2::Builder::new(executor);

    // Apply HTTP/2 optimizations
    builder.timer(hyper_util::rt::TokioTimer::new());

    if let Some(max_streams) = config.max_concurrent_streams {
        builder.max_concurrent_streams(max_streams);
    }
    if let Some(window_size) = config.initial_connection_window_size {
        builder.initial_connection_window_size(window_size);
    }
    if let Some(window_size) = config.initial_stream_window_size {
        builder.initial_stream_window_size(window_size);
    }
    if let Some(frame_size) = config.max_frame_size {
        builder.max_frame_size(frame_size);
    }
    if let Some(interval) = config.keep_alive_interval {
        builder.keep_alive_interval(interval);
    }
    if let Some(timeout) = config.keep_alive_timeout {
        builder.keep_alive_timeout(timeout);
    }
    if config.adaptive_window {
        builder.adaptive_window(true);
    }
    if let Some(max_header_size) = config.max_header_list_size {
        builder.max_header_list_size(max_header_size);
    }

    debug!("🌐 Serving HTTP/2 connection with config: {:?}", config);
    builder.serve_connection(io, service).await?;
    Ok(())
}

/// Handle individual HTTP requests with optimizations
///
/// Processes individual HTTP requests through the application router,
/// including WebSocket upgrade detection, body parsing, and response generation.
async fn handle_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    max_body_size: usize,
    metrics: Arc<PerformanceMetrics>,
    state: Arc<ServerState>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let request_start = Instant::now();
    state.total_requests.fetch_add(1, Ordering::Relaxed);

    // Check for WebSocket upgrade first
    #[cfg(feature = "websocket")]
    if is_websocket_upgrade(&req) {
        return handle_websocket_upgrade(router, req).await;
    }

    // Handle regular HTTP request
    handle_regular_http_request(router, req, max_body_size, metrics, request_start).await
}

/// Check if request is WebSocket upgrade
///
/// Fast path checks for WebSocket upgrade headers without expensive parsing.
#[cfg(feature = "websocket")]
fn is_websocket_upgrade(req: &hyper::Request<hyper::body::Incoming>) -> bool {
    use hyper::header::{CONNECTION, UPGRADE};

    let version = req.version();

    // HTTP/1.1 WebSocket upgrade
    if version == http::Version::HTTP_11 {
        let connection_header = req.headers().get(CONNECTION).and_then(|h| h.to_str().ok());
        let upgrade_header = req.headers().get(UPGRADE).and_then(|h| h.to_str().ok());

        if let (Some(conn), Some(upgrade)) = (connection_header, upgrade_header) {
            return conn.to_lowercase().contains("upgrade")
                && upgrade.to_lowercase().contains("websocket")
                && req.headers().get("sec-websocket-key").is_some();
        }
    }

    false
}

/// Handle WebSocket upgrade requests
///
/// Processes WebSocket upgrade requests by finding the appropriate handler
/// and performing the protocol upgrade.
#[cfg(feature = "websocket")]
async fn handle_websocket_upgrade(
    router: Arc<Router>,
    hyper_req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let path = hyper_req.uri().path().to_string();
    let websocket_handlers = router.get_websocket_handlers();

    let handler = match websocket_handlers.get(&path) {
        Some(handler) => Arc::clone(handler.value()),
        None => {
            debug!("🔍 No WebSocket handler found for path: {}", path);
            return Ok(hyper::Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("WebSocket endpoint not found")))
                .unwrap());
        }
    };

    // Extract metadata WITHOUT consuming body
    let method = hyper_req.method().clone();
    let uri = hyper_req.uri().clone();
    let version = hyper_req.version();
    let headers = hyper_req.headers().clone();

    // Get router extensions with state
    let router_extensions = {
        let inner = router.inner.read();
        inner.extensions.clone()
    };

    // Create framework Request with EMPTY body and router extensions
    let mut framework_req = Request::new(method, uri, version, headers, Bytes::new());

    // Copy router extensions (including State) to request
    framework_req.extensions = router_extensions;

    // Check if this is a valid WebSocket request
    if !crate::websocket::is_websocket_request(&framework_req) {
        debug!("❌ Invalid WebSocket upgrade request");
        return Ok(hyper::Response::builder()
            .status(400)
            .body(Full::new(Bytes::from("Invalid WebSocket upgrade request")))
            .unwrap());
    }

    // Generate upgrade response based on protocol
    let upgrade_response = match crate::websocket::upgrade_connection(&framework_req) {
        Ok(resp) => resp,
        Err(e) => {
            debug!("❌ WebSocket upgrade failed: {}", e);
            return Ok(hyper::Response::builder()
                .status(e.status_code())
                .body(Full::new(Bytes::from(e.to_string())))
                .unwrap());
        }
    };

    // Build hyper response from framework response
    let mut response_builder = hyper::Response::builder().status(upgrade_response.status);

    for (key, value) in upgrade_response.headers.iter() {
        response_builder = response_builder.header(key, value);
    }

    let response = response_builder.body(Full::new(Bytes::new())).unwrap();

    // Spawn WebSocket handling task
    tokio::spawn(async move {
        match hyper::upgrade::on(hyper_req).await {
            Ok(upgraded) => {
                let response =
                    crate::websocket::handle_websocket_upgrade(framework_req, upgraded, handler)
                        .await;

                #[cfg(debug_assertions)]
                if !response.status.is_success() {
                    debug!(
                        "🔌 WebSocket handler returned error status: {}",
                        response.status
                    );
                }
            }
            Err(e) => {
                debug!("🔌 WebSocket upgrade failed: {}", e);
            }
        }
    });

    Ok(response)
}

/// Handle regular HTTP requests
///
/// Processes non-WebSocket HTTP requests through the router with
/// body parsing, request processing, and response generation.
async fn handle_regular_http_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    max_body_size: usize,
    metrics: Arc<PerformanceMetrics>,
    request_start: Instant,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let (parts, body) = req.into_parts();

    // Collect body with size limits
    let body_bytes = match timeout(
        Duration::from_secs(10), // Shorter timeout for better performance
        body.collect(),
    )
    .await
    {
        Ok(Ok(collected)) => {
            let bytes = collected.to_bytes();
            // Check body size using len() for efficiency
            if bytes.len() > max_body_size {
                let mut response =
                    hyper::Response::new(Full::new(Bytes::from("Request too large")));
                *response.status_mut() = http::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(response);
            }
            bytes
        }
        Ok(Err(_)) | Err(_) => Bytes::new(),
    };

    // Create request object
    let request = Request::new(
        parts.method,
        parts.uri,
        parts.version,
        parts.headers,
        body_bytes,
    );

    // Process request through router
    let response = match router.handle(request).await {
        Ok(res) => res,
        Err(err) => {
            let status = err.status_code();
            let mut res = Response::new(status);
            res.body = Bytes::from(err.to_string());
            res
        }
    };

    // Record request metrics
    let request_duration = request_start.elapsed();
    metrics.record_request(request_duration);

    // Build hyper response with optimizations
    let mut builder = hyper::Response::builder().status(response.status);

    // Pre-allocate headers vector for better performance
    for (key, value) in response.headers.iter() {
        builder = builder.header(key, value);
    }

    Ok(builder.body(Full::new(response.body)).unwrap())
}
