//! Object pooling for high-performance request handling

use crate::Result;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for object pooling
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial pool size
    pub initial_size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Timeout for acquiring objects
    pub acquire_timeout: Duration,
    /// Enable object validation
    pub validate_objects: bool,
    /// Enable statistics collection
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

/// Pool statistics
#[derive(Debug, Default)]
pub struct PoolStats {
    /// Total objects created
    pub total_created: AtomicUsize,
    /// Total objects reused
    pub total_reused: AtomicUsize,
    /// Current pool size
    pub current_size: AtomicUsize,
    /// Maximum pool size reached
    pub max_size_reached: AtomicUsize,
    /// Acquire timeouts
    pub acquire_timeouts: AtomicUsize,
    /// Average acquire time
    pub avg_acquire_time: Mutex<Duration>,
}

/// Generic object pool for high-performance scenarios
pub struct ObjectPool<T: Poolable> {
    objects: Mutex<VecDeque<T>>,
    config: PoolConfig,
    stats: PoolStats,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

/// Trait for poolable objects
pub trait Poolable: Sized + Send + 'static {
    /// Reset object to initial state
    fn reset(&mut self);
    /// Validate object before reuse
    fn is_valid(&self) -> bool;
}

impl<T: Poolable> ObjectPool<T> {
    /// Create a new object pool
    pub fn new(config: PoolConfig, factory: impl Fn() -> T + Send + Sync + 'static) -> Self {
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

    /// Warm up the pool by creating initial objects
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
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Clear the pool
    pub fn clear(&self) {
        let mut objects = self.objects.lock();
        objects.clear();
        self.stats.current_size.store(0, Ordering::Relaxed);
    }
}

/// A pooled object that returns to the pool when dropped
pub struct PooledObject<'a, T: Poolable> {
    object: Option<T>,
    pool: &'a ObjectPool<T>,
    acquire_time: Instant,
}

impl<'a, T: Poolable> PooledObject<'a, T> {
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

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<'a, T: Poolable> DerefMut for PooledObject<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

impl<'a, T: Poolable> Drop for PooledObject<'a, T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            self.pool.release(object);
        }
    }
}

/// Pool configuration for different scenarios
impl PoolConfig {
    /// Configuration for high-RPS scenarios
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

// Implement Poolable for common types

impl Poolable for Vec<u8> {
    fn reset(&mut self) {
        self.clear();
    }

    fn is_valid(&self) -> bool {
        self.capacity() <= 1024 * 1024 // 1MB max capacity
    }
}

impl Poolable for String {
    fn reset(&mut self) {
        self.clear();
    }

    fn is_valid(&self) -> bool {
        self.capacity() <= 1024 * 1024 // 1MB max capacity
    }
}

/// Global pools for common objects
pub struct ObjectPools {
    pub buffer_pool: Arc<ObjectPool<Vec<u8>>>,
    pub string_pool: Arc<ObjectPool<String>>,
}

impl ObjectPools {
    /// Create global pools with default configuration
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
