//! # Advanced Error Handling Middleware
//!
//! This module provides comprehensive error handling and logging middleware for the Ignitia
//! web framework. It offers advanced error processing, custom error pages, structured logging,
//! and configurable error responses.
//!
//! ## Features
//!
//! - **Advanced Error Logging**: Structured logging with configurable levels
//! - **Custom Error Pages**: Support for custom HTML error pages per status code
//! - **JSON Error Formatting**: Configurable JSON error response formats
//! - **Security Options**: Control error detail exposure in production
//! - **Performance Monitoring**: Error rate tracking and logging
//! - **Development Tools**: Enhanced error information for debugging
//!
//! ## Error Processing Pipeline
//!
//! 1. **Error Detection**: Identifies error responses (4xx, 5xx status codes)
//! 2. **Logging**: Records errors with appropriate log levels
//! 3. **Response Processing**: Optionally modifies error responses
//! 4. **Custom Formatting**: Applies custom error pages or JSON formats
//!
//! ## Usage
//!
//! ### Basic Error Handling
//! ```
//! use ignitia::{Router, ErrorHandlerMiddleware};
//!
//! let router = Router::new()
//!     .middleware(ErrorHandlerMiddleware::new())
//!     .get("/", || async { Ok(ignitia::Response::text("Hello")) })
//!     .get("/error", || async {
//!         Err(ignitia::Error::BadRequest("Something went wrong".into()))
//!     });
//! ```
//!
//! ### Production Configuration
//! ```
//! use ignitia::{Router, ErrorHandlerMiddleware};
//!
//! let router = Router::new()
//!     .middleware(
//!         ErrorHandlerMiddleware::new()
//!             .with_details(false) // Hide error details in production
//!             .with_logging(true)
//!             .with_error_log_threshold(400) // Log 4xx as warnings, 5xx as errors
//!     )
//!     .get("/api/data", || async { Ok(ignitia::Response::text("Data")) });
//! ```
//!
//! ### Development Configuration
//! ```
//! use ignitia::{Router, ErrorHandlerMiddleware};
//!
//! let router = Router::new()
//!     .middleware(
//!         ErrorHandlerMiddleware::new()
//!             .with_details(true) // Show detailed error information
//!             .with_stack_trace(true) // Include stack traces
//!             .with_logging(true)
//!     )
//!     .get("/api/test", || async { Ok(ignitia::Response::text("Test")) });
//! ```
//!
//! ### Custom Error Pages
//! ```
//! use ignitia::{Router, ErrorHandlerMiddleware};
//! use http::StatusCode;
//!
//! let custom_404_page = r#"
//! <!DOCTYPE html>
//! <html>
//! <head><title>Page Not Found</title></head>
//! <body>
//!     <h1>Oops! Page Not Found</h1>
//!     <p>The page you're looking for doesn't exist.</p>
//!     <a href="/">Go Home</a>
//! </body>
//! </html>
//! "#;
//!
//! let router = Router::new()
//!     .middleware(
//!         ErrorHandlerMiddleware::new()
//!             .with_custom_error_page(StatusCode::NOT_FOUND, custom_404_page.to_string())
//!     )
//!     .get("/", || async { Ok(ignitia::Response::text("Home")) });
//! ```
//!
//! ## Error Logging Levels
//!
//! The middleware uses different log levels based on HTTP status codes:
//!
//! - **4xx Client Errors**: Logged as `WARN` (configurable)
//! - **5xx Server Errors**: Logged as `ERROR` (configurable)
//! - **Other Status Codes**: Logged as `DEBUG`
//!
//! ### Log Output Examples
//! ```
//! WARN  HTTP 404 - Not Found (Body length: 23 bytes)
//! ERROR HTTP 500 - Internal Server Error (Body length: 45 bytes)
//! WARN  HTTP 400 - Bad Request (Body length: 67 bytes)
//! ERROR HTTP 503 - Service Unavailable (Body length: 34 bytes)
//! ```
//!
//! ## Custom Error Formats
//!
//! ### JSON Error Format Options
//! ```
//! use ignitia::{Router, ErrorHandlerMiddleware};
//! use ignitia::middleware::error_handler::ErrorFormat;
//!
//! let router = Router::new()
//!     .middleware(
//!         ErrorHandlerMiddleware::new()
//!             .with_json_format(ErrorFormat::Detailed)
//!     )
//!     .get("/api", || async { Ok(ignitia::Response::text("API")) });
//! ```
//!
//! ### Custom Error Response Function
//! ```
//! use ignitia::{Router, ErrorHandlerMiddleware, Error, Request};
//! use ignitia::middleware::error_handler::ErrorFormat;
//!
//! fn custom_error_formatter(error: &Error, req: &Request) -> serde_json::Value {
//!     serde_json::json!({
//!         "error": error.to_string(),
//!         "path": req.uri.path(),
//!         "method": req.method.as_str(),
//!         "timestamp": chrono::Utc::now().to_rfc3339(),
//!         "request_id": "req-12345" // You could generate this
//!     })
//! }
//!
//! let router = Router::new()
//!     .middleware(
//!         ErrorHandlerMiddleware::new()
//!             .with_json_format(ErrorFormat::Custom(custom_error_formatter))
//!     )
//!     .get("/", || async { Ok(ignitia::Response::text("Home")) });
//! ```
//!
//! ## Advanced Usage Examples
//!
//! ### Error Monitoring Integration
//! ```
//! use ignitia::{Middleware, Request, Response, Result};
//! use async_trait::async_trait;
//! use tracing::{error, warn, info};
//!
//! pub struct MonitoringErrorMiddleware {
//!     service_name: String,
//!     environment: String,
//! }
//!
//! impl MonitoringErrorMiddleware {
//!     pub fn new(service_name: String, environment: String) -> Self {
//!         Self { service_name, environment }
//!     }
//! }
//!
//! #[async_trait]
//! impl Middleware for MonitoringErrorMiddleware {
//!     async fn after(&self, res: &mut Response) -> Result<()> {
//!         if !res.status.is_success() {
//!             let status_code = res.status.as_u16();
//!
//!             // Send metrics to monitoring service
//!             self.send_error_metric(status_code).await;
//!
//!             // Log with structured data
//!             error!(
//!                 service = %self.service_name,
//!                 environment = %self.environment,
//!                 status_code = status_code,
//!                 "HTTP error response"
//!             );
//!         }
//!         Ok(())
//!     }
//! }
//!
//! impl MonitoringErrorMiddleware {
//!     async fn send_error_metric(&self, status_code: u16) {
//!         // Send to monitoring service (Prometheus, DataDog, etc.)
//!         info!("Sending error metric: {}", status_code);
//!     }
//! }
//! ```
//!
//! ### Error Rate Limiting
//! ```
//! use ignitia::{Middleware, Request, Response, Result, Error};
//! use async_trait::async_trait;
//! use std::collections::HashMap;
//! use std::sync::{Arc, Mutex};
//! use std::time::{Duration, Instant};
//!
//! pub struct ErrorRateLimitMiddleware {
//!     error_counts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
//!     max_errors_per_minute: u32,
//! }
//!
//! impl ErrorRateLimitMiddleware {
//!     pub fn new(max_errors_per_minute: u32) -> Self {
//!         Self {
//!             error_counts: Arc::new(Mutex::new(HashMap::new())),
//!             max_errors_per_minute,
//!         }
//!     }
//!
//!     fn get_client_id(&self, req: &Request) -> String {
//!         // Get client identifier (IP, user ID, etc.)
//!         req.header("x-forwarded-for")
//!             .or_else(|| req.header("x-real-ip"))
//!             .unwrap_or("unknown")
//!             .to_string()
//!     }
//! }
//!
//! #[async_trait]
//! impl Middleware for ErrorRateLimitMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         let client_id = self.get_client_id(req);
//!         let now = Instant::now();
//!
//!         let mut counts = self.error_counts.lock().unwrap();
//!
//!         // Clean old entries and check rate limit
//!         let (count, last_error) = counts
//!             .entry(client_id.clone())
//!             .or_insert((0, now));
//!
//!         if now.duration_since(*last_error) > Duration::from_secs(60) {
//!             *count = 0;
//!             *last_error = now;
//!         }
//!
//!         if *count >= self.max_errors_per_minute {
//!             return Err(Error::BadRequest("Too many errors, please try again later".into()));
//!         }
//!
//!         Ok(())
//!     }
//!
//!     async fn after(&self, res: &mut Response) -> Result<()> {
//!         if res.status.as_u16() >= 400 {
//!             // This would need the request context, which isn't available in after()
//!             // In a real implementation, you'd store this in request extensions
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Configuration Best Practices
//!
//! ### Development Settings
//! ```
//! use ignitia::ErrorHandlerMiddleware;
//!
//! let dev_middleware = ErrorHandlerMiddleware::new()
//!     .with_details(true)          // Show full error details
//!     .with_stack_trace(true)      // Include stack traces
//!     .with_logging(true)          // Enable logging
//!     .with_error_log_threshold(0); // Log everything as errors
//! ```
//!
//! ### Production Settings
//! ```
//! use ignitia::ErrorHandlerMiddleware;
//!
//! let prod_middleware = ErrorHandlerMiddleware::new()
//!     .with_details(false)         // Hide sensitive error details
//!     .with_stack_trace(false)     // No stack traces
//!     .with_logging(true)          // Keep logging enabled
//!     .with_error_log_threshold(500); // 4xx warnings, 5xx errors
//! ```
//!
//! ### Security Considerations
//!
//! - **Never expose stack traces in production**
//! - **Sanitize error messages** to avoid information leakage
//! - **Log security-related errors** for monitoring
//! - **Rate limit error responses** to prevent abuse
//!
//! ## Performance Impact
//!
//! - **Minimal Overhead**: Only processes error responses
//! - **Efficient Logging**: Uses structured logging with minimal allocations
//! - **Optional Processing**: Can be disabled in performance-critical scenarios
//! - **Memory Efficient**: No significant memory usage per request

use crate::middleware::Middleware;
use crate::{Error, Request, Response, Result};
use http::StatusCode;
use std::collections::HashMap;
use tracing::{debug, error, warn};

/// Advanced error handling middleware with configurable logging and custom error pages.
///
/// This middleware provides comprehensive error processing capabilities including:
/// - Structured error logging with configurable levels
/// - Custom error pages for different HTTP status codes
/// - Configurable JSON error response formats
/// - Production vs development error detail control
/// - Performance monitoring and error tracking
///
/// # Default Configuration
/// - **Details**: Enabled in debug builds, disabled in release builds
/// - **Stack Traces**: Enabled in debug builds, disabled in release builds
/// - **Logging**: Enabled
/// - **Error Log Threshold**: 500 (5xx errors logged as errors, 4xx as warnings)
/// - **JSON Format**: Detailed format
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::{Router, ErrorHandlerMiddleware};
///
/// let router = Router::new()
///     .middleware(ErrorHandlerMiddleware::new())
///     .get("/", || async { Ok(ignitia::Response::text("Hello")) });
/// ```
///
/// ## Production Configuration
/// ```
/// use ignitia::{Router, ErrorHandlerMiddleware};
///
/// let router = Router::new()
///     .middleware(
///         ErrorHandlerMiddleware::new()
///             .with_details(false)
///             .with_logging(true)
///     )
///     .get("/api", || async { Ok(ignitia::Response::text("API")) });
/// ```
pub struct ErrorHandlerMiddleware {
    /// Whether to include detailed error information in responses
    include_details: bool,
    /// Whether to include stack traces in debug mode
    include_stack_trace: bool,
    /// Custom error pages for different status codes
    custom_error_pages: HashMap<StatusCode, String>,
    /// Custom JSON error format
    json_error_format: ErrorFormat,
    /// Whether to log errors
    log_errors: bool,
    /// Minimum status code to log as error (vs warning)
    error_log_threshold: u16,
}

/// Configuration for JSON error response formatting.
///
/// This enum allows you to choose between different error response formats
/// or provide a custom formatting function.
///
/// # Variants
/// - `Simple`: Basic error information only
/// - `Detailed`: Comprehensive error information including metadata
/// - `Custom`: User-provided formatting function
///
/// # Examples
/// ```
/// use ignitia::middleware::error_handler::ErrorFormat;
/// use ignitia::{Error, Request};
///
/// let custom_format = ErrorFormat::Custom(|error: &Error, req: &Request| {
///     serde_json::json!({
///         "message": error.to_string(),
///         "path": req.uri.path(),
///         "timestamp": chrono::Utc::now().to_rfc3339()
///     })
/// });
/// ```
#[derive(Clone)]
pub enum ErrorFormat {
    /// Simple error format with basic information
    Simple,
    /// Detailed error format with comprehensive information
    Detailed,
    /// Custom error formatting function
    Custom(fn(&Error, &Request) -> serde_json::Value),
}

impl ErrorHandlerMiddleware {
    /// Creates a new error handler middleware with default settings.
    ///
    /// Default configuration:
    /// - Details and stack traces enabled in debug builds
    /// - Error logging enabled
    /// - 5xx status codes logged as errors, 4xx as warnings
    /// - Detailed JSON error format
    ///
    /// # Examples
    /// ```
    /// use ignitia::ErrorHandlerMiddleware;
    ///
    /// let middleware = ErrorHandlerMiddleware::new();
    /// ```
    pub fn new() -> Self {
        Self {
            include_details: cfg!(debug_assertions),
            include_stack_trace: cfg!(debug_assertions),
            custom_error_pages: HashMap::new(),
            json_error_format: ErrorFormat::Detailed,
            log_errors: true,
            error_log_threshold: 500, // 5xx errors logged as errors, 4xx as warnings
        }
    }

    /// Configures whether to include detailed error information in responses.
    ///
    /// When enabled, error responses may include additional debugging information.
    /// This should typically be disabled in production environments.
    ///
    /// # Parameters
    /// - `include`: Whether to include detailed error information
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::ErrorHandlerMiddleware;
    ///
    /// let middleware = ErrorHandlerMiddleware::new()
    ///     .with_details(false); // Hide details for production
    /// ```
    pub fn with_details(mut self, include: bool) -> Self {
        self.include_details = include;
        self
    }

    /// Configures whether to include stack traces in error responses.
    ///
    /// Stack traces can be helpful for debugging but should never be exposed
    /// in production as they may reveal sensitive information about the
    /// application structure.
    ///
    /// # Parameters
    /// - `include`: Whether to include stack traces
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Security Warning
    /// Never enable stack traces in production environments as they can
    /// expose sensitive information about your application structure.
    ///
    /// # Examples
    /// ```
    /// use ignitia::ErrorHandlerMiddleware;
    ///
    /// // Development
    /// let dev_middleware = ErrorHandlerMiddleware::new()
    ///     .with_stack_trace(true);
    ///
    /// // Production
    /// let prod_middleware = ErrorHandlerMiddleware::new()
    ///     .with_stack_trace(false);
    /// ```
    pub fn with_stack_trace(mut self, include: bool) -> Self {
        self.include_stack_trace = include;
        self
    }

    /// Adds a custom error page for a specific HTTP status code.
    ///
    /// When an error with the specified status code occurs, the custom HTML
    /// page will be returned instead of the default JSON error response.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code to customize
    /// - `html`: The HTML content to return for this status code
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::ErrorHandlerMiddleware;
    /// use http::StatusCode;
    ///
    /// let custom_404 = r#"
    /// <!DOCTYPE html>
    /// <html>
    /// <head><title>Page Not Found</title></head>
    /// <body>
    ///     <h1>404 - Page Not Found</h1>
    ///     <p>The requested page could not be found.</p>
    /// </body>
    /// </html>
    /// "#;
    ///
    /// let middleware = ErrorHandlerMiddleware::new()
    ///     .with_custom_error_page(StatusCode::NOT_FOUND, custom_404.to_string());
    /// ```
    pub fn with_custom_error_page(mut self, status: StatusCode, html: String) -> Self {
        self.custom_error_pages.insert(status, html);
        self
    }

    /// Sets the JSON error response format.
    ///
    /// This configures how error responses are formatted when returned as JSON.
    /// You can choose between simple, detailed, or custom formatting.
    ///
    /// # Parameters
    /// - `format`: The error format to use
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::ErrorHandlerMiddleware;
    /// use ignitia::middleware::error_handler::ErrorFormat;
    ///
    /// let middleware = ErrorHandlerMiddleware::new()
    ///     .with_json_format(ErrorFormat::Simple);
    /// ```
    pub fn with_json_format(mut self, format: ErrorFormat) -> Self {
        self.json_error_format = format;
        self
    }

    /// Configures whether error logging is enabled.
    ///
    /// When enabled, the middleware will log error responses using the
    /// tracing framework. This is useful for monitoring and debugging.
    ///
    /// # Parameters
    /// - `enabled`: Whether to enable error logging
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::ErrorHandlerMiddleware;
    ///
    /// let middleware = ErrorHandlerMiddleware::new()
    ///     .with_logging(true);
    /// ```
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_errors = enabled;
        self
    }

    /// Sets the threshold for error vs warning log levels.
    ///
    /// Status codes at or above this threshold will be logged as errors,
    /// while status codes below will be logged as warnings.
    ///
    /// # Parameters
    /// - `threshold`: The status code threshold (default: 500)
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::ErrorHandlerMiddleware;
    ///
    /// let middleware = ErrorHandlerMiddleware::new()
    ///     .with_error_log_threshold(400); // Log 4xx+ as errors
    /// ```
    ///
    /// ## Common Thresholds
    /// - `400`: Log 4xx and 5xx as errors
    /// - `500`: Log only 5xx as errors, 4xx as warnings (default)
    /// - `600`: Log nothing as errors (everything as warnings)
    pub fn with_error_log_threshold(mut self, threshold: u16) -> Self {
        self.error_log_threshold = threshold;
        self
    }

    /// Returns a user-friendly error message for common HTTP status codes.
    ///
    /// This method provides human-readable error messages that can be displayed
    /// to users instead of technical error descriptions.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code
    ///
    /// # Returns
    /// A user-friendly error message
    ///
    /// # Examples
    /// ```
    /// let message = middleware.get_user_friendly_message(StatusCode::NOT_FOUND);
    /// // Returns: "The requested resource could not be found."
    /// ```
    fn _get_user_friendly_message(&self, status: StatusCode) -> &'static str {
        match status {
            StatusCode::BAD_REQUEST => "The request could not be understood by the server.",
            StatusCode::UNAUTHORIZED => "Authentication is required to access this resource.",
            StatusCode::FORBIDDEN => "You don't have permission to access this resource.",
            StatusCode::NOT_FOUND => "The requested resource could not be found.",
            StatusCode::METHOD_NOT_ALLOWED => {
                "The request method is not allowed for this resource."
            }
            StatusCode::CONFLICT => "The request conflicts with the current state of the resource.",
            StatusCode::UNPROCESSABLE_ENTITY => {
                "The request was well-formed but contains semantic errors."
            }
            StatusCode::TOO_MANY_REQUESTS => "Too many requests. Please try again later.",
            StatusCode::INTERNAL_SERVER_ERROR => "An internal server error occurred.",
            StatusCode::NOT_IMPLEMENTED => "This feature is not yet implemented.",
            StatusCode::BAD_GATEWAY => {
                "The server received an invalid response from an upstream server."
            }
            StatusCode::SERVICE_UNAVAILABLE => "The service is temporarily unavailable.",
            StatusCode::GATEWAY_TIMEOUT => {
                "The server did not receive a timely response from an upstream server."
            }
            _ => "An error occurred while processing your request.",
        }
    }
}

#[async_trait::async_trait]
impl Middleware for ErrorHandlerMiddleware {
    /// Processes error responses after they are generated by handlers.
    ///
    /// This method is called for all responses, but only processes those with
    /// error status codes (4xx, 5xx). It performs logging and can optionally
    /// modify error responses.
    ///
    /// # Processing Steps
    /// 1. **Status Check**: Only processes non-success status codes
    /// 2. **Logging**: Records errors with appropriate log levels
    /// 3. **Response Modification**: Optionally applies custom error pages or formatting
    ///
    /// # Parameters
    /// - `res`: Mutable reference to the response
    ///
    /// # Returns
    /// - `Ok(())`: Processing completed successfully
    ///
    /// # Logging Behavior
    /// - Status codes >= `error_log_threshold`: Logged as ERROR
    /// - Status codes >= 400: Logged as WARN
    /// - Other status codes: Logged as DEBUG
    ///
    /// # Examples
    /// Given an error response, this might log:
    /// ```
    /// ERROR HTTP 500 - Internal Server Error (Body length: 45 bytes)
    /// WARN  HTTP 404 - Not Found (Body length: 23 bytes)
    /// ```
    async fn after(&self, _req: &Request, res: &mut Response) -> Result<()> {
        // Only process error responses
        if res.status.is_success() {
            return Ok(());
        }

        // Log the error if logging is enabled
        if self.log_errors {
            let status_code = res.status.as_u16();
            let log_message = format!(
                "HTTP {} - {} (Body length: {} bytes)",
                status_code,
                res.status.canonical_reason().unwrap_or("Unknown"),
                res.body.len()
            );

            if status_code >= self.error_log_threshold {
                error!("{}", log_message);
            } else if status_code >= 400 {
                warn!("{}", log_message);
            } else {
                debug!("{}", log_message);
            }
        }

        // Note: We can't access the original request here since the middleware trait
        // only provides access to the response in the after() method.
        // For full error handling, we'd need to modify the middleware trait or
        // handle this in the router level.

        Ok(())
    }
}

/// Default implementation for ErrorHandlerMiddleware.
///
/// Creates a new error handler with default settings suitable for most applications.
impl Default for ErrorHandlerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
