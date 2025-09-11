//! # Multipart Form Data Support
//!
//! This module provides comprehensive support for parsing multipart/form-data requests,
//! commonly used for file uploads and form submissions containing binary data.
//!
//! ## Features
//!
//! - **Streaming Parser**: Memory-efficient parsing of large multipart requests
//! - **Automatic File Handling**: Large files are automatically written to temporary files
//! - **Configurable Limits**: Protection against DoS attacks with customizable size limits
//! - **Type-Safe Extraction**: Integration with Ignitia's request extraction system
//! - **Async Support**: Fully asynchronous parsing for non-blocking I/O
//!
//! ## Usage
//!
//! ### Basic Usage
//!
//! ```
//! use ignitia::{Multipart, Response, Result};
//!
//! async fn upload_handler(mut multipart: Multipart) -> Result<Response> {
//!     while let Some(field) = multipart.next_field().await? {
//!         if field.is_file() {
//!             // Handle file upload
//!             let file_name = field.file_name().unwrap_or("unknown");
//!             let bytes = field.bytes().await?;
//!             println!("Uploaded file: {} ({} bytes)", file_name, bytes.len());
//!         } else {
//!             // Handle text field
//!             let text = field.text().await?;
//!             println!("Field {}: {}", field.name(), text);
//!         }
//!     }
//!
//!     Ok(Response::text("Upload successful"))
//! }
//! ```
//!
//! ### Configuration
//!
//! ```
//! use ignitia::multipart::{Multipart, MultipartConfig};
//!
//! let config = MultipartConfig {
//!     max_request_size: 50 * 1024 * 1024, // 50MB
//!     max_field_size: 10 * 1024 * 1024,   // 10MB per field
//!     file_size_threshold: 1024 * 1024,   // 1MB before writing to disk
//!     max_fields: 50,                     // Maximum 50 fields
//! };
//! ```
//!
//! ## Security Considerations
//!
//! - **Size Limits**: Always configure appropriate size limits for your use case
//! - **File Validation**: Validate file types and contents before processing
//! - **Temporary Files**: Temporary files are automatically cleaned up when dropped
//! - **Memory Usage**: Large files are automatically streamed to disk to prevent memory exhaustion

pub mod error;
pub mod field;
pub mod parser;

pub use error::{MultipartError, MultipartRejection};
pub use field::{Field, FileField, TextField};
pub use parser::Multipart;

use crate::handler::extractor::FromRequest;
use crate::{Request, Result};

// Re-export for convenience
pub use bytes::Bytes;
pub use tempfile::NamedTempFile;

/// Configuration options for multipart form parsing.
///
/// This struct allows you to customize the behavior of the multipart parser,
/// including setting limits to protect against denial-of-service attacks and
/// configuring memory vs. disk usage thresholds.
///
/// # Examples
///
/// ```
/// use ignitia::multipart::MultipartConfig;
///
/// // Create a restrictive configuration for public APIs
/// let config = MultipartConfig {
///     max_request_size: 5 * 1024 * 1024,   // 5MB total
///     max_field_size: 1 * 1024 * 1024,     // 1MB per field
///     file_size_threshold: 512 * 1024,     // 512KB before disk
///     max_fields: 20,                      // Maximum 20 fields
/// };
///
/// // Create a permissive configuration for internal services
/// let internal_config = MultipartConfig {
///     max_request_size: 100 * 1024 * 1024, // 100MB total
///     max_field_size: 50 * 1024 * 1024,    // 50MB per field
///     file_size_threshold: 2 * 1024 * 1024, // 2MB before disk
///     max_fields: 100,                     // Maximum 100 fields
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MultipartConfig {
    /// Maximum size for the entire multipart request in bytes.
    ///
    /// If the total size of all fields exceeds this limit, parsing will fail
    /// with a `MultipartError::RequestTooLarge` error.
    ///
    /// **Default**: 10MB (10,485,760 bytes)
    pub max_request_size: usize,

    /// Maximum size for individual fields kept in memory in bytes.
    ///
    /// Fields larger than this limit will cause parsing to fail with a
    /// `MultipartError::FieldTooLarge` error. This prevents any single
    /// field from consuming excessive memory.
    ///
    /// **Default**: 1MB (1,048,576 bytes)
    pub max_field_size: usize,

    /// Size threshold for writing files to disk instead of keeping them in memory.
    ///
    /// Fields larger than this size will be automatically written to temporary
    /// files instead of being kept in memory. This helps manage memory usage
    /// for large file uploads.
    ///
    /// **Default**: 256KB (262,144 bytes)
    pub file_size_threshold: usize,

    /// Maximum number of fields allowed in a single multipart request.
    ///
    /// This prevents attacks that send many small fields to consume server
    /// resources. Parsing will fail with `MultipartError::TooManyFields`
    /// if this limit is exceeded.
    ///
    /// **Default**: 100 fields
    pub max_fields: usize,
}

impl Default for MultipartConfig {
    /// Creates a default multipart configuration with sensible limits.
    ///
    /// The default configuration is designed to be secure for most web applications
    /// while allowing reasonable file upload sizes.
    ///
    /// # Default Values
    ///
    /// - `max_request_size`: 10MB
    /// - `max_field_size`: 1MB
    /// - `file_size_threshold`: 256KB
    /// - `max_fields`: 100
    fn default() -> Self {
        Self {
            max_request_size: 10 * 1024 * 1024, // 10MB
            max_field_size: 1 * 1024 * 1024,    // 1MB
            file_size_threshold: 256 * 1024,    // 256KB
            max_fields: 100,
        }
    }
}

impl MultipartConfig {
    /// Creates a new multipart configuration with the given maximum request size.
    ///
    /// Other values are set to defaults proportional to the request size.
    ///
    /// # Arguments
    ///
    /// * `max_request_size` - Maximum total size of the multipart request in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartConfig;
    ///
    /// let config = MultipartConfig::with_max_size(50 * 1024 * 1024); // 50MB
    /// ```
    pub fn with_max_size(max_request_size: usize) -> Self {
        Self {
            max_request_size,
            max_field_size: (max_request_size / 10).max(1024 * 1024), // 10% or 1MB minimum
            file_size_threshold: (max_request_size / 40).max(256 * 1024), // 2.5% or 256KB minimum
            max_fields: 100,
        }
    }

    /// Creates a configuration suitable for large file uploads.
    ///
    /// This configuration allows larger files and sets a higher threshold
    /// before writing to disk, making it suitable for applications that
    /// need to handle large file uploads.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartConfig;
    ///
    /// let config = MultipartConfig::large_files();
    /// assert_eq!(config.max_request_size, 100 * 1024 * 1024); // 100MB
    /// ```
    pub fn large_files() -> Self {
        Self {
            max_request_size: 100 * 1024 * 1024,  // 100MB
            max_field_size: 50 * 1024 * 1024,     // 50MB
            file_size_threshold: 2 * 1024 * 1024, // 2MB
            max_fields: 50,
        }
    }

    /// Creates a restrictive configuration suitable for public APIs.
    ///
    /// This configuration has smaller limits to protect against abuse
    /// in public-facing applications where upload sizes should be limited.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartConfig;
    ///
    /// let config = MultipartConfig::restrictive();
    /// assert_eq!(config.max_request_size, 2 * 1024 * 1024); // 2MB
    /// ```
    pub fn restrictive() -> Self {
        Self {
            max_request_size: 2 * 1024 * 1024, // 2MB
            max_field_size: 512 * 1024,        // 512KB
            file_size_threshold: 128 * 1024,   // 128KB
            max_fields: 20,
        }
    }

    /// Sets the maximum request size.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum total size in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartConfig;
    ///
    /// let config = MultipartConfig::default()
    ///     .max_request_size(20 * 1024 * 1024); // 20MB
    /// ```
    pub fn max_request_size(mut self, size: usize) -> Self {
        self.max_request_size = size;
        self
    }

    /// Sets the maximum field size.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum field size in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartConfig;
    ///
    /// let config = MultipartConfig::default()
    ///     .max_field_size(5 * 1024 * 1024); // 5MB per field
    /// ```
    pub fn max_field_size(mut self, size: usize) -> Self {
        self.max_field_size = size;
        self
    }

    /// Sets the file size threshold for writing to disk.
    ///
    /// # Arguments
    ///
    /// * `size` - Threshold size in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartConfig;
    ///
    /// let config = MultipartConfig::default()
    ///     .file_size_threshold(1024 * 1024); // 1MB threshold
    /// ```
    pub fn file_size_threshold(mut self, size: usize) -> Self {
        self.file_size_threshold = size;
        self
    }

    /// Sets the maximum number of fields.
    ///
    /// # Arguments
    ///
    /// * `count` - Maximum number of fields
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartConfig;
    ///
    /// let config = MultipartConfig::default()
    ///     .max_fields(50); // Maximum 50 fields
    /// ```
    pub fn max_fields(mut self, count: usize) -> Self {
        self.max_fields = count;
        self
    }
}

impl FromRequest for Multipart {
    /// Extracts multipart data from an HTTP request.
    ///
    /// This implementation automatically checks for the correct Content-Type header
    /// and extracts the boundary parameter required for parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request doesn't have a Content-Type header
    /// - The Content-Type is not `multipart/form-data`
    /// - The boundary parameter is missing from the Content-Type header
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Multipart, Response, Result};
    ///
    /// async fn handler(multipart: Multipart) -> Result<Response> {
    ///     // The multipart data is automatically extracted from the request
    ///     Ok(Response::text("Received multipart data"))
    /// }
    /// ```
    fn from_request(req: &Request) -> Result<Self> {
        // Check content type
        let content_type = req
            .header("content-type")
            .ok_or_else(|| crate::Error::BadRequest("Missing Content-Type header".into()))?;

        if !content_type.starts_with("multipart/form-data") {
            return Err(crate::Error::BadRequest(
                "Request is not multipart/form-data".into(),
            ));
        }

        // Extract boundary
        let boundary = extract_boundary(content_type)
            .ok_or_else(|| crate::Error::BadRequest("Missing boundary in Content-Type".into()))?;

        Ok(Multipart::new(
            req.body.clone(),
            boundary,
            MultipartConfig::default(),
        ))
    }
}

/// Extracts the boundary parameter from a Content-Type header.
///
/// The boundary is used to separate different parts of the multipart data.
/// It's typically specified as `boundary=something` in the Content-Type header.
///
/// # Arguments
///
/// * `content_type` - The Content-Type header value
///
/// # Returns
///
/// The boundary string without quotes, or `None` if not found.
///
/// # Examples
///
/// ```
/// use ignitia::multipart::extract_boundary;
///
/// let ct = "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW";
/// assert_eq!(extract_boundary(ct), Some("----WebKitFormBoundary7MA4YWxkTrZu0gW".to_string()));
/// ```
fn extract_boundary(content_type: &str) -> Option<String> {
    content_type.split(';').find_map(|part| {
        let part = part.trim();
        if part.starts_with("boundary=") {
            Some(part[9..].trim_matches('"').to_string())
        } else {
            None
        }
    })
}
