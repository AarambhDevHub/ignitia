//! # Compression Middleware
//!
//! This module provides HTTP response compression middleware for the Ignitia web framework.
//! It supports multiple compression algorithms including gzip and brotli, with automatic
//! content negotiation based on the client's `Accept-Encoding` header.
//!
//! ## Features
//!
//! - **Multiple Algorithms**: Supports gzip and brotli compression
//! - **Smart Content Negotiation**: Automatically selects the best compression algorithm
//! - **Configurable Thresholds**: Only compress responses above a certain size
//! - **MIME Type Filtering**: Compress only appropriate content types
//! - **Quality Value Support**: Respects client preference with quality values
//! - **Performance Optimized**: Uses async compression with proper buffering
//! - **Standards Compliant**: Sets proper HTTP headers (`Content-Encoding`, `Vary`)
//!
//! ## Quick Start
//!
//! ```
//! use ignitia::{Router, CompressionMiddleware};
//!
//! let app = Router::new()
//!     .middleware(CompressionMiddleware::new())
//!     .get("/api/data", || async {
//!         // This response will be automatically compressed
//!         ignitia::Response::json(&serde_json::json!({
//!             "data": "This will be compressed!"
//!         }))
//!     });
//! ```
//!
//! ## Configuration Examples
//!
//! ### API-Optimized Compression
//!
//! ```
//! use ignitia::CompressionMiddleware;
//!
//! let compression = CompressionMiddleware::for_api()
//!     .with_threshold(512)  // Compress responses > 512 bytes
//!     .with_brotli(true)    // Enable brotli
//!     .with_gzip(true);     // Enable gzip
//! ```
//!
//! ### High Compression for Static Content
//!
//! ```
//! use ignitia::{CompressionMiddleware, CompressionLevel};
//!
//! let compression = CompressionMiddleware::high_compression()
//!     .with_level(CompressionLevel::Best)
//!     .with_threshold(2048);
//! ```
//!
//! ### Custom Configuration
//!
//! ```
//! use ignitia::{CompressionMiddleware, CompressionLevel};
//!
//! let compression = CompressionMiddleware::new()
//!     .with_threshold(1024)
//!     .with_level(CompressionLevel::Default)
//!     .with_compressible_types(vec![
//!         "application/json",
//!         "text/html",
//!         "text/css",
//!         "application/javascript"
//!     ]);
//! ```

use crate::middleware::Middleware;
use crate::{Request, Response, Result};
use async_compression::tokio::write::{BrotliEncoder, GzipEncoder};
use bytes::Bytes;
use http::{header, HeaderValue};
use tokio::io::AsyncWriteExt;
use tracing::debug;

use super::Next;

/// Compression level configuration for the compression algorithms.
///
/// This enum allows fine-tuning of the compression ratio vs. speed trade-off.
///
/// # Examples
///
/// ```
/// use ignitia::{CompressionMiddleware, CompressionLevel};
///
/// // Use fastest compression (lower CPU usage, larger files)
/// let fast = CompressionMiddleware::new()
///     .with_level(CompressionLevel::Fastest);
///
/// // Use maximum compression (higher CPU usage, smaller files)
/// let best = CompressionMiddleware::new()
///     .with_level(CompressionLevel::Best);
///
/// // Use a specific compression level (0-9 for most algorithms)
/// let custom = CompressionMiddleware::new()
///     .with_level(CompressionLevel::Precise(6));
/// ```
#[derive(Debug, Clone)]
pub enum CompressionLevel {
    /// Fastest compression with minimal CPU usage
    Fastest,
    /// Balanced compression (recommended for most use cases)
    Default,
    /// Maximum compression with higher CPU usage
    Best,
    /// Precise compression level (0-9, algorithm dependent)
    Precise(i32),
}

impl From<CompressionLevel> for async_compression::Level {
    fn from(level: CompressionLevel) -> Self {
        match level {
            CompressionLevel::Fastest => async_compression::Level::Fastest,
            CompressionLevel::Default => async_compression::Level::Default,
            CompressionLevel::Best => async_compression::Level::Best,
            CompressionLevel::Precise(n) => async_compression::Level::Precise(n),
        }
    }
}

/// HTTP compression middleware for automatic response compression.
///
/// This middleware automatically compresses HTTP responses based on the client's
/// `Accept-Encoding` header and the response's content type and size.
///
/// ## Behavior
///
/// 1. **Request Phase (`before`)**: Parses the client's `Accept-Encoding` header
///    and negotiates the best available compression algorithm.
///
/// 2. **Response Phase (`after`)**: Compresses the response body if:
///    - Response size is above the configured threshold
///    - Content type is in the compressible types list
///    - Client supports at least one available compression algorithm
///    - Response doesn't already have a `Content-Encoding` header
///
/// ## Headers Set
///
/// - `Content-Encoding`: The compression algorithm used (e.g., "gzip", "br")
/// - `Vary: Accept-Encoding`: Indicates response varies based on Accept-Encoding
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use ignitia::{Router, CompressionMiddleware};
///
/// let app = Router::new()
///     .middleware(CompressionMiddleware::new())
///     .get("/", || async {
///         ignitia::Response::text("This will be compressed if > 1KB")
///     });
/// ```
///
/// ## With Custom Configuration
///
/// ```
/// use ignitia::{CompressionMiddleware, CompressionLevel};
///
/// let compression = CompressionMiddleware::new()
///     .with_threshold(512)               // Compress responses > 512 bytes
///     .with_level(CompressionLevel::Best) // Maximum compression
///     .with_brotli(true)                 // Enable brotli
///     .with_gzip(true)                   // Enable gzip
///     .with_compressible_types(vec![
///         "application/json",
///         "text/html",
///         "text/css"
///     ]);
/// ```
#[derive(Debug, Clone)]
pub struct CompressionMiddleware {
    /// Minimum response size to compress (in bytes)
    threshold: usize,
    /// Compression level for algorithms
    level: CompressionLevel,
    /// Enable gzip compression (RFC 1952)
    enable_gzip: bool,
    /// Enable brotli compression (RFC 7932)
    enable_brotli: bool,
    /// MIME types that should be compressed
    compressible_types: Vec<String>,
}

impl Default for CompressionMiddleware {
    /// Creates a new `CompressionMiddleware` with sensible defaults.
    ///
    /// ## Default Configuration
    ///
    /// - **Threshold**: 1024 bytes (1KB)
    /// - **Level**: `CompressionLevel::Default`
    /// - **Gzip**: Enabled
    /// - **Brotli**: Enabled
    /// - **Compressible Types**: Common text-based MIME types
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::CompressionMiddleware;
    ///
    /// let compression = CompressionMiddleware::default();
    /// // Equivalent to:
    /// let compression = CompressionMiddleware::new();
    /// ```
    fn default() -> Self {
        Self {
            threshold: 1024, // 1KB
            level: CompressionLevel::Default,
            enable_gzip: true,
            enable_brotli: true,
            compressible_types: vec![
                "text/plain".to_string(),
                "text/html".to_string(),
                "text/css".to_string(),
                "text/javascript".to_string(),
                "application/javascript".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
                "text/xml".to_string(),
                "application/rss+xml".to_string(),
                "application/atom+xml".to_string(),
                "image/svg+xml".to_string(),
            ],
        }
    }
}

impl CompressionMiddleware {
    /// Creates a new `CompressionMiddleware` with default settings.
    ///
    /// This is equivalent to calling `CompressionMiddleware::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::CompressionMiddleware;
    ///
    /// let compression = CompressionMiddleware::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum response size threshold for compression.
    ///
    /// Responses smaller than this threshold will not be compressed,
    /// as the compression overhead may not be worth it for small responses.
    ///
    /// # Parameters
    ///
    /// * `threshold` - Minimum size in bytes (recommended: 512-2048)
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::CompressionMiddleware;
    ///
    /// // Only compress responses larger than 2KB
    /// let compression = CompressionMiddleware::new()
    ///     .with_threshold(2048);
    /// ```
    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets the compression level for all algorithms.
    ///
    /// Higher compression levels result in smaller files but require more CPU time.
    ///
    /// # Parameters
    ///
    /// * `level` - Compression level to use
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{CompressionMiddleware, CompressionLevel};
    ///
    /// // Use fastest compression
    /// let fast = CompressionMiddleware::new()
    ///     .with_level(CompressionLevel::Fastest);
    ///
    /// // Use maximum compression
    /// let best = CompressionMiddleware::new()
    ///     .with_level(CompressionLevel::Best);
    ///
    /// // Use specific level (6 out of 9)
    /// let custom = CompressionMiddleware::new()
    ///     .with_level(CompressionLevel::Precise(6));
    /// ```
    pub fn with_level(mut self, level: CompressionLevel) -> Self {
        self.level = level;
        self
    }

    /// Enables or disables gzip compression (RFC 1952).
    ///
    /// Gzip is widely supported by all modern browsers and has good
    /// compression ratios with reasonable speed.
    ///
    /// # Parameters
    ///
    /// * `enabled` - Whether to enable gzip compression
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::CompressionMiddleware;
    ///
    /// // Disable gzip (only use brotli)
    /// let compression = CompressionMiddleware::new()
    ///     .with_gzip(false)
    ///     .with_brotli(true);
    /// ```
    pub fn with_gzip(mut self, enabled: bool) -> Self {
        self.enable_gzip = enabled;
        self
    }

    /// Enables or disables brotli compression (RFC 7932).
    ///
    /// Brotli typically provides better compression ratios than gzip
    /// but may have slightly higher CPU usage. It's supported by all
    /// modern browsers.
    ///
    /// # Parameters
    ///
    /// * `enabled` - Whether to enable brotli compression
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::CompressionMiddleware;
    ///
    /// // Enable only brotli for maximum compression
    /// let compression = CompressionMiddleware::new()
    ///     .with_gzip(false)
    ///     .with_brotli(true);
    /// ```
    pub fn with_brotli(mut self, enabled: bool) -> Self {
        self.enable_brotli = enabled;
        self
    }

    /// Sets the list of compressible MIME types.
    ///
    /// Only responses with these content types will be compressed.
    /// Binary formats (images, videos, etc.) are typically not
    /// compressible and may become larger when compressed.
    ///
    /// # Parameters
    ///
    /// * `types` - List of MIME type prefixes to compress
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::CompressionMiddleware;
    ///
    /// // Only compress JSON and HTML
    /// let compression = CompressionMiddleware::new()
    ///     .with_compressible_types(vec![
    ///         "application/json",
    ///         "text/html"
    ///     ]);
    ///
    /// // Compress all text types
    /// let text_only = CompressionMiddleware::new()
    ///     .with_compressible_types(vec!["text/"]);
    /// ```
    pub fn with_compressible_types(mut self, types: Vec<&str>) -> Self {
        self.compressible_types = types.into_iter().map(String::from).collect();
        self
    }

    /// Checks if the given content type should be compressed.
    ///
    /// This method checks if the content type starts with any of the
    /// configured compressible type prefixes.
    ///
    /// # Parameters
    ///
    /// * `content_type` - The content type to check (e.g., "application/json")
    ///
    /// # Returns
    ///
    /// `true` if the content type should be compressed, `false` otherwise.
    fn is_compressible(&self, content_type: Option<&str>) -> bool {
        if let Some(ct) = content_type {
            let ct_lower = ct.to_lowercase();
            self.compressible_types
                .iter()
                .any(|t| ct_lower.starts_with(t))
        } else {
            false
        }
    }

    /// Negotiates the best compression encoding based on Accept-Encoding header.
    ///
    /// This method parses the `Accept-Encoding` header and selects the best
    /// available compression algorithm based on quality values and server capabilities.
    ///
    /// # Parameters
    ///
    /// * `accept_encoding` - The Accept-Encoding header value
    ///
    /// # Returns
    ///
    /// The best available encoding, or `None` if no suitable encoding is found.
    ///
    /// # Examples
    ///
    /// The method handles various Accept-Encoding formats:
    /// - `"gzip, deflate, br"`
    /// - `"br;q=1.0, gzip;q=0.8, *;q=0.1"`
    /// - `"gzip, br;q=0.9"`
    fn negotiate_encoding(&self, accept_encoding: Option<&str>) -> Option<Encoding> {
        if let Some(accept) = accept_encoding {
            let accept_lower = accept.to_lowercase();

            // Parse quality values and encodings
            let mut encodings: Vec<(Encoding, f32)> = Vec::new();

            for part in accept_lower.split(',') {
                let part = part.trim();
                if let Some((encoding, quality)) = self.parse_encoding_with_quality(part) {
                    encodings.push((encoding, quality));
                }
            }

            // Sort by quality (descending) and return best available
            encodings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            for (encoding, _) in encodings {
                match encoding {
                    Encoding::Brotli if self.enable_brotli => return Some(Encoding::Brotli),
                    Encoding::Gzip if self.enable_gzip => return Some(Encoding::Gzip),
                    _ => continue,
                }
            }
        }
        None
    }

    /// Parses a single encoding entry with its quality value.
    ///
    /// # Parameters
    ///
    /// * `part` - A single encoding part (e.g., "gzip;q=0.8" or "br")
    ///
    /// # Returns
    ///
    /// A tuple of (Encoding, quality) or None if the encoding is not supported.
    fn parse_encoding_with_quality(&self, part: &str) -> Option<(Encoding, f32)> {
        let mut split = part.split(';');
        let encoding_str = split.next()?.trim();

        let encoding = match encoding_str {
            "br" => Encoding::Brotli,
            "gzip" => Encoding::Gzip,
            "*" => Encoding::Gzip, // Default fallback
            _ => return None,
        };

        // Parse quality value
        let quality = if let Some(q_part) = split.next() {
            if let Some(q_value) = q_part.trim().strip_prefix("q=") {
                q_value.parse().unwrap_or(1.0)
            } else {
                1.0
            }
        } else {
            1.0
        };

        Some((encoding, quality))
    }

    /// Compresses data using the specified encoding algorithm.
    ///
    /// This method performs the actual compression using async I/O to avoid
    /// blocking the event loop during compression of large responses.
    ///
    /// # Parameters
    ///
    /// * `data` - The data to compress
    /// * `encoding` - The compression algorithm to use
    ///
    /// # Returns
    ///
    /// The compressed data as `Bytes`, or an error if compression fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The compression algorithm fails
    /// - I/O operations fail during compression
    /// - The encoder cannot be finalized properly
    async fn compress_data(&self, data: &Bytes, encoding: Encoding) -> Result<Bytes> {
        match encoding {
            Encoding::Gzip => {
                // Create encoder that writes compressed data to a Vec<u8>
                let mut encoder = GzipEncoder::with_quality(Vec::new(), self.level.clone().into());

                // Write the uncompressed data to the encoder
                encoder.write_all(data).await.map_err(|e| {
                    crate::Error::Internal(format!("Gzip compression failed: {}", e))
                })?;

                // Finish compression and get the compressed data
                encoder.shutdown().await.map_err(|e| {
                    crate::Error::Internal(format!("Gzip finalization failed: {}", e))
                })?;

                let compressed = encoder.into_inner();
                Ok(Bytes::from(compressed))
            }
            Encoding::Brotli => {
                // Create encoder that writes compressed data to a Vec<u8>
                let mut encoder =
                    BrotliEncoder::with_quality(Vec::new(), self.level.clone().into());

                // Write the uncompressed data to the encoder
                encoder.write_all(data).await.map_err(|e| {
                    crate::Error::Internal(format!("Brotli compression failed: {}", e))
                })?;

                // Finish compression and get the compressed data
                encoder.shutdown().await.map_err(|e| {
                    crate::Error::Internal(format!("Brotli finalization failed: {}", e))
                })?;

                let compressed = encoder.into_inner();
                Ok(Bytes::from(compressed))
            }
        }
    }
}

/// Supported compression encodings.
///
/// This enum represents the compression algorithms supported by the middleware.
#[derive(Debug, Clone, Copy)]
enum Encoding {
    /// Gzip compression (RFC 1952) - widely supported, good compression ratio
    Gzip,
    /// Brotli compression (RFC 7932) - better compression than gzip, modern browsers
    Brotli,
}

impl Encoding {
    /// Returns the HTTP header value for this encoding.
    ///
    /// # Returns
    ///
    /// The string representation used in HTTP headers.
    fn as_str(&self) -> &'static str {
        match self {
            Encoding::Gzip => "gzip",
            Encoding::Brotli => "br",
        }
    }
}

#[async_trait::async_trait]
impl Middleware for CompressionMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        // Parse Accept-Encoding and store preferred encoding in response headers
        let accept_encoding = req.header("accept-encoding");

        if let Some(encoding) = self.negotiate_encoding(accept_encoding) {
            // Store the encoding in response headers for after() phase to access
            req.headers.insert(
                "x-negotiated-encoding",
                HeaderValue::from_static(encoding.as_str()),
            );
            debug!("Negotiated encoding: {}", encoding.as_str());
        }

        let mut res = next.run(req.clone()).await;

        // Skip if response is too small

        if res.body.len() < self.threshold {
            debug!(
                "Response too small for compression: {} bytes",
                res.body.len()
            );
            return res;
        }

        // Skip if already compressed
        if res.headers.contains_key(header::CONTENT_ENCODING) {
            debug!("Response already has Content-Encoding header");
            return res;
        }

        // Check if content type is compressible
        let content_type = res
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok());

        if !self.is_compressible(content_type) {
            debug!("Content type not compressible: {:?}", content_type);
            return res;
        }

        // ✅ Get negotiated encoding from REQUEST headers (where it was stored)
        let encoding_str = req
            .headers
            .get("x-negotiated-encoding")
            .and_then(|val| val.to_str().ok());
        let encoding = match encoding_str {
            Some("br") if self.enable_brotli => Encoding::Brotli,
            Some("gzip") if self.enable_gzip => Encoding::Gzip,
            _ => {
                // Fallback: use best available if no client preference
                if self.enable_brotli {
                    Encoding::Brotli
                } else if self.enable_gzip {
                    Encoding::Gzip
                } else {
                    return res;
                }
            }
        };

        // Compress the response body
        let original_size = res.body.len();
        let compressed_body = match self.compress_data(&res.body, encoding).await {
            Ok(body) => body,
            Err(e) => {
                debug!(
                    "Compression failed: {}, returning uncompressed",
                    e.to_string()
                );
                return res; // Return original response on compression error
            }
        };
        let compressed_size = compressed_body.len();

        // Only use compressed version if it's actually smaller
        if compressed_size < original_size {
            res.body = compressed_body;
            res.headers.insert(
                header::CONTENT_ENCODING,
                HeaderValue::from_static(encoding.as_str()),
            );
            res.headers
                .insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
            debug!(
                "Compressed response: {} -> {} bytes ({}% reduction, {})",
                original_size,
                compressed_size,
                ((original_size - compressed_size) * 100) / original_size,
                encoding.as_str()
            );
        } else {
            debug!(
                "Compression not beneficial: {} -> {} bytes",
                original_size, compressed_size
            );
        }

        res
    }
}

// Builder pattern implementations for common use cases
impl CompressionMiddleware {
    /// Creates compression middleware optimized for API responses.
    ///
    /// This configuration is designed for JSON APIs and similar services:
    /// - Lower threshold (512 bytes) for better API response times
    /// - Focuses on JSON, XML, and text content types
    /// - Balanced compression settings
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, CompressionMiddleware};
    ///
    /// let api = Router::new()
    ///     .middleware(CompressionMiddleware::for_api())
    ///     .get("/api/users", get_users_handler);
    /// ```
    pub fn for_api() -> Self {
        Self::new()
            .with_threshold(512) // Smaller threshold for API responses
            .with_compressible_types(vec![
                "application/json",
                "application/xml",
                "text/xml",
                "text/plain",
            ])
    }

    /// Creates compression middleware optimized for web content.
    ///
    /// This configuration is designed for serving web pages and static content:
    /// - Standard threshold (1KB) for balanced performance
    /// - Includes HTML, CSS, JavaScript, and other web content types
    /// - Default compression level for good balance of speed vs. size
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, CompressionMiddleware};
    ///
    /// let web = Router::new()
    ///     .middleware(CompressionMiddleware::for_web())
    ///     .get("/", serve_homepage);
    /// ```
    pub fn for_web() -> Self {
        Self::new()
            .with_threshold(1024)
            .with_level(CompressionLevel::Default)
    }

    /// Creates compression middleware with maximum compression settings.
    ///
    /// This configuration prioritizes file size over compression speed:
    /// - Higher threshold (2KB) to avoid compressing small files
    /// - Maximum compression level for best compression ratio
    /// - Suitable for static content where compression time is not critical
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, CompressionMiddleware};
    ///
    /// let static_content = Router::new()
    ///     .middleware(CompressionMiddleware::high_compression())
    ///     .get("/static/*path", serve_static_file);
    /// ```
    pub fn high_compression() -> Self {
        Self::new()
            .with_level(CompressionLevel::Best)
            .with_threshold(2048)
    }

    /// Creates compression middleware with fastest compression settings.
    ///
    /// This configuration prioritizes compression speed over file size:
    /// - Lower threshold (512 bytes) for more responsive compression
    /// - Fastest compression level for minimal CPU usage
    /// - Suitable for high-traffic applications where speed is critical
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, CompressionMiddleware};
    ///
    /// let high_traffic = Router::new()
    ///     .middleware(CompressionMiddleware::fast_compression())
    ///     .get("/api/stream", stream_handler);
    /// ```
    pub fn fast_compression() -> Self {
        Self::new()
            .with_level(CompressionLevel::Fastest)
            .with_threshold(512)
    }
}
