//! High-performance server optimizations for maximum RPS throughput

use crate::{Request, Response, Result};
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

/// Performance configuration for high-RPS scenarios
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable SO_REUSEPORT for better load distribution
    pub reuse_port: bool,
    /// Enable TCP_NODELAY for lower latency
    pub tcp_nodelay: bool,
    /// Enable SO_REUSEADDR
    pub reuse_addr: bool,
    /// TCP keep-alive configuration
    pub keep_alive: Option<Duration>,
    /// Socket send buffer size
    pub send_buffer_size: Option<usize>,
    /// Socket receive buffer size
    pub recv_buffer_size: Option<usize>,
    /// Connection backlog size
    pub backlog: u32,
    /// Enable CPU affinity for worker threads
    pub cpu_affinity: bool,
    /// Number of worker threads (0 = auto-detect)
    pub worker_threads: usize,
    /// Enable fast path optimizations
    pub fast_path: bool,
    /// Enable zero-copy optimizations where possible
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
    /// Create config optimized for maximum RPS
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

    /// Create config for high-throughput APIs
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
}

/// High-performance metrics collection
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Total requests processed
    pub requests_total: AtomicU64,
    /// Requests per second (rolling average)
    pub requests_per_second: AtomicU64,
    /// Active connections
    pub active_connections: AtomicUsize,
    /// Response times histogram
    pub response_times: RwLock<Vec<Duration>>,
    /// Memory usage tracking
    pub memory_usage: AtomicUsize,
    /// CPU usage percentage
    pub cpu_usage: AtomicU64,
    /// Error count
    pub error_count: AtomicU64,
    /// Last metrics update
    pub last_update: Mutex<Instant>,
}

impl PerformanceMetrics {
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

    /// Record a request completion
    pub fn record_request(&self, response_time: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);

        // Update response times (keep last 10k)
        let mut times = self.response_times.write();
        if times.len() >= 10000 {
            times.drain(0..1000); // Remove oldest 1000 entries
        }
        times.push(response_time);
    }

    /// Get current RPS
    pub fn current_rps(&self) -> u64 {
        self.requests_per_second.load(Ordering::Relaxed)
    }

    /// Get average response time
    pub fn avg_response_time(&self) -> Duration {
        let times = self.response_times.read();
        if times.is_empty() {
            return Duration::from_millis(0);
        }

        let total: Duration = times.iter().sum();
        total / times.len() as u32
    }

    /// Get 95th percentile response time
    pub fn p95_response_time(&self) -> Duration {
        let mut times = self.response_times.read().clone();
        if times.is_empty() {
            return Duration::from_millis(0);
        }

        times.sort();
        let index = (times.len() as f64 * 0.95) as usize;
        times[index.min(times.len() - 1)]
    }
}

/// High-performance TCP listener with optimizations
pub struct OptimizedTcpListener {
    listener: TcpListener,
    config: PerformanceConfig,
    metrics: Arc<PerformanceMetrics>,
}

impl OptimizedTcpListener {
    /// Create an optimized TCP listener
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

    /// Accept connections with optimizations
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

    /// Get performance metrics
    pub fn metrics(&self) -> Arc<PerformanceMetrics> {
        Arc::clone(&self.metrics)
    }
}

/// Create an optimized socket for high performance
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

/// Optimize individual TCP connection
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

/// Fast path request processing for high RPS scenarios
pub struct FastPathProcessor {
    router: Arc<crate::Router>,
    metrics: Arc<PerformanceMetrics>,
    cache: DashMap<String, Arc<Response>>,
}

impl FastPathProcessor {
    pub fn new(router: Arc<crate::Router>, metrics: Arc<PerformanceMetrics>) -> Self {
        Self {
            router,
            metrics,
            cache: DashMap::with_capacity(1000),
        }
    }

    /// Process request using fast path optimizations
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

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}
