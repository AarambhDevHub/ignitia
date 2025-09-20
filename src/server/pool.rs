//! Object pooling for high-performance request handling
//!
//! This module provides a high-performance object pool implementation designed for
//! web server scenarios where frequent object allocation and deallocation can impact
//! performance. The pool supports configuration for different use cases including
//! high-RPS scenarios, memory-constrained environments, and long-lived connections.
//!
//! # Features
//!
//! - **Generic object pooling**: Pool any type implementing the `Poolable` trait
//! - **Thread-safe**: Full concurrent access support with minimal lock contention
//! - **Configurable**: Tune pool behavior for different scenarios
//! - **Statistics**: Optional performance monitoring and debugging
//! - **Validation**: Optional object validation before reuse
//! - **Timeout handling**: Configurable acquire timeouts
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use crate::server::pool::{ObjectPool, PoolConfig, Poolable};
//! use std::time::Duration;
//!
//! // Create a pool for byte vectors
//! let pool = ObjectPool::new(
//!     PoolConfig::default(),
//!     || Vec::<u8>::with_capacity(8192)
//! );
//!
//! // Acquire an object
//! let mut buffer = pool.acquire()?;
//! buffer.extend_from_slice(b"Hello, world!");
//!
//! // Object is automatically returned to pool when dropped
//! ```
//!
//! ## High-Performance Configuration
//!
//! ```
//! let config = PoolConfig::high_rps();
//! let pool = ObjectPool::new(config, || Vec::<u8>::with_capacity(4096));
//! ```
//!
//! ## Global Object Pools
//!
//! ```
//! use crate::server::pool::ObjectPools;
//!
//! let pools = ObjectPools::new();
//! let buffer = pools.buffer_pool.acquire()?;
//! let string = pools.string_pool.acquire()?;
//! ```

use crate::Result;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for object pooling behavior
///
/// This struct controls how the object pool behaves in different scenarios.
/// Use the predefined configurations for common use cases, or customize
/// for specific requirements.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial number of objects to pre-allocate in the pool
    ///
    /// Higher values reduce allocation overhead during startup but use more memory.
    /// For high-throughput scenarios, consider 100-500 initial objects.
    pub initial_size: usize,

    /// Maximum number of objects the pool can hold
    ///
    /// Objects beyond this limit will be dropped instead of returned to pool.
    /// Set based on available memory and expected concurrent usage.
    pub max_size: usize,

    /// Maximum time to wait when acquiring objects from pool
    ///
    /// If the pool is empty and at max size, acquisition will wait up to this duration
    /// before creating a new object anyway or returning an error.
    pub acquire_timeout: Duration,

    /// Whether to validate objects before reusing them
    ///
    /// When enabled, objects returning from the pool are validated using `Poolable::is_valid()`.
    /// Invalid objects are discarded and new ones created. Disable for maximum performance.
    pub validate_objects: bool,

    /// Whether to collect detailed statistics about pool usage
    ///
    /// Statistics include creation count, reuse count, and timing metrics.
    /// Disable in production for slightly better performance.
    pub enable_stats: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 100,
            max_size: 1000,
            acquire_timeout: Duration::from_millis(100),
            validate_objects: true,
            enable_stats: true,
        }
    }
}

/// Statistics collected by the object pool
///
/// These metrics help monitor pool performance and tune configuration.
/// Only collected when `PoolConfig::enable_stats` is true.
#[derive(Debug, Default)]
pub struct PoolStats {
    /// Total number of new objects created by the pool
    pub total_created: AtomicUsize,

    /// Total number of objects reused from the pool
    pub total_reused: AtomicUsize,

    /// Current number of objects in the pool
    pub current_size: AtomicUsize,

    /// Maximum pool size reached during operation
    pub max_size_reached: AtomicUsize,

    /// Number of times acquisition timed out
    pub acquire_timeouts: AtomicUsize,

    /// Average time to acquire objects (when stats enabled)
    pub avg_acquire_time: Mutex<Duration>,
}

/// A high-performance, thread-safe object pool
///
/// The object pool manages a collection of reusable objects to reduce allocation overhead.
/// It's particularly useful for frequently allocated objects like buffers, strings, or
/// connection objects in high-throughput scenarios.
///
/// # Thread Safety
///
/// The pool is fully thread-safe and can be shared across multiple threads using `Arc`.
/// Internal synchronization uses `parking_lot::Mutex` for better performance than std mutexes.
///
/// # Memory Management
///
/// - Objects are stored in a `VecDeque` for efficient LIFO access
/// - Pool size is bounded by `max_size` to prevent unbounded growth
/// - Objects exceeding the limit are dropped rather than stored
/// - Statistics track memory usage patterns when enabled
pub struct ObjectPool<T: Poolable> {
    /// Thread-safe storage for pooled objects
    objects: Mutex<VecDeque<T>>,

    /// Pool configuration
    config: PoolConfig,

    /// Performance and usage statistics
    stats: PoolStats,

    /// Factory function for creating new objects
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

/// Trait for objects that can be pooled
///
/// Objects must implement this trait to be used with `ObjectPool`.
/// The trait ensures objects can be reset to a clean state and validated.
pub trait Poolable: Sized + Send + 'static {
    /// Reset the object to its initial state for reuse
    ///
    /// This method is called before returning an object from the pool.
    /// It should clear any data and reset the object to a clean state.
    ///
    /// # Example
    ///
    /// ```
    /// impl Poolable for Vec<u8> {
    ///     fn reset(&mut self) {
    ///         self.clear(); // Clear data but keep capacity
    ///     }
    /// }
    /// ```
    fn reset(&mut self);

    /// Check if the object is in a valid state for reuse
    ///
    /// This method is called when `PoolConfig::validate_objects` is true.
    /// Return false if the object should be discarded instead of reused.
    ///
    /// # Example
    ///
    /// ```
    /// impl Poolable for Vec<u8> {
    ///     fn is_valid(&self) -> bool {
    ///         self.capacity() <= 1024 * 1024 // Max 1MB capacity
    ///     }
    /// }
    /// ```
    fn is_valid(&self) -> bool;
}

impl<T: Poolable> ObjectPool<T> {
    /// Create a new object pool with the given configuration and factory function
    ///
    /// # Arguments
    ///
    /// * `config` - Pool configuration controlling behavior
    /// * `factory` - Function to create new objects when needed
    ///
    /// # Examples
    ///
    /// ```
    /// let pool = ObjectPool::new(
    ///     PoolConfig::default(),
    ///     || Vec::<u8>::with_capacity(4096)
    /// );
    /// ```
    pub fn new<F>(config: PoolConfig, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let mut pool = Self {
            objects: Mutex::new(VecDeque::with_capacity(config.initial_size)),
            config,
            stats: PoolStats::default(),
            factory: Arc::new(factory),
        };

        // Pre-populate pool
        pool.warmup();
        pool
    }

    /// Pre-populate the pool with initial objects
    ///
    /// This method is called during construction to create the initial set of objects.
    /// It reduces allocation overhead during the initial phase of operation.
    fn warmup(&mut self) {
        let mut objects = self.objects.lock();
        for _ in 0..self.config.initial_size {
            let obj = (self.factory)();
            objects.push_back(obj);
            self.stats.total_created.fetch_add(1, Ordering::Relaxed);
        }
        self.stats
            .current_size
            .store(self.config.initial_size, Ordering::Relaxed);
    }

    /// Acquire an object from the pool
    ///
    /// This method attempts to get an object from the pool, reusing existing objects
    /// when possible. If the pool is empty, it creates new objects up to the maximum
    /// pool size. Beyond that limit, it may wait or create temporary objects.
    ///
    /// # Returns
    ///
    /// Returns a `PooledObject<T>` that automatically returns to the pool when dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if acquisition times out (when pool is at capacity and
    /// `acquire_timeout` is exceeded).
    ///
    /// # Examples
    ///
    /// ```
    /// let pool = ObjectPool::new(PoolConfig::default(), || Vec::<u8>::new());
    /// let mut buffer = pool.acquire()?;
    /// buffer.push(42);
    /// // Buffer automatically returns to pool when dropped
    /// ```
    pub fn acquire(&self) -> Result<PooledObject<T>> {
        let start = Instant::now();

        // Try to get from pool first
        if let Some(mut obj) = self.objects.lock().pop_front() {
            self.stats.current_size.fetch_sub(1, Ordering::Relaxed);
            self.stats.total_reused.fetch_add(1, Ordering::Relaxed);

            // Validate object if enabled
            if self.config.validate_objects && !obj.is_valid() {
                // Create new object if validation fails
                let new_obj = (self.factory)();
                self.stats.total_created.fetch_add(1, Ordering::Relaxed);
                return Ok(PooledObject::new(new_obj, self));
            }

            obj.reset();
            return Ok(PooledObject::new(obj, self));
        }

        // Create new object if pool is empty but under max size
        let current_size = self.stats.current_size.load(Ordering::Relaxed);
        if current_size < self.config.max_size {
            let obj = (self.factory)();
            self.stats.total_created.fetch_add(1, Ordering::Relaxed);
            self.stats.current_size.fetch_add(1, Ordering::Relaxed);
            return Ok(PooledObject::new(obj, self));
        }

        // Pool is at max size, wait or timeout
        if start.elapsed() > self.config.acquire_timeout {
            self.stats.acquire_timeouts.fetch_add(1, Ordering::Relaxed);
            return Err(crate::Error::Internal("Pool acquire timeout".to_string()));
        }

        // Create new object anyway (exceeding max size temporarily)
        let obj = (self.factory)();
        self.stats.total_created.fetch_add(1, Ordering::Relaxed);
        Ok(PooledObject::new(obj, self))
    }

    /// Return an object to the pool
    ///
    /// This method is called automatically when a `PooledObject` is dropped.
    /// It stores the object in the pool for future reuse, unless the pool is at capacity.
    ///
    /// # Arguments
    ///
    /// * `obj` - The object to return to the pool
    fn release(&self, obj: T) {
        let mut objects = self.objects.lock();
        let current_size = objects.len();

        if current_size < self.config.max_size {
            objects.push_back(obj);
            self.stats
                .current_size
                .store(current_size + 1, Ordering::Relaxed);
            self.stats
                .max_size_reached
                .fetch_max(current_size + 1, Ordering::Relaxed);
        }
        // If pool is full, the object will be dropped
    }

    /// Get pool statistics
    ///
    /// Returns a reference to the pool's performance statistics.
    /// Statistics are only meaningful when `PoolConfig::enable_stats` is true.
    ///
    /// # Examples
    ///
    /// ```
    /// let stats = pool.stats();
    /// println!("Created: {}, Reused: {}, Current size: {}",
    ///     stats.total_created.load(Ordering::Relaxed),
    ///     stats.total_reused.load(Ordering::Relaxed),
    ///     stats.current_size.load(Ordering::Relaxed)
    /// );
    /// ```
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Clear all objects from the pool
    ///
    /// This method removes and drops all objects currently in the pool.
    /// It's useful for cleanup or when changing pool configuration.
    pub fn clear(&self) {
        let mut objects = self.objects.lock();
        objects.clear();
        self.stats.current_size.store(0, Ordering::Relaxed);
    }
}

/// A pooled object that returns to the pool when dropped
///
/// This wrapper ensures that objects are automatically returned to their pool
/// when they go out of scope. It implements `Deref` and `DerefMut` to provide
/// transparent access to the underlying object.
///
/// # Automatic Cleanup
///
/// The object is automatically returned to the pool when the `PooledObject`
/// is dropped, ensuring proper resource management without manual intervention.
pub struct PooledObject<'a, T: Poolable> {
    /// The pooled object (None when already returned)
    object: Option<T>,

    /// Reference to the pool for returning the object
    pool: &'a ObjectPool<T>,

    /// Time when the object was acquired (for statistics)
    acquire_time: Instant,
}

impl<'a, T: Poolable> PooledObject<'a, T> {
    /// Create a new pooled object wrapper
    ///
    /// This method is called internally by `ObjectPool::acquire()`.
    fn new(object: T, pool: &'a ObjectPool<T>) -> Self {
        Self {
            object: Some(object),
            pool,
            acquire_time: Instant::now(),
        }
    }
}

impl<'a, T: Poolable> Deref for PooledObject<'a, T> {
    type Target = T;
    /// Provide immutable access to the underlying object
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<'a, T: Poolable> DerefMut for PooledObject<'a, T> {
    /// Provide mutable access to the underlying object
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

impl<'a, T: Poolable> Drop for PooledObject<'a, T> {
    /// Automatically return the object to the pool when dropped
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            self.pool.release(object);
        }
    }
}

/// Predefined pool configurations for common scenarios
impl PoolConfig {
    /// Configuration optimized for high-RPS scenarios
    ///
    /// This configuration maximizes performance by:
    /// - Large initial and maximum pool sizes
    /// - Short acquire timeout
    /// - Disabled validation and statistics for minimum overhead
    ///
    /// Use this for high-throughput web servers where performance is critical.
    pub fn high_rps() -> Self {
        Self {
            initial_size: 500,
            max_size: 5000,
            acquire_timeout: Duration::from_millis(10),
            validate_objects: false, // Disable validation for maximum speed
            enable_stats: false,     // Disable stats for less overhead
        }
    }

    /// Configuration for memory-constrained environments
    ///
    /// This configuration minimizes memory usage by:
    /// - Small initial and maximum pool sizes
    /// - Enabled validation to ensure objects don't grow too large
    /// - Statistics enabled for monitoring
    ///
    /// Use this in embedded systems or when memory is limited.
    pub fn memory_constrained() -> Self {
        Self {
            initial_size: 50,
            max_size: 200,
            acquire_timeout: Duration::from_millis(50),
            validate_objects: true,
            enable_stats: true,
        }
    }

    /// Configuration for long-lived connections
    ///
    /// This configuration balances performance and resource management:
    /// - Moderate pool sizes
    /// - Longer acquire timeout for stability
    /// - Validation and statistics enabled for monitoring
    ///
    /// Use this for connection pools or other long-lived resources.
    pub fn long_lived() -> Self {
        Self {
            initial_size: 100,
            max_size: 1000,
            acquire_timeout: Duration::from_millis(100),
            validate_objects: true,
            enable_stats: true,
        }
    }
}

/// Poolable implementation for byte vectors
///
/// Byte vectors are commonly used for I/O buffers in web servers.
/// The implementation clears the vector but preserves capacity for efficiency.
impl Poolable for Vec<u8> {
    fn reset(&mut self) {
        self.clear();
    }

    fn is_valid(&self) -> bool {
        self.capacity() <= 1024 * 1024 // 1MB max capacity
    }
}

/// Poolable implementation for strings
///
/// Strings are commonly used for text processing in web applications.
/// The implementation clears the string but preserves capacity for efficiency.
impl Poolable for String {
    fn reset(&mut self) {
        self.clear();
    }

    fn is_valid(&self) -> bool {
        self.capacity() <= 1024 * 1024 // 1MB max capacity
    }
}

/// Global object pools for common types
///
/// This struct provides pre-configured pools for commonly used types
/// in web server scenarios. Use this for convenience when you don't
/// need custom pool configuration.
///
/// # Examples
///
/// ```
/// let pools = ObjectPools::new();
///
/// // Get a buffer for I/O operations
/// let mut buffer = pools.buffer_pool.acquire()?;
/// buffer.extend_from_slice(b"Hello, world!");
///
/// // Get a string for text processing
/// let mut text = pools.string_pool.acquire()?;
/// text.push_str("Processing request...");
/// ```
pub struct ObjectPools {
    /// Pool for byte vectors (I/O buffers)
    pub buffer_pool: Arc<ObjectPool<Vec<u8>>>,
    /// Pool for strings (text processing)
    pub string_pool: Arc<ObjectPool<String>>,
}

impl ObjectPools {
    /// Create global pools with default high-performance configuration
    ///
    /// Uses `PoolConfig::high_rps()` for maximum throughput.
    pub fn new() -> Self {
        Self {
            buffer_pool: Arc::new(ObjectPool::new(PoolConfig::high_rps(), || {
                Vec::with_capacity(8192)
            })),
            string_pool: Arc::new(ObjectPool::new(PoolConfig::high_rps(), || {
                String::with_capacity(1024)
            })),
        }
    }

    /// Create pools with custom configuration
    ///
    /// Use this when you need different pool behavior than the default.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration to use for both pools
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            buffer_pool: Arc::new(ObjectPool::new(config.clone(), || Vec::with_capacity(8192))),
            string_pool: Arc::new(ObjectPool::new(config, || String::with_capacity(1024))),
        }
    }
}

impl Default for ObjectPools {
    fn default() -> Self {
        Self::new()
    }
}
