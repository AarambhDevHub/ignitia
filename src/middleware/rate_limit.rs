//! Rate Limiting Middleware for Ignitia Web Framework
//!
//! This middleware provides flexible, high-performance rate limiting capabilities
//! based on various key extraction strategies (IP, user ID, API key, etc.).
//!
//! # Features
//! - Sliding window algorithm for smooth rate limiting
//! - Configurable key extraction (IP, custom functions)
//! - Automatic cleanup of expired entries
//! - HTTP 429 responses with proper headers
//! - Memory-efficient implementation with bounded storage
//! - Async-friendly with non-blocking operations
//!
//! # Examples
//!
//! ## Basic Usage (IP-based)
//! ```
//! use ignitia::{Router, RateLimitingMiddleware};
//!
//! let router = Router::new()
//!     .middleware(RateLimitingMiddleware::per_minute(100))
//!     .get("/api", handler);
//! ```
//!
//! ## Custom Key Extraction
//! ```
//! use ignitia::{Router, RateLimitConfig, RateLimitingMiddleware};
//! use std::time::Duration;
//!
//! let config = RateLimitConfig::new(50, Duration::from_secs(60))
//!     .with_key_extractor(|req| {
//!         // Rate limit by API key instead of IP
//!         req.header("x-api-key")
//!             .map(|key| format!("api_key:{}", key))
//!             .unwrap_or_else(|| "anonymous".to_string())
//!     })
//!     .with_error_message("API rate limit exceeded. Please wait before making more requests.");
//!
//! let router = Router::new()
//!     .middleware(RateLimitingMiddleware::new(config))
//!     .get("/api", handler);
//! ```

use crate::middleware::Middleware;
use crate::{Request, Response};
use http::StatusCode;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

use super::Next;

/// Configuration for the rate limiting middleware
///
/// This struct allows fine-tuning of rate limiting behavior including
/// the number of allowed requests, time windows, key extraction logic,
/// and response customization.
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed within the time window
    pub max_requests: usize,

    /// Duration of the sliding time window
    pub window: Duration,

    /// Optional custom function to extract rate limiting key from request
    /// If None, defaults to IP-based extraction
    pub key_extractor: Option<Arc<dyn Fn(&Request) -> String + Send + Sync>>,

    /// Custom error message returned when rate limit is exceeded
    pub error_message: String,

    /// Whether to include rate limit information in response headers
    pub include_headers: bool,

    /// Whether to use burst allowance (allows temporary spikes above the limit)
    pub allow_burst: bool,

    /// Burst multiplier (e.g., 1.5 allows 50% more requests temporarily)
    pub burst_multiplier: f32,
}

impl std::fmt::Debug for RateLimitConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimitConfig")
            .field("max_requests", &self.max_requests)
            .field("window", &self.window)
            .field("key_extractor", &"<function>") // Can't debug function pointers
            .field("error_message", &self.error_message)
            .field("include_headers", &self.include_headers)
            .field("allow_burst", &self.allow_burst)
            .field("burst_multiplier", &self.burst_multiplier)
            .finish()
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60), // 1 minute
            key_extractor: None,
            error_message: "Rate limit exceeded. Please try again later.".to_string(),
            include_headers: true,
            allow_burst: false,
            burst_multiplier: 1.5,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit configuration with specified max requests and window
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed in the time window
    /// * `window` - Duration of the time window
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use ignitia::RateLimitConfig;
    ///
    /// let config = RateLimitConfig::new(50, Duration::from_secs(30));
    /// ```
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            ..Default::default()
        }
    }

    /// Create configuration for requests per minute
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per minute
    ///
    /// # Examples
    /// ```
    /// let config = RateLimitConfig::per_minute(100); // 100 requests per minute
    /// ```
    pub fn per_minute(max_requests: usize) -> Self {
        Self::new(max_requests, Duration::from_secs(60))
    }

    /// Create configuration for requests per second
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per second
    ///
    /// # Examples
    /// ```
    /// let config = RateLimitConfig::per_second(10); // 10 requests per second
    /// ```
    pub fn per_second(max_requests: usize) -> Self {
        Self::new(max_requests, Duration::from_secs(1))
    }

    /// Create configuration for requests per hour
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per hour
    pub fn per_hour(max_requests: usize) -> Self {
        Self::new(max_requests, Duration::from_secs(3600))
    }

    /// Set a custom key extraction function
    ///
    /// The key extractor function receives a request and returns a string
    /// that uniquely identifies the client for rate limiting purposes.
    ///
    /// # Arguments
    /// * `extractor` - Function that extracts the key from a request
    ///
    /// # Examples
    /// ```
    /// let config = RateLimitConfig::per_minute(100)
    ///     .with_key_extractor(|req| {
    ///         // Rate limit by user ID from JWT token
    ///         req.header("authorization")
    ///             .and_then(|auth| extract_user_id_from_jwt(auth))
    ///             .unwrap_or_else(|| req.header("x-forwarded-for")
    ///                 .unwrap_or("unknown").to_string())
    ///     });
    /// ```
    pub fn with_key_extractor<F>(mut self, extractor: F) -> Self
    where
        F: Fn(&Request) -> String + Send + Sync + 'static,
    {
        self.key_extractor = Some(Arc::new(extractor));
        self
    }

    /// Set custom error message for rate limit exceeded responses
    ///
    /// # Arguments
    /// * `message` - Custom error message
    ///
    /// # Examples
    /// ```
    /// let config = RateLimitConfig::per_minute(100)
    ///     .with_error_message("Slow down! You're making too many requests.");
    /// ```
    pub fn with_error_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = message.into();
        self
    }

    /// Enable or disable rate limit headers in responses
    ///
    /// When enabled, responses include headers like:
    /// - `X-RateLimit-Limit`: Maximum requests allowed
    /// - `X-RateLimit-Remaining`: Requests remaining in current window
    /// - `X-RateLimit-Reset`: When the rate limit resets
    ///
    /// # Arguments
    /// * `include` - Whether to include headers
    pub fn with_headers(mut self, include: bool) -> Self {
        self.include_headers = include;
        self
    }

    /// Enable burst allowance for temporary traffic spikes
    ///
    /// When enabled, allows temporary spikes above the base limit
    /// using the burst multiplier.
    ///
    /// # Arguments
    /// * `multiplier` - Burst multiplier (e.g., 1.5 = 50% more requests)
    ///
    /// # Examples
    /// ```
    /// let config = RateLimitConfig::per_minute(100)
    ///     .with_burst(1.8); // Allow up to 180 requests temporarily
    /// ```
    pub fn with_burst(mut self, multiplier: f32) -> Self {
        self.allow_burst = true;
        self.burst_multiplier = multiplier.max(1.0);
        self
    }

    /// Disable burst allowance
    pub fn without_burst(mut self) -> Self {
        self.allow_burst = false;
        self
    }
}

/// Tracks request timestamps within a sliding window for a specific key
#[derive(Debug, Clone)]
struct RequestWindow {
    /// Timestamps of recent requests within the window
    timestamps: Vec<Instant>,
    /// Last time this window was cleaned up
    last_cleanup: Instant,
    /// Total number of requests seen (for metrics)
    total_requests: u64,
}

impl RequestWindow {
    /// Create a new empty request window
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            last_cleanup: Instant::now(),
            total_requests: 0,
        }
    }

    /// Remove expired timestamps outside the sliding window
    ///
    /// # Arguments
    /// * `window` - Duration of the sliding window
    fn cleanup_expired(&mut self, window: Duration) {
        let now = Instant::now();
        let cutoff = now.checked_sub(window).unwrap_or(now);

        // Remove timestamps older than the window
        self.timestamps.retain(|&timestamp| timestamp > cutoff);
        self.last_cleanup = now;

        trace!(
            "Cleaned up expired timestamps, {} remaining",
            self.timestamps.len()
        );
    }

    /// Check if a new request is allowed and record it if so
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed in window
    /// * `window` - Duration of the sliding window
    /// * `allow_burst` - Whether to allow burst requests
    /// * `burst_multiplier` - Multiplier for burst allowance
    ///
    /// # Returns
    /// `true` if the request is allowed, `false` if rate limited
    fn is_allowed(
        &mut self,
        max_requests: usize,
        window: Duration,
        allow_burst: bool,
        burst_multiplier: f32,
    ) -> bool {
        self.cleanup_expired(window);

        let current_count = self.timestamps.len();
        let effective_limit = if allow_burst {
            ((max_requests as f32) * burst_multiplier) as usize
        } else {
            max_requests
        };

        if current_count >= effective_limit {
            debug!(
                "Rate limit exceeded: {} >= {}",
                current_count, effective_limit
            );
            false
        } else {
            // Allow the request and record it
            let now = Instant::now();
            self.timestamps.push(now);
            self.total_requests += 1;

            trace!("Request allowed: {}/{}", current_count + 1, effective_limit);
            true
        }
    }

    /// Get the number of remaining requests in the current window
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed in window
    ///
    /// # Returns
    /// Number of requests remaining
    fn remaining_requests(&self, max_requests: usize) -> usize {
        max_requests.saturating_sub(self.timestamps.len())
    }

    /// Get the time when the rate limit will reset (oldest request + window)
    ///
    /// # Arguments
    /// * `window` - Duration of the sliding window
    ///
    /// # Returns
    /// Optional timestamp when the limit resets
    fn reset_time(&self, window: Duration) -> Option<Instant> {
        self.timestamps.first().map(|&first| first + window)
    }

    /// Get total number of requests seen (for analytics)
    fn total_requests(&self) -> u64 {
        self.total_requests
    }
}

/// Rate limiting middleware implementation
///
/// This middleware implements a sliding window rate limiter that can be
/// configured with various key extraction strategies and limits.
///
/// # Thread Safety
/// The middleware is fully thread-safe and can handle concurrent requests
/// efficiently using async RwLock for the internal request tracking.
pub struct RateLimitingMiddleware {
    /// Configuration for the rate limiter
    config: RateLimitConfig,
    /// Storage for request windows per key
    windows: Arc<RwLock<HashMap<String, RequestWindow>>>,
    /// Cleanup task handle (for proper shutdown)
    _cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl RateLimitingMiddleware {
    /// Create new rate limiting middleware with the given configuration
    ///
    /// # Arguments
    /// * `config` - Rate limiting configuration
    ///
    /// # Examples
    /// ```
    /// use ignitia::{RateLimitConfig, RateLimitingMiddleware};
    /// use std::time::Duration;
    ///
    /// let config = RateLimitConfig::new(100, Duration::from_secs(60))
    ///     .with_error_message("Too many requests");
    /// let middleware = RateLimitingMiddleware::new(config);
    /// ```
    pub fn new(config: RateLimitConfig) -> Self {
        let windows = Arc::new(RwLock::new(HashMap::new()));

        // Start background cleanup task
        let cleanup_handle = Self::start_cleanup_task(Arc::clone(&windows), config.window);

        Self {
            config,
            windows,
            _cleanup_handle: Some(cleanup_handle),
        }
    }

    /// Create rate limiting middleware with requests per minute limit
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per minute
    ///
    /// # Examples
    /// ```
    /// let middleware = RateLimitingMiddleware::per_minute(100);
    /// ```
    pub fn per_minute(max_requests: usize) -> Self {
        Self::new(RateLimitConfig::per_minute(max_requests))
    }

    /// Create rate limiting middleware with requests per second limit
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per second
    pub fn per_second(max_requests: usize) -> Self {
        Self::new(RateLimitConfig::per_second(max_requests))
    }

    /// Create rate limiting middleware with requests per hour limit
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per hour
    pub fn per_hour(max_requests: usize) -> Self {
        Self::new(RateLimitConfig::per_hour(max_requests))
    }

    /// Start background cleanup task to remove stale entries
    ///
    /// # Arguments
    /// * `windows` - Shared reference to the windows storage
    /// * `window_duration` - Duration to determine stale entries
    ///
    /// # Returns
    /// Handle to the cleanup task
    fn start_cleanup_task(
        windows: Arc<RwLock<HashMap<String, RequestWindow>>>,
        window_duration: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Clean every 5 minutes

            loop {
                interval.tick().await;

                let mut map = windows.write().await;
                let now = Instant::now();
                let cleanup_threshold = window_duration * 3; // Keep entries 3x window duration

                let initial_size = map.len();
                map.retain(|_, window| now.duration_since(window.last_cleanup) < cleanup_threshold);

                let removed = initial_size - map.len();
                if removed > 0 {
                    debug!("Cleaned up {} stale rate limit entries", removed);
                }
            }
        })
    }

    /// Extract the rate limiting key from a request
    ///
    /// Uses the configured key extractor if available, otherwise falls back
    /// to IP-based extraction.
    ///
    /// # Arguments
    /// * `req` - The incoming request
    ///
    /// # Returns
    /// String key uniquely identifying the client
    fn extract_key(&self, req: &Request) -> String {
        if let Some(ref extractor) = self.config.key_extractor {
            extractor(req)
        } else {
            self.extract_ip_key(req)
        }
    }

    /// Default key extractor based on client IP address
    ///
    /// Checks headers in order of preference:
    /// 1. `X-Forwarded-For` (proxy/load balancer)
    /// 2. `X-Real-IP` (nginx, cloudflare)
    /// 3. Falls back to "unknown" if no IP is available
    ///
    /// # Arguments
    /// * `req` - The incoming request
    ///
    /// # Returns
    /// String key based on IP address
    fn extract_ip_key(&self, req: &Request) -> String {
        // Check X-Forwarded-For header (most common for proxied requests)
        if let Some(forwarded) = req.header("x-forwarded-for") {
            if let Some(ip) = forwarded.split(',').next() {
                let clean_ip = ip.trim();
                if !clean_ip.is_empty() {
                    return format!("ip:{}", clean_ip);
                }
            }
        }

        // Check X-Real-IP header
        if let Some(real_ip) = req.header("x-real-ip") {
            let clean_ip = real_ip.trim();
            if !clean_ip.is_empty() {
                return format!("ip:{}", clean_ip);
            }
        }

        // Fallback - in production you'd want to get this from the connection
        "ip:unknown".to_string()
    }

    /// Get current statistics for a given key
    ///
    /// # Arguments
    /// * `key` - The rate limiting key
    ///
    /// # Returns
    /// Optional statistics for the key
    pub async fn get_stats(&self, key: &str) -> Option<RateLimitStats> {
        let windows = self.windows.read().await;
        windows.get(key).map(|window| RateLimitStats {
            current_requests: window.timestamps.len(),
            total_requests: window.total_requests(),
            remaining_requests: window.remaining_requests(self.config.max_requests),
            reset_time: window.reset_time(self.config.window),
        })
    }

    /// Clear all rate limit data (useful for testing)
    pub async fn clear_all(&self) {
        let mut windows = self.windows.write().await;
        windows.clear();
        debug!("Cleared all rate limit data");
    }
}

/// Statistics for a rate limit key
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Current number of requests in the window
    pub current_requests: usize,
    /// Total number of requests ever seen for this key
    pub total_requests: u64,
    /// Number of requests remaining in the current window
    pub remaining_requests: usize,
    /// When the rate limit will reset
    pub reset_time: Option<Instant>,
}

/// Information about the current rate limit state for a request
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Maximum requests allowed in the window
    pub limit: usize,
    /// Remaining requests in current window
    pub remaining: usize,
    /// Whether the rate limit was exceeded
    pub exceeded: bool,
    /// When the rate limit resets
    pub reset_time: Option<Instant>,
    /// Key used for rate limiting
    pub key: String,
}

#[async_trait::async_trait]
impl Middleware for RateLimitingMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        let key = self.extract_key(&req);

        debug!("Checking rate limit for key: {}", key);

        let mut windows = self.windows.write().await;
        let window = windows
            .entry(key.clone())
            .or_insert_with(RequestWindow::new);

        let is_allowed = window.is_allowed(
            self.config.max_requests,
            self.config.window,
            self.config.allow_burst,
            self.config.burst_multiplier,
        );

        let remaining = window.remaining_requests(self.config.max_requests);
        let reset_time = window.reset_time(self.config.window);

        if !is_allowed {
            warn!("Rate limit exceeded for key: {}", key);

            // Store rate limit info in request extensions for potential header use
            if self.config.include_headers {
                req.insert_extension(RateLimitInfo {
                    limit: self.config.max_requests,
                    remaining: 0,
                    exceeded: true,
                    reset_time,
                    key: key.clone(),
                });
            }

            return Response::json(RateLimitError {
                message: self.config.error_message.clone(),
                limit: self.config.max_requests,
                window: self.config.window,
                key,
                reset_time,
            })
            .with_status(StatusCode::TOO_MANY_REQUESTS);
        }

        // Store successful rate limit info
        if self.config.include_headers {
            req.insert_extension(RateLimitInfo {
                limit: self.config.max_requests,
                remaining,
                exceeded: false,
                reset_time,
                key: key.clone(),
            });
        }

        trace!("Rate limit check passed for key: {}", key);
        let mut res = next.run(req.clone()).await;

        // Only add headers if rate limiting info is available and headers are enabled
        if !self.config.include_headers {
            return res;
        }

        // Try to get rate limit info from request extensions
        if let Some(rate_limit_info) = req.get_extension::<RateLimitInfo>() {
            // Add X-RateLimit-Limit header
            if let Ok(limit_value) = rate_limit_info.limit.to_string().parse() {
                res.headers.insert("x-ratelimit-limit", limit_value);
            }

            // Add X-RateLimit-Remaining header
            if let Ok(remaining_value) = rate_limit_info.remaining.to_string().parse() {
                res.headers.insert("x-ratelimit-remaining", remaining_value);
            }

            // Add X-RateLimit-Reset header (Unix timestamp when limit resets)
            if let Some(reset_time) = rate_limit_info.reset_time {
                if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                {
                    let reset_duration = reset_time.duration_since(std::time::Instant::now());
                    let reset_timestamp = now.as_secs() + reset_duration.as_secs();

                    if let Ok(reset_value) = reset_timestamp.to_string().parse() {
                        res.headers.insert("x-ratelimit-reset", reset_value);
                    }
                }
            }

            // Add X-RateLimit-Reset-After header (seconds until reset)
            if let Some(reset_time) = rate_limit_info.reset_time {
                let now = std::time::Instant::now();
                if reset_time > now {
                    let seconds_until_reset = reset_time.duration_since(now).as_secs();
                    if let Ok(reset_after_value) = seconds_until_reset.to_string().parse() {
                        res.headers
                            .insert("x-ratelimit-reset-after", reset_after_value);
                    }
                }
            }

            // Add retry-after header for rate limited responses
            if rate_limit_info.exceeded && res.status == http::StatusCode::TOO_MANY_REQUESTS {
                let retry_after = self.config.window.as_secs();
                if let Ok(retry_value) = retry_after.to_string().parse() {
                    res.headers.insert("retry-after", retry_value);
                }
            }

            // Add custom headers with rate limit key info (for debugging in dev mode)
            #[cfg(debug_assertions)]
            {
                if let Ok(key_value) = rate_limit_info.key.parse() {
                    res.headers.insert("x-ratelimit-key", key_value);
                }
            }

            tracing::trace!(
                "Added rate limit headers: limit={}, remaining={}, exceeded={}",
                rate_limit_info.limit,
                rate_limit_info.remaining,
                rate_limit_info.exceeded
            );
        }

        res
    }
}

/// Custom error type for rate limit exceeded
#[derive(Debug, Serialize)]
pub struct RateLimitError {
    /// Custom error message
    pub message: String,
    /// Rate limit (requests per window)
    pub limit: usize,
    /// Time window duration
    pub window: Duration,
    /// Key that exceeded the limit
    pub key: String,
    /// When the limit resets
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_instant")]
    pub reset_time: Option<Instant>,
}

fn serialize_instant<S>(
    instant: &Option<Instant>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match instant {
        Some(i) => {
            let now = Instant::now();
            let remaining = i.saturating_duration_since(now);
            serializer.serialize_u64(remaining.as_secs())
        }
        None => serializer.serialize_none(),
    }
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (limit: {} requests per {:?}, key: {})",
            self.message, self.limit, self.window, self.key
        )
    }
}

// Drop implementation to clean up background task
impl Drop for RateLimitingMiddleware {
    fn drop(&mut self) {
        if let Some(handle) = self._cleanup_handle.take() {
            handle.abort();
        }
    }
}
