//! High-performance server module with advanced optimizations

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
use tokio::net::TcpListener;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use tokio_rustls::TlsAcceptor;

#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
use crate::websocket::upgrade::generate_accept_key;

/// High-performance HTTP/HTTPS server with advanced optimizations
pub struct Server {
    /// Application router
    router: Arc<Router>,
    /// Server bind address
    addr: SocketAddr,
    /// Server configuration
    config: ServerConfig,
    /// Performance configuration
    perf_config: PerformanceConfig,
    /// Performance metrics collection
    metrics: Arc<PerformanceMetrics>,
    /// Server state tracking
    state: Arc<ServerState>,

    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    /// TLS acceptor for HTTPS
    tls_acceptor: Option<TlsAcceptor>,
}

/// Server state for monitoring and graceful shutdown
#[derive(Debug)]
struct ServerState {
    /// Server running state
    running: AtomicBool,
    /// Active connection count
    active_connections: AtomicUsize,
    /// Total requests processed
    total_requests: AtomicU64,
    /// Server start time
    start_time: RwLock<Option<Instant>>,
}

impl ServerState {
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
    pub fn with_config(router: Router, addr: SocketAddr, config: ServerConfig) -> Self {
        let mut server = Self::new(router, addr);
        server.config = config;
        server
    }

    /// Create server optimized for maximum RPS
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
    pub fn with_performance_config(mut self, config: PerformanceConfig) -> Self {
        self.perf_config = config;
        self
    }

    /// Set server configuration
    pub fn with_server_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable HTTPS with custom TLS configuration
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub fn with_tls(mut self, tls_config: tls::TlsConfig) -> Result<Self, tls::TlsError> {
        let acceptor = tls_config.build()?;
        self.tls_acceptor = Some(acceptor);
        self.config.tls = Some(tls_config);
        Ok(self)
    }

    /// Enable HTTPS with certificate and key files
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
    #[cfg(all(feature = "tls", feature = "self-signed"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "tls", feature = "self-signed"))))]
    pub fn with_self_signed_cert(self, domain: &str) -> Result<Self, tls::TlsError> {
        let (cert_pem, key_pem) = tls::TlsConfig::generate_self_signed(domain)?;
        let tls_config = tls::TlsConfig::new("self_signed_cert.pem", "self_signed_key.pem");
        self.with_tls(tls_config)
    }

    /// Enable HTTP to HTTPS redirect
    pub fn redirect_to_https(mut self, https_port: u16) -> Self {
        self.config = self.config.redirect_to_https(https_port);
        self
    }

    /// Get server metrics
    pub fn metrics(&self) -> Arc<PerformanceMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get server uptime
    pub fn uptime(&self) -> Option<Duration> {
        self.state.start_time.read().map(|start| start.elapsed())
    }

    /// Get active connection count
    pub fn active_connections(&self) -> usize {
        self.state.active_connections.load(Ordering::Relaxed)
    }

    /// Get total requests processed
    pub fn total_requests(&self) -> u64 {
        self.state.total_requests.load(Ordering::Relaxed)
    }

    /// Start the high-performance server
    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Igniting high-performance server...");

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

        if self.config.http2.enable_prior_knowledge {
            info!("🌐 H2C (HTTP/2 Cleartext) enabled");
        }

        info!("📊 Performance optimizations enabled:");
        info!(
            "   - Zero-copy optimizations: {}",
            self.perf_config.zero_copy
        );
        info!("   - Connection backlog: {}", self.perf_config.backlog);

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
async fn handle_connection<I>(
    io: TokioIo<I>,
    router: Arc<Router>,
    config: ServerConfig,
    addr: SocketAddr,
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
#[cfg(feature = "websocket")]
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

/// Handle WebSocket upgrade requests
#[cfg(feature = "websocket")]
async fn handle_websocket_upgrade(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    use hyper::header::SEC_WEBSOCKET_KEY;

    let path = req.uri().path();
    let websocket_handlers = router.get_websocket_handlers();
    let handler = match websocket_handlers.get(path) {
        Some(handler) => Arc::clone(handler.value()),
        None => {
            debug!("🔍 No WebSocket handler found for path: {}", path);
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
                    .unwrap());
            }
        },
        None => {
            return Ok(hyper::Response::builder()
                .status(400)
                .body(Full::new(Bytes::from("Missing Sec-WebSocket-Key")))
                .unwrap());
        }
    };

    let accept_key = generate_accept_key(websocket_key);
    let mut response = hyper::Response::builder()
        .status(101)
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-accept", accept_key);

    // Handle protocol negotiation efficiently
    if let Some(protocols) = req.headers().get("sec-websocket-protocol") {
        if let Some(protocol) = protocols
            .to_str()
            .ok()
            .and_then(|protocols_str| protocols_str.split(',').find(|p| !p.trim().is_empty()))
        {
            if let Ok(protocol_value) = protocol.trim().parse::<http::HeaderValue>() {
                response = response.header("sec-websocket-protocol", protocol_value);
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
                    debug!("🔌 WebSocket handler error: {}", e);
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
            res.body = Arc::new(Bytes::from(err.to_string()));
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

    Ok(builder.body(Full::new((*response.body).clone())).unwrap())
}
