//! # Error Handling Module
//!
//! This module provides comprehensive error handling capabilities for the Ignitia web framework.
//! It includes a rich error type system, custom error traits, standardized error responses,
//! and utilities for easy error conversion and handling.
//!
//! ## Features
//!
//! - **Rich Error Types**: Predefined error variants for common web application scenarios
//! - **Custom Error Support**: Trait-based system for application-specific errors
//! - **Standardized Responses**: Consistent JSON error response format
//! - **Easy Conversion**: Helper traits and macros for seamless error handling
//! - **Flexible Error Handlers**: Support for custom error processing and formatting
//!
//! ## Quick Start
//!
//! ```
//! use ignitia::{Error, Result, ErrorExt};
//!
//! // Using predefined errors
//! fn validate_input(input: &str) -> Result<String> {
//!     if input.is_empty() {
//!         return Err(Error::validation("Input cannot be empty"));
//!     }
//!     Ok(input.to_string())
//! }
//!
//! // Using error conversion helpers
//! fn parse_number(s: &str) -> Result<i32> {
//!     s.parse().validation_error()
//! }
//! ```
//!
//! ## Custom Error Types
//!
//! ```
//! use ignitia::define_error;
//! use http::StatusCode;
//!
//! define_error! {
//!     UserError {
//!         InvalidCredentials(StatusCode::UNAUTHORIZED, "invalid_credentials", "AUTH_001"),
//!         AccountLocked(StatusCode::FORBIDDEN, "account_locked", "AUTH_002"),
//!     }
//! }
//! ```

use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

use crate::{Request, Response};

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use crate::server::tls::TlsError;

/// The main error type for the Ignitia web framework.
///
/// This enum represents all possible errors that can occur during request processing.
/// It provides both predefined error variants for common scenarios and support for
/// custom application-specific errors.
///
/// # Error Categories
///
/// - **Client Errors (4xx)**: `NotFound`, `MethodNotAllowed`, `BadRequest`, `Unauthorized`, `Forbidden`, `Validation`
/// - **Server Errors (5xx)**: `Internal`, `Database`, `ExternalService`
/// - **System Errors**: `Io`, `Hyper`, `Json`
/// - **Custom Errors**: `Custom` variant for application-specific errors
///
/// # Examples
///
/// ```
/// use ignitia::Error;
///
/// // Creating specific errors
/// let not_found = Error::not_found("/api/users/123");
/// let validation = Error::validation("Email format is invalid");
/// let unauthorized = Error::unauthorized();
///
/// // All errors can be converted to HTTP responses
/// let response = Response::from(not_found);
/// ```
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP 404 - Resource not found error.
    ///
    /// Used when a requested resource (route, file, database record) cannot be found.
    ///
    /// # Example
    /// ```
    /// let error = Error::not_found("/api/users/999");
    /// ```
    #[error("Route not found: {0}")]
    NotFound(String),

    /// HTTP 405 - Method not allowed error.
    ///
    /// Used when the HTTP method is not supported for the requested resource.
    ///
    /// # Example
    /// ```
    /// let error = Error::MethodNotAllowed("POST not allowed on /api/users".to_string());
    /// ```
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    /// HTTP 500 - Internal server error.
    ///
    /// Used for unexpected server-side errors that don't fit other categories.
    ///
    /// # Example
    /// ```
    /// let error = Error::internal("Unexpected server error occurred");
    /// ```
    #[error("Internal server error: {0}")]
    Internal(String),

    /// HTTP 400 - Bad request error.
    ///
    /// Used when the client request is malformed or contains invalid data.
    ///
    /// # Example
    /// ```
    /// let error = Error::bad_request("Missing required field: email");
    /// ```
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// HTTP 401 - Unauthorized error.
    ///
    /// Used when authentication is required but not provided or invalid.
    ///
    /// # Example
    /// ```
    /// let error = Error::unauthorized("Invalid API key provided");
    /// ```
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// HTTP 403 - Forbidden error.
    ///
    /// Used when the user is authenticated but doesn't have permission for the resource.
    ///
    /// # Example
    /// ```
    /// let error = Error::forbidden("Insufficient permissions to access this resource");
    /// ```
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// HTTP 429 - Too Many Requests error.
    ///
    /// Used when rate limiting is applied and the client has exceeded the allowed request rate.
    ///
    /// # Example
    /// ```
    /// let error = Error::too_many_requests("Rate limit exceeded. Try again in 60 seconds");
    /// ```
    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    /// HTTP 400 - Validation error.
    ///
    /// Used specifically for input validation failures.
    ///
    /// # Example
    /// ```
    /// let error = Error::validation("Password must be at least 8 characters");
    /// ```
    #[error("Validation failed: {0}")]
    Validation(String),

    /// HTTP 500 - Database error.
    ///
    /// Used for database-related errors (connection failures, query errors, etc.).
    ///
    /// # Example
    /// ```
    /// let error = Error::Database("Connection timeout".to_string());
    /// ```
    #[error("Database error: {0}")]
    Database(String),

    /// HTTP 500 - External service error.
    ///
    /// Used when external API calls or service integrations fail.
    ///
    /// # Example
    /// ```
    /// let error = Error::ExternalService("Payment gateway unavailable".to_string());
    /// ```
    #[error("External service error: {0}")]
    ExternalService(String),

    /// I/O error wrapper.
    ///
    /// Automatically converts `std::io::Error` into framework errors.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Hyper HTTP library error wrapper.
    ///
    /// Automatically converts `hyper::Error` into framework errors.
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    /// JSON serialization/deserialization error wrapper.
    ///
    /// Automatically converts `serde_json::Error` into framework errors.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TLS/SSL configuration and connection error.
    ///
    /// Used when TLS-related operations fail, such as certificate loading,
    /// handshake failures, or SSL/TLS configuration issues.
    ///
    /// This error type is only available when the "tls" feature is enabled.
    ///
    /// # Common Scenarios
    /// - Certificate file not found or invalid
    /// - Private key parsing failures
    /// - TLS handshake errors
    /// - Invalid TLS configuration
    /// - Self-signed certificate generation failures (when "self-signed" feature is enabled)
    ///
    /// # Example
    /// ```
    /// use ignitia::{Server, Router};
    ///
    /// #[tokio::main]
    /// async fn main() -> ignitia::Result<()> {
    ///     let app = Router::new()
    ///         .get("/", || async { Ok(ignitia::Response::text("Hello HTTPS!")) });
    ///
    ///     let server = Server::new(app, "127.0.0.1:8443".parse().unwrap())
    ///         .enable_https("cert.pem", "key.pem")?; // May return TLS error
    ///
    ///     server.ignitia().await?;
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    #[error("TLS error: {0}")]
    Tls(#[from] TlsError),

    /// Custom application-specific error.
    ///
    /// Allows applications to define their own error types while maintaining
    /// compatibility with the framework's error handling system.
    ///
    /// # Example
    /// ```
    /// use ignitia::{Error, CustomError};
    /// use http::StatusCode;
    ///
    /// #[derive(Debug)]
    /// struct MyCustomError(String);
    ///
    /// impl std::fmt::Display for MyCustomError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "Custom error: {}", self.0)
    ///     }
    /// }
    ///
    /// impl CustomError for MyCustomError {
    ///     fn status_code(&self) -> StatusCode { StatusCode::BAD_REQUEST }
    ///     fn error_type(&self) -> &'static str { "custom_error" }
    /// }
    ///
    /// let custom_error = Error::Custom(Box::new(MyCustomError("something went wrong".to_string())));
    /// ```
    #[error("Custom error: {0}")]
    Custom(Box<dyn CustomError>),
}

/// Trait for custom error types that can be converted to HTTP responses.
///
/// This trait allows applications to define their own error types while ensuring
/// they can be properly converted to HTTP responses with appropriate status codes
/// and metadata.
///
/// # Required Methods
///
/// - `status_code()`: Returns the HTTP status code for this error
/// - `error_type()`: Returns a string identifier for the error type
///
/// # Optional Methods
///
/// - `error_code()`: Returns an application-specific error code
/// - `metadata()`: Returns additional error metadata as JSON
///
/// # Example
///
/// ```
/// use ignitia::CustomError;
/// use http::StatusCode;
/// use serde_json::json;
///
/// #[derive(Debug)]
/// struct ValidationError {
///     field: String,
///     message: String,
/// }
///
/// impl std::fmt::Display for ValidationError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "Validation error on {}: {}", self.field, self.message)
///     }
/// }
///
/// impl CustomError for ValidationError {
///     fn status_code(&self) -> StatusCode {
///         StatusCode::BAD_REQUEST
///     }
///
///     fn error_type(&self) -> &'static str {
///         "validation_error"
///     }
///
///     fn error_code(&self) -> Option<String> {
///         Some("VALIDATION_001".to_string())
///     }
///
///     fn metadata(&self) -> Option<serde_json::Value> {
///         Some(json!({
///             "field": self.field,
///             "validation_type": "format"
///         }))
///     }
/// }
/// ```
pub trait CustomError: fmt::Debug + fmt::Display + Send + Sync + 'static {
    /// Returns the HTTP status code that should be used for this error.
    fn status_code(&self) -> StatusCode;

    /// Returns a string identifier for this error type.
    ///
    /// This should be a stable, machine-readable identifier that clients
    /// can use to programmatically handle specific error types.
    fn error_type(&self) -> &'static str;

    /// Returns an optional application-specific error code.
    ///
    /// This can be used for more granular error identification within
    /// your application's error taxonomy.
    fn error_code(&self) -> Option<String> {
        None
    }

    /// Returns optional metadata associated with this error.
    ///
    /// This can include additional context, validation details,
    /// or any other relevant information that might help clients
    /// understand and handle the error.
    fn metadata(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Standard error response format used throughout the framework.
///
/// This struct provides a consistent JSON structure for error responses,
/// ensuring that clients can reliably parse and handle errors.
///
/// # Fields
///
/// - `error`: Human-readable error name (typically the HTTP status reason phrase)
/// - `message`: Detailed error message describing what went wrong
/// - `status`: HTTP status code as a number
/// - `error_type`: Machine-readable error type identifier (optional)
/// - `error_code`: Application-specific error code (optional)
/// - `metadata`: Additional error context as JSON (optional)
/// - `timestamp`: ISO 8601 timestamp when the error occurred (optional)
///
/// # JSON Example
///
/// ```
/// {
///   "error": "Bad Request",
///   "message": "Validation failed: Email format is invalid",
///   "status": 400,
///   "error_type": "validation_error",
///   "error_code": "VALIDATION_001",
///   "metadata": {
///     "field": "email",
///     "expected_format": "email"
///   },
///   "timestamp": "2023-12-07T10:30:00Z"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Human-readable error name (e.g., "Bad Request", "Not Found")
    pub error: String,
    /// Detailed error message describing what went wrong
    pub message: String,
    /// HTTP status code as a number
    pub status: u16,
    /// Machine-readable error type identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    /// Application-specific error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Additional error context as JSON
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 timestamp when the error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl Error {
    /// Returns the HTTP status code that should be used for this error.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    /// use http::StatusCode;
    ///
    /// let error = Error::not_found("/api/users/123");
    /// assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    /// ```
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            Error::BadRequest(_) | Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            Error::Database(_) | Error::ExternalService(_) => StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(feature = "tls")]
            #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
            Error::Tls(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Custom(custom) => custom.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns a machine-readable error type identifier.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::validation("Invalid email");
    /// assert_eq!(error.error_type(), "validation_error");
    /// ```
    pub fn error_type(&self) -> &'static str {
        match self {
            Error::NotFound(_) => "not_found",
            Error::MethodNotAllowed(_) => "method_not_allowed",
            Error::BadRequest(_) => "bad_request",
            Error::Unauthorized(_) => "unauthorized",
            Error::Forbidden(_) => "forbidden",
            Error::TooManyRequests(_) => "too_many_requests",
            Error::Validation(_) => "validation_error",
            Error::Database(_) => "database_error",
            Error::ExternalService(_) => "external_service_error",
            #[cfg(feature = "tls")]
            #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
            Error::Tls(_) => "tls_error",
            Error::Custom(custom) => custom.error_type(),
            _ => "internal_server_error",
        }
    }

    // Fast path constructors for common errors

    /// Creates a new "Not Found" error with the specified path.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::not_found("/api/users/123");
    /// ```
    #[inline]
    pub fn not_found(path: &str) -> Self {
        Error::NotFound(path.to_string())
    }

    /// Creates a new "Bad Request" error with the specified message.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::bad_request("Missing required field: email");
    /// ```
    #[inline]
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Error::BadRequest(msg.into())
    }

    /// Creates a new validation error with the specified message.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::validation("Password must be at least 8 characters");
    /// ```
    #[inline]
    pub fn validation(msg: impl Into<String>) -> Self {
        Error::Validation(msg.into())
    }

    /// Creates a new "Unauthorized" error with the specified message.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::unauthorized("Invalid API key provided");
    /// let error2 = Error::unauthorized("Authentication token has expired");
    /// ```
    #[inline]
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Error::Unauthorized(msg.into())
    }

    /// Creates a new "Forbidden" error with the specified message.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::forbidden("Insufficient permissions to access this resource");
    /// let error2 = Error::forbidden("Admin access required");
    /// ```
    #[inline]
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Error::Forbidden(msg.into())
    }

    /// Creates a new "Too Many Requests" error with the specified message.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::too_many_requests("Rate limit exceeded. Try again in 60 seconds");
    /// let error2 = Error::too_many_requests("API quota exceeded for this hour");
    /// ```
    #[inline]
    pub fn too_many_requests(msg: impl Into<String>) -> Self {
        Error::TooManyRequests(msg.into())
    }

    /// Creates a new "Internal Server Error" with the specified message.
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::internal("Database connection failed");
    /// ```
    #[inline]
    pub fn internal(msg: impl Into<String>) -> Self {
        Error::Internal(msg.into())
    }

    /// Converts this error to a standardized error response.
    ///
    /// # Parameters
    ///
    /// - `include_timestamp`: Whether to include a timestamp in the response
    ///
    /// # Example
    /// ```
    /// use ignitia::Error;
    ///
    /// let error = Error::validation("Invalid email format");
    /// let response = error.to_response(true);
    /// assert_eq!(response.status, 400);
    /// assert_eq!(response.error_type, Some("validation_error".to_string()));
    /// ```
    pub fn to_response(&self, include_timestamp: bool) -> ErrorResponse {
        let status = self.status_code();

        ErrorResponse {
            error: status
                .canonical_reason()
                .unwrap_or("Unknown Error")
                .to_string(),
            message: self.to_string(),
            status: status.as_u16(),
            error_type: Some(self.error_type().to_string()),
            error_code: match self {
                Error::Custom(custom) => custom.error_code(),
                _ => None,
            },
            metadata: match self {
                Error::Custom(custom) => custom.metadata(),
                _ => None,
            },
            timestamp: if include_timestamp {
                Some(chrono::Utc::now().to_rfc3339())
            } else {
                None
            },
        }
    }
}

/// Type alias for `Result<T, Error>`.
///
/// This provides a convenient shorthand for functions that return
/// the framework's error type.
///
/// # Example
/// ```
/// use ignitia::Result;
///
/// fn validate_email(email: &str) -> Result<String> {
///     if email.contains('@') {
///         Ok(email.to_string())
///     } else {
///         Err(Error::validation("Invalid email format"))
///     }
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Macro for easily defining custom error enums with automatic trait implementations.
///
/// This macro generates a custom error enum with automatic implementations of
/// `Display`, `CustomError`, and conversion to the framework's `Error` type.
///
/// # Syntax
/// ```
/// define_error! {
///     ErrorName {
///         Variant1(StatusCode, "error_type"),
///         Variant2(StatusCode, "error_type", "error_code"),
///     }
/// }
/// ```
///
/// # Example
/// ```
/// use ignitia::define_error;
/// use http::StatusCode;
///
/// define_error! {
///     UserError {
///         InvalidCredentials(StatusCode::UNAUTHORIZED, "invalid_credentials", "AUTH_001"),
///         AccountLocked(StatusCode::FORBIDDEN, "account_locked", "AUTH_002"),
///         ProfileNotFound(StatusCode::NOT_FOUND, "profile_not_found"),
///     }
/// }
///
/// // Usage
/// let error = UserError::InvalidCredentials("Wrong password".to_string());
/// let framework_error: Error = error.into();
/// ```
#[macro_export]
macro_rules! define_error {
    (
        $name:ident {
            $($variant:ident($status:expr, $error_type:expr $(, $code:expr)?)),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub enum $name {
            $($variant(String)),*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant(msg) => write!(f, "{}", msg)),*
                }
            }
        }

        impl $crate::error::CustomError for $name {
            fn status_code(&self) -> http::StatusCode {
                match self {
                    $(Self::$variant(_) => $status),*
                }
            }

            fn error_type(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => $error_type),*
                }
            }

            fn error_code(&self) -> Option<String> {
                match self {
                    $(
                        Self::$variant(_) => {
                            #[allow(unused_variables)]
                            let code: Option<&str> = None;
                            $(let code = Some($code);)?
                            code.map(String::from)
                        }
                    )*
                }
            }
        }

        impl From<$name> for $crate::error::Error {
            fn from(err: $name) -> Self {
                $crate::error::Error::Custom(Box::new(err))
            }
        }
    };
}

/// Helper trait for easy error conversion from standard `Result` types.
///
/// This trait provides convenient methods to convert standard library
/// and third-party errors into the framework's error types.
///
/// # Example
/// ```
/// use ignitia::{ErrorExt, Result};
///
/// fn parse_number(s: &str) -> Result<i32> {
///     s.parse().validation_error()
/// }
///
/// fn read_file(path: &str) -> Result<String> {
///     std::fs::read_to_string(path).internal_error()
/// }
/// ```
pub trait ErrorExt<T> {
    /// Converts the error to a "Bad Request" error.
    fn bad_request(self) -> Result<T>;
    /// Converts the error to an "Unauthorized" error.
    fn unauthorized(self) -> Result<T>;
    /// Converts the error to a "Forbidden" error.
    fn forbidden(self) -> Result<T>;
    /// Converts the error to an "Internal Server Error".
    fn internal_error(self) -> Result<T>;
    /// Converts the error to a validation error.
    fn validation_error(self) -> Result<T>;
}

impl<T, E> ErrorExt<T> for std::result::Result<T, E>
where
    E: fmt::Display,
{
    /// Converts any error to a "Bad Request" error.
    ///
    /// # Example
    /// ```
    /// use ignitia::ErrorExt;
    ///
    /// let result: Result<i32, _> = "not_a_number".parse().bad_request();
    /// ```
    fn bad_request(self) -> Result<T> {
        self.map_err(|e| Error::bad_request(e.to_string()))
    }

    /// Converts any error to an "Unauthorized" error.
    ///
    /// # Example
    /// ```
    /// use ignitia::ErrorExt;
    ///
    /// let result = authenticate_user().unauthorized();
    /// ```
    fn unauthorized(self) -> Result<T> {
        self.map_err(|e| Error::unauthorized(e.to_string()))
    }

    /// Converts any error to a "Forbidden" error.
    ///
    /// # Example
    /// ```
    /// use ignitia::ErrorExt;
    ///
    /// let result = check_permissions().forbidden();
    /// ```
    fn forbidden(self) -> Result<T> {
        self.map_err(|e| Error::forbidden(e.to_string()))
    }

    /// Converts any error to an "Internal Server Error".
    ///
    /// # Example
    /// ```
    /// use ignitia::ErrorExt;
    ///
    /// let result = database_operation().internal_error();
    /// ```
    fn internal_error(self) -> Result<T> {
        self.map_err(|e| Error::internal(e.to_string()))
    }

    /// Converts any error to a validation error.
    ///
    /// # Example
    /// ```
    /// use ignitia::ErrorExt;
    ///
    /// let result: Result<i32, _> = "not_a_number".parse().validation_error();
    /// ```
    fn validation_error(self) -> Result<T> {
        self.map_err(|e| Error::validation(e.to_string()))
    }
}

/// Trait for custom error handlers that process errors before sending responses.
///
/// This trait allows applications to customize how errors are processed,
/// logged, or transformed before being sent to clients.
///
/// # Example
/// ```
/// use ignitia::{ErrorHandler, Error, Response};
///
/// struct CustomErrorHandler;
///
/// impl ErrorHandler for CustomErrorHandler {
///     fn handle_error(&self, error: Error, req: Option<&Request>) -> Response {
///         // Custom error processing logic
///         eprintln!("Error occurred: {}", error);
///         Response::from(error)
///     }
/// }
/// ```
pub trait ErrorHandler: Send + Sync + 'static {
    /// Handles an error and returns an HTTP response.
    ///
    /// # Parameters
    /// - `error`: The error that occurred
    /// - `req`: Optional reference to the original request
    fn handle_error(&self, error: Error, req: Option<&Request>) -> Response;
}

// Implementation for function pointers
impl<F> ErrorHandler for F
where
    F: Fn(Error) -> Response + Send + Sync + 'static,
{
    fn handle_error(&self, error: Error, _req: Option<&Request>) -> Response {
        self(error)
    }
}

/// Trait for error handlers that need access to the original request.
///
/// This trait is similar to `ErrorHandler` but requires access to the
/// original request, which can be useful for context-aware error handling.
///
/// # Example
/// ```
/// use ignitia::{ErrorHandlerWithRequest, Error, Request, Response};
///
/// struct ContextAwareErrorHandler;
///
/// impl ErrorHandlerWithRequest for ContextAwareErrorHandler {
///     fn handle_error_with_request(&self, error: Error, req: &Request) -> Response {
///         // Use request context for error handling
///         let user_agent = req.header("user-agent").unwrap_or("unknown");
///         eprintln!("Error for {}: {}", user_agent, error);
///         Response::from(error)
///     }
/// }
/// ```
pub trait ErrorHandlerWithRequest: Send + Sync + 'static {
    /// Handles an error with access to the original request.
    ///
    /// # Parameters
    /// - `error`: The error that occurred
    /// - `req`: Reference to the original request
    fn handle_error_with_request(&self, error: Error, req: &Request) -> Response;
}

impl<F> ErrorHandlerWithRequest for F
where
    F: Fn(Error, &Request) -> Response + Send + Sync + 'static,
{
    fn handle_error_with_request(&self, error: Error, req: &Request) -> Response {
        self(error, req)
    }
}

/// Wrapper enum to unify both error handler types.
///
/// This enum allows the framework to work with both simple error handlers
/// and request-aware error handlers in a unified way.
#[derive(Clone)]
pub enum ErrorHandlerType {
    /// Simple error handler that doesn't need request context
    Simple(Arc<dyn ErrorHandler>),
    /// Error handler that requires access to the original request
    WithRequest(Arc<dyn ErrorHandlerWithRequest>),
}

impl ErrorHandlerType {
    /// Handles an error using the appropriate handler type.
    ///
    /// # Parameters
    /// - `error`: The error to handle
    /// - `req`: Optional reference to the original request
    ///
    /// # Returns
    /// An HTTP response generated by the error handler
    pub fn handle(&self, error: Error, req: Option<&Request>) -> Response {
        match self {
            ErrorHandlerType::Simple(handler) => handler.handle_error(error, req),
            ErrorHandlerType::WithRequest(handler) => {
                if let Some(req) = req {
                    handler.handle_error_with_request(error, req)
                } else {
                    // Fallback to default error handling if no request available
                    Response::from(error)
                }
            }
        }
    }
}
