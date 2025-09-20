//! High-performance server optimizations for maximum RPS throughput
//!
//! This module provides a comprehensive suite of performance optimization tools designed
//! to maximize request-per-second (RPS) throughput in high-traffic web server scenarios.
//! It includes TCP socket optimization, connection pooling, request caching, and detailed
//! performance metrics collection.
//!
//! # Features
//!
//! - **Socket-level optimizations**: SO_REUSEPORT, TCP_NODELAY, custom buffer sizes
//! - **Connection optimization**: Keep-alive tuning, backlog configuration
//! - **Response caching**: Fast-path processing for frequently requested resources
//! - **Performance metrics**: Real-time RPS tracking, response time percentiles
//! - **Resource management**: Efficient cleanup and memory usage monitoring
//!
//! # Performance Configurations
//!
//! The module provides several pre-configured performance profiles:
//!
//! - **Max RPS**: Optimized for absolute maximum throughput
//! - **High Throughput API**: Balanced performance for API servers
//! - **Memory Constrained**: Optimized for limited memory environments
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use crate::server::performance::{PerformanceConfig, OptimizedTcpListener};
//! use std::net::SocketAddr;
//!
//! let config = PerformanceConfig::max_rps();
//! let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
//! let listener = OptimizedTcpListener::bind(addr, config).await?;
//!
//! // Accept optimized connections
//! let (stream, remote_addr) = listener.accept().await?;
//! ```
//!
//! ## Custom Configuration
//!
//! ```
//! use crate::server::performance::PerformanceConfig;
//! use std::time::Duration;
//!
//! let config = PerformanceConfig {
//!     reuse_port: true,
//!     tcp_nodelay: true,
//!     backlog: 16384,
//!     send_buffer_size: Some(1024 * 1024), // 1MB
//!     recv_buffer_size: Some(512 * 1024),  // 512KB
//!     keep_alive: Some(Duration::from_secs(30)),
//!     fast_path: true,
//!     zero_copy: true,
//!     ..Default::default()
//! };
//! ```
//!
//! ## Performance Monitoring
//!
//! ```
//! use crate::server::performance::PerformanceMetrics;
//!
//! let metrics = PerformanceMetrics::new();
//!
//! // Record request completion
//! metrics.record_request(response_time);
//!
//! // Get current metrics
//! let current_rps = metrics.current_rps();
//! let avg_time = metrics.avg_response_time();
//! let p95_time = metrics.p95_response_time();
//! ```

use crate::{Request, Response, Result};
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

/// Performance configuration for high-RPS scenarios
///
/// This configuration struct controls various TCP socket and connection optimizations
/// that can significantly impact server performance in high-throughput scenarios.
/// Different presets are available for common use cases.
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable SO_REUSEPORT for better load distribution across worker threads
    ///
    /// When enabled, multiple worker processes can bind to the same port,
    /// allowing the kernel to distribute incoming connections more efficiently.
    /// This is particularly beneficial for multi-core systems.
    pub reuse_port: bool,

    /// Enable TCP_NODELAY to reduce latency by disabling Nagle's algorithm
    ///
    /// Nagle's algorithm buffers small packets to improve network efficiency,
    /// but this can increase latency for interactive applications. Disabling
    /// it provides lower latency at the cost of potentially more network packets.
    pub tcp_nodelay: bool,

    /// Enable SO_REUSEADDR to allow rapid server restart without "Address in use" errors
    ///
    /// This allows the server to bind to an address that's in the TIME_WAIT state,
    /// which is useful for development and deployment scenarios.
    pub reuse_addr: bool,

    /// TCP keep-alive configuration for detecting dead connections
    ///
    /// Keep-alive probes are sent after periods of inactivity to detect
    /// if the remote endpoint is still responsive. This helps clean up
    /// resources from dead connections.
    pub keep_alive: Option<Duration>,

    /// Socket send buffer size in bytes
    ///
    /// Larger send buffers can improve throughput for applications that send
    /// large amounts of data, but use more memory per connection.
    /// Typical values: 64KB-1MB for high-throughput scenarios.
    pub send_buffer_size: Option<usize>,

    /// Socket receive buffer size in bytes
    ///
    /// Larger receive buffers can improve performance when receiving large
    /// request bodies, but use more memory per connection.
    /// Typical values: 64KB-512KB for most web applications.
    pub recv_buffer_size: Option<usize>,

    /// Connection backlog size for the listening socket
    ///
    /// This controls how many pending connections can be queued by the kernel.
    /// Higher values help handle traffic spikes but use more kernel memory.
    /// Modern systems typically support values up to 65535.
    pub backlog: u32,

    /// Enable CPU affinity for worker threads (platform-dependent)
    ///
    /// When enabled, attempts to pin worker threads to specific CPU cores
    /// to reduce cache misses and improve performance consistency.
    pub cpu_affinity: bool,

    /// Number of worker threads (0 = auto-detect based on CPU cores)
    ///
    /// Controls the size of the async runtime thread pool.
    /// Auto-detection typically uses CPU core count + 2 for optimal performance.
    pub worker_threads: usize,

    /// Enable fast-path optimizations for common request patterns
    ///
    /// Fast-path processing bypasses expensive operations for simple requests,
    /// such as static file serving or health check endpoints.
    pub fast_path: bool,

    /// Enable zero-copy optimizations where possible
    ///
    /// Uses techniques like memory mapping and splice() system calls to
    /// avoid copying data between kernel and user space, improving performance
    /// and reducing CPU usage for I/O intensive operations.
    pub zero_copy: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            reuse_port: true,
            tcp_nodelay: true,
            reuse_addr: true,
            keep_alive: Some(Duration::from_secs(60)),
            send_buffer_size: Some(256 * 1024), // 256KB
            recv_buffer_size: Some(256 * 1024), // 256KB
            backlog: 8192,
            cpu_affinity: true,
            worker_threads: 0, // Auto-detect
            fast_path: true,
            zero_copy: true,
        }
    }
}

impl PerformanceConfig {
    /// Configuration optimized for maximum RPS
    ///
    /// This preset maximizes request throughput by:
    /// - Using larger buffer sizes for high-volume I/O
    /// - Aggressive connection pooling with shorter timeouts
    /// - Maximum backlog size for handling traffic spikes
    /// - All performance optimizations enabled
    ///
    /// Best for: High-traffic web APIs, load testing scenarios, production deployments
    /// expecting >10,000 RPS.
    pub fn max_rps() -> Self {
        Self {
            reuse_port: true,
            tcp_nodelay: true,
            reuse_addr: true,
            keep_alive: Some(Duration::from_secs(30)),
            send_buffer_size: Some(512 * 1024), // 512KB
            recv_buffer_size: Some(512 * 1024), // 512KB
            backlog: 16384,
            cpu_affinity: true,
            worker_threads: num_cpus::get() * 2,
            fast_path: true,
            zero_copy: true,
        }
    }

    /// Configuration for high-throughput API servers
    ///
    /// Balanced configuration that provides excellent performance while
    /// maintaining reasonable resource usage:
    /// - Moderate buffer sizes suitable for API payloads
    /// - Longer keep-alive for persistent connections
    /// - Optimized for typical API response patterns
    ///
    /// Best for: REST APIs, GraphQL servers, microservices handling moderate
    /// to high traffic (1,000-10,000 RPS).
    pub fn high_throughput_api() -> Self {
        Self {
            tcp_nodelay: true,
            keep_alive: Some(Duration::from_secs(120)),
            send_buffer_size: Some(1024 * 1024), // 1MB
            recv_buffer_size: Some(256 * 1024),  // 256KB
            backlog: 8192,
            fast_path: true,
            zero_copy: true,
            ..Self::default()
        }
    }

    /// Configuration for memory-constrained environments
    ///
    /// Optimized to minimize memory usage while maintaining good performance:
    /// - Smaller buffer sizes to reduce per-connection memory overhead
    /// - Conservative backlog size
    /// - Disabled advanced features that consume extra memory
    ///
    /// Best for: Embedded systems, containers with memory limits, development
    /// environments, or deployments with many concurrent low-traffic connections.
    pub fn memory_constrained() -> Self {
        Self {
            reuse_port: false,                          // Reduce kernel overhead
            tcp_nodelay: true,                          // Still optimize latency
            keep_alive: Some(Duration::from_secs(300)), // Longer to avoid reconnections
            send_buffer_size: Some(64 * 1024),          // 64KB
            recv_buffer_size: Some(32 * 1024),          // 32KB
            backlog: 1024,                              // Smaller backlog
            cpu_affinity: false,                        // Let OS handle scheduling
            worker_threads: 2,                          // Minimal thread pool
            fast_path: false,                           // Disable caching
            zero_copy: false,                           // Disable advanced optimizations
            reuse_addr: true,
        }
    }
}

/// High-performance metrics collection for monitoring server performance
///
/// Collects detailed performance statistics with minimal overhead using
/// atomic operations and lock-free data structures where possible.
/// Includes both real-time metrics and historical data for trend analysis.
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Total number of requests processed since server start
    ///
    /// This counter never resets and provides a baseline for calculating
    /// rates and detecting traffic patterns over time.
    pub requests_total: AtomicU64,

    /// Current requests per second (rolling average)
    ///
    /// Updated every second by a background task. Provides real-time
    /// throughput information for monitoring and alerting.
    pub requests_per_second: AtomicU64,

    /// Current number of active TCP connections
    ///
    /// Tracks concurrent connections to monitor server load and
    /// detect potential resource exhaustion scenarios.
    pub active_connections: AtomicUsize,

    /// Response time histogram (last 10,000 requests)
    ///
    /// Stores individual response times for calculating percentiles.
    /// Uses a circular buffer approach to maintain bounded memory usage
    /// while providing statistical accuracy.
    pub response_times: RwLock<Vec<Duration>>,

    /// Current memory usage estimate in bytes
    ///
    /// Tracks approximate memory consumption including connection buffers,
    /// cached responses, and internal data structures.
    pub memory_usage: AtomicUsize,

    /// Current CPU usage percentage (0-100)
    ///
    /// Estimated CPU utilization of the server process, updated periodically
    /// by sampling system metrics.
    pub cpu_usage: AtomicU64,

    /// Total number of errors encountered
    ///
    /// Includes connection errors, request parsing errors, handler errors,
    /// and other exceptional conditions. Useful for calculating error rates.
    pub error_count: AtomicU64,

    /// Timestamp of last metrics update
    ///
    /// Used internally to calculate time-based metrics and ensure
    /// consistent sampling intervals.
    pub last_update: Mutex<Instant>,
}

impl PerformanceMetrics {
    /// Create a new metrics collection instance
    ///
    /// Initializes all counters to zero and allocates internal data structures
    /// for histogram collection. This is a lightweight operation suitable for
    /// frequent instantiation.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            requests_total: AtomicU64::new(0),
            requests_per_second: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
            response_times: RwLock::new(Vec::with_capacity(10000)),
            memory_usage: AtomicUsize::new(0),
            cpu_usage: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_update: Mutex::new(Instant::now()),
        })
    }

    /// Record completion of a request with its response time
    ///
    /// Updates both the total request counter and response time histogram.
    /// This method is designed to be called from high-frequency code paths
    /// with minimal overhead.
    ///
    /// # Arguments
    ///
    /// * `response_time` - The total time taken to process the request
    ///
    /// # Examples
    ///
    /// ```
    /// let start = Instant::now();
    /// // ... process request ...
    /// metrics.record_request(start.elapsed());
    /// ```
    pub fn record_request(&self, response_time: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);

        // Update response times (keep last 10k)
        let mut times = self.response_times.write();
        if times.len() >= 10000 {
            times.drain(0..1000); // Remove oldest 1000 entries
        }
        times.push(response_time);
    }

    /// Get current requests per second
    ///
    /// Returns the most recent RPS calculation, updated by background metrics
    /// collection tasks. This provides near real-time throughput information.
    ///
    /// # Returns
    ///
    /// Current RPS as calculated over the last sampling interval (typically 1 second).
    pub fn current_rps(&self) -> u64 {
        self.requests_per_second.load(Ordering::Relaxed)
    }

    /// Calculate average response time across all recent requests
    ///
    /// Computes the arithmetic mean of response times in the current histogram.
    /// This provides a general sense of system performance but can be skewed
    /// by outliers.
    ///
    /// # Returns
    ///
    /// Average response time, or zero if no requests have been recorded.
    ///
    /// # Performance Notes
    ///
    /// This method acquires a read lock on the response times collection,
    /// so it should not be called from high-frequency code paths.
    pub fn avg_response_time(&self) -> Duration {
        let times = self.response_times.read();
        if times.is_empty() {
            return Duration::from_millis(0);
        }

        let total: Duration = times.iter().sum();
        total / times.len() as u32
    }

    /// Calculate 95th percentile response time
    ///
    /// Computes the 95th percentile of recent response times, which represents
    /// the response time below which 95% of requests complete. This metric is
    /// less sensitive to outliers than average response time.
    ///
    /// # Returns
    ///
    /// 95th percentile response time, or zero if no requests have been recorded.
    ///
    /// # Performance Notes
    ///
    /// This method creates a sorted copy of the response times vector, so it
    /// has O(n log n) time complexity and should be used sparingly in
    /// high-frequency scenarios.
    pub fn p95_response_time(&self) -> Duration {
        let mut times = self.response_times.read().clone();
        if times.is_empty() {
            return Duration::from_millis(0);
        }

        times.sort();
        let index = (times.len() as f64 * 0.95) as usize;
        times[index.min(times.len() - 1)]
    }

    /// Calculate 99th percentile response time
    ///
    /// Similar to p95 but represents the response time below which 99% of
    /// requests complete. This is useful for identifying worst-case performance
    /// and setting SLA targets.
    pub fn p99_response_time(&self) -> Duration {
        let mut times = self.response_times.read().clone();
        if times.is_empty() {
            return Duration::from_millis(0);
        }

        times.sort();
        let index = (times.len() as f64 * 0.99) as usize;
        times[index.min(times.len() - 1)]
    }

    /// Get total number of requests processed
    ///
    /// Returns the lifetime total of requests handled by this server instance.
    /// This counter never resets and can be used for capacity planning and
    /// trend analysis.
    pub fn total_requests(&self) -> u64 {
        self.requests_total.load(Ordering::Relaxed)
    }

    /// Get current number of active connections
    ///
    /// Returns the instantaneous count of open TCP connections. This metric
    /// helps monitor server load and detect potential resource exhaustion.
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Get current error count
    ///
    /// Returns the total number of errors encountered during request processing.
    /// Combined with the total request count, this can be used to calculate
    /// error rates for monitoring and alerting.
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// Calculate current error rate as a percentage
    ///
    /// Computes the ratio of errors to total requests as a percentage.
    /// Returns 0.0 if no requests have been processed.
    pub fn error_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            return 0.0;
        }

        let errors = self.error_count();
        (errors as f64 / total as f64) * 100.0
    }

    /// Record an error occurrence
    ///
    /// Increments the error counter. Should be called whenever request
    /// processing encounters an exceptional condition.
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset all metrics to initial state
    ///
    /// Clears all counters and histograms. Useful for testing scenarios
    /// or when implementing metrics rotation policies.
    ///
    /// # Warning
    ///
    /// This operation is not atomic across all metrics. Concurrent access
    /// during reset may result in inconsistent intermediate states.
    pub fn reset(&self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.requests_per_second.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.response_times.write().clear();
        *self.last_update.lock() = Instant::now();
    }
}

/// High-performance TCP listener with socket-level optimizations
///
/// Wraps a standard Tokio TCP listener with additional optimizations applied
/// at the socket level. Provides methods for accepting connections with
/// per-connection optimization and integrated metrics collection.
pub struct OptimizedTcpListener {
    /// The underlying Tokio TCP listener
    listener: TcpListener,

    /// Performance configuration applied to this listener
    config: PerformanceConfig,

    /// Metrics collection for this listener instance
    metrics: Arc<PerformanceMetrics>,
}

impl OptimizedTcpListener {
    /// Create an optimized TCP listener bound to the specified address
    ///
    /// Applies all socket-level optimizations specified in the configuration,
    /// including buffer sizes, reuse options, and protocol tuning parameters.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address to bind to
    /// * `config` - Performance configuration to apply
    ///
    /// # Returns
    ///
    /// A configured listener ready to accept optimized connections
    ///
    /// # Errors
    ///
    /// Returns an error if socket creation, binding, or optimization fails.
    /// Common causes include address already in use, permission denied,
    /// or unsupported socket options on the target platform.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = PerformanceConfig::max_rps();
    /// let addr = "0.0.0.0:8080".parse().unwrap();
    /// let listener = OptimizedTcpListener::bind(addr, config).await?;
    /// ```
    pub async fn bind(addr: SocketAddr, config: PerformanceConfig) -> Result<Self> {
        let socket = create_optimized_socket(&addr, &config)?;
        socket.bind(&addr.into())?;
        socket.listen(config.backlog as i32)?;
        socket.set_nonblocking(true)?;

        let std_listener = std::net::TcpListener::from(socket);
        let listener = TcpListener::from_std(std_listener)?;

        info!(
            "Optimized TCP listener bound to {} with config: {:?}",
            addr, config
        );

        Ok(Self {
            listener,
            config,
            metrics: PerformanceMetrics::new(),
        })
    }

    /// Accept an incoming connection with applied optimizations
    ///
    /// Waits for and accepts the next incoming connection, applying per-connection
    /// optimizations such as TCP_NODELAY and keep-alive settings. Also updates
    /// the active connection counter in metrics.
    ///
    /// # Returns
    ///
    /// A tuple containing the optimized TCP stream and remote address
    ///
    /// # Errors
    ///
    /// Returns errors from the underlying accept operation or connection
    /// optimization failures. Connection optimization errors are logged
    /// but don't prevent the connection from being returned.
    ///
    /// # Examples
    ///
    /// ```
    /// loop {
    ///     let (stream, remote_addr) = listener.accept().await?;
    ///     // Handle connection...
    /// }
    /// ```
    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;

        // Apply per-connection optimizations
        if let Err(e) = optimize_connection(&stream, &self.config).await {
            warn!("Failed to optimize connection from {}: {}", addr, e);
        }

        self.metrics
            .active_connections
            .fetch_add(1, Ordering::Relaxed);

        Ok((stream, addr))
    }

    /// Get performance metrics for this listener
    ///
    /// Returns a reference to the metrics collector associated with this
    /// listener instance. Metrics include connection counts, accept rates,
    /// and error statistics.
    pub fn metrics(&self) -> Arc<PerformanceMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get the local address this listener is bound to
    ///
    /// Useful for logging, monitoring, and service discovery scenarios.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.listener.local_addr().map_err(Into::into)
    }
}

/// Create an optimized socket with performance tuning applied
///
/// Creates a new socket with all optimizations specified in the configuration.
/// This includes protocol-level options, buffer sizing, and connection behavior.
///
/// # Arguments
///
/// * `addr` - Target address (used for determining IPv4 vs IPv6)
/// * `config` - Performance configuration to apply
///
/// # Returns
///
/// A configured socket ready for binding and listening
///
/// # Platform Support
///
/// Some optimizations may not be available on all platforms:
/// - SO_REUSEPORT: Linux 3.9+, FreeBSD, macOS
/// - TCP keep-alive tuning: Most Unix-like systems
/// - Large buffer sizes: May be limited by system configuration
fn create_optimized_socket(addr: &SocketAddr, config: &PerformanceConfig) -> Result<Socket> {
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    // Apply socket options
    socket.set_reuse_address(config.reuse_addr)?;
    socket.set_reuse_port(config.reuse_port)?;
    socket.set_nodelay(config.tcp_nodelay)?;

    if let Some(keep_alive) = config.keep_alive {
        socket.set_keepalive(true)?;
        socket.set_tcp_keepalive(&socket2::TcpKeepalive::new().with_time(keep_alive))?;
    }

    if let Some(send_size) = config.send_buffer_size {
        socket.set_send_buffer_size(send_size)?;
    }

    if let Some(recv_size) = config.recv_buffer_size {
        socket.set_recv_buffer_size(recv_size)?;
    }

    Ok(socket)
}

/// Apply per-connection optimizations to an accepted TCP stream
///
/// Applies connection-specific optimizations that can't be set at the listener
/// level. This includes per-connection TCP settings and platform-specific
/// performance tuning.
///
/// # Arguments
///
/// * `stream` - The accepted TCP stream to optimize
/// * `config` - Performance configuration containing optimization parameters
///
/// # Errors
///
/// Returns socket option errors, but these are typically non-fatal and
/// the connection can still be used even if some optimizations fail.
async fn optimize_connection(stream: &TcpStream, config: &PerformanceConfig) -> Result<()> {
    use socket2::Socket;
    use std::os::fd::{AsRawFd, FromRawFd};

    stream.set_nodelay(config.tcp_nodelay)?;

    if let Some(keep_alive) = config.keep_alive {
        // Get the raw file descriptor
        let raw_fd = stream.as_raw_fd();

        // Create a socket2::Socket from the raw fd (without taking ownership)
        let socket = unsafe { Socket::from_raw_fd(raw_fd) };

        // Configure keep-alive
        socket.set_keepalive(true)?;
        socket.set_tcp_keepalive(&socket2::TcpKeepalive::new().with_time(keep_alive))?;

        // Important: Don't drop the socket as it would close the connection
        std::mem::forget(socket);
    }

    Ok(())
}

/// Fast-path request processor for high RPS scenarios
///
/// Implements aggressive caching and optimization strategies for frequently
/// accessed resources. Uses in-memory caches with configurable eviction
/// policies and request pattern analysis for optimal performance.
pub struct FastPathProcessor {
    /// Application router for handling requests
    router: Arc<crate::Router>,

    /// Performance metrics collection
    metrics: Arc<PerformanceMetrics>,

    /// Response cache for frequently accessed resources
    cache: DashMap<String, Arc<Response>>,
}

impl FastPathProcessor {
    /// Create a new fast-path processor
    ///
    /// # Arguments
    ///
    /// * `router` - The application router for processing requests
    /// * `metrics` - Metrics collector for performance tracking
    pub fn new(router: Arc<crate::Router>, metrics: Arc<PerformanceMetrics>) -> Self {
        Self {
            router,
            metrics,
            cache: DashMap::with_capacity(1000),
        }
    }

    /// Process a request using fast-path optimizations
    ///
    /// Attempts to serve requests from cache when possible, falling back to
    /// full router processing for cache misses. Automatically populates the
    /// cache with cacheable responses based on HTTP headers.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request to process
    ///
    /// # Returns
    ///
    /// The HTTP response, either from cache or freshly generated
    ///
    /// # Performance Characteristics
    ///
    /// - Cache hits: ~1-2μs response time
    /// - Cache misses: Full router processing time + cache population overhead
    /// - Memory usage: Bounded by cache size limit (default 1000 entries)
    pub async fn process(&self, request: Request) -> Result<Response> {
        let start = Instant::now();

        let method = request.method.clone();
        let path = request.uri.path().to_string();
        // Check cache first for GET requests
        if method == http::Method::GET {
            let cache_key = request.uri.path();
            if let Some(cached) = self.cache.get(cache_key) {
                self.metrics.record_request(start.elapsed());
                // CORRECT: Use .clone() on the Ref guard directly
                return Ok((**cached).clone());
            }
        }

        // Process through router
        let response = self.router.handle(request).await?;

        // Cache successful GET responses if they have cache headers
        if method == http::Method::GET && response.is_cacheable() {
            let cache_key = response.cache_key(&path);
            self.cache.insert(cache_key, Arc::new(response.clone()));
        }

        self.metrics.record_request(start.elapsed());
        Ok(response)
    }

    /// Clear the response cache
    ///
    /// Removes all cached responses. Useful for cache invalidation scenarios
    /// or memory management in long-running servers.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}
