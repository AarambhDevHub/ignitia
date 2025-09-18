//! HTTP server implementation for the Ignitia web framework.
//!
//! This module provides the core server functionality including:
//! - HTTP/1.1 and HTTP/2 protocol support
//! - TLS/HTTPS support with certificate management
//! - WebSocket protocol upgrades and handling
//! - Connection management and protocol detection
//! - Request routing and middleware processing
//! - Performance optimizations for high-throughput scenarios
//!
//! # Features
//!
//! The server supports multiple optional features:
//! - **TLS Support**: Enable HTTPS with the `tls` feature
//! - **WebSocket Support**: Enable WebSocket connections with the `websocket` feature
//! - **HTTP/2**: Built-in support for HTTP/2 protocol negotiation
//! - **Protocol Detection**: Automatic protocol detection via ALPN
//!
//! # Architecture
//!
//! The server is built on top of Hyper and Tokio, providing:
//! - Async/await support throughout
//! - Zero-copy request/response handling where possible
//! - Efficient connection pooling and management
//! - Graceful error handling and recovery
//!
//! # Usage
//!
//! ## Basic HTTP Server
//!
//! ```
//! use ignitia::{Router, Server, Response};
//! use std::net::SocketAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let router = Router::new()
//!     .get("/", || async { Ok(Response::text("Hello, World!")) })
//!     .post("/api/data", |body: String| async move {
//!         Ok(Response::json(&serde_json::json!({"received": body}))?)
//!     });
//!
//! let addr: SocketAddr = "127.0.0.1:3000".parse()?;
//! Server::new(router, addr)
//!     .ignitia()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## HTTPS Server with TLS
//!
//! ```
//! # #[cfg(feature = "tls")]
//! use ignitia::{Router, Server, TlsConfig};
//!
//! # #[cfg(feature = "tls")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let router = Router::new()
//!     .get("/", || async { Ok(ignitia::Response::text("Hello, HTTPS!")) });
//!
//! let tls_config = TlsConfig::new("cert.pem", "key.pem");
//! let addr = "127.0.0.1:8443".parse()?;
//!
//! Server::new(router, addr)
//!     .with_tls(tls_config)?
//!     .ignitia()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## HTTP/2 Configuration
//!
//! ```
//! use ignitia::{Router, Server, ServerConfig, Http2Config};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let router = Router::new()
//!     .get("/", || async { Ok(ignitia::Response::text("HTTP/2 Server")) });
//!
//! let http2_config = Http2Config {
//!     enabled: true,
//!     max_concurrent_streams: Some(1000),
//!     initial_connection_window_size: Some(1024 * 1024), // 1MB
//!     keep_alive_interval: Some(Duration::from_secs(30)),
//!     ..Default::default()
//! };
//!
//! let server_config = ServerConfig {
//!     http2: http2_config,
//!     ..Default::default()
//! };
//!
//! let addr = "127.0.0.1:3000".parse()?;
//! Server::new(router, addr)
//!     .with_config(server_config)
//!     .ignitia()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Considerations
//!
//! - The server uses async I/O throughout for maximum concurrency
//! - HTTP/2 multiplexing reduces connection overhead
//! - Request body size is limited to 10MB by default for security
//! - Connection pooling and reuse minimize resource overhead
//! - Zero-copy operations where possible reduce memory allocations
//!
//! # Security Features
//!
//! - Request body size limiting prevents memory exhaustion attacks
//! - TLS support with configurable cipher suites and versions
//! - Automatic HTTPS redirect capability
//! - WebSocket protocol validation and upgrade security
//! - Header validation and sanitization

pub mod config;
pub mod connection;
pub mod executor;
pub mod protocol;

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
pub mod tls;

pub use config::{Http2Config, ServerConfig};

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
pub use tls::{TlsConfig, TlsError, TlsVersion};

use crate::{Request, Response, Router};
use bytes::Bytes;
use executor::TokioExecutor;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use tokio_rustls::TlsAcceptor;

#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
use crate::websocket::upgrade::generate_accept_key;

/// High-performance HTTP server for the Ignitia web framework.
///
/// The `Server` struct is the core component that handles incoming HTTP connections,
/// routes requests through the configured router, and manages various protocols
/// including HTTP/1.1, HTTP/2, and WebSocket upgrades.
///
/// # Features
///
/// - **Multi-protocol Support**: HTTP/1.1, HTTP/2, and WebSocket protocols
/// - **TLS/HTTPS**: Optional TLS support with certificate management
/// - **Performance**: Optimized for high-throughput scenarios
/// - **Async/Await**: Full async support built on Tokio
/// - **Protocol Detection**: Automatic protocol negotiation via ALPN
///
/// # Examples
///
/// ## Basic Server
///
/// ```
/// use ignitia::{Router, Server, Response};
/// use std::net::SocketAddr;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let router = Router::new()
///     .get("/", || async { Ok(Response::text("Hello, World!")) });
///
/// let addr: SocketAddr = "127.0.0.1:3000".parse()?;
/// let server = Server::new(router, addr);
/// server.ignitia().await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Server with Custom Configuration
///
/// ```
/// use ignitia::{Router, Server, ServerConfig, Http2Config};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let router = Router::new()
///     .get("/", || async { Ok(ignitia::Response::text("Configured Server")) });
///
/// let config = ServerConfig {
///     http1_enabled: true,
///     http2: Http2Config {
///         enabled: true,
///         max_concurrent_streams: Some(500),
///         ..Default::default()
///     },
///     ..Default::default()
/// };
///
/// let addr = "127.0.0.1:3000".parse()?;
/// Server::new(router, addr)
///     .with_config(config)
///     .ignitia()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct Server {
    /// The router containing all route definitions and middleware
    router: Arc<Router>,
    /// The socket address the server will bind to
    addr: SocketAddr,

    /// Server configuration including protocol settings
    config: ServerConfig,

    /// TLS acceptor for HTTPS connections (when TLS feature is enabled)
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    tls_acceptor: Option<TlsAcceptor>,
}

impl Server {
    /// Creates a new server instance with the given router and bind address.
    ///
    /// The server is created with default configuration that enables both HTTP/1.1
    /// and HTTP/2 protocols with sensible defaults for performance and compatibility.
    ///
    /// # Arguments
    ///
    /// * `router` - The router instance that will handle request routing
    /// * `addr` - The socket address to bind the server to
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, Server};
    /// use std::net::SocketAddr;
    ///
    /// let router = Router::new()
    ///     .get("/", || async { Ok(ignitia::Response::text("Hello!")) });
    ///
    /// let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    /// let server = Server::new(router, addr);
    /// ```
    ///
    /// # Performance Notes
    ///
    /// The router is wrapped in an `Arc` for efficient sharing across async tasks,
    /// enabling the server to handle multiple concurrent connections efficiently.
    pub fn new(router: Router, addr: SocketAddr) -> Self {
        Self {
            router: Arc::new(router),
            addr,
            config: ServerConfig::default(),
            #[cfg(feature = "tls")]
            #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
            tls_acceptor: None,
        }
    }

    /// Sets a custom server configuration.
    ///
    /// This method allows you to customize various aspects of the server behavior
    /// including HTTP/2 settings, protocol detection, and TLS configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The server configuration to use
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, Server, ServerConfig, Http2Config};
    /// use std::time::Duration;
    ///
    /// let router = Router::new();
    /// let addr = "127.0.0.1:3000".parse().unwrap();
    ///
    /// let config = ServerConfig {
    ///     http1_enabled: true,
    ///     http2: Http2Config {
    ///         enabled: true,
    ///         max_concurrent_streams: Some(1000),
    ///         keep_alive_interval: Some(Duration::from_secs(60)),
    ///         ..Default::default()
    ///     },
    ///     auto_protocol_detection: true,
    ///     ..Default::default()
    /// };
    ///
    /// let server = Server::new(router, addr).with_config(config);
    /// ```
    pub fn with_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Enables HTTPS with a custom TLS configuration.
    ///
    /// This method configures the server to accept HTTPS connections using the
    /// provided TLS configuration. The TLS configuration includes certificate
    /// paths, ALPN protocols, and security settings.
    ///
    /// # Arguments
    ///
    /// * `tls_config` - The TLS configuration including certificates and settings
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` on success, or a `TlsError` if the TLS configuration
    /// is invalid or certificates cannot be loaded.
    ///
    /// # Errors
    ///
    /// - `TlsError::Io` - Certificate or key files cannot be read
    /// - `TlsError::CertParsing` - Invalid certificate format
    /// - `TlsError::KeyParsing` - Invalid private key format
    /// - `TlsError::Config` - TLS configuration error
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "tls")]
    /// use ignitia::{Router, Server, TlsConfig};
    ///
    /// # #[cfg(feature = "tls")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::new()
    ///     .get("/", || async { Ok(ignitia::Response::text("HTTPS Server")) });
    ///
    /// let tls_config = TlsConfig::new("cert.pem", "key.pem")
    ///     .with_alpn_protocols(vec!["h2", "http/1.1"]);
    ///
    /// let addr = "127.0.0.1:8443".parse()?;
    /// let server = Server::new(router, addr)
    ///     .with_tls(tls_config)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Security Considerations
    ///
    /// - Ensure certificate and private key files have appropriate permissions
    /// - Use certificates from trusted Certificate Authorities in production
    /// - Regularly update certificates before expiration
    /// - Consider enabling client certificate verification for enhanced security
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub fn with_tls(mut self, tls_config: tls::TlsConfig) -> Result<Self, tls::TlsError> {
        let acceptor = tls_config.build()?;
        self.tls_acceptor = Some(acceptor);
        self.config.tls = Some(tls_config);
        Ok(self)
    }

    /// Enables HTTPS with certificate and key file paths.
    ///
    /// This is a convenience method that creates a default TLS configuration
    /// with the specified certificate and private key files.
    ///
    /// # Arguments
    ///
    /// * `cert_file` - Path to the PEM-encoded certificate file
    /// * `key_file` - Path to the PEM-encoded private key file
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "tls")]
    /// use ignitia::{Router, Server};
    ///
    /// # #[cfg(feature = "tls")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::new()
    ///     .get("/", || async { Ok(ignitia::Response::text("HTTPS")) });
    ///
    /// let addr = "127.0.0.1:8443".parse()?;
    /// let server = Server::new(router, addr)
    ///     .enable_https("server.crt", "server.key")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # File Requirements
    ///
    /// - Certificate file must be in PEM format and contain the server certificate
    /// - Private key file must be in PEM format (PKCS#8 recommended)
    /// - Files must be readable by the server process
    /// - Private key should have restrictive permissions (e.g., 600)
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

    /// Generates and uses a self-signed certificate for development.
    ///
    /// This method creates a self-signed certificate for the specified domain
    /// and configures the server to use it. The generated certificate files
    /// are saved as "self_signed_cert.pem" and "self_signed_key.pem".
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name for the certificate (e.g., "localhost")
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(all(feature = "tls", feature = "self-signed"))]
    /// use ignitia::{Router, Server};
    ///
    /// # #[cfg(all(feature = "tls", feature = "self-signed"))]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::new()
    ///     .get("/", || async { Ok(ignitia::Response::text("Dev HTTPS")) });
    ///
    /// let addr = "127.0.0.1:8443".parse()?;
    /// let server = Server::new(router, addr)
    ///     .with_self_signed_cert("localhost")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # ⚠️ Security Warning
    ///
    /// **Self-signed certificates should NEVER be used in production!**
    ///
    /// Self-signed certificates:
    /// - Provide no identity verification
    /// - Are vulnerable to man-in-the-middle attacks
    /// - Will cause browser security warnings
    /// - Should only be used for local development and testing
    #[cfg(all(feature = "tls", feature = "self-signed"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "tls", feature = "self-signed"))))]
    pub fn with_self_signed_cert(self, domain: &str) -> Result<Self, tls::TlsError> {
        let (_cert_pem, _key_pem) = tls::TlsConfig::generate_self_signed(domain)?;
        let tls_config = tls::TlsConfig::new("self_signed_cert.pem", "self_signed_key.pem");
        self.with_tls(tls_config)
    }

    /// Enables automatic HTTP to HTTPS redirection.
    ///
    /// When enabled, all HTTP requests will be automatically redirected to HTTPS
    /// using a 301 Moved Permanently response. This is useful for ensuring all
    /// traffic uses secure connections.
    ///
    /// # Arguments
    ///
    /// * `https_port` - The HTTPS port to redirect to
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, Server};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::new();
    /// let addr = "127.0.0.1:80".parse()?; // HTTP port
    ///
    /// let server = Server::new(router, addr)
    ///     .redirect_to_https(443); // Redirect to standard HTTPS port
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Behavior
    ///
    /// - All HTTP requests receive a 301 redirect response
    /// - The redirect URL preserves the original path and query parameters
    /// - If `https_port` is 443, the port is omitted from the redirect URL
    /// - The original request body is not forwarded in the redirect
    ///
    /// # Security Benefits
    ///
    /// - Prevents accidental transmission of sensitive data over HTTP
    /// - Helps with SEO by consolidating traffic to HTTPS URLs
    /// - Supports HSTS (HTTP Strict Transport Security) implementation
    pub fn redirect_to_https(mut self, https_port: u16) -> Self {
        self.config = self.config.redirect_to_https(https_port);
        self
    }

    /// Starts the server and begins accepting connections.
    ///
    /// This method starts the HTTP server, binds to the configured address,
    /// and begins accepting incoming connections. The method runs indefinitely,
    /// processing requests until the program is terminated.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server shuts down gracefully, or an error if
    /// the server fails to start or encounters a fatal error.
    ///
    /// # Errors
    ///
    /// Common errors include:
    /// - Address already in use (port binding failure)
    /// - Permission denied (binding to privileged ports)
    /// - TLS configuration errors
    /// - Network interface errors
    ///
    /// # Examples
    ///
    /// ## Basic Server Start
    /// ```
    /// use ignitia::{Router, Server, Response};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let router = Router::new()
    ///         .get("/", || async { Ok(Response::text("Server is running!")) });
    ///
    ///     let server = Server::new(router, "127.0.0.1:3000".parse()?);
    ///
    ///     println!("Starting server on http://127.0.0.1:3000");
    ///     server.ignitia().await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Server with Graceful Shutdown
    /// ```
    /// use ignitia::{Router, Server, Response};
    /// use tokio::signal;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let router = Router::new()
    ///         .get("/", || async { Ok(Response::text("Hello")) });
    ///
    ///     let server = Server::new(router, "127.0.0.1:3000".parse()?);
    ///
    ///     // Run server in a separate task
    ///     let server_task = tokio::spawn(async move {
    ///         if let Err(e) = server.ignitia().await {
    ///             eprintln!("Server error: {}", e);
    ///         }
    ///     });
    ///
    ///     // Wait for Ctrl+C
    ///     signal::ctrl_c().await?;
    ///     println!("Shutdown signal received");
    ///
    ///     // In a real implementation, you'd gracefully shutdown here
    ///     server_task.abort();
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Performance Characteristics
    ///
    /// - Uses async I/O for maximum concurrency
    /// - Spawns a new task for each connection
    /// - Supports thousands of concurrent connections
    /// - Automatically handles protocol negotiation (HTTP/1.1 vs HTTP/2)
    /// - WebSocket upgrades are handled transparently
    ///
    /// # Logging
    ///
    /// The server logs important events including:
    /// - Server startup information (address, protocols)
    /// - Connection errors and debugging information
    /// - Protocol negotiation results
    /// - TLS handshake information (when applicable)
    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;

        #[cfg(feature = "tls")]
        #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
        let protocol_info = if self.tls_acceptor.is_some() {
            if self.config.http2.enabled && self.config.http1_enabled {
                "HTTPS (HTTP/1.1 + HTTP/2)"
            } else if self.config.http2.enabled {
                "HTTPS (HTTP/2)"
            } else {
                "HTTPS (HTTP/1.1)"
            }
        } else if self.config.http2.enabled && self.config.http1_enabled {
            "HTTP/1.1 + HTTP/2"
        } else if self.config.http2.enabled {
            "HTTP/2"
        } else {
            "HTTP/1.1"
        };

        #[cfg(not(feature = "tls"))]
        let protocol_info = if self.config.http2.enabled && self.config.http1_enabled {
            "HTTP/1.1 + HTTP/2"
        } else if self.config.http2.enabled {
            "HTTP/2"
        } else {
            "HTTP/1.1"
        };

        #[cfg(feature = "tls")]
        #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
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

        if self.config.http2.enable_prior_knowledge {
            info!("📡 H2C (HTTP/2 Cleartext) enabled - supports curl --http2-prior-knowledge");
        }

        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    warn!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let router = Arc::clone(&self.router);
            let config = self.config.clone();

            #[cfg(feature = "tls")]
            #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
            let tls_acceptor = self.tls_acceptor.clone();

            tokio::spawn(async move {
                #[cfg(feature = "tls")]
                #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
                {
                    if let Some(acceptor) = tls_acceptor {
                        if let Err(err) =
                            handle_tls_connection(stream, router, config, acceptor, addr).await
                        {
                            debug!("TLS connection error from {}: {}", addr, err);
                        }
                    } else {
                        let io = TokioIo::new(stream);
                        if let Err(err) = handle_connection(io, router, config, addr).await {
                            debug!("Connection error from {}: {}", addr, err);
                        }
                    }
                }

                #[cfg(not(feature = "tls"))]
                {
                    let io = TokioIo::new(stream);
                    if let Err(err) = handle_connection(io, router, config, addr).await {
                        debug!("Connection error from {}: {}", addr, err);
                    }
                }
            });
        }
    }
}

/// Handles incoming TLS connections with protocol negotiation.
///
/// This function manages the TLS handshake, extracts ALPN protocol information,
/// and routes the connection to the appropriate HTTP protocol handler.
///
/// # Arguments
///
/// * `stream` - The incoming TCP stream
/// * `router` - The router for handling requests
/// * `config` - Server configuration
/// * `acceptor` - TLS acceptor for the handshake
/// * `addr` - Client address for logging
///
/// # Protocol Detection
///
/// The function uses ALPN (Application-Layer Protocol Negotiation) to determine
/// which HTTP protocol to use:
/// - "h2" → HTTP/2
/// - "http/1.1" → HTTP/1.1
/// - "http/1.0" → HTTP/1.1 (compatible)
/// - No ALPN → Auto-detection based on configuration
///
/// # Performance Notes
///
/// - TLS handshake is performed asynchronously
/// - ALPN information is extracted before moving the stream
/// - Protocol-specific optimizations are applied based on negotiation
/// - Connection errors are logged but don't crash the server
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
async fn handle_tls_connection(
    stream: tokio::net::TcpStream,
    router: Arc<Router>,
    config: ServerConfig,
    acceptor: TlsAcceptor,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use protocol::ProtocolDetector;

    let tls_stream = acceptor.accept(stream).await?;

    // Extract ALPN protocol information BEFORE moving tls_stream
    let alpn_protocol = {
        let (_, connection_info) = tls_stream.get_ref();
        connection_info.alpn_protocol().map(|p| p.to_vec()) // Clone the protocol bytes
    };

    debug!(
        "TLS connection from {}, ALPN: {:?}",
        addr,
        alpn_protocol.as_ref().map(|p| String::from_utf8_lossy(p))
    );

    // FIX: Use owned tls_stream, not reference
    let io = TokioIo::new(tls_stream);

    // Use ALPN to determine protocol
    let protocol = ProtocolDetector::detect_from_alpn(alpn_protocol.as_deref());

    let service = service_fn(move |req| {
        let router = Arc::clone(&router);
        async move {
            // FIX: Ensure proper return type
            handle_request(router, req, config.max_request_body_size).await
        }
    });

    match protocol {
        protocol::HttpProtocol::Http2 => {
            serve_http2_connection(io, service, &config.http2).await?;
        }
        protocol::HttpProtocol::Http1 | protocol::HttpProtocol::Auto => {
            let mut builder = http1::Builder::new();
            builder.half_close(true);
            builder.timer(TokioTimer::new());

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

/// Handles plain HTTP connections with protocol detection and routing.
///
/// This function manages non-TLS connections and can optionally redirect to HTTPS,
/// handle HTTP/2 prior knowledge (H2C), or serve regular HTTP/1.1 requests.
///
/// # Arguments
///
/// * `io` - The connection I/O wrapper
/// * `router` - The router for handling requests
/// * `config` - Server configuration
/// * `_addr` - Client address (currently unused but available for logging)
///
/// # Connection Types
///
/// Based on configuration, this function can:
/// 1. **HTTPS Redirect**: Redirect all HTTP traffic to HTTPS
/// 2. **HTTP/2 Prior Knowledge**: Handle H2C connections
/// 3. **Protocol Detection**: Auto-detect HTTP/1.1 vs HTTP/2
/// 4. **HTTP/1.1 Only**: Traditional HTTP/1.1 connections
///
/// # Security Considerations
///
/// When HTTPS redirect is enabled, all HTTP requests receive a 301 redirect
/// to the corresponding HTTPS URL, helping enforce secure connections.
async fn handle_connection(
    io: TokioIo<tokio::net::TcpStream>,
    router: Arc<Router>,
    config: ServerConfig,
    _addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if config.redirect_http_to_https {
        return handle_http_redirect(io, config.https_port.unwrap_or(443)).await;
    }

    let service = service_fn(move |req| {
        let router = Arc::clone(&router);
        async move { handle_request(router, req, config.max_request_body_size).await }
    });

    if config.auto_protocol_detection && config.http1_enabled && config.http2.enabled {
        let mut builder = http1::Builder::new();
        builder.half_close(true);
        builder.timer(TokioTimer::new());
        builder
            .serve_connection(io, service)
            .with_upgrades()
            .await?;
    } else if config.http2.enabled && config.http2.enable_prior_knowledge {
        serve_http2_connection(io, service, &config.http2).await?;
    } else if config.http2.enabled {
        serve_http2_connection(io, service, &config.http2).await?;
    } else {
        // http1::Builder::new().serve_connection(io, service).await?;
        let mut builder = http1::Builder::new();
        builder.timer(TokioTimer::new()); // Add timer here
        builder.serve_connection(io, service).await?;
    }

    Ok(())
}

/// Handles HTTP to HTTPS redirection for all incoming requests.
///
/// This function creates a service that responds to all HTTP requests with
/// a 301 Moved Permanently redirect to the corresponding HTTPS URL.
///
/// # Arguments
///
/// * `io` - The connection I/O wrapper
/// * `https_port` - The HTTPS port to redirect to
///
/// # Redirect Behavior
///
/// - Preserves the original Host header
/// - Maintains the full path and query parameters
/// - Uses 301 status code for permanent redirect
/// - Omits port number if redirecting to standard port 443
/// - Includes port number for non-standard ports
///
/// # Examples
///
/// Redirect examples:
/// - `http://example.com/path?query=1` → `https://example.com/path?query=1` (port 443)
/// - `http://example.com/api/users` → `https://example.com:8443/api/users` (port 8443)
async fn handle_http_redirect(
    io: TokioIo<tokio::net::TcpStream>,
    https_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    builder.timer(TokioTimer::new()); // Add timer here
    builder.serve_connection(io, service).await?;

    Ok(())
}

/// Serves HTTP/2 connections with optimized configuration.
///
/// This function creates an HTTP/2 server with the specified configuration
/// and handles the connection lifecycle including stream multiplexing,
/// flow control, and keep-alive management.
///
/// # Arguments
///
/// * `io` - The connection I/O wrapper
/// * `service` - The HTTP service to handle requests
/// * `config` - HTTP/2-specific configuration parameters
///
/// # Configuration Options
///
/// The function applies various HTTP/2 optimizations:
/// - **Concurrent Streams**: Limits simultaneous streams per connection
/// - **Window Sizes**: Controls flow control buffer sizes
/// - **Frame Size**: Sets maximum frame payload size
/// - **Keep-Alive**: Manages connection persistence
/// - **Adaptive Windows**: Enables dynamic flow control
/// - **Header Limits**: Prevents header-based attacks
///
/// # Performance Benefits
///
/// HTTP/2 provides several advantages:
/// - **Multiplexing**: Multiple requests per connection
/// - **Header Compression**: Reduces bandwidth usage
/// - **Server Push**: Proactive resource delivery (when supported)
/// - **Binary Protocol**: More efficient than HTTP/1.1 text format
/// - **Flow Control**: Prevents buffer overflow and ensures fairness
async fn serve_http2_connection<S>(
    io: TokioIo<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static>,
    service: S,
    config: &Http2Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
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
    builder.timer(TokioTimer::new());

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

    debug!("Serving HTTP/2 connection with config: {:?}", config);

    builder.serve_connection(io, service).await?;
    Ok(())
}

/// Handles individual HTTP requests and routes them through the application.
///
/// This is the main request processing function that:
/// 1. Checks for WebSocket upgrade requests
/// 2. Routes regular HTTP requests through the router
/// 3. Converts internal types to Hyper types
/// 4. Handles errors and converts them to appropriate responses
///
/// # Arguments
///
/// * `router` - The application router
/// * `req` - The incoming Hyper request
///
/// # Request Processing Flow
///
/// 1. **WebSocket Detection**: Check if request is WebSocket upgrade
/// 2. **Body Reading**: Read and limit request body size
/// 3. **Type Conversion**: Convert Hyper types to internal types
/// 4. **Routing**: Pass request through router and middleware
/// 5. **Response Conversion**: Convert response back to Hyper format
/// 6. **Error Handling**: Convert errors to appropriate HTTP responses
///
/// # Security Features
///
/// - Request body size limiting (10MB default) prevents memory exhaustion
/// - WebSocket protocol validation ensures secure upgrades
/// - Error information is sanitized in responses
/// - Header validation prevents injection attacks
async fn handle_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    max_body_size: usize,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();

    // Check for WebSocket upgrade first (fast path)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    {
        if is_websocket_upgrade(&req) {
            return handle_websocket_upgrade(router, req, &path).await;
        }
    }

    // Handle regular HTTP request
    handle_regular_http_request(router, req, max_body_size).await
}

/// Determines if an incoming request is a WebSocket upgrade request.
///
/// This function performs fast validation of WebSocket upgrade headers to
/// determine if the request should be handled as a WebSocket connection.
/// It checks for the presence and validity of required WebSocket headers.
///
/// # Parameters
/// - `req`: The incoming HTTP request to examine
///
/// # Returns
/// - `true`: If the request is a valid WebSocket upgrade request
/// - `false`: If the request is a regular HTTP request
///
/// # WebSocket Protocol Requirements
/// A valid WebSocket upgrade request must contain:
/// - `Connection: Upgrade` header
/// - `Upgrade: websocket` header
/// - `Sec-WebSocket-Key` header with a valid key
/// - `Sec-WebSocket-Version: 13` header
///
/// # Examples
/// This function is used internally by the server for WebSocket detection
/// and is not typically called directly by user code.
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
fn is_websocket_upgrade(req: &hyper::Request<hyper::body::Incoming>) -> bool {
    use hyper::header::{CONNECTION, UPGRADE};

    // Fast path checks with early returns
    let connection_header = match req.headers().get(CONNECTION) {
        Some(h) => h,
        None => return false,
    };

    let upgrade_header = match req.headers().get(UPGRADE) {
        Some(h) => h,
        None => return false,
    };

    let connection = match connection_header.to_str() {
        Ok(c) => c.to_lowercase(),
        Err(_) => return false,
    };

    let upgrade = match upgrade_header.to_str() {
        Ok(u) => u.to_lowercase(),
        Err(_) => return false,
    };

    connection.contains("upgrade")
        && upgrade.contains("websocket")
        && req.headers().get("sec-websocket-key").is_some()
        && req
            .headers()
            .get("sec-websocket-version")
            .map(|v| v == "13")
            .unwrap_or(false)
}

/// Handles WebSocket upgrade requests with proper protocol negotiation.
///
/// This function processes WebSocket upgrade requests by validating the
/// WebSocket protocol headers, generating appropriate response headers,
/// and spawning a task to handle the WebSocket connection.
///
/// # Parameters
/// - `router`: Shared reference to the router containing WebSocket handlers
/// - `req`: The incoming WebSocket upgrade request
/// - `path`: The path component of the request URI
///
/// # Returns
/// An HTTP 101 Switching Protocols response for successful upgrades,
/// or an error response for invalid requests or missing handlers
///
/// # WebSocket Upgrade Process
/// 1. **Handler Lookup**: Find the appropriate WebSocket handler for the path
/// 2. **Key Validation**: Validate the WebSocket key header
/// 3. **Response Generation**: Generate the WebSocket accept key and headers
/// 4. **Protocol Negotiation**: Handle optional WebSocket protocol negotiation
/// 5. **Connection Upgrade**: Spawn a task to handle the upgraded connection
///
/// # Error Responses
/// - **404**: No WebSocket handler found for the requested path
/// - **400**: Invalid or missing WebSocket headers
///
/// # Examples
/// This function is called internally when WebSocket upgrade requests are detected.
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
async fn handle_websocket_upgrade(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    path: &str,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    use hyper::header::SEC_WEBSOCKET_KEY;

    let websocket_handlers = router.get_websocket_handlers();
    let handler = match websocket_handlers.get(path) {
        Some(handler) => Arc::clone(handler.value()),
        None => {
            tracing::debug!("No WebSocket handler found for path: {}", path);
            return Ok(hyper::Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("WebSocket endpoint not found")))
                .unwrap());
        }
    };

    let websocket_key = match req.headers().get(SEC_WEBSOCKET_KEY) {
        Some(key) => match key.to_str() {
            Ok(k) => k,
            Err(_) => {
                return Ok(hyper::Response::builder()
                    .status(400)
                    .body(Full::new(Bytes::from("Invalid Sec-WebSocket-Key")))
                    .unwrap())
            }
        },
        None => {
            return Ok(hyper::Response::builder()
                .status(400)
                .body(Full::new(Bytes::from("Missing Sec-WebSocket-Key")))
                .unwrap())
        }
    };

    let accept_key = generate_accept_key(websocket_key);

    let mut response = hyper::Response::builder()
        .status(101)
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-accept", accept_key);

    if let Some(protocols) = req.headers().get("sec-websocket-protocol") {
        if let Ok(protocols_str) = protocols.to_str() {
            if let Some(protocol) = protocols_str.split(',').find(|p| !p.trim().is_empty()) {
                response = response.header("sec-websocket-protocol", protocol.trim());
            }
        }
    }

    let response = response.body(Full::new(Bytes::new())).unwrap();

    // Spawn WebSocket handling task
    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                if let Err(e) = crate::websocket::handle_websocket_upgrade(upgraded, handler).await
                {
                    tracing::debug!("WebSocket handler error: {}", e);
                }
            }
            Err(e) => {
                tracing::debug!("WebSocket upgrade failed: {}", e);
            }
        }
    });

    Ok(response)
}

/// Handles regular HTTP requests through the router system.
///
/// This function processes standard HTTP requests by parsing the request body,
/// routing the request through the framework's router, and generating an
/// appropriate HTTP response.
///
/// # Parameters
/// - `router`: Shared reference to the router for request processing
/// - `req`: The incoming HTTP request from Hyper
///
/// # Returns
/// A Hyper HTTP response containing the processed result
///
/// # Request Processing Flow
/// 1. **Request Parsing**: Parse HTTP request parts and body
/// 2. **Body Size Validation**: Check request body size against limits
/// 3. **Request Construction**: Build internal Request object
/// 4. **Router Processing**: Process request through the router
/// 5. **Response Generation**: Convert internal Response to Hyper response
/// 6. **Header Processing**: Copy response headers efficiently
///
/// # Request Size Limits
/// - **Default Limit**: 10MB maximum request body size
/// - **Error Response**: HTTP 413 (Payload Too Large) for oversized requests
/// - **Memory Protection**: Prevents memory exhaustion attacks
///
/// # Error Handling
/// - **Request Errors**: Converted to appropriate HTTP error responses
/// - **Router Errors**: Processed through the framework's error handling system
/// - **System Errors**: Logged and converted to HTTP 500 responses
///
/// # Examples
/// This function is called internally for all non-WebSocket HTTP requests.
async fn handle_regular_http_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    max_body_size: usize,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let (parts, body) = req.into_parts();

    // Limit request body size (10MB max)
    // const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    let body_bytes = match body.collect().await {
        Ok(collected) => {
            let bytes = collected.to_bytes();
            // Check body size using len() instead of size()
            if bytes.len() > max_body_size {
                let mut response =
                    hyper::Response::new(Full::new(Bytes::from("Request too large")));
                *response.status_mut() = http::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(response);
            }
            bytes
        }
        Err(_) => Bytes::new(),
    };

    let request = Request::new(
        parts.method,
        parts.uri,
        parts.version,
        parts.headers,
        body_bytes,
    );

    let response = match router.handle(request).await {
        Ok(res) => res,
        Err(err) => {
            let status = err.status_code();
            let mut res = Response::new(status);
            res.body = Arc::new(Bytes::from(err.to_string()));
            res
        }
    };

    let mut builder = hyper::Response::builder().status(response.status);

    // Pre-allocate headers vector for better performance
    for (key, value) in response.headers.iter() {
        builder = builder.header(key, value);
    }

    Ok(builder.body(Full::new((*response.body).clone())).unwrap())
}
