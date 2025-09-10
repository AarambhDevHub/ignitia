//! Tokio-based executor implementation for Hyper HTTP/2 connections.
//!
//! This module provides the `TokioExecutor` which integrates Hyper's HTTP/2
//! server with Tokio's async runtime. It handles spawning of futures for
//! concurrent request processing and connection management.
//!
//! # Overview
//!
//! The executor is responsible for:
//! - Spawning HTTP/2 connection handlers
//! - Managing concurrent request processing
//! - Integrating with Tokio's task scheduler
//! - Providing efficient async I/O for HTTP/2 streams
//!
//! # Performance Characteristics
//!
//! - **Zero-copy spawning**: Futures are moved directly to Tokio tasks
//! - **Work-stealing scheduler**: Tokio automatically load-balances across CPU cores
//! - **Efficient async I/O**: Uses epoll/kqueue for optimal performance
//! - **Memory efficient**: Tasks are heap-allocated only when necessary
//!
//! # Examples
//!
//! ## Basic usage with HTTP/2 builder
//! ```
//! use ignitia::server::executor::TokioExecutor;
//! use hyper::server::conn::http2;
//!
//! let executor = TokioExecutor;
//! let mut builder = http2::Builder::new(executor);
//!
//! // Configure and use the builder for HTTP/2 connections
//! builder.max_concurrent_streams(1000);
//! ```
//!
//! ## Integration with server configuration
//! ```
//! use ignitia::{Server, Router, ServerConfig, Http2Config};
//! use std::net::SocketAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let router = Router::new();
//! let addr: SocketAddr = "127.0.0.1:3000".parse()?;
//!
//! let config = ServerConfig {
//!     http2: Http2Config {
//!         enabled: true,
//!         max_concurrent_streams: Some(2000),
//!         ..Default::default()
//!     },
//!     ..Default::default()
//! };
//!
//! let server = Server::new(router, addr).with_config(config);
//! // TokioExecutor is used internally for HTTP/2 connections
//! # Ok(())
//! # }
//! ```

use std::future::Future;

/// Tokio-based executor for HTTP/2 connection handling.
///
/// `TokioExecutor` implements Hyper's `Executor` trait to spawn futures on
/// Tokio's async runtime. This integration allows HTTP/2 connections to
/// benefit from Tokio's work-stealing scheduler and efficient I/O handling.
///
/// # Design Philosophy
///
/// The executor follows a "spawn and forget" pattern where each HTTP/2 stream
/// or connection management task is spawned as an independent Tokio task.
/// This provides:
///
/// - **Isolation**: Each request/connection runs independently
/// - **Parallelism**: Multiple CPU cores can process requests simultaneously
/// - **Fault tolerance**: Task panics don't affect other connections
/// - **Backpressure**: Tokio's scheduler naturally handles load balancing
///
/// # Memory Management
///
/// - Futures are moved to the heap only when spawned
/// - Completed tasks are automatically cleaned up by Tokio
/// - No manual memory management required
/// - Memory usage scales with active connection count
///
/// # Error Handling
///
/// Task spawning with `tokio::spawn` is infallible - the returned `JoinHandle`
/// can be used to await completion and handle any panics or errors within
/// the spawned future.
///
/// # Examples
///
/// ## Manual instantiation
/// ```
/// use ignitia::server::executor::TokioExecutor;
///
/// let executor = TokioExecutor;
///
/// // The executor can be cloned cheaply
/// let executor_clone = executor.clone();
/// assert_eq!(std::mem::size_of_val(&executor), 0); // Zero-sized type
/// ```
///
/// ## Custom future spawning
/// ```
/// use ignitia::server::executor::TokioExecutor;
/// use hyper::rt::Executor;
///
/// # async fn example() {
/// let executor = TokioExecutor;
///
/// let future = async {
///     println!("Hello from spawned task!");
///     42
/// };
///
/// // Spawn the future on Tokio runtime
/// executor.execute(future);
/// # }
/// ```
///
/// # Performance Considerations
///
/// ## CPU Utilization
/// - Tasks automatically distribute across available CPU cores
/// - Work-stealing prevents core starvation
/// - CPU-bound tasks should yield periodically with `tokio::task::yield_now()`
///
/// ## Memory Usage
/// - Each spawned task has ~2KB overhead on 64-bit systems
/// - Stack size is dynamic and grows as needed
/// - Consider task pools for very high-frequency short-lived tasks
///
/// ## I/O Performance
/// - Tokio uses efficient system calls (epoll, kqueue, IOCP)
/// - Non-blocking I/O prevents thread pool exhaustion
/// - Built-in support for TCP_NODELAY and other optimizations
#[derive(Clone)]
pub struct TokioExecutor;

impl TokioExecutor {
    /// Create a new `TokioExecutor` instance.
    ///
    /// Since `TokioExecutor` is a zero-sized type with no internal state,
    /// this constructor is essentially free and mainly provided for
    /// explicit initialization patterns.
    ///
    /// # Returns
    /// A new `TokioExecutor` instance
    ///
    /// # Examples
    /// ```
    /// use ignitia::server::executor::TokioExecutor;
    ///
    /// let executor = TokioExecutor::new();
    ///
    /// // Equivalent to using the unit struct directly
    /// let executor2 = TokioExecutor;
    /// ```
    ///
    /// # Performance
    /// This function compiles to a no-op in release builds since
    /// `TokioExecutor` contains no data.
    pub fn new() -> Self {
        Self
    }
}

/// Implementation of Hyper's `Executor` trait for Tokio integration.
///
/// This implementation bridges Hyper's HTTP/2 server with Tokio's async runtime
/// by spawning futures as independent Tokio tasks. The trait is generic over
/// future types to support various HTTP/2 operations.
///
/// # Type Parameters
///
/// * `F` - The future type to execute
///   - Must implement `Future` with any output type
///   - Must be `Send + 'static` for task spawning
///   - Output must be `Send + 'static` for result handling
///
/// # Task Spawning Behavior
///
/// When `execute()` is called:
/// 1. The future is moved to Tokio's task queue
/// 2. Tokio scheduler assigns it to an available worker thread
/// 3. The future runs to completion independently
/// 4. Results/errors are handled within the spawned context
///
/// # Thread Safety
///
/// The executor is fully thread-safe:
/// - Can be called from any thread with Tokio runtime
/// - Spawned tasks may run on different threads
/// - No shared state between executor instances
///
/// # Examples
///
/// ## HTTP/2 connection handling
/// ```
/// use ignitia::server::executor::TokioExecutor;
/// use hyper::rt::Executor;
/// use std::future::ready;
///
/// # async fn example() {
/// let executor = TokioExecutor;
///
/// // Simulate HTTP/2 connection task
/// let connection_future = async {
///     // Handle HTTP/2 streams, process requests, etc.
///     println!("Processing HTTP/2 connection");
/// };
///
/// // Spawn the connection handler
/// executor.execute(connection_future);
/// # }
/// ```
///
/// ## Error handling in spawned tasks
/// ```
/// use ignitia::server::executor::TokioExecutor;
/// use hyper::rt::Executor;
///
/// # async fn example() {
/// let executor = TokioExecutor;
///
/// let error_prone_future = async {
///     // Errors within spawned tasks don't propagate automatically
///     if let Err(e) = some_fallible_operation().await {
///         eprintln!("Task error: {}", e);
///         // Handle error within task context
///     }
/// };
///
/// executor.execute(error_prone_future);
///
/// async fn some_fallible_operation() -> Result<(), Box<dyn std::error::Error>> {
///     Ok(())
/// }
/// # }
/// ```
///
/// # Integration with HTTP/2 Builder
/// ```
/// use ignitia::server::executor::TokioExecutor;
/// use hyper::server::conn::http2;
/// use std::time::Duration;
///
/// # async fn setup_http2() {
/// let executor = TokioExecutor;
/// let mut builder = http2::Builder::new(executor);
///
/// // Configure HTTP/2 settings
/// builder
///     .max_concurrent_streams(1000)
///     .initial_stream_window_size(65536)
///     .keep_alive_interval(Duration::from_secs(60));
///
/// // Builder now uses TokioExecutor for spawning connection tasks
/// # }
/// ```
impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    /// Execute a future by spawning it as a Tokio task.
    ///
    /// This method takes ownership of the future and spawns it on Tokio's
    /// current runtime. The future will run concurrently with other tasks
    /// and the calling code can continue immediately.
    ///
    /// # Arguments
    /// * `fut` - The future to execute
    ///
    /// # Spawning Semantics
    ///
    /// - **Non-blocking**: This method returns immediately
    /// - **Independent execution**: The spawned task runs separately
    /// - **Error isolation**: Panics in spawned tasks don't affect the caller
    /// - **Automatic cleanup**: Completed tasks are cleaned up by Tokio
    ///
    /// # Runtime Requirements
    ///
    /// This method must be called from within a Tokio runtime context.
    /// Calling it outside a runtime will result in a panic.
    ///
    /// # Performance Notes
    ///
    /// - **Cheap to call**: Spawning overhead is minimal (~microseconds)
    /// - **Memory efficient**: Only allocates when necessary
    /// - **CPU scalable**: Tasks distribute across available cores
    /// - **I/O optimized**: Leverages Tokio's efficient I/O reactor
    ///
    /// # Examples
    ///
    /// ## Basic usage
    /// ```
    /// use ignitia::server::executor::TokioExecutor;
    /// use hyper::rt::Executor;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let executor = TokioExecutor;
    ///
    /// let work = async {
    ///     println!("Doing work in background");
    ///     // Simulate some async work
    ///     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    ///     println!("Work completed");
    /// };
    ///
    /// // Spawn and continue
    /// executor.execute(work);
    /// println!("Spawned task, continuing...");
    ///
    /// // Give spawned task time to complete
    /// tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    /// # }
    /// ```
    ///
    /// ## With error handling
    /// ```
    /// use ignitia::server::executor::TokioExecutor;
    /// use hyper::rt::Executor;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let executor = TokioExecutor;
    ///
    /// let risky_work = async {
    ///     match risky_operation().await {
    ///         Ok(result) => println!("Success: {}", result),
    ///         Err(e) => eprintln!("Error in spawned task: {}", e),
    ///     }
    /// };
    ///
    /// executor.execute(risky_work);
    ///
    /// async fn risky_operation() -> Result<String, &'static str> {
    ///     Ok("All good!".to_string())
    /// }
    /// # }
    /// ```
    ///
    /// # Panic Behavior
    ///
    /// If the spawned future panics:
    /// - The panic is contained within the spawned task
    /// - Other tasks and the main thread continue normally
    /// - The panic can be observed via the `JoinHandle` if needed
    ///
    /// # Memory Management
    ///
    /// Spawned tasks are automatically cleaned up when they complete.
    /// No manual cleanup is required, but long-running tasks should
    /// be designed to terminate gracefully when appropriate.
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

impl Default for TokioExecutor {
    /// Create a default `TokioExecutor` instance.
    ///
    /// Equivalent to `TokioExecutor::new()` or `TokioExecutor` direct construction.
    /// Provided for consistency with Rust patterns and generic programming.
    ///
    /// # Returns
    /// A new `TokioExecutor` instance
    ///
    /// # Examples
    /// ```
    /// use ignitia::server::executor::TokioExecutor;
    ///
    /// let executor = TokioExecutor::default();
    ///
    /// // All of these are equivalent
    /// let executor1 = TokioExecutor::new();
    /// let executor2 = TokioExecutor;
    /// let executor3 = TokioExecutor::default();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}
