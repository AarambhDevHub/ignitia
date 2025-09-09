//! # HTTP Response Builder Module
//!
//! This module provides a flexible builder pattern for constructing HTTP responses in the Ignitia
//! web framework. The ResponseBuilder allows for fluent, method-chained response construction with
//! comprehensive header management, content type handling, and flexible body assignment.
//!
//! ## Features
//!
//! - **Fluent Builder Pattern**: Method chaining for readable response construction
//! - **Multiple Content Types**: Support for JSON, HTML, text, and binary responses
//! - **Header Management**: Easy header manipulation with type safety
//! - **Status Code Handling**: Support for both enum and numeric status codes
//! - **Error Handling**: Comprehensive error handling for serialization and header validation
//!
//! ## Usage Examples
//!
//! ### Basic Builder Usage
//! ```
//! use ignitia::ResponseBuilder;
//! use http::StatusCode;
//!
//! // Simple text response
//! let response = ResponseBuilder::new()
//!     .status(StatusCode::OK)
//!     .text("Hello, World!")
//!     .build();
//!
//! // Custom status code with numeric value
//! let response = ResponseBuilder::new()
//!     .status_code(201)
//!     .text("Resource created")
//!     .build();
//! ```
//!
//! ### JSON Response Building
//! ```
//! use ignitia::ResponseBuilder;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let data = json!({
//!     "success": true,
//!     "data": {
//!         "id": 123,
//!         "name": "Example Item"
//!     },
//!     "timestamp": "2023-01-01T12:00:00Z"
//! });
//!
//! let response = ResponseBuilder::new()
//!     .status_code(200)
//!     .json(&data)?
//!     .build();
//! # Ok(())
//! # }
//! ```
//!
//! ### Custom Headers and Advanced Building
//! ```
//! use ignitia::ResponseBuilder;
//! use http::{HeaderName, HeaderValue, StatusCode};
//!
//! let response = ResponseBuilder::new()
//!     .status(StatusCode::OK)
//!     .header("X-Custom-Header", "custom-value")
//!     .header("Cache-Control", "no-cache, no-store")
//!     .header("X-RateLimit-Remaining", "99")
//!     .html("<h1>Custom Response</h1>")
//!     .build();
//! ```
//!
//! ## Advanced Usage Patterns
//!
//! ### API Response Builder
//! ```
//! use ignitia::{ResponseBuilder, Result};
//! use serde::Serialize;
//! use http::StatusCode;
//!
//! #[derive(Serialize)]
//! struct ApiResponse<T> {
//!     success: bool,
//!     data: Option<T>,
//!     message: String,
//!     timestamp: String,
//! }
//!
//! fn build_api_response<T: Serialize>(
//!     data: Option<T>,
//!     message: impl Into<String>,
//!     status: StatusCode,
//! ) -> Result<ignitia::Response> {
//!     let api_response = ApiResponse {
//!         success: status.is_success(),
//!         data,
//!         message: message.into(),
//!         timestamp: chrono::Utc::now().to_rfc3339(),
//!     };
//!
//!     ResponseBuilder::new()
//!         .status(status)
//!         .header("X-API-Version", "1.0")
//!         .json(&api_response)
//!         .map(|builder| builder.build())
//! }
//! ```
//!
//! ### File Download Builder
//! ```
//! use ignitia::ResponseBuilder;
//! use bytes::Bytes;
//!
//! fn build_file_download(
//!     file_content: Bytes,
//!     filename: &str,
//!     content_type: &str,
//! ) -> ignitia::Response {
//!     ResponseBuilder::new()
//!         .status_code(200)
//!         .header("Content-Type", content_type)
//!         .header("Content-Disposition",
//!             format!("attachment; filename=\"{}\"", filename))
//!         .header("Content-Length", file_content.len().to_string())
//!         .header("Cache-Control", "no-cache")
//!         .body(file_content)
//!         .build()
//! }
//! ```
//!
//! ## Error Handling Patterns
//!
//! ### Safe JSON Building with Error Recovery
//! ```
//! use ignitia::{ResponseBuilder, Response};
//! use serde::Serialize;
//!
//! fn safe_json_response<T: Serialize>(data: T) -> ignitia::Response {
//!     match ResponseBuilder::new().json(&data) {
//!         Ok(builder) => builder.build(),
//!         Err(e) => {
//!             // Fallback to error response if JSON serialization fails
//!             ResponseBuilder::new()
//!                 .status_code(500)
//!                 .text(format!("JSON serialization failed: {}", e))
//!                 .build()
//!         }
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Efficient Header Management
//! The builder uses HeaderMap internally for efficient header storage and manipulation:
//! - Headers are validated during insertion
//! - Invalid headers are silently ignored to prevent panics
//! - Memory is pre-allocated for common response sizes
//!
//! ### Builder Reuse Pattern
//! ```
//! use ignitia::ResponseBuilder;
//!
//! // Create a base builder for common response patterns
//! fn create_base_response() -> ResponseBuilder {
//!     ResponseBuilder::new()
//!         .header("X-API-Version", "1.0")
//!         .header("X-Powered-By", "Ignitia")
//! }
//!
//! // Use the base for specific responses
//! fn success_response(message: &str) -> ignitia::Response {
//!     create_base_response()
//!         .status_code(200)
//!         .text(message)
//!         .build()
//! }
//!
//! fn error_response(error: &str) -> ignitia::Response {
//!     create_base_response()
//!         .status_code(500)
//!         .text(error)
//!         .build()
//! }
//! ```

use super::Response;
use crate::error::Result;
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use serde::Serialize;

/// HTTP response builder providing fluent construction of responses.
///
/// The `ResponseBuilder` enables method chaining to construct responses with custom
/// status codes, headers, and body content. It provides type-safe header management
/// and supports multiple content formats including JSON, HTML, and plain text.
///
/// # Builder Pattern
/// All methods consume `self` and return `Self`, enabling fluent method chaining.
/// The builder must be finalized with the `build()` method to create a `Response`.
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::ResponseBuilder;
/// use http::StatusCode;
///
/// let response = ResponseBuilder::new()
///     .status(StatusCode::OK)
///     .text("Hello, World!")
///     .build();
/// ```
///
/// ## Method Chaining
/// ```
/// use ignitia::ResponseBuilder;
///
/// let response = ResponseBuilder::new()
///     .status_code(201)
///     .header("Location", "/users/123")
///     .header("X-Created-At", "2023-01-01T12:00:00Z")
///     .text("User created successfully")
///     .build();
/// ```
pub struct ResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
    body: Option<Bytes>,
}

impl ResponseBuilder {
    /// Creates a new ResponseBuilder with default values.
    ///
    /// The builder starts with:
    /// - Status: 200 OK
    /// - Headers: Empty HeaderMap
    /// - Body: None (will be empty when built)
    ///
    /// # Returns
    /// A new ResponseBuilder instance ready for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use http::StatusCode;
    ///
    /// let builder = ResponseBuilder::new();
    /// let response = builder.build();
    ///
    /// assert_eq!(response.status, StatusCode::OK);
    /// assert!(response.body.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: None,
        }
    }

    /// Sets the HTTP status code (builder pattern).
    ///
    /// This method consumes the builder and returns it with the updated status code,
    /// enabling fluent method chaining.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code to set
    ///
    /// # Returns
    /// The builder with the updated status code
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use http::StatusCode;
    ///
    /// let response = ResponseBuilder::new()
    ///     .status(StatusCode::CREATED)
    ///     .text("Resource created")
    ///     .build();
    ///
    /// assert_eq!(response.status, StatusCode::CREATED);
    /// ```
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Sets the HTTP status code using a numeric value (builder pattern).
    ///
    /// This is a convenience method that accepts a u16 status code. Invalid
    /// status codes are silently ignored to prevent panics.
    ///
    /// # Parameters
    /// - `status_code`: The numeric HTTP status code (e.g., 200, 404, 500)
    ///
    /// # Returns
    /// The builder with the updated status code (if valid)
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .status_code(404)
    ///     .text("Not Found")
    ///     .build();
    ///
    /// assert_eq!(response.status.as_u16(), 404);
    /// ```
    ///
    /// ## Invalid Status Codes
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use http::StatusCode;
    ///
    /// // Invalid status codes are ignored
    /// let response = ResponseBuilder::new()
    ///     .status_code(9999) // Invalid, ignored
    ///     .text("Test")
    ///     .build();
    ///
    /// // Status remains the default (200 OK)
    /// assert_eq!(response.status, StatusCode::OK);
    /// ```
    pub fn status_code(mut self, status_code: u16) -> Self {
        if let Ok(status) = StatusCode::from_u16(status_code) {
            self.status = status;
        }
        self
    }

    /// Sets the response body using raw bytes (builder pattern).
    ///
    /// This method accepts any type that can be converted to `Bytes` and sets
    /// it as the response body. Note that this will overwrite any existing body.
    ///
    /// # Parameters
    /// - `body`: The body content (String, &str, Vec<u8>, Bytes, etc.)
    ///
    /// # Returns
    /// The builder with the updated body
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use bytes::Bytes;
    ///
    /// // From string
    /// let response = ResponseBuilder::new()
    ///     .with_body("Hello, World!")
    ///     .build();
    ///
    /// // From bytes
    /// let data = Bytes::from("Binary data");
    /// let response = ResponseBuilder::new()
    ///     .with_body(data)
    ///     .build();
    ///
    /// // From vector
    /// let data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello"
    /// let response = ResponseBuilder::new()
    ///     .with_body(data)
    ///     .build();
    /// ```
    pub fn with_body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Adds an HTTP header (builder pattern).
    ///
    /// This method attempts to convert the key and value into valid HTTP header
    /// components. If the conversion fails (e.g., invalid characters), the header
    /// is silently ignored to prevent panics.
    ///
    /// # Type Parameters
    /// - `K`: Header name type (must implement `TryInto<HeaderName>`)
    /// - `V`: Header value type (must implement `TryInto<HeaderValue>`)
    ///
    /// # Parameters
    /// - `key`: The header name
    /// - `value`: The header value
    ///
    /// # Returns
    /// The builder with the added header (if valid)
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use http::{HeaderName, HeaderValue};
    ///
    /// let response = ResponseBuilder::new()
    ///     .header("Content-Type", "application/json")
    ///     .header("X-Custom-Header", "custom-value")
    ///     .header("Cache-Control", "max-age=3600")
    ///     .text("Hello")
    ///     .build();
    /// ```
    ///
    /// ## Type-Safe Headers
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use http::{HeaderName, HeaderValue};
    ///
    /// let header_name = HeaderName::from_static("x-request-id");
    /// let header_value = HeaderValue::from_static("12345");
    ///
    /// let response = ResponseBuilder::new()
    ///     .header(header_name, header_value)
    ///     .text("Request processed")
    ///     .build();
    /// ```
    ///
    /// ## Dynamic Headers
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// fn build_response_with_id(request_id: u64) -> ignitia::Response {
    ///     ResponseBuilder::new()
    ///         .header("X-Request-ID", request_id.to_string())
    ///         .header("X-Timestamp", chrono::Utc::now().timestamp().to_string())
    ///         .text("Response with metadata")
    ///         .build()
    /// }
    /// ```
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
        K::Error: std::fmt::Debug,
        V::Error: std::fmt::Debug,
    {
        if let (Ok(name), Ok(val)) = (key.try_into(), value.try_into()) {
            self.headers.insert(name, val);
        }
        self
    }

    /// Sets JSON body with automatic serialization and content-type header.
    ///
    /// This method serializes the provided data to JSON, sets the appropriate
    /// content-type header, and stores the result as the response body. It returns
    /// a Result because serialization can fail.
    ///
    /// # Type Parameters
    /// - `T`: The type to serialize (must implement `Serialize`)
    ///
    /// # Parameters
    /// - `data`: The data to serialize as JSON
    ///
    /// # Returns
    /// - `Ok(Self)`: Builder with JSON body and content-type header set
    /// - `Err(Error)`: JSON serialization error
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use serde::Serialize;
    /// use serde_json::json;
    ///
    /// #[derive(Serialize)]
    /// struct ApiResponse {
    ///     success: bool,
    ///     message: String,
    ///     data: serde_json::Value,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api_response = ApiResponse {
    ///     success: true,
    ///     message: "Operation completed".to_string(),
    ///     data: json!({"id": 123, "name": "Test"}),
    /// };
    ///
    /// let response = ResponseBuilder::new()
    ///     .status_code(200)
    ///     .json(&api_response)?
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Error Handling
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use serde_json::json;
    ///
    /// fn safe_json_builder(data: serde_json::Value) -> ignitia::Response {
    ///     match ResponseBuilder::new().json(&data) {
    ///         Ok(builder) => builder.build(),
    ///         Err(e) => {
    ///             ResponseBuilder::new()
    ///                 .status_code(500)
    ///                 .text(format!("Serialization error: {}", e))
    ///                 .build()
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// ## Complex Data Structures
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use serde::Serialize;
    /// use std::collections::HashMap;
    ///
    /// #[derive(Serialize)]
    /// struct PaginatedResponse<T> {
    ///     data: Vec<T>,
    ///     pagination: PaginationInfo,
    /// }
    ///
    /// #[derive(Serialize)]
    /// struct PaginationInfo {
    ///     page: u32,
    ///     per_page: u32,
    ///     total: u32,
    ///     pages: u32,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let users = vec!["Alice", "Bob", "Charlie"];
    /// let response_data = PaginatedResponse {
    ///     data: users,
    ///     pagination: PaginationInfo {
    ///         page: 1,
    ///         per_page: 10,
    ///         total: 3,
    ///         pages: 1,
    ///     },
    /// };
    ///
    /// let response = ResponseBuilder::new()
    ///     .header("X-Total-Count", "3")
    ///     .json(&response_data)?
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self> {
        let body = serde_json::to_vec(data)?;
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        self.body = Some(Bytes::from(body));
        Ok(self)
    }

    /// Sets plain text body with appropriate content-type header.
    ///
    /// This method sets the response body to UTF-8 encoded text and adds the
    /// appropriate content-type header with charset specification.
    ///
    /// # Type Parameters
    /// - `T`: Text type (must implement `Into<String>`)
    ///
    /// # Parameters
    /// - `text`: The text content to set as the body
    ///
    /// # Returns
    /// The builder with text body and content-type header set
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .status_code(200)
    ///     .text("Hello, World!")
    ///     .build();
    ///
    /// // Dynamic text generation
    /// let user_name = "Alice";
    /// let response = ResponseBuilder::new()
    ///     .text(format!("Welcome, {}!", user_name))
    ///     .build();
    /// ```
    ///
    /// ## Multi-line Text
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// let report = r#"
    /// System Status Report
    /// ===================
    ///
    /// CPU Usage: 45%
    /// Memory Usage: 78%
    /// Disk Usage: 23%
    ///
    /// All systems operational.
    /// "#;
    ///
    /// let response = ResponseBuilder::new()
    ///     .header("Content-Disposition", "attachment; filename=\"report.txt\"")
    ///     .text(report)
    ///     .build();
    /// ```
    pub fn text<T: Into<String>>(mut self, text: T) -> Self {
        let text = text.into();
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        self.body = Some(Bytes::from(text));
        self
    }

    /// Sets HTML body with appropriate content-type header.
    ///
    /// This method sets the response body to HTML content and adds the appropriate
    /// content-type header with UTF-8 charset specification.
    ///
    /// # Type Parameters
    /// - `T`: HTML type (must implement `Into<String>`)
    ///
    /// # Parameters
    /// - `html`: The HTML content to set as the body
    ///
    /// # Returns
    /// The builder with HTML body and content-type header set
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// let html = r#"
    /// <!DOCTYPE html>
    /// <html>
    /// <head>
    ///     <title>Welcome</title>
    ///     <meta charset="UTF-8">
    /// </head>
    /// <body>
    ///     <h1>Welcome to Our Site</h1>
    ///     <p>This is a sample HTML response.</p>
    /// </body>
    /// </html>
    /// "#;
    ///
    /// let response = ResponseBuilder::new()
    ///     .html(html)
    ///     .build();
    /// ```
    ///
    /// ## Dynamic HTML Generation
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// fn generate_user_profile(name: &str, email: &str) -> ignitia::Response {
    ///     let html = format!(r#"
    ///     <!DOCTYPE html>
    ///     <html>
    ///     <head>
    ///         <title>User Profile - {}</title>
    ///         <style>
    ///             body {{ font-family: Arial, sans-serif; margin: 40px; }}
    ///             .profile {{ border: 1px solid #ccc; padding: 20px; }}
    ///         </style>
    ///     </head>
    ///     <body>
    ///         <div class="profile">
    ///             <h1>User Profile</h1>
    ///             <p><strong>Name:</strong> {}</p>
    ///             <p><strong>Email:</strong> {}</p>
    ///         </div>
    ///     </body>
    ///     </html>
    ///     "#, name, name, email);
    ///
    ///     ResponseBuilder::new()
    ///         .header("Cache-Control", "no-cache")
    ///         .html(html)
    ///         .build()
    /// }
    /// ```
    ///
    /// ## Template Integration
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// // Example with a simple template system
    /// fn render_template(template: &str, variables: &[(&str, &str)]) -> String {
    ///     let mut result = template.to_string();
    ///     for (key, value) in variables {
    ///         result = result.replace(&format!("{{{{{}}}}}", key), value);
    ///     }
    ///     result
    /// }
    ///
    /// fn template_response() -> ignitia::Response {
    ///     let template = r#"
    ///     <html>
    ///     <body>
    ///         <h1>Hello, {{name}}!</h1>
    ///         <p>Today is {{date}}.</p>
    ///     </body>
    ///     </html>
    ///     "#;
    ///
    ///     let variables = [
    ///         ("name", "Alice"),
    ///         ("date", "2023-01-01"),
    ///     ];
    ///
    ///     let rendered_html = render_template(template, &variables);
    ///
    ///     ResponseBuilder::new()
    ///         .html(rendered_html)
    ///         .build()
    /// }
    /// ```
    pub fn html<T: Into<String>>(mut self, html: T) -> Self {
        let html = html.into();
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        self.body = Some(Bytes::from(html));
        self
    }

    /// Sets the response body (builder pattern).
    ///
    /// This method sets the raw body content without modifying headers.
    /// Use this for binary content or when you need full control over the body.
    ///
    /// # Type Parameters
    /// - `T`: Body type (must implement `Into<Bytes>`)
    ///
    /// # Parameters
    /// - `body`: The body content
    ///
    /// # Returns
    /// The builder with the updated body
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use bytes::Bytes;
    ///
    /// // Binary data
    /// let image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    /// let response = ResponseBuilder::new()
    ///     .header("Content-Type", "image/jpeg")
    ///     .body(image_data)
    ///     .build();
    ///
    /// // From Bytes
    /// let data = Bytes::from("Custom content");
    /// let response = ResponseBuilder::new()
    ///     .header("Content-Type", "application/octet-stream")
    ///     .body(data)
    ///     .build();
    /// ```
    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Builds and returns the final Response.
    ///
    /// This method consumes the builder and creates a `Response` instance with
    /// all the configured properties. If no body was set, the response will
    /// have an empty body.
    ///
    /// # Returns
    /// A complete `Response` instance
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    /// use http::StatusCode;
    ///
    /// let response = ResponseBuilder::new()
    ///     .status(StatusCode::CREATED)
    ///     .header("Location", "/users/123")
    ///     .text("User created successfully")
    ///     .build();
    ///
    /// assert_eq!(response.status, StatusCode::CREATED);
    /// assert!(response.headers.contains_key("location"));
    /// ```
    ///
    /// ## Empty Body Handling
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// // Response without explicit body
    /// let response = ResponseBuilder::new()
    ///     .status_code(204) // No Content
    ///     .build();
    ///
    /// assert!(response.body.is_empty());
    /// ```
    pub fn build(self) -> Response {
        Response {
            status: self.status,
            headers: self.headers,
            body: self.body.unwrap_or_else(|| Bytes::new()),
        }
    }
}

impl Default for ResponseBuilder {
    /// Creates a default ResponseBuilder.
    ///
    /// This is equivalent to calling `ResponseBuilder::new()`.
    ///
    /// # Examples
    /// ```
    /// use ignitia::ResponseBuilder;
    ///
    /// let builder = ResponseBuilder::default();
    /// let response = builder.text("Hello").build();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}
