//! Server configuration module for the Ignitia web framework.
//!
//! This module provides comprehensive configuration options for HTTP/1.1, HTTP/2,
//! and TLS settings. It allows fine-tuning of server behavior including connection
//! limits, timeouts, protocol settings, and security configurations.
//!
//! # Examples
//!
//! ## Basic HTTP server configuration
//! ```
//! use ignitia::ServerConfig;
//!
//! let config = ServerConfig::default();
//! ```
//!
//! ## HTTP/2 with custom settings
//! ```
//! use ignitia::{ServerConfig, Http2Config};
//! use std::time::Duration;
//!
//! let http2_config = Http2Config {
//!     enabled: true,
//!     max_concurrent_streams: Some(2000),
//!     initial_connection_window_size: Some(2 * 1024 * 1024), // 2MB
//!     keep_alive_interval: Some(Duration::from_secs(30)),
//!     ..Default::default()
//! };
//!
//! let config = ServerConfig {
//!     http2: http2_config,
//!     ..Default::default()
//! };
//! ```
//!
//! ## HTTPS with TLS configuration
//! ```
//! # #[cfg(feature = "tls")]
//! use ignitia::{ServerConfig, TlsConfig};
//!
//! # #[cfg(feature = "tls")]
//! let tls_config = TlsConfig::new("cert.pem", "key.pem");
//! # #[cfg(feature = "tls")]
//! let config = ServerConfig::default().with_tls(tls_config);
//! ```

use std::time::Duration;

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use super::tls::TlsConfig;

/// HTTP/2 protocol configuration settings.
///
/// This struct provides fine-grained control over HTTP/2 behavior including
/// connection management, flow control, and performance optimizations.
///
/// # Protocol Specifications
///
/// These settings correspond to HTTP/2 parameters defined in RFC 7540.
/// Incorrect values may cause connection failures or performance degradation.
///
/// # Performance Considerations
///
/// - Higher `max_concurrent_streams` allows more parallelism but uses more memory
/// - Larger window sizes improve throughput for high-bandwidth connections
/// - Keep-alive settings help detect dead connections but add overhead
/// - Adaptive window sizing automatically adjusts flow control for optimal performance
///
/// # Examples
///
/// ## High-performance configuration
/// ```
/// use ignitia::Http2Config;
/// use std::time::Duration;
///
/// let config = Http2Config {
///     enabled: true,
///     max_concurrent_streams: Some(5000),           // High concurrency
///     initial_connection_window_size: Some(4 * 1024 * 1024), // 4MB
///     initial_stream_window_size: Some(256 * 1024), // 256KB
///     max_frame_size: Some(32 * 1024),              // 32KB frames
///     keep_alive_interval: Some(Duration::from_secs(30)),
///     adaptive_window: true,                         // Auto-tune flow control
///     ..Default::default()
/// };
/// ```
///
/// ## Conservative configuration
/// ```
/// use ignitia::Http2Config;
/// use std::time::Duration;
///
/// let config = Http2Config {
///     enabled: true,
///     max_concurrent_streams: Some(100),            // Lower concurrency
///     initial_connection_window_size: Some(512 * 1024), // 512KB
///     keep_alive_interval: Some(Duration::from_secs(120)),
///     adaptive_window: false,                        // Manual flow control
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Http2Config {
    /// Enable HTTP/2 support.
    ///
    /// When `true`, the server will negotiate HTTP/2 via ALPN (with TLS) or
    /// support HTTP/2 prior knowledge (without TLS).
    ///
    /// # Default
    /// `true`
    pub enabled: bool,

    /// Enable HTTP/2 prior knowledge (H2C - HTTP/2 Cleartext).
    ///
    /// When `true`, allows clients to send HTTP/2 requests directly without
    /// upgrade negotiation. Useful for client-server scenarios where both
    /// endpoints support HTTP/2.
    ///
    /// # Security Note
    /// H2C bypasses the normal HTTP/1.1 upgrade mechanism. Ensure clients
    /// are trusted when enabling this feature.
    ///
    /// # Default
    /// `false`
    ///
    /// # Examples
    /// ```
    /// // Enable for curl --http2-prior-knowledge support
    /// use ignitia::Http2Config;
    ///
    /// let config = Http2Config {
    ///     enable_prior_knowledge: true,
    ///     ..Default::default()
    /// };
    /// ```
    pub enable_prior_knowledge: bool,

    /// Maximum number of concurrent streams per connection.
    ///
    /// Controls how many parallel requests a single HTTP/2 connection can handle.
    /// Higher values allow more concurrency but consume more server resources.
    ///
    /// # Range
    /// - Minimum: 1
    /// - Maximum: 2^31 - 1 (per HTTP/2 spec)
    /// - Recommended: 100-5000 depending on server capacity
    ///
    /// # Default
    /// `Some(1000)`
    ///
    /// # Performance Impact
    /// - **Low values (1-100)**: Limits parallelism, may cause client bottlenecks
    /// - **Medium values (100-1000)**: Good balance for most applications
    /// - **High values (1000+)**: Maximum performance but higher memory usage
    pub max_concurrent_streams: Option<u32>,

    /// Initial connection-level flow control window size in bytes.
    ///
    /// Determines how much data can be sent before receiving a WINDOW_UPDATE frame.
    /// Affects overall connection throughput, especially for large responses.
    ///
    /// # Range
    /// - Minimum: 65,535 bytes (HTTP/2 spec minimum)
    /// - Maximum: 2^31 - 1 bytes
    /// - Recommended: 1MB - 4MB for high-bandwidth connections
    ///
    /// # Default
    /// `Some(1024 * 1024)` (1MB)
    ///
    /// # Tuning Guidelines
    /// ```
    /// use ignitia::Http2Config;
    ///
    /// // For high-bandwidth applications (video streaming, file uploads)
    /// let high_bandwidth = Http2Config {
    ///     initial_connection_window_size: Some(4 * 1024 * 1024), // 4MB
    ///     ..Default::default()
    /// };
    ///
    /// // For low-latency APIs (REST, JSON responses)
    /// let low_latency = Http2Config {
    ///     initial_connection_window_size: Some(256 * 1024), // 256KB
    ///     ..Default::default()
    /// };
    /// ```
    pub initial_connection_window_size: Option<u32>,

    /// Initial per-stream flow control window size in bytes.
    ///
    /// Controls flow control for individual HTTP/2 streams (requests/responses).
    /// Smaller than connection window size to allow fair sharing among streams.
    ///
    /// # Range
    /// - Minimum: 65,535 bytes (HTTP/2 spec minimum)
    /// - Maximum: 2^31 - 1 bytes
    /// - Recommended: 64KB - 256KB
    ///
    /// # Default
    /// `Some(64 * 1024)` (64KB)
    ///
    /// # Relationship to Connection Window
    /// Stream window should typically be smaller than connection window:
    /// ```
    /// connection_window >= streams * stream_window / concurrency_factor
    /// ```
    pub initial_stream_window_size: Option<u32>,

    /// Maximum HTTP/2 frame size in bytes.
    ///
    /// Larger frames reduce per-frame overhead but increase memory usage and
    /// can impact latency for multiplexed connections.
    ///
    /// # Range
    /// - Minimum: 16,384 bytes (HTTP/2 spec minimum)
    /// - Maximum: 16,777,215 bytes (2^24 - 1)
    /// - Recommended: 16KB - 64KB
    ///
    /// # Default
    /// `Some(16 * 1024)` (16KB)
    ///
    /// # Trade-offs
    /// - **Small frames (16KB)**: Lower latency, higher CPU overhead
    /// - **Large frames (64KB+)**: Higher throughput, more memory usage
    pub max_frame_size: Option<u32>,

    /// HTTP/2 connection keep-alive ping interval.
    ///
    /// Sends PING frames to detect dead connections and maintain NAT mappings.
    /// Helps identify network issues before they cause request timeouts.
    ///
    /// # Default
    /// `Some(Duration::from_secs(60))` (60 seconds)
    ///
    /// # Recommendations
    /// - **High-frequency clients**: 30-60 seconds
    /// - **Long-lived connections**: 60-120 seconds
    /// - **Mobile/unreliable networks**: 30 seconds
    /// - **Internal networks**: 120+ seconds
    ///
    /// # Examples
    /// ```
    /// use ignitia::Http2Config;
    /// use std::time::Duration;
    ///
    /// // Aggressive keep-alive for mobile clients
    /// let mobile_config = Http2Config {
    ///     keep_alive_interval: Some(Duration::from_secs(30)),
    ///     keep_alive_timeout: Some(Duration::from_secs(10)),
    ///     ..Default::default()
    /// };
    /// ```
    pub keep_alive_interval: Option<Duration>,

    /// HTTP/2 keep-alive timeout duration.
    ///
    /// How long to wait for a PING response before considering the connection dead.
    /// Should be shorter than `keep_alive_interval` to allow for retry cycles.
    ///
    /// # Default
    /// `Some(Duration::from_secs(20))` (20 seconds)
    ///
    /// # Guidelines
    /// - Set to 1/3 to 1/2 of `keep_alive_interval`
    /// - Account for network round-trip time
    /// - Consider client processing delays
    pub keep_alive_timeout: Option<Duration>,

    /// Enable adaptive window sizing for flow control.
    ///
    /// When `true`, automatically adjusts flow control windows based on
    /// connection performance. Improves throughput without manual tuning.
    ///
    /// # Default
    /// `true`
    ///
    /// # Benefits
    /// - Automatically optimizes for connection characteristics
    /// - Reduces need for manual window size tuning
    /// - Adapts to varying network conditions
    ///
    /// # When to Disable
    /// - Predictable, controlled network environments
    /// - Need deterministic flow control behavior
    /// - Debugging flow control issues
    pub adaptive_window: bool,

    /// Maximum HTTP/2 header list size in bytes.
    ///
    /// Limits the total size of request/response headers to prevent
    /// memory exhaustion attacks and excessive resource usage.
    ///
    /// # Range
    /// - Recommended: 8KB - 64KB
    /// - Consider your application's header requirements
    ///
    /// # Default
    /// `Some(16 * 1024)` (16KB)
    ///
    /// # Security Considerations
    /// Large header limits can enable DoS attacks. Balance application
    /// needs with security requirements.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Http2Config;
    ///
    /// // For applications with large cookies/JWT tokens
    /// let large_headers = Http2Config {
    ///     max_header_list_size: Some(32 * 1024), // 32KB
    ///     ..Default::default()
    /// };
    ///
    /// // Security-focused configuration
    /// let secure_config = Http2Config {
    ///     max_header_list_size: Some(8 * 1024), // 8KB
    ///     ..Default::default()
    /// };
    /// ```
    pub max_header_list_size: Option<u32>,
}

impl Default for Http2Config {
    /// Creates a default HTTP/2 configuration optimized for general web applications.
    ///
    /// These defaults provide a good balance between performance, resource usage,
    /// and compatibility. They work well for most web applications without tuning.
    ///
    /// # Default Values
    /// - `enabled`: `true` - HTTP/2 enabled
    /// - `enable_prior_knowledge`: `false` - Require negotiation
    /// - `max_concurrent_streams`: `1000` - Good parallelism without excessive memory
    /// - `initial_connection_window_size`: `1MB` - Suitable for typical web content
    /// - `initial_stream_window_size`: `64KB` - Balances fairness and performance
    /// - `max_frame_size`: `16KB` - HTTP/2 spec recommendation
    /// - `keep_alive_interval`: `60s` - Standard keep-alive frequency
    /// - `keep_alive_timeout`: `20s` - Reasonable timeout for ping responses
    /// - `adaptive_window`: `true` - Auto-optimize flow control
    /// - `max_header_list_size`: `16KB` - Accommodates most header scenarios
    ///
    /// # Examples
    /// ```
    /// use ignitia::Http2Config;
    ///
    /// let config = Http2Config::default();
    /// assert_eq!(config.enabled, true);
    /// assert_eq!(config.max_concurrent_streams, Some(1000));
    /// ```
    fn default() -> Self {
        Self {
            enabled: true,
            enable_prior_knowledge: false,
            max_concurrent_streams: Some(1000),
            initial_connection_window_size: Some(1024 * 1024), // 1MB
            initial_stream_window_size: Some(64 * 1024),       // 64KB
            max_frame_size: Some(16 * 1024),                   // 16KB
            keep_alive_interval: Some(Duration::from_secs(60)),
            keep_alive_timeout: Some(Duration::from_secs(20)),
            adaptive_window: true,
            max_header_list_size: Some(16 * 1024), // 16KB
        }
    }
}

/// Main server configuration containing all protocol and security settings.
///
/// `ServerConfig` is the primary configuration structure that combines HTTP/1.1,
/// HTTP/2, TLS, and general server settings. It supports both development and
/// production scenarios with flexible configuration options.
///
/// # Protocol Support
///
/// The server can operate in several modes:
/// - **HTTP/1.1 only**: Traditional HTTP with optional upgrade support
/// - **HTTP/2 only**: Modern binary protocol with multiplexing
/// - **Dual protocol**: Automatic negotiation between HTTP/1.1 and HTTP/2
/// - **HTTPS**: TLS encryption with ALPN protocol negotiation
///
/// # Configuration Patterns
///
/// ## Development Server
/// ```
/// use ignitia::ServerConfig;
///
/// let dev_config = ServerConfig {
///     http1_enabled: true,
///     auto_protocol_detection: true,
///     ..Default::default()
/// };
/// ```
///
/// ## Production HTTPS Server
/// ```
/// # #[cfg(feature = "tls")]
/// use ignitia::{ServerConfig, TlsConfig};
///
/// # #[cfg(feature = "tls")]
/// let tls_config = TlsConfig::new("cert.pem", "key.pem")
///     .with_alpn_protocols(vec!["h2", "http/1.1"]);
///
/// # #[cfg(feature = "tls")]
/// let prod_config = ServerConfig::default()
///     .with_tls(tls_config);
/// ```
///
/// ## HTTP to HTTPS Redirect
/// ```
/// use ignitia::ServerConfig;
///
/// let redirect_config = ServerConfig::default()
///     .redirect_to_https(443);
/// ```
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Enable HTTP/1.1 protocol support.
    ///
    /// When `true`, the server accepts HTTP/1.1 connections and can handle
    /// traditional HTTP requests. Can be combined with HTTP/2 for dual-protocol support.
    ///
    /// # Default
    /// `true`
    ///
    /// # Use Cases
    /// - **Legacy client support**: Some clients only support HTTP/1.1
    /// - **Debugging**: HTTP/1.1 is easier to debug with standard tools
    /// - **Gradual migration**: Maintain compatibility during HTTP/2 rollout
    ///
    /// # Performance Considerations
    /// HTTP/1.1 uses more connections for parallelism, which can exhaust
    /// server resources faster than HTTP/2's multiplexing.
    pub http1_enabled: bool,

    /// HTTP/2 configuration settings.
    ///
    /// Contains all HTTP/2-specific options including flow control, concurrency,
    /// and performance tuning parameters.
    ///
    /// # Examples
    /// ```
    /// use ignitia::{ServerConfig, Http2Config};
    /// use std::time::Duration;
    ///
    /// let custom_http2 = Http2Config {
    ///     max_concurrent_streams: Some(2000),
    ///     keep_alive_interval: Some(Duration::from_secs(30)),
    ///     ..Default::default()
    /// };
    ///
    /// let config = ServerConfig {
    ///     http2: custom_http2,
    ///     ..Default::default()
    /// };
    /// ```
    pub http2: Http2Config,

    /// Enable automatic protocol detection and negotiation.
    ///
    /// When `true`, the server automatically determines whether to use HTTP/1.1
    /// or HTTP/2 based on client capabilities and negotiation results.
    ///
    /// # Default
    /// `true`
    ///
    /// # Negotiation Methods
    /// - **ALPN (TLS)**: Client and server negotiate protocol during TLS handshake
    /// - **HTTP/2 Prior Knowledge**: Client sends HTTP/2 directly without upgrade
    /// - **HTTP/1.1 Upgrade**: Client requests upgrade via HTTP/1.1 headers
    ///
    /// # When to Disable
    /// - Force specific protocol version
    /// - Simplified deployment scenarios
    /// - Legacy system integration
    pub auto_protocol_detection: bool,

    /// TLS (Transport Layer Security) configuration.
    ///
    /// When provided, enables HTTPS support with customizable cipher suites,
    /// protocol versions, and certificate settings.
    ///
    /// # Security Best Practices
    /// - Use TLS 1.2 or higher for production
    /// - Configure strong cipher suites
    /// - Implement proper certificate management
    /// - Consider HSTS headers for enhanced security
    ///
    /// # Examples
    /// ```
    /// # #[cfg(feature = "tls")]
    /// use ignitia::{ServerConfig, TlsConfig, TlsVersion};
    ///
    /// # #[cfg(feature = "tls")]
    /// let tls_config = TlsConfig::new("server.crt", "server.key")
    ///     .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
    ///     .with_alpn_protocols(vec!["h2", "http/1.1"]);
    ///
    /// # #[cfg(feature = "tls")]
    /// let config = ServerConfig::default().with_tls(tls_config);
    /// ```
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub tls: Option<TlsConfig>,

    /// Automatically redirect HTTP requests to HTTPS.
    ///
    /// When `true`, all HTTP requests receive a 301 redirect to the equivalent
    /// HTTPS URL. Useful for enforcing encrypted connections.
    ///
    /// # Default
    /// `false`
    ///
    /// # Requirements
    /// - Must specify `https_port`
    /// - HTTPS server must be running on the specified port
    /// - Consider HSTS headers for additional security
    ///
    /// # Security Benefits
    /// - Prevents accidental plaintext communication
    /// - Helps meet compliance requirements
    /// - Protects against protocol downgrade attacks
    pub redirect_http_to_https: bool,

    /// HTTPS port number for redirect responses.
    ///
    /// Used when `redirect_http_to_https` is `true` to construct redirect URLs.
    /// Standard HTTPS port is 443, but custom ports are supported.
    ///
    /// # Default
    /// `None`
    ///
    /// # Examples
    /// ```
    /// use ignitia::ServerConfig;
    ///
    /// // Standard HTTPS port
    /// let config = ServerConfig::default().redirect_to_https(443);
    ///
    /// // Custom HTTPS port
    /// let dev_config = ServerConfig::default().redirect_to_https(8443);
    /// ```
    pub https_port: Option<u16>,
}

impl Default for ServerConfig {
    /// Creates a default server configuration suitable for development and testing.
    ///
    /// The default configuration enables both HTTP/1.1 and HTTP/2 with automatic
    /// protocol detection. TLS is disabled by default for easier development setup.
    ///
    /// # Default Values
    /// - `http1_enabled`: `true` - Support legacy clients
    /// - `http2`: `Http2Config::default()` - Optimized HTTP/2 settings
    /// - `auto_protocol_detection`: `true` - Automatic negotiation
    /// - `tls`: `None` - No TLS (HTTP only)
    /// - `redirect_http_to_https`: `false` - No automatic redirect
    /// - `https_port`: `None` - No HTTPS port specified
    ///
    /// # Production Considerations
    /// For production deployments, consider:
    /// - Enabling TLS with proper certificates
    /// - Configuring HTTP to HTTPS redirect
    /// - Tuning HTTP/2 settings for your workload
    /// - Implementing proper logging and monitoring
    ///
    /// # Examples
    /// ```
    /// use ignitia::ServerConfig;
    ///
    /// let config = ServerConfig::default();
    /// assert_eq!(config.http1_enabled, true);
    /// assert_eq!(config.auto_protocol_detection, true);
    /// # #[cfg(feature = "tls")]
    /// assert!(config.tls.is_none());
    /// ```
    fn default() -> Self {
        Self {
            http1_enabled: true,
            http2: Http2Config::default(),
            auto_protocol_detection: true,
            #[cfg(feature = "tls")]
            tls: None,
            redirect_http_to_https: false,
            https_port: None,
        }
    }
}

impl ServerConfig {
    /// Configure the server to use TLS encryption.
    ///
    /// Enables HTTPS support with the provided TLS configuration. The TLS config
    /// includes certificate paths, cipher suites, protocol versions, and ALPN settings.
    ///
    /// # Arguments
    /// * `tls_config` - Complete TLS configuration including certificates and security settings
    ///
    /// # Returns
    /// Modified `ServerConfig` with TLS enabled
    ///
    /// # Examples
    /// ```
    /// # #[cfg(feature = "tls")]
    /// use ignitia::{ServerConfig, TlsConfig};
    ///
    /// # #[cfg(feature = "tls")]
    /// let tls_config = TlsConfig::new("server.crt", "private.key")
    ///     .with_alpn_protocols(vec!["h2", "http/1.1"]);
    ///
    /// # #[cfg(feature = "tls")]
    /// let server_config = ServerConfig::default()
    ///     .with_tls(tls_config);
    ///
    /// # #[cfg(feature = "tls")]
    /// assert!(server_config.is_tls_enabled());
    /// ```
    ///
    /// # Security Considerations
    /// - Ensure certificate files are properly secured
    /// - Use strong cipher suites and recent TLS versions
    /// - Consider certificate rotation and renewal processes
    /// - Implement proper key management practices
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls = Some(tls_config);
        self
    }

    /// Enable automatic HTTP to HTTPS redirection.
    ///
    /// Configures the server to send 301 Moved Permanently responses for all HTTP
    /// requests, redirecting clients to the equivalent HTTPS URL.
    ///
    /// # Arguments
    /// * `https_port` - Port number where HTTPS server is listening
    ///
    /// # Returns
    /// Modified `ServerConfig` with redirect enabled
    ///
    /// # Redirect Behavior
    /// - Preserves the original path and query string
    /// - Uses 301 status code for permanent redirect
    /// - Handles both standard (443) and custom HTTPS ports
    ///
    /// # Examples
    /// ```
    /// use ignitia::ServerConfig;
    ///
    /// // Redirect to standard HTTPS port
    /// let config = ServerConfig::default().redirect_to_https(443);
    ///
    /// // Redirect to custom port for development
    /// let dev_config = ServerConfig::default().redirect_to_https(8443);
    ///
    /// assert_eq!(config.redirect_http_to_https, true);
    /// assert_eq!(config.https_port, Some(443));
    /// ```
    ///
    /// # Production Deployment
    /// ```
    /// # #[cfg(feature = "tls")]
    /// use ignitia::{ServerConfig, TlsConfig};
    ///
    /// # #[cfg(feature = "tls")]
    /// let production_config = ServerConfig::default()
    ///     .with_tls(TlsConfig::new("cert.pem", "key.pem"))
    ///     .redirect_to_https(443);
    /// ```
    ///
    /// # Security Benefits
    /// - Enforces encrypted communication
    /// - Prevents accidental plaintext data transmission
    /// - Improves compliance with security standards
    /// - Protects against man-in-the-middle attacks
    pub fn redirect_to_https(mut self, https_port: u16) -> Self {
        self.redirect_http_to_https = true;
        self.https_port = Some(https_port);
        self
    }

    /// Check if TLS/HTTPS is enabled in the current configuration.
    ///
    /// Returns `true` if a TLS configuration has been provided, indicating
    /// that the server will support HTTPS connections.
    ///
    /// # Returns
    /// `true` if TLS is configured, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// use ignitia::ServerConfig;
    ///
    /// let http_config = ServerConfig::default();
    /// assert_eq!(http_config.is_tls_enabled(), false);
    ///
    /// # #[cfg(feature = "tls")]
    /// {
    ///     use ignitia::TlsConfig;
    ///
    ///     let https_config = ServerConfig::default()
    ///         .with_tls(TlsConfig::new("cert.pem", "key.pem"));
    ///     assert_eq!(https_config.is_tls_enabled(), true);
    /// }
    /// ```
    ///
    /// # Use Cases
    /// - Conditional logic based on security configuration
    /// - Validation of server setup
    /// - Dynamic port binding decisions
    /// - Logging and monitoring setup
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub fn is_tls_enabled(&self) -> bool {
        self.tls.is_some()
    }

    /// Check if TLS/HTTPS is enabled (always returns false when TLS feature is disabled).
    ///
    /// This version is available when the `tls` feature is not enabled in your
    /// `Cargo.toml`. It always returns `false` since TLS support is not compiled in.
    ///
    /// # Returns
    /// Always `false` when TLS feature is disabled
    ///
    /// # Feature Flag Notice
    /// To enable TLS support, add the `tls` feature to your `Cargo.toml`:
    /// ```
    /// [dependencies]
    /// ignitia = { version = "*", features = ["tls"] }
    /// ```
    #[cfg(not(feature = "tls"))]
    pub fn is_tls_enabled(&self) -> bool {
        false
    }
}
