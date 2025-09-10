//! # Request ID Middleware
//!
//! This module provides HTTP request ID middleware for the Ignitia web framework.
//! It generates unique identifiers for each request to enable distributed tracing,
//! request correlation, and improved debugging capabilities.
//!
//! ## Features
//!
//! - **Unique ID Generation**: Creates UUIDs or custom IDs for each request
//! - **Header Propagation**: Uses standard X-Request-ID header
//! - **Client ID Support**: Respects existing request IDs from clients
//! - **Validation**: Ensures request ID format and length constraints
//! - **Context Integration**: Makes request ID available throughout request lifecycle
//! - **Logging Integration**: Structured logging with request correlation
//!
//! ## Quick Start
//!
//! ```
//! use ignitia::{Router, RequestIdMiddleware};
//!
//! let app = Router::new()
//!     .middleware(RequestIdMiddleware::new())
//!     .get("/api/users", || async {
//!         // Request ID automatically available in logs and context
//!         ignitia::Response::json(&users)
//!     });
//! ```

use std::str::FromStr;

use crate::middleware::Middleware;
use crate::{Request, Response, Result};
use http::header::HeaderValue;
use http::HeaderName;
use tracing::{debug, info};
use uuid::Uuid;

/// Request ID header name (standard convention)
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Maximum allowed length for request IDs (security constraint)
const MAX_REQUEST_ID_LENGTH: usize = 200;

/// Minimum allowed length for request IDs
const MIN_REQUEST_ID_LENGTH: usize = 8;

/// Request ID generation strategy
#[derive(Debug, Clone)]
pub enum IdGenerator {
    /// UUID v4 (default) - cryptographically random
    Uuid,
    /// Nanoid - URL-safe, shorter than UUID
    NanoId { length: usize },
    /// Custom function for ID generation
    Custom(fn() -> String),
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::Uuid
    }
}

impl IdGenerator {
    /// Generate a new request ID using the configured strategy
    pub fn generate(&self) -> String {
        match self {
            IdGenerator::Uuid => Uuid::new_v4().to_string(),
            IdGenerator::NanoId { length } => generate_nanoid(*length),
            IdGenerator::Custom(func) => func(),
        }
    }
}

/// HTTP Request ID middleware for distributed tracing and request correlation.
///
/// This middleware automatically assigns unique identifiers to HTTP requests,
/// enabling better debugging, logging, and distributed tracing capabilities.
///
/// ## Behavior
///
/// 1. **Request Processing**:
///    - Checks for existing `X-Request-ID` header from client
///    - Validates existing ID format and length
///    - Generates new ID if missing or invalid
///    - Sets ID in request context for downstream use
///
/// 2. **Response Processing**:
///    - Always includes `X-Request-ID` header in response
///    - Enables client-side request correlation
///    - Supports debugging and audit trails
///
/// ## Security Considerations
///
/// - Request IDs are limited to 200 characters maximum
/// - Only ASCII alphanumeric and safe characters allowed
/// - Invalid IDs are replaced with server-generated ones
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use ignitia::{Router, RequestIdMiddleware};
///
/// let app = Router::new()
///     .middleware(RequestIdMiddleware::new())
///     .get("/health", || async {
///         // Request ID available in tracing spans automatically
///         ignitia::Response::text("OK")
///     });
/// ```
///
/// ## Custom Configuration
///
/// ```
/// use ignitia::{RequestIdMiddleware, IdGenerator};
///
/// let request_id_mw = RequestIdMiddleware::new()
///     .with_generator(IdGenerator::NanoId { length: 16 })
///     .with_header_name("x-trace-id")
///     .with_validation(true);
/// ```
///
/// ## With Logging Integration
///
/// ```
/// use ignitia::{RequestIdMiddleware, LoggerMiddleware};
///
/// let app = Router::new()
///     .middleware(RequestIdMiddleware::new())  // Must come first
///     .middleware(LoggerMiddleware::new())     // Will include request ID
///     .get("/api/data", handler);
/// ```
#[derive(Debug, Clone)]
pub struct RequestIdMiddleware {
    /// ID generation strategy
    generator: IdGenerator,
    /// Header name for request ID (default: "x-request-id")
    header_name: String,
    /// Whether to validate incoming request IDs
    validate_incoming: bool,
    /// Whether to include request ID in structured logging
    enable_logging: bool,
}

impl Default for RequestIdMiddleware {
    /// Creates a new `RequestIdMiddleware` with sensible defaults.
    ///
    /// ## Default Configuration
    ///
    /// - **Generator**: UUID v4
    /// - **Header**: "x-request-id"
    /// - **Validation**: Enabled
    /// - **Logging**: Enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::RequestIdMiddleware;
    ///
    /// let middleware = RequestIdMiddleware::default();
    /// // Equivalent to:
    /// let middleware = RequestIdMiddleware::new();
    /// ```
    fn default() -> Self {
        Self {
            generator: IdGenerator::default(),
            header_name: REQUEST_ID_HEADER.to_string(),
            validate_incoming: true,
            enable_logging: true,
        }
    }
}

impl RequestIdMiddleware {
    /// Creates a new `RequestIdMiddleware` with default settings.
    ///
    /// This is equivalent to calling `RequestIdMiddleware::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ID generation strategy.
    ///
    /// # Parameters
    ///
    /// * `generator` - The ID generation strategy to use
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{RequestIdMiddleware, IdGenerator};
    ///
    /// // Use shorter NanoIDs
    /// let middleware = RequestIdMiddleware::new()
    ///     .with_generator(IdGenerator::NanoId { length: 12 });
    ///
    /// // Use custom generator
    /// let middleware = RequestIdMiddleware::new()
    ///     .with_generator(IdGenerator::Custom(|| {
    ///         format!("req_{}", chrono::Utc::now().timestamp_millis())
    ///     }));
    /// ```
    pub fn with_generator(mut self, generator: IdGenerator) -> Self {
        self.generator = generator;
        self
    }

    /// Sets the header name for the request ID.
    ///
    /// # Parameters
    ///
    /// * `header_name` - The HTTP header name to use
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::RequestIdMiddleware;
    ///
    /// // Use custom header name
    /// let middleware = RequestIdMiddleware::new()
    ///     .with_header_name("x-trace-id");
    ///
    /// // Use correlation ID header
    /// let middleware = RequestIdMiddleware::new()
    ///     .with_header_name("x-correlation-id");
    /// ```
    pub fn with_header_name(mut self, header_name: &str) -> Self {
        self.header_name = header_name.to_lowercase();
        self
    }

    /// Enables or disables validation of incoming request IDs.
    ///
    /// When enabled, incoming request IDs are validated for format and length.
    /// Invalid IDs are replaced with server-generated ones.
    ///
    /// # Parameters
    ///
    /// * `validate` - Whether to validate incoming request IDs
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::RequestIdMiddleware;
    ///
    /// // Trust all client-provided request IDs
    /// let middleware = RequestIdMiddleware::new()
    ///     .with_validation(false);
    /// ```
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_incoming = validate;
        self
    }

    /// Enables or disables structured logging integration.
    ///
    /// When enabled, request IDs are automatically included in log spans.
    ///
    /// # Parameters
    ///
    /// * `enable` - Whether to enable logging integration
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// Validates a request ID string.
    ///
    /// Checks format, length, and character constraints for security.
    ///
    /// # Parameters
    ///
    /// * `request_id` - The request ID to validate
    ///
    /// # Returns
    ///
    /// `true` if the request ID is valid, `false` otherwise.
    fn is_valid_request_id(&self, request_id: &str) -> bool {
        if request_id.len() < MIN_REQUEST_ID_LENGTH || request_id.len() > MAX_REQUEST_ID_LENGTH {
            return false;
        }

        // Allow ASCII letters, digits, hyphens, and underscores
        request_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }

    /// Extracts or generates a request ID for the given request.
    ///
    /// # Parameters
    ///
    /// * `req` - The HTTP request to process
    ///
    /// # Returns
    ///
    /// A valid request ID string.
    fn get_or_generate_request_id(&self, req: &Request) -> String {
        // Try to get existing request ID from headers
        if let Some(existing_id) = req.header(&self.header_name) {
            if !self.validate_incoming || self.is_valid_request_id(existing_id) {
                debug!("Using client-provided request ID: {}", existing_id);
                return existing_id.to_string();
            } else {
                debug!(
                    "Invalid client request ID, generating new one: {}",
                    existing_id
                );
            }
        }

        // Generate new request ID
        let new_id = self.generator.generate();
        debug!("Generated new request ID: {}", new_id);
        new_id
    }
}

#[async_trait::async_trait]
impl Middleware for RequestIdMiddleware {
    /// Processes the request and assigns a unique request ID.
    ///
    /// This method extracts existing request IDs from headers or generates
    /// new ones, then stores the ID for use in the response phase and logging.
    ///
    /// # Parameters
    ///
    /// * `req` - The HTTP request to process
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if processing fails.
    async fn before(&self, req: &mut Request) -> Result<()> {
        let request_id = self.get_or_generate_request_id(req);

        let header_name = HeaderName::from_str(self.header_name.as_str())
            .map_err(|e| crate::Error::Internal(format!("Invalid header name: {}", e)))?;

        // Store request ID in request headers for after() phase
        req.headers.insert(
            header_name,
            HeaderValue::from_str(&request_id)
                .map_err(|e| crate::Error::Internal(format!("Invalid request ID: {}", e)))?,
        );

        // Create tracing span with request ID for structured logging
        if self.enable_logging {
            info!(
                request_id = %request_id,
                method = %req.method,
                uri = %req.uri,
                "Processing request"
            );
        }

        Ok(())
    }

    /// Processes the response and includes the request ID in response headers.
    ///
    /// This ensures the client receives the request ID for correlation and debugging.
    ///
    /// # Parameters
    ///
    /// * `req` - The original HTTP request
    /// * `res` - The HTTP response to process
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if processing fails.
    async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        // Get request ID from request headers (set in before() phase)
        if let Some(request_id) = req.header(&self.header_name) {
            let header_name = HeaderName::from_str(self.header_name.as_str())
                .map_err(|e| crate::Error::Internal(format!("Invalid header name: {}", e)))?;

            // Set request ID in response headers
            res.headers.insert(
                header_name,
                HeaderValue::from_str(request_id)
                    .map_err(|e| crate::Error::Internal(format!("Invalid request ID: {}", e)))?,
            );

            // Log response with request ID
            if self.enable_logging {
                info!(
                    request_id = %request_id,
                    status = %res.status.as_u16(),
                    "Request completed"
                );
            }
        }

        Ok(())
    }
}

// Preset configurations for common use cases
impl RequestIdMiddleware {
    /// Creates request ID middleware optimized for microservices.
    ///
    /// This configuration is designed for distributed systems:
    /// - Short NanoID for reduced header size
    /// - Standard X-Request-ID header
    /// - Strict validation enabled
    /// - Logging enabled for observability
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, RequestIdMiddleware};
    ///
    /// let api = Router::new()
    ///     .middleware(RequestIdMiddleware::for_microservices())request_id.rs
    ///     .get("/api/users", get_users_handler);
    /// ```
    pub fn for_microservices() -> Self {
        Self::new()
            .with_generator(IdGenerator::NanoId { length: 16 })
            .with_validation(true)
            .with_logging(true)
    }

    /// Creates request ID middleware for development and debugging.
    ///
    /// This configuration prioritizes debugging convenience:
    /// - UUID for maximum uniqueness
    /// - Custom trace header
    /// - Relaxed validation
    /// - Verbose logging
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, RequestIdMiddleware};
    ///
    /// let dev_app = Router::new()
    ///     .middleware(RequestIdMiddleware::for_development())
    ///     .get("/debug", debug_handler);
    /// ```
    pub fn for_development() -> Self {
        Self::new()
            .with_generator(IdGenerator::Uuid)
            .with_header_name("x-trace-id")
            .with_validation(false)
            .with_logging(true)
    }

    /// Creates request ID middleware for high-performance scenarios.
    ///
    /// This configuration minimizes overhead:
    /// - Short NanoID for performance
    /// - Minimal validation
    /// - Reduced logging
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, RequestIdMiddleware};
    ///
    /// let fast_api = Router::new()
    ///     .middleware(RequestIdMiddleware::for_performance())
    ///     .get("/api/fast", fast_handler);
    /// ```
    pub fn for_performance() -> Self {
        Self::new()
            .with_generator(IdGenerator::NanoId { length: 12 })
            .with_validation(false)
            .with_logging(false)
    }
}

/// Helper function to generate NanoID
fn generate_nanoid(length: usize) -> String {
    use rand::Rng;

    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..ALPHABET.len());
            ALPHABET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Request, Response};
    use bytes::Bytes;
    use http::{HeaderMap, Method, Uri, Version};

    /// Test helper to create a mock request
    fn mock_request() -> Request {
        Request::new(
            Method::GET,
            Uri::from_static("/"),
            Version::HTTP_11,
            HeaderMap::new(),
            Bytes::new(),
        )
    }

    /// Test helper to create a mock request with request ID header
    fn mock_request_with_id(request_id: &str) -> Request {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", request_id.parse().unwrap());
        Request::new(
            Method::GET,
            Uri::from_static("/"),
            Version::HTTP_11,
            headers,
            Bytes::new(),
        )
    }

    #[test]
    fn test_request_id_validation() {
        let middleware = RequestIdMiddleware::new();

        // Valid request IDs
        assert!(middleware.is_valid_request_id("12345678"));
        assert!(middleware.is_valid_request_id("abc-123_def"));
        assert!(middleware.is_valid_request_id(&"a".repeat(200)));

        // Invalid request IDs
        assert!(!middleware.is_valid_request_id("1234567")); // Too short
        assert!(!middleware.is_valid_request_id(&"a".repeat(201))); // Too long
        assert!(!middleware.is_valid_request_id("abc@123")); // Invalid chars
        assert!(!middleware.is_valid_request_id("")); // Empty
    }

    #[test]
    fn test_id_generators() {
        // Test UUID generator
        let uuid_gen = IdGenerator::Uuid;
        let id1 = uuid_gen.generate();
        let id2 = uuid_gen.generate();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID v4 length

        // Test NanoID generator
        let nano_gen = IdGenerator::NanoId { length: 12 };
        let id3 = nano_gen.generate();
        assert_eq!(id3.len(), 12);

        // Test custom generator
        let custom_gen = IdGenerator::Custom(|| "test-123".to_string());
        let id4 = custom_gen.generate();
        assert_eq!(id4, "test-123");
    }

    #[tokio::test]
    async fn test_request_id_middleware_new_id() {
        let middleware = RequestIdMiddleware::new();
        let mut request = mock_request();
        let mut response = Response::new(http::StatusCode::OK);

        // Before phase should generate new request ID
        middleware.before(&mut request).await.unwrap();

        let request_id = request.header("x-request-id").unwrap();
        assert!(!request_id.is_empty());

        // After phase should set response header
        middleware.after(&request, &mut response).await.unwrap();

        let response_id = response
            .headers
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(request_id, response_id);
    }

    #[tokio::test]
    async fn test_request_id_middleware_existing_id() {
        let middleware = RequestIdMiddleware::new();
        let mut request = mock_request_with_id("existing-request-123");
        let mut response = Response::new(http::StatusCode::OK);

        // Before phase should preserve existing valid ID
        middleware.before(&mut request).await.unwrap();

        let request_id = request.header("x-request-id").unwrap();
        assert_eq!(request_id, "existing-request-123");

        // After phase should use same ID
        middleware.after(&request, &mut response).await.unwrap();

        let response_id = response
            .headers
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(response_id, "existing-request-123");
    }

    #[tokio::test]
    async fn test_request_id_middleware_invalid_id() {
        let middleware = RequestIdMiddleware::new(); // Validation enabled by default
        let mut request = mock_request_with_id("invalid@id!");
        let mut response = Response::new(http::StatusCode::OK);

        // Before phase should replace invalid ID
        middleware.before(&mut request).await.unwrap();

        let request_id = request.header("x-request-id").unwrap();
        assert_ne!(request_id, "invalid@id!");
        assert!(middleware.is_valid_request_id(request_id));

        // After phase should use new valid ID
        middleware.after(&request, &mut response).await.unwrap();

        let response_id = response
            .headers
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(response_id, request_id);
    }

    #[test]
    fn test_preset_configurations() {
        let microservices = RequestIdMiddleware::for_microservices();
        assert!(matches!(
            microservices.generator,
            IdGenerator::NanoId { length: 16 }
        ));
        assert!(microservices.validate_incoming);

        let development = RequestIdMiddleware::for_development();
        assert_eq!(development.header_name, "x-trace-id");
        assert!(!development.validate_incoming);

        let performance = RequestIdMiddleware::for_performance();
        assert!(matches!(
            performance.generator,
            IdGenerator::NanoId { length: 12 }
        ));
        assert!(!performance.enable_logging);
    }

    #[test]
    fn test_builder_pattern() {
        let middleware = RequestIdMiddleware::new()
            .with_generator(IdGenerator::NanoId { length: 8 })
            .with_header_name("x-custom-id")
            .with_validation(false)
            .with_logging(false);

        assert!(matches!(
            middleware.generator,
            IdGenerator::NanoId { length: 8 }
        ));
        assert_eq!(middleware.header_name, "x-custom-id");
        assert!(!middleware.validate_incoming);
        assert!(!middleware.enable_logging);
    }
}
