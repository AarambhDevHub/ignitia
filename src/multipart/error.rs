//! # Multipart Error Types
//!
//! This module defines error types specific to multipart form data parsing.
//! All errors implement the necessary traits for integration with Ignitia's
//! error handling system.
//!
//! ## Error Categories
//!
//! - **Validation Errors**: Invalid boundaries, field formats, etc.
//! - **Size Limit Errors**: Field or request size exceeded configured limits
//! - **I/O Errors**: File system operations and network errors
//! - **Parsing Errors**: Malformed multipart data
//!
//! ## Error Handling
//!
//! ```
//! use ignitia::multipart::{Multipart, MultipartError};
//!
//! async fn handle_upload(mut multipart: Multipart) {
//!     match multipart.next_field().await {
//!         Ok(Some(field)) => {
//!             // Process field
//!         }
//!         Ok(None) => {
//!             // No more fields
//!         }
//!         Err(MultipartError::FieldTooLarge { field_name, max_size }) => {
//!             eprintln!("Field '{}' exceeds {} bytes", field_name, max_size);
//!         }
//!         Err(err) => {
//!             eprintln!("Multipart error: {}", err);
//!         }
//!     }
//! }
//! ```

use crate::error::CustomError;
use http::StatusCode;
use std::fmt;

/// Errors that can occur during multipart form data parsing.
///
/// This enum covers all possible error conditions that may arise when
/// processing multipart/form-data requests, from validation issues to
/// resource limit violations.
///
/// # Error Categories
///
/// ## Validation Errors
/// - `InvalidBoundary`: The boundary parameter is malformed
/// - `InvalidField`: A field has invalid format or headers
/// - `InvalidUtf8`: Text field contains invalid UTF-8 sequences
///
/// ## Size Limit Errors
/// - `FieldTooLarge`: An individual field exceeds the configured size limit
/// - `TooManyFields`: The request contains more fields than allowed
/// - `RequestTooLarge`: The entire request exceeds the size limit
///
/// ## System Errors
/// - `Io`: File system or network I/O errors
/// - `Parse`: Low-level parsing errors from the multipart library
///
/// # Examples
///
/// ```
/// use ignitia::multipart::MultipartError;
///
/// match error {
///     MultipartError::FieldTooLarge { field_name, max_size } => {
///         println!("Field '{}' is too large (max: {} bytes)", field_name, max_size);
///     }
///     MultipartError::TooManyFields { max_fields } => {
///         println!("Too many fields (max: {})", max_fields);
///     }
///     MultipartError::InvalidBoundary => {
///         println!("Invalid multipart boundary");
///     }
///     _ => {
///         println!("Other multipart error: {}", error);
///     }
/// }
/// ```
#[derive(Debug)]
pub enum MultipartError {
    /// The multipart boundary parameter is invalid or malformed.
    ///
    /// This occurs when the Content-Type header contains an invalid
    /// boundary parameter, or when the boundary is not properly formatted
    /// according to RFC 2046 specifications.
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 400 Bad Request.
    InvalidBoundary,

    /// A field exceeds the maximum allowed size.
    ///
    /// This error is thrown when an individual field's content exceeds
    /// the `max_field_size` limit configured in `MultipartConfig`.
    ///
    /// # Fields
    ///
    /// - `field_name`: The name of the field that exceeded the limit
    /// - `max_size`: The maximum allowed size in bytes
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 413 Payload Too Large.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartError;
    ///
    /// let error = MultipartError::FieldTooLarge {
    ///     field_name: "upload_file".to_string(),
    ///     max_size: 1024 * 1024, // 1MB
    /// };
    /// ```
    FieldTooLarge {
        /// Name of the field that exceeded the size limit
        field_name: String,
        /// Maximum allowed size in bytes
        max_size: usize,
    },

    /// The request contains too many fields.
    ///
    /// This error occurs when the number of fields in the multipart request
    /// exceeds the `max_fields` limit configured in `MultipartConfig`.
    ///
    /// # Fields
    ///
    /// - `max_fields`: The maximum number of fields allowed
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 413 Payload Too Large.
    TooManyFields {
        /// Maximum number of fields allowed
        max_fields: usize,
    },

    /// The entire request size exceeds the configured limit.
    ///
    /// This error is thrown when the total size of all fields combined
    /// exceeds the `max_request_size` limit configured in `MultipartConfig`.
    ///
    /// # Fields
    ///
    /// - `max_size`: The maximum allowed request size in bytes
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 413 Payload Too Large.
    RequestTooLarge {
        /// Maximum allowed request size in bytes
        max_size: usize,
    },

    /// A field has invalid format or headers.
    ///
    /// This error occurs when a multipart field is malformed, has missing
    /// required headers, or contains invalid header values.
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 400 Bad Request.
    InvalidField(String),

    /// An I/O error occurred during parsing.
    ///
    /// This wraps standard I/O errors that may occur when reading from
    /// the request stream, writing temporary files, or performing other
    /// file system operations.
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 400 Bad Request (though it could indicate
    /// a server-side issue in some cases).
    Io(std::io::Error),

    /// A text field contains invalid UTF-8 sequences.
    ///
    /// This error occurs when attempting to parse a field as text, but
    /// the field content is not valid UTF-8.
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 400 Bad Request.
    InvalidUtf8(std::str::Utf8Error),

    /// A low-level parsing error occurred.
    ///
    /// This wraps errors from the underlying multipart parsing library,
    /// typically indicating malformed multipart data that doesn't conform
    /// to RFC standards.
    ///
    /// # HTTP Status
    ///
    /// This error maps to HTTP 400 Bad Request.
    Parse(String),
}

impl fmt::Display for MultipartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultipartError::InvalidBoundary => {
                write!(f, "Invalid multipart boundary")
            }
            MultipartError::FieldTooLarge {
                field_name,
                max_size,
            } => {
                write!(
                    f,
                    "Field '{}' exceeds maximum size of {} bytes",
                    field_name, max_size
                )
            }
            MultipartError::TooManyFields { max_fields } => {
                write!(f, "Too many fields, maximum allowed: {}", max_fields)
            }
            MultipartError::RequestTooLarge { max_size } => {
                write!(f, "Request exceeds maximum size of {} bytes", max_size)
            }
            MultipartError::InvalidField(msg) => {
                write!(f, "Invalid field: {}", msg)
            }
            MultipartError::Io(err) => {
                write!(f, "I/O error: {}", err)
            }
            MultipartError::InvalidUtf8(err) => {
                write!(f, "Invalid UTF-8: {}", err)
            }
            MultipartError::Parse(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for MultipartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MultipartError::Io(err) => Some(err),
            MultipartError::InvalidUtf8(err) => Some(err),
            _ => None,
        }
    }
}

impl CustomError for MultipartError {
    /// Returns the appropriate HTTP status code for this error.
    ///
    /// Size-related errors return 413 Payload Too Large, while
    /// validation and parsing errors return 400 Bad Request.
    fn status_code(&self) -> StatusCode {
        match self {
            MultipartError::FieldTooLarge { .. }
            | MultipartError::TooManyFields { .. }
            | MultipartError::RequestTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    /// Returns a string identifier for the error type.
    ///
    /// This is used in structured error responses to help clients
    /// programmatically identify and handle different error types.
    fn error_type(&self) -> &'static str {
        match self {
            MultipartError::InvalidBoundary => "invalid_boundary",
            MultipartError::FieldTooLarge { .. } => "field_too_large",
            MultipartError::TooManyFields { .. } => "too_many_fields",
            MultipartError::RequestTooLarge { .. } => "request_too_large",
            MultipartError::InvalidField(_) => "invalid_field",
            MultipartError::Io(_) => "io_error",
            MultipartError::InvalidUtf8(_) => "invalid_utf8",
            MultipartError::Parse(_) => "parse_error",
        }
    }

    /// Returns additional metadata for structured error responses.
    ///
    /// For size-related errors, this includes the limits that were exceeded.
    fn metadata(&self) -> Option<serde_json::Value> {
        match self {
            MultipartError::FieldTooLarge {
                field_name,
                max_size,
            } => Some(serde_json::json!({
                "field_name": field_name,
                "max_size": max_size,
                "limit_type": "field_size"
            })),
            MultipartError::TooManyFields { max_fields } => Some(serde_json::json!({
                "max_fields": max_fields,
                "limit_type": "field_count"
            })),
            MultipartError::RequestTooLarge { max_size } => Some(serde_json::json!({
                "max_size": max_size,
                "limit_type": "request_size"
            })),
            _ => None,
        }
    }
}

impl From<MultipartError> for crate::Error {
    /// Converts a MultipartError into the framework's main Error type.
    ///
    /// This allows multipart errors to be seamlessly integrated with
    /// Ignitia's error handling and response generation system.
    fn from(err: MultipartError) -> Self {
        crate::Error::Custom(Box::new(err))
    }
}

impl From<std::io::Error> for MultipartError {
    /// Converts standard I/O errors into MultipartError::Io.
    ///
    /// This provides automatic conversion for file system and network
    /// errors that may occur during multipart processing.
    fn from(err: std::io::Error) -> Self {
        MultipartError::Io(err)
    }
}

impl From<std::str::Utf8Error> for MultipartError {
    /// Converts UTF-8 validation errors into MultipartError::InvalidUtf8.
    ///
    /// This provides automatic conversion when attempting to parse
    /// binary field data as UTF-8 text.
    fn from(err: std::str::Utf8Error) -> Self {
        MultipartError::InvalidUtf8(err)
    }
}

/// Rejection type for multipart extraction failures.
///
/// This type is returned when multipart data extraction fails during
/// request processing. It wraps the underlying `MultipartError` and
/// provides additional context for debugging.
///
/// # Usage
///
/// This type is primarily used internally by the framework's request
/// extraction system. Application code typically works with `MultipartError`
/// directly rather than this rejection wrapper.
///
/// # Examples
///
/// ```
/// use ignitia::multipart::{MultipartRejection, MultipartError};
///
/// let rejection = MultipartRejection(MultipartError::InvalidBoundary);
/// println!("Extraction failed: {}", rejection);
/// ```
#[derive(Debug)]
pub struct MultipartRejection(pub MultipartError);

impl fmt::Display for MultipartRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Multipart extraction failed: {}", self.0)
    }
}

impl std::error::Error for MultipartRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<MultipartError> for MultipartRejection {
    /// Converts a MultipartError into a MultipartRejection.
    fn from(err: MultipartError) -> Self {
        MultipartRejection(err)
    }
}
