//! Body Size Limit Middleware
//!
//! This middleware provides configurable request body size limits to protect your server
//! from oversized requests. It returns a 413 Payload Too Large status when limits are exceeded.
//!
//! # Examples
//!
//! ```
//! use ignitia::{Router, BodySizeLimitMiddleware};
//!
//! let app = Router::new()
//!     // Global 5MB limit
//!     .middleware(BodySizeLimitMiddleware::megabytes(5))
//!     .post("/upload", upload_handler);
//!
//! // Route-specific limits
//! let app = Router::new()
//!     .post("/avatar", LayeredHandler::new(avatar_handler)
//!         .layer(BodySizeLimitMiddleware::kilobytes(500))) // 500KB for avatars
//!     .post("/document", LayeredHandler::new(doc_handler)
//!         .layer(BodySizeLimitMiddleware::megabytes(10))); // 10MB for documents
//! ```

use crate::middleware::Middleware;
use crate::{Error, Request, Result};
use http::StatusCode;
use tracing::warn;

/// Body Size Limit Middleware
///
/// This middleware enforces maximum request body size limits and rejects requests
/// that exceed the configured limit with a 413 Payload Too Large status code.
///
/// The middleware is designed to be:
/// - **Flexible**: Configure limits in bytes, KB, MB, or GB
/// - **Route-specific**: Apply different limits to different routes
/// - **User-friendly**: Provides human-readable error messages
/// - **Metadata-rich**: Returns detailed error information including actual vs. allowed sizes
/// - **Fast**: Checks body size before processing to minimize resource usage
#[derive(Debug, Clone)]
pub struct BodySizeLimitMiddleware {
    /// Maximum allowed body size in bytes
    max_size: usize,
    /// Custom error message (optional)
    custom_message: Option<String>,
    /// Whether to log rejected requests
    log_rejections: bool,
    /// Include detailed size information in error response
    include_size_info: bool,
}

impl BodySizeLimitMiddleware {
    /// Create a new body size limit middleware with the specified maximum size
    ///
    /// # Arguments
    /// * `max_size` - Maximum allowed body size in bytes
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::BodySizeLimitMiddleware;
    ///
    /// // Limit to 1MB (1,048,576 bytes)
    /// let middleware = BodySizeLimitMiddleware::new(1024 * 1024);
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            custom_message: None,
            log_rejections: true,
            include_size_info: true,
        }
    }

    /// Create a body size limit with a custom error message
    ///
    /// # Arguments
    /// * `max_size` - Maximum allowed body size in bytes
    /// * `message` - Custom error message to display when limit is exceeded
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::with_message(
    ///     1024 * 1024,
    ///     "Profile picture must be smaller than 1MB"
    /// );
    /// ```
    pub fn with_message(max_size: usize, message: impl Into<String>) -> Self {
        Self {
            max_size,
            custom_message: Some(message.into()),
            log_rejections: true,
            include_size_info: true,
        }
    }

    /// Custom error message for body size limit
    ///
    /// # Arguments
    /// * `message` - Custom error message to display when limit is exceeded
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::message(
    ///     "Profile picture must be smaller than 1MB"
    /// );
    /// ```
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.custom_message = Some(message.into());
        self
    }

    /// Convenience method to create megabyte-based limits
    ///
    /// # Arguments
    /// * `mb` - Size limit in megabytes
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::megabytes(5); // 5MB limit
    /// ```
    pub fn megabytes(mb: usize) -> Self {
        Self::new(mb * 1024 * 1024)
    }

    /// Convenience method to create kilobyte-based limits
    ///
    /// # Arguments
    /// * `kb` - Size limit in kilobytes
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::kilobytes(512); // 512KB limit
    /// ```
    pub fn kilobytes(kb: usize) -> Self {
        Self::new(kb * 1024)
    }

    /// Convenience method to create gigabyte-based limits (use with caution!)
    ///
    /// # Arguments
    /// * `gb` - Size limit in gigabytes
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::gigabytes(1); // 1GB limit
    /// ```
    ///
    /// # Warning
    /// Large limits can consume significant server resources. Use with caution!
    pub fn gigabytes(gb: usize) -> Self {
        Self::new(gb * 1024 * 1024 * 1024)
    }

    /// Enable or disable logging of rejected requests
    ///
    /// # Arguments
    /// * `enabled` - Whether to log rejected requests
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::megabytes(5)
    ///     .with_logging(false); // Disable logging
    /// ```
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_rejections = enabled;
        self
    }

    /// Enable or disable detailed size information in error responses
    ///
    /// # Arguments
    /// * `enabled` - Whether to include size information in errors
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::megabytes(5)
    ///     .with_size_info(false); // Hide size details from users
    /// ```
    pub fn with_size_info(mut self, enabled: bool) -> Self {
        self.include_size_info = enabled;
        self
    }

    /// Get the current maximum size limit in bytes
    ///
    /// # Returns
    /// The maximum allowed body size in bytes
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::megabytes(5);
    /// assert_eq!(middleware.max_size(), 5 * 1024 * 1024);
    /// ```
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Format size in human-readable format
    ///
    /// Converts byte sizes to appropriate units (bytes, KB, MB, GB) for display.
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    ///
    /// # Returns
    /// Human-readable size string
    ///
    /// # Examples
    /// ```
    /// assert_eq!(BodySizeLimitMiddleware::format_size(1024), "1.0 KB");
    /// assert_eq!(BodySizeLimitMiddleware::format_size(1536), "1.5 KB");
    /// assert_eq!(BodySizeLimitMiddleware::format_size(1048576), "1.0 MB");
    /// ```
    pub fn format_size(size: usize) -> String {
        const UNITS: &[&str] = &["bytes", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as usize, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}
/// Create predefined middleware configurations for common use cases
impl BodySizeLimitMiddleware {
    /// Standard configuration for JSON APIs (1MB limit)
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::json_api();
    /// ```
    pub fn json_api() -> Self {
        Self::megabytes(1).message("Request body too large for JSON API (max 1MB)")
    }

    /// Standard configuration for file uploads (10MB limit)
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::file_upload();
    /// ```
    pub fn file_upload() -> Self {
        Self::megabytes(10).message("File too large for upload (max 10MB)")
    }

    /// Standard configuration for avatar/profile images (500KB limit)
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::avatar_upload();
    /// ```
    pub fn avatar_upload() -> Self {
        Self::kilobytes(500).message("Avatar image too large (max 500KB)")
    }

    /// Strict configuration for form data (64KB limit)
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::form_data();
    /// ```
    pub fn form_data() -> Self {
        Self::kilobytes(64).message("Form data too large (max 64KB)")
    }
}

#[async_trait::async_trait]
impl Middleware for BodySizeLimitMiddleware {
    /// Check request body size before processing
    ///
    /// This method is called before the request handler and will reject requests
    /// that exceed the configured size limit.
    async fn before(&self, req: &mut Request) -> Result<()> {
        let body_size = req.body.len();

        // Fast path: if body size is within limit, continue
        if body_size <= self.max_size {
            return Ok(());
        }

        // Log the rejection if enabled
        if self.log_rejections {
            warn!(
                "Request body size limit exceeded: {} > {} ({})",
                Self::format_size(body_size),
                Self::format_size(self.max_size),
                req.uri.path()
            );
        }

        // Create appropriate error message
        let message = if let Some(custom_msg) = &self.custom_message {
            custom_msg.clone()
        } else if self.include_size_info {
            format!(
                "Request body size ({}) exceeds maximum allowed size ({})",
                Self::format_size(body_size),
                Self::format_size(self.max_size)
            )
        } else {
            "Request body too large".to_string()
        };

        // Create a custom error that will return 413 Payload Too Large
        Err(Error::Custom(Box::new(PayloadTooLargeError {
            message,
            current_size: body_size,
            max_size: self.max_size,
            include_metadata: self.include_size_info,
        })))
    }
}

/// Custom error for payload too large scenarios
///
/// This error type implements the CustomError trait to provide detailed
/// error information including metadata about the size limits.
#[derive(Debug)]
struct PayloadTooLargeError {
    message: String,
    current_size: usize,
    max_size: usize,
    include_metadata: bool,
}

impl std::fmt::Display for PayloadTooLargeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl crate::error::CustomError for PayloadTooLargeError {
    fn status_code(&self) -> StatusCode {
        StatusCode::PAYLOAD_TOO_LARGE // 413
    }

    fn error_type(&self) -> &'static str {
        "payload_too_large"
    }

    fn error_code(&self) -> Option<String> {
        Some("BODY_SIZE_LIMIT_EXCEEDED".to_string())
    }

    fn metadata(&self) -> Option<serde_json::Value> {
        if !self.include_metadata {
            return None;
        }

        Some(serde_json::json!({
            "current_size_bytes": self.current_size,
            "max_size_bytes": self.max_size,
            "current_size_formatted": BodySizeLimitMiddleware::format_size(self.current_size),
            "max_size_formatted": BodySizeLimitMiddleware::format_size(self.max_size),
            "exceeded_by_bytes": self.current_size.saturating_sub(self.max_size),
            "exceeded_by_formatted": BodySizeLimitMiddleware::format_size(
                self.current_size.saturating_sub(self.max_size)
            )
        }))
    }
}

/// Builder pattern implementation for advanced configuration
impl BodySizeLimitMiddleware {
    /// Create a new builder for advanced configuration
    ///
    /// # Examples
    /// ```
    /// let middleware = BodySizeLimitMiddleware::builder()
    ///     .max_size_mb(5)
    ///     .message("Custom error message")
    ///     .disable_logging()
    ///     .hide_size_info()
    ///     .build();
    /// ```
    pub fn builder() -> BodySizeLimitBuilder {
        BodySizeLimitBuilder::new()
    }
}

/// Builder for creating BodySizeLimitMiddleware with advanced configuration
#[derive(Debug)]
pub struct BodySizeLimitBuilder {
    max_size: Option<usize>,
    custom_message: Option<String>,
    log_rejections: bool,
    include_size_info: bool,
}

impl BodySizeLimitBuilder {
    /// Create a new builder
    fn new() -> Self {
        Self {
            max_size: None,
            custom_message: None,
            log_rejections: true,
            include_size_info: true,
        }
    }

    /// Set maximum size in bytes
    pub fn max_size(mut self, bytes: usize) -> Self {
        self.max_size = Some(bytes);
        self
    }

    /// Set maximum size in kilobytes
    pub fn max_size_kb(mut self, kb: usize) -> Self {
        self.max_size = Some(kb * 1024);
        self
    }

    /// Set maximum size in megabytes
    pub fn max_size_mb(mut self, mb: usize) -> Self {
        self.max_size = Some(mb * 1024 * 1024);
        self
    }

    /// Set maximum size in gigabytes
    pub fn max_size_gb(mut self, gb: usize) -> Self {
        self.max_size = Some(gb * 1024 * 1024 * 1024);
        self
    }

    /// Set custom error message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.custom_message = Some(message.into());
        self
    }

    /// Disable logging of rejected requests
    pub fn disable_logging(mut self) -> Self {
        self.log_rejections = false;
        self
    }

    /// Hide size information from error responses
    pub fn hide_size_info(mut self) -> Self {
        self.include_size_info = false;
        self
    }

    /// Build the middleware
    pub fn build(self) -> BodySizeLimitMiddleware {
        let max_size = self.max_size.unwrap_or(10 * 1024 * 1024); // Default 10MB

        BodySizeLimitMiddleware {
            max_size,
            custom_message: self.custom_message,
            log_rejections: self.log_rejections,
            include_size_info: self.include_size_info,
        }
    }
}
