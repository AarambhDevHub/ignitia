//! # HTTP Response Generation Module
//!
//! This module provides comprehensive HTTP response generation capabilities for the Ignitia web framework.
//! It includes response building, content type handling, status code management, and error response generation
//! with efficient serialization and flexible customization options.
//!
//! ## Features
//!
//! - **Multiple Response Formats**: JSON, HTML, text, and binary response generation
//! - **Status Code Management**: Easy status code setting with validation
//! - **Header Management**: Flexible header manipulation and content type handling
//! - **Error Response Generation**: Automatic error-to-response conversion with structured formats
//! - **Builder Pattern Support**: Fluent response building with method chaining
//! - **Performance Optimized**: Efficient serialization and memory usage
//!
//! ## Response Types
//!
//! ### JSON Responses
//! - Automatic serialization with proper content-type headers
//! - Support for any type implementing `Serialize`
//! - Comprehensive error handling for serialization failures
//!
//! ### HTML Responses
//! - Proper content-type headers with charset specification
//! - Template integration support
//! - XSS protection considerations
//!
//! ### Text Responses
//! - UTF-8 encoded plain text with proper headers
//! - Support for various text formats
//!
//! ### Binary Responses
//! - Efficient handling of binary data
//! - Flexible content-type specification
//! - Support for file downloads and streaming
//!
//! ## Usage Examples
//!
//! ### Basic Response Creation
//! ```
//! use ignitia::{Response, Result};
//! use http::StatusCode;
//!
//! // Simple text response
//! async fn hello_handler() -> Result<Response> {
//!     Ok(Response::text("Hello, World!"))
//! }
//!
//! // JSON response
//! async fn json_handler() -> Result<Response> {
//!     let data = serde_json::json!({
//!         "message": "Hello, World!",
//!         "timestamp": chrono::Utc::now()
//!     });
//!     Response::json(data)
//! }
//!
//! // HTML response
//! async fn html_handler() -> Result<Response> {
//!     let html = r#"
//!         <!DOCTYPE html>
//!         <html>
//!         <head><title>Hello</title></head>
//!         <body><h1>Hello, World!</h1></body>
//!         </html>
//!     "#;
//!     Ok(Response::html(html))
//! }
//! ```
//!
//! ### Custom Status Codes
//! ```
//! use ignitia::Response;
//! use http::StatusCode;
//!
//! async fn custom_status_handler() -> ignitia::Result<Response> {
//!     Ok(Response::text("Created successfully")
//!         .with_status(StatusCode::CREATED))
//! }
//!
//! async fn not_found_handler() -> ignitia::Result<Response> {
//!     Ok(Response::text("Resource not found")
//!         .with_status_code(404))
//! }
//! ```
//!
//! ### Working with Headers
//! ```
//! use ignitia::Response;
//! use http::{HeaderMap, HeaderName, HeaderValue};
//!
//! async fn custom_headers_handler() -> ignitia::Result<Response> {
//!     let mut response = Response::text("Custom headers example");
//!
//!     // Add custom headers
//!     response.headers.insert(
//!         HeaderName::from_static("x-custom-header"),
//!         HeaderValue::from_static("custom-value")
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("cache-control"),
//!         HeaderValue::from_static("no-cache, no-store, must-revalidate")
//!     );
//!
//!     Ok(response)
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ### API Response Patterns
//! ```
//! use ignitia::{Response, Result};
//! use serde::{Deserialize, Serialize};
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
//! #[derive(Serialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! async fn get_user_handler() -> Result<Response> {
//!     let user = User {
//!         id: 1,
//!         name: "Alice".to_string(),
//!         email: "alice@example.com".to_string(),
//!     };
//!
//!     let api_response = ApiResponse {
//!         success: true,
//!         data: Some(user),
//!         message: "User retrieved successfully".to_string(),
//!         timestamp: chrono::Utc::now().to_rfc3339(),
//!     };
//!
//!     Response::json(api_response)
//! }
//!
//! async fn user_not_found_handler() -> Result<Response> {
//!     let api_response = ApiResponse::<()> {
//!         success: false,
//!         data: None,
//!         message: "User not found".to_string(),
//!         timestamp: chrono::Utc::now().to_rfc3339(),
//!     };
//!
//!     let mut response = Response::json(api_response)?;
//!     response.status = StatusCode::NOT_FOUND;
//!     Ok(response)
//! }
//! ```
//!
//! ### File Download Responses
//! ```
//! use ignitia::Response;
//! use bytes::Bytes;
//! use http::{HeaderName, HeaderValue};
//!
//! async fn download_handler() -> ignitia::Result<Response> {
//!     // Simulate file content
//!     let file_content = b"Hello, this is a downloadable file!";
//!     let filename = "example.txt";
//!
//!     let mut response = Response::new(http::StatusCode::OK);
//!     response.body = Bytes::from(&file_content[..]);
//!
//!     // Set appropriate headers for file download
//!     response.headers.insert(
//!         HeaderName::from_static("content-type"),
//!         HeaderValue::from_static("application/octet-stream")
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("content-disposition"),
//!         HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
//!             .map_err(|_| ignitia::Error::Internal("Invalid filename".into()))?
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("content-length"),
//!         HeaderValue::from_str(&file_content.len().to_string())
//!             .map_err(|_| ignitia::Error::Internal("Invalid content length".into()))?
//!     );
//!
//!     Ok(response)
//! }
//! ```
//!
//! ### Streaming Responses
//! ```
//! use ignitia::Response;
//! use bytes::Bytes;
//! use http::{HeaderName, HeaderValue};
//!
//! async fn streaming_handler() -> ignitia::Result<Response> {
//!     // Server-Sent Events example
//!     let event_data = "data: Hello from server!\n\n";
//!
//!     let mut response = Response::new(http::StatusCode::OK);
//!     response.body = Bytes::from(event_data);
//!
//!     response.headers.insert(
//!         HeaderName::from_static("content-type"),
//!         HeaderValue::from_static("text/event-stream")
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("cache-control"),
//!         HeaderValue::from_static("no-cache")
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("connection"),
//!         HeaderValue::from_static("keep-alive")
//!     );
//!
//!     Ok(response)
//! }
//! ```
//!
//! ## Error Response Handling
//!
//! ### Automatic Error Conversion
//! ```
//! use ignitia::{Response, Error, Result};
//! use http::StatusCode;
//!
//! async fn error_example_handler() -> Result<Response> {
//!     // This will automatically be converted to an error response
//!     Err(Error::NotFound("User not found".to_string()))
//! }
//!
//! // The framework automatically converts errors to responses:
//! // {
//! //   "error": "Not Found",
//! //   "message": "User not found",
//! //   "status": 404,
//! //   "error_type": "not_found",
//! //   "timestamp": "2023-01-01T12:00:00Z"
//! // }
//! ```
//!
//! ### Custom Error Responses
//! ```
//! use ignitia::{Response, Error, Result};
//! use serde_json::json;
//!
//! async fn custom_error_handler() -> Result<Response> {
//!     let error_messages = vec![
//!         "Name is required".to_string(),
//!         "Email format is invalid".to_string(),
//!     ];
//!
//!     Response::validation_error(error_messages)
//! }
//!
//! async fn api_error_handler() -> Result<Response> {
//!     let error_response = json!({
//!         "error": {
//!             "code": "RATE_LIMITED",
//!             "message": "Too many requests",
//!             "retry_after": 60
//!         }
//!     });
//!
//!     let mut response = Response::json(error_response)?;
//!     response.status = http::StatusCode::TOO_MANY_REQUESTS;
//!     Ok(response)
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Memory Efficiency
//! - Uses `bytes::Bytes` for zero-copy operations
//! - Efficient serialization with pre-allocated buffers
//! - Minimal header allocation overhead
//!
//! ### Serialization Performance
//! ```
//! use ignitia::Response;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct LargeData {
//!     items: Vec<String>,
//! }
//!
//! async fn optimized_json_handler() -> ignitia::Result<Response> {
//!     let data = LargeData {
//!         items: (0..1000).map(|i| format!("Item {}", i)).collect(),
//!     };
//!
//!     // Efficient JSON serialization with error handling
//!     match Response::json(data) {
//!         Ok(response) => Ok(response),
//!         Err(e) => {
//!             tracing::error!("JSON serialization failed: {}", e);
//!             Ok(Response::text("Internal server error")
//!                 .with_status(http::StatusCode::INTERNAL_SERVER_ERROR))
//!         }
//!     }
//! }
//! ```
//!
//! ## Security Considerations
//!
//! ### Content Type Security
//! - Always set appropriate content-type headers
//! - Validate content before setting as HTML
//! - Use proper encoding for text responses
//!
//! ### XSS Prevention
//! ```
//! use ignitia::Response;
//!
//! async fn safe_html_handler(user_input: &str) -> ignitia::Result<Response> {
//!     // Escape user input to prevent XSS
//!     let escaped_input = html_escape::encode_text(user_input);
//!
//!     let safe_html = format!(
//!         r#"<!DOCTYPE html>
//!         <html>
//!         <head><title>Safe Output</title></head>
//!         <body><h1>Hello, {}</h1></body>
//!         </html>"#,
//!         escaped_input
//!     );
//!
//!     Ok(Response::html(safe_html))
//! }
//! ```
//!
//! ### Security Headers
//! ```
//! use ignitia::Response;
//! use http::{HeaderName, HeaderValue};
//!
//! async fn secure_response_handler() -> ignitia::Result<Response> {
//!     let mut response = Response::html("<h1>Secure Page</h1>");
//!
//!     // Add security headers
//!     response.headers.insert(
//!         HeaderName::from_static("x-content-type-options"),
//!         HeaderValue::from_static("nosniff")
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("x-frame-options"),
//!         HeaderValue::from_static("DENY")
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("x-xss-protection"),
//!         HeaderValue::from_static("1; mode=block")
//!     );
//!
//!     response.headers.insert(
//!         HeaderName::from_static("content-security-policy"),
//!         HeaderValue::from_static("default-src 'self'")
//!     );
//!
//!     Ok(response)
//! }
//! ```
//!
//! ## Testing Response Generation
//!
//! ### Unit Testing
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use http::StatusCode;
//!
//!     #[tokio::test]
//!     async fn test_text_response() {
//!         let response = Response::text("Hello, World!");
//!
//!         assert_eq!(response.status, StatusCode::OK);
//!         assert_eq!(
//!             response.headers.get("content-type").unwrap(),
//!             "text/plain; charset=utf-8"
//!         );
//!         assert_eq!(response.body, "Hello, World!");
//!     }
//!
//!     #[tokio::test]
//!     async fn test_json_response() {
//!         let data = serde_json::json!({"message": "test"});
//!         let response = Response::json(data).unwrap();
//!
//!         assert_eq!(response.status, StatusCode::OK);
//!         assert_eq!(
//!             response.headers.get("content-type").unwrap(),
//!             "application/json"
//!         );
//!     }
//!
//!     #[tokio::test]
//!     async fn test_error_conversion() {
//!         let error = ignitia::Error::NotFound("test".to_string());
//!         let response = Response::from(error);
//!
//!         assert_eq!(response.status, StatusCode::NOT_FOUND);
//!     }
//! }
//! ```

pub mod builder;
pub mod status;

use std::sync::Arc;

use crate::error::Result;
use bytes::Bytes;
use http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode};
use serde::Serialize;

/// HTTP response representation containing status, headers, and body.
///
/// The `Response` struct encapsulates all data needed to send an HTTP response,
/// including the status code, headers, and body content. It provides convenient
/// methods for creating responses with different content types and formats.
///
/// # Structure
/// - **status**: HTTP status code (200, 404, 500, etc.)
/// - **headers**: HTTP headers as a HeaderMap
/// - **body**: Response body as bytes
///
/// # Examples
///
/// ## Basic Response Creation
/// ```
/// use ignitia::Response;
/// use http::StatusCode;
///
/// // Create a simple text response
/// let response = Response::text("Hello, World!");
/// assert_eq!(response.status, StatusCode::OK);
///
/// // Create a response with custom status
/// let response = Response::new(StatusCode::CREATED);
/// ```
///
/// ## JSON Responses
/// ```
/// use ignitia::Response;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let data = json!({
///     "message": "Success",
///     "data": {
///         "id": 123,
///         "name": "Example"
///     }
/// });
///
/// let response = Response::json(data)?;
/// # Ok(())
/// # }
/// ```
///
/// ## HTML Responses
/// ```
/// use ignitia::Response;
///
/// let html = r#"
/// <!DOCTYPE html>
/// <html>
/// <head><title>Hello</title></head>
/// <body><h1>Hello, World!</h1></body>
/// </html>
/// "#;
///
/// let response = Response::html(html);
/// ```
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code
    pub status: StatusCode,
    /// HTTP response headers
    pub headers: HeaderMap,
    /// Response body as bytes
    pub body: Arc<Bytes>,
}

impl Response {
    /// Creates a new Response with the specified status code.
    ///
    /// This is the basic constructor for Response instances. It creates a response
    /// with the given status code, empty headers, and an empty body.
    ///
    /// # Parameters
    /// - `status`: The HTTP status code for the response
    ///
    /// # Returns
    /// A new Response instance
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    /// use http::StatusCode;
    ///
    /// let response = Response::new(StatusCode::OK);
    /// assert_eq!(response.status, StatusCode::OK);
    /// assert!(response.body.is_empty());
    ///
    /// let error_response = Response::new(StatusCode::INTERNAL_SERVER_ERROR);
    /// assert_eq!(error_response.status, StatusCode::INTERNAL_SERVER_ERROR);
    /// ```
    #[inline]
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Arc::new(Bytes::new()),
        }
    }

    /// Sets the status code of the response (builder pattern).
    ///
    /// This method consumes the response and returns it with the new status code,
    /// enabling fluent method chaining.
    ///
    /// # Parameters
    /// - `status`: The new status code to set
    ///
    /// # Returns
    /// The response with the updated status code
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    /// use http::StatusCode;
    ///
    /// let response = Response::text("Created successfully")
    ///     .with_status(StatusCode::CREATED);
    /// assert_eq!(response.status, StatusCode::CREATED);
    /// ```
    #[inline]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Sets the status code using a numeric value (builder pattern).
    ///
    /// This is a convenience method that accepts a u16 status code and converts
    /// it to a StatusCode. Invalid status codes are ignored.
    ///
    /// # Parameters
    /// - `status_code`: The numeric status code (e.g., 200, 404, 500)
    ///
    /// # Returns
    /// The response with the updated status code (if valid)
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    ///
    /// let response = Response::text("Not Found")
    ///     .with_status_code(404);
    /// assert_eq!(response.status.as_u16(), 404);
    ///
    /// // Invalid status codes are ignored
    /// let response = Response::text("Test")
    ///     .with_status_code(9999); // Invalid, ignored
    /// ```
    #[inline]
    pub fn with_status_code(mut self, status_code: u16) -> Self {
        if let Ok(status) = StatusCode::from_u16(status_code) {
            self.status = status;
        }
        self
    }

    /// Sets the response body (builder pattern).
    ///
    /// This method accepts any type that can be converted to `Bytes` and sets
    /// it as the response body.
    ///
    /// # Parameters
    /// - `body`: The body content (String, &str, Vec<u8>, Bytes, etc.)
    ///
    /// # Returns
    /// The response with the updated body
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    /// use bytes::Bytes;
    ///
    /// // From string
    /// let response = Response::new(http::StatusCode::OK)
    ///     .with_body("Hello, World!");
    ///
    /// // From bytes
    /// let data = Bytes::from("Binary data");
    /// let response = Response::new(http::StatusCode::OK)
    ///     .with_body(data);
    /// ```
    #[inline]
    pub fn with_body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = Arc::new(body.into());
        self
    }

    // Zero-copy body sharing
    pub fn body_shared(&self) -> Arc<Bytes> {
        Arc::clone(&self.body)
    }

    /// Creates a successful response (200 OK).
    ///
    /// This is a convenience method for creating responses with a 200 OK status.
    ///
    /// # Returns
    /// A new response with status 200 OK
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    /// use http::StatusCode;
    ///
    /// let response = Response::ok();
    /// assert_eq!(response.status, StatusCode::OK);
    /// ```
    #[inline]
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    /// Creates a not found response (404 Not Found).
    ///
    /// This is a convenience method for creating 404 responses.
    ///
    /// # Returns
    /// A new response with status 404 Not Found
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    /// use http::StatusCode;
    ///
    /// let response = Response::not_found();
    /// assert_eq!(response.status, StatusCode::NOT_FOUND);
    /// ```
    #[inline]
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    /// Creates an internal server error response (500 Internal Server Error).
    ///
    /// This is a convenience method for creating 500 error responses.
    ///
    /// # Returns
    /// A new response with status 500 Internal Server Error
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    /// use http::StatusCode;
    ///
    /// let response = Response::internal_error();
    /// assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
    /// ```
    #[inline]
    pub fn internal_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Creates a JSON response with automatic serialization.
    ///
    /// This method serializes the provided data to JSON and creates a response
    /// with the appropriate content-type header. It returns a Result because
    /// serialization can fail.
    ///
    /// # Type Parameters
    /// - `T`: The type to serialize (must implement `Serialize`)
    ///
    /// # Parameters
    /// - `data`: The data to serialize as JSON
    ///
    /// # Returns
    /// - `Ok(Response)`: Successfully created JSON response
    /// - `Err(Error)`: JSON serialization error
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    /// use serde::Serialize;
    /// use serde_json::json;
    ///
    /// #[derive(Serialize)]
    /// struct ApiResponse {
    ///     success: bool,
    ///     message: String,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // With custom struct
    /// let data = ApiResponse {
    ///     success: true,
    ///     message: "Operation completed".to_string(),
    /// };
    /// let response = Response::json(data)?;
    ///
    /// // With serde_json::Value
    /// let data = json!({
    ///     "users": [
    ///         {"id": 1, "name": "Alice"},
    ///         {"id": 2, "name": "Bob"}
    ///     ],
    ///     "total": 2
    /// });
    /// let response = Response::json(data)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Error Handling
    /// ```
    /// use ignitia::Response;
    /// use serde_json::json;
    ///
    /// async fn safe_json_handler() -> ignitia::Result<Response> {
    ///     let data = json!({"key": "value"});
    ///
    ///     match Response::json(data) {
    ///         Ok(response) => Ok(response),
    ///         Err(e) => {
    ///             tracing::error!("JSON serialization failed: {}", e);
    ///             Ok(Response::text("Internal server error")
    ///                 .with_status_code(500))
    ///         }
    ///     }
    /// }
    /// ```
    pub fn json<T: Serialize>(data: T) -> Result<Self> {
        let body = serde_json::to_vec(&data)?;
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        response.body = Arc::new(Bytes::from(body));
        Ok(response)
    }

    /// Creates a plain text response.
    ///
    /// This method creates a response with UTF-8 encoded text and sets the
    /// appropriate content-type header.
    ///
    /// # Parameters
    /// - `text`: The text content (String, &str, or anything that converts to String)
    ///
    /// # Returns
    /// A new response with the text content
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    ///
    /// let response = Response::text("Hello, World!");
    /// assert_eq!(
    ///     response.headers.get("content-type").unwrap(),
    ///     "text/plain; charset=utf-8"
    /// );
    ///
    /// let response = Response::text(format!("User ID: {}", 123));
    /// ```
    ///
    /// ## Multi-line Text
    /// ```
    /// use ignitia::Response;
    ///
    /// let text = r#"
    /// Line 1
    /// Line 2
    /// Line 3
    /// "#;
    /// let response = Response::text(text);
    /// ```
    #[inline]
    pub fn text(text: impl Into<String>) -> Self {
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.body = Arc::new(Bytes::from(text.into()));
        response
    }

    /// Creates an HTML response.
    ///
    /// This method creates a response with HTML content and sets the appropriate
    /// content-type header with UTF-8 charset.
    ///
    /// # Parameters
    /// - `html`: The HTML content (String, &str, or anything that converts to String)
    ///
    /// # Returns
    /// A new response with the HTML content
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    ///
    /// let html = r#"
    /// <!DOCTYPE html>
    /// <html>
    /// <head>
    ///     <title>Hello Page</title>
    ///     <meta charset="UTF-8">
    /// </head>
    /// <body>
    ///     <h1>Hello, World!</h1>
    ///     <p>Welcome to our website!</p>
    /// </body>
    /// </html>
    /// "#;
    ///
    /// let response = Response::html(html);
    /// assert_eq!(
    ///     response.headers.get("content-type").unwrap(),
    ///     "text/html; charset=utf-8"
    /// );
    /// ```
    ///
    /// ## Dynamic HTML Generation
    /// ```
    /// use ignitia::Response;
    ///
    /// fn generate_user_page(username: &str, email: &str) -> Response {
    ///     let html = format!(r#"
    ///     <!DOCTYPE html>
    ///     <html>
    ///     <head><title>User Profile</title></head>
    ///     <body>
    ///         <h1>User Profile</h1>
    ///         <p><strong>Username:</strong> {}</p>
    ///         <p><strong>Email:</strong> {}</p>
    ///     </body>
    ///     </html>
    ///     "#, username, email);
    ///
    ///     Response::html(html)
    /// }
    /// ```
    ///
    /// ## Security Note
    /// When generating HTML with user input, always escape the input to prevent XSS:
    /// ```
    /// use ignitia::Response;
    ///
    /// fn safe_html_response(user_input: &str) -> Response {
    ///     // Escape user input (you would use a proper HTML escape function)
    ///     let escaped_input = user_input
    ///         .replace('&', "&amp;")
    ///         .replace('<', "&lt;")
    ///         .replace('>', "&gt;")
    ///         .replace('"', "&quot;")
    ///         .replace('\'', "&#x27;");
    ///
    ///     let html = format!("<p>Hello, {}</p>", escaped_input);
    ///     Response::html(html)
    /// }
    /// ```
    #[inline]
    pub fn html(html: impl Into<String>) -> Self {
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        response.body = Arc::new(Bytes::from(html.into()));
        response
    }

    /// Creates a temporary redirect response (HTTP 302 Found).
    ///
    /// This is the most commonly used redirect method. The client will make a new request
    /// to the provided location, but the original URL should be used for future requests.
    /// The HTTP method may change to GET for the redirected request.
    ///
    /// # Arguments
    ///
    /// * `location` - The URL to redirect to
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    /// ```
    /// use ignitia::Response;
    ///
    /// let response = Response::redirect("/dashboard");
    /// assert_eq!(response.status, ignitia::StatusCode::FOUND);
    /// assert_eq!(
    ///     response.headers.get("location").unwrap(),
    ///     "/dashboard"
    /// );
    /// ```
    ///
    /// ## Redirect After Login
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn login_handler() -> ignitia::Result<Response> {
    ///     // Authenticate user...
    ///     Ok(Response::redirect("/dashboard"))
    /// }
    /// ```
    ///
    /// ## Conditional Redirect
    /// ```
    /// use ignitia::Response;
    ///
    /// fn redirect_based_on_role(user_role: &str) -> Response {
    ///     match user_role {
    ///         "admin" => Response::redirect("/admin/dashboard"),
    ///         "user" => Response::redirect("/user/profile"),
    ///         _ => Response::redirect("/login"),
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn redirect(location: impl Into<String>) -> Self {
        Self::redirect_with_status(StatusCode::FOUND, location)
    }

    /// Creates a permanent redirect response (HTTP 301 Moved Permanently).
    ///
    /// Use this when a resource has permanently moved to a new location. Search engines
    /// and browsers will update their records to use the new URL. The HTTP method may
    /// change to GET for the redirected request.
    ///
    /// # Arguments
    ///
    /// * `location` - The new permanent URL location
    ///
    /// # Examples
    ///
    /// ## Basic Permanent Redirect
    /// ```
    /// use ignitia::Response;
    ///
    /// let response = Response::permanent_redirect("/new-location");
    /// assert_eq!(response.status, ignitia::StatusCode::MOVED_PERMANENTLY);
    /// ```
    ///
    /// ## Redirecting Old URLs
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn handle_old_blog_url() -> ignitia::Result<Response> {
    ///     // Old blog structure: /blog/2023/article-title
    ///     // New blog structure: /articles/article-title
    ///     Ok(Response::permanent_redirect("/articles/migrating-to-new-blog"))
    /// }
    /// ```
    ///
    /// ## SEO-Friendly Redirects
    /// ```
    /// use ignitia::Response;
    ///
    /// fn redirect_old_product_page(old_id: u32, new_slug: &str) -> Response {
    ///     // Permanently redirect old product IDs to new slug-based URLs
    ///     Response::permanent_redirect(&format!("/products/{}", new_slug))
    /// }
    /// ```
    #[inline]
    pub fn permanent_redirect(location: impl Into<String>) -> Self {
        Self::redirect_with_status(StatusCode::MOVED_PERMANENTLY, location)
    }

    /// Creates a redirect response with a custom HTTP status code.
    ///
    /// This method allows you to specify any redirect status code (3xx series).
    /// The response includes an HTML fallback page for browsers that don't
    /// automatically follow redirects.
    ///
    /// # Arguments
    ///
    /// * `status` - The HTTP status code for the redirect (typically 3xx)
    /// * `location` - The URL to redirect to
    ///
    /// # Examples
    ///
    /// ## Custom Status Redirect
    /// ```
    /// use ignitia::{Response, StatusCode};
    ///
    /// let response = Response::redirect_with_status(
    ///     StatusCode::FOUND,
    ///     "/custom-redirect"
    /// );
    /// ```
    ///
    /// ## Multiple Choice Redirect
    /// ```
    /// use ignitia::{Response, StatusCode};
    ///
    /// fn handle_ambiguous_request() -> Response {
    ///     Response::redirect_with_status(
    ///         StatusCode::MULTIPLE_CHOICES,
    ///         "/choose-option"
    ///     )
    /// }
    /// ```
    ///
    /// ## Temporary Maintenance Redirect
    /// ```
    /// use ignitia::{Response, StatusCode};
    ///
    /// fn maintenance_redirect() -> Response {
    ///     Response::redirect_with_status(
    ///         StatusCode::TEMPORARY_REDIRECT,
    ///         "/maintenance"
    ///     )
    /// }
    /// ```
    pub fn redirect_with_status(status: StatusCode, location: impl Into<String>) -> Self {
        let location_str = location.into();

        let mut response = Self::new(status);
        response.headers.insert(
            header::LOCATION,
            HeaderValue::from_str(&location_str).unwrap_or_else(|_| HeaderValue::from_static("/")),
        );

        // Add a simple HTML body for browsers that don't handle redirects automatically
        let html_body = format!(
            r#"<!DOCTYPE html>
    <html>
    <head>
        <title>Redirect</title>
        <meta http-equiv="refresh" content="0; url={}">
    </head>
    <body>
        <p>Redirecting to <a href="{}">{}</a></p>
    </body>
    </html>"#,
            html_escape(&location_str),
            html_escape(&location_str),
            html_escape(&location_str)
        );

        response.headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        response.body = Arc::new(bytes::Bytes::from(html_body));

        response
    }

    /// Creates a "See Other" redirect response (HTTP 303 See Other).
    ///
    /// This redirect is ideal for the POST-redirect-GET pattern, where after
    /// processing a POST request, you redirect the client to a GET endpoint.
    /// This prevents duplicate form submissions if the user refreshes the page.
    ///
    /// # Arguments
    ///
    /// * `location` - The URL to redirect to (typically a GET endpoint)
    ///
    /// # Examples
    ///
    /// ## POST-Redirect-GET Pattern
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn process_form() -> ignitia::Result<Response> {
    ///     // Process form data...
    ///     // Save to database...
    ///
    ///     // Redirect to success page to prevent duplicate submissions
    ///     Ok(Response::see_other("/form-success"))
    /// }
    /// ```
    ///
    /// ## After User Registration
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn register_user() -> ignitia::Result<Response> {
    ///     // Create new user account...
    ///
    ///     // Redirect to welcome page
    ///     Ok(Response::see_other("/welcome"))
    /// }
    /// ```
    ///
    /// ## Shopping Cart Checkout
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn checkout_handler() -> ignitia::Result<Response> {
    ///     // Process payment...
    ///     // Update inventory...
    ///
    ///     // Redirect to order confirmation
    ///     Ok(Response::see_other("/order-confirmation"))
    /// }
    /// ```
    #[inline]
    pub fn see_other(location: impl Into<String>) -> Self {
        Self::redirect_with_status(StatusCode::SEE_OTHER, location)
    }

    /// Creates a temporary redirect that preserves the HTTP method (HTTP 307 Temporary Redirect).
    ///
    /// Unlike 302 redirects, this guarantees that the client will use the same HTTP method
    /// when making the redirected request. Use this when the method preservation is important.
    ///
    /// # Arguments
    ///
    /// * `location` - The temporary URL to redirect to
    ///
    /// # Examples
    ///
    /// ## Method-Preserving Redirect
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn api_endpoint() -> ignitia::Result<Response> {
    ///     // Temporarily redirect POST to another server
    ///     Ok(Response::temporary_redirect("/api/v2/endpoint"))
    /// }
    /// ```
    ///
    /// ## Load Balancing Redirect
    /// ```
    /// use ignitia::Response;
    ///
    /// fn balance_load() -> Response {
    ///     // Redirect to less busy server while preserving method
    ///     Response::temporary_redirect("https://server2.example.com/api")
    /// }
    /// ```
    #[inline]
    pub fn temporary_redirect(location: impl Into<String>) -> Self {
        Self::redirect_with_status(StatusCode::TEMPORARY_REDIRECT, location)
    }

    /// Creates a permanent redirect that preserves the HTTP method (HTTP 308 Permanent Redirect).
    ///
    /// This is like 301 but guarantees the client will use the same HTTP method.
    /// Use this for permanent moves where method preservation is crucial.
    ///
    /// # Arguments
    ///
    /// * `location` - The new permanent URL
    ///
    /// # Examples
    ///
    /// ## API Version Migration
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn old_api_endpoint() -> ignitia::Result<Response> {
    ///     // Permanently moved to new API version, preserve HTTP method
    ///     Ok(Response::permanent_redirect_308("/api/v2/users"))
    /// }
    /// ```
    #[inline]
    pub fn permanent_redirect_308(location: impl Into<String>) -> Self {
        Self::redirect_with_status(StatusCode::PERMANENT_REDIRECT, location)
    }

    /// Creates a redirect response without an HTML body (useful for APIs).
    ///
    /// This method creates a minimal redirect response with only the necessary headers,
    /// making it ideal for REST APIs or situations where you don't want the HTML fallback.
    ///
    /// # Arguments
    ///
    /// * `status` - The HTTP status code for the redirect
    /// * `location` - The URL to redirect to
    ///
    /// # Examples
    ///
    /// ## API Redirect
    /// ```
    /// use ignitia::{Response, StatusCode};
    ///
    /// async fn api_redirect() -> ignitia::Result<Response> {
    ///     Ok(Response::redirect_empty(
    ///         StatusCode::FOUND,
    ///         "https://api.example.com/v2/endpoint"
    ///     ))
    /// }
    /// ```
    ///
    /// ## Minimal Redirect
    /// ```
    /// use ignitia::{Response, StatusCode};
    ///
    /// fn lightweight_redirect() -> Response {
    ///     Response::redirect_empty(
    ///         StatusCode::MOVED_PERMANENTLY,
    ///         "/new-location"
    ///     )
    /// }
    /// ```
    pub fn redirect_empty(status: StatusCode, location: impl Into<String>) -> Self {
        let location_str = location.into();

        let mut response = Self::new(status);
        response.headers.insert(
            header::LOCATION,
            HeaderValue::from_str(&location_str).unwrap_or_else(|_| HeaderValue::from_static("/")),
        );

        response
    }

    /// Convenience method for redirecting to a login page.
    ///
    /// Creates a temporary redirect (302) to the specified login path.
    /// This is a commonly used pattern in web applications for authentication flows.
    ///
    /// # Arguments
    ///
    /// * `login_path` - The path to the login page
    ///
    /// # Examples
    ///
    /// ## Basic Login Redirect
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn protected_handler() -> ignitia::Result<Response> {
    ///     // Check authentication...
    ///     if !user_authenticated {
    ///         return Ok(Response::redirect_to_login("/auth/login"));
    ///     }
    ///
    ///     // Continue with protected logic...
    ///     Ok(Response::text("Welcome to protected area"))
    /// }
    /// ```
    ///
    /// ## With Return URL
    /// ```
    /// use ignitia::Response;
    ///
    /// fn redirect_with_return_url(current_path: &str) -> Response {
    ///     let login_url = format!("/login?return_to={}",
    ///         urlencoding::encode(current_path));
    ///     Response::redirect_to_login(login_url)
    /// }
    /// ```
    #[inline]
    pub fn redirect_to_login(login_path: impl Into<String>) -> Self {
        Self::redirect(login_path)
    }

    /// Convenience method for redirecting after a successful POST request.
    ///
    /// Uses HTTP 303 (See Other) to implement the POST-redirect-GET pattern,
    /// preventing duplicate form submissions when users refresh the page.
    ///
    /// # Arguments
    ///
    /// * `location` - The success page URL to redirect to
    ///
    /// # Examples
    ///
    /// ## Form Submission Success
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn contact_form_handler() -> ignitia::Result<Response> {
    ///     // Process contact form...
    ///     // Send email...
    ///     // Save to database...
    ///
    ///     Ok(Response::redirect_after_post("/contact/thank-you"))
    /// }
    /// ```
    ///
    /// ## E-commerce Order
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn place_order() -> ignitia::Result<Response> {
    ///     // Process order...
    ///     // Charge payment...
    ///     // Update inventory...
    ///
    ///     Ok(Response::redirect_after_post("/orders/success"))
    /// }
    /// ```
    #[inline]
    pub fn redirect_after_post(location: impl Into<String>) -> Self {
        Self::see_other(location)
    }

    /// Convenience method for redirecting moved content.
    ///
    /// Creates a permanent redirect (301) for content that has been permanently moved.
    /// This is ideal for SEO as search engines will update their indexes.
    ///
    /// # Arguments
    ///
    /// * `new_location` - The new permanent location of the content
    ///
    /// # Examples
    ///
    /// ## Content Migration
    /// ```
    /// use ignitia::Response;
    ///
    /// async fn old_article_handler() -> ignitia::Result<Response> {
    ///     // Article has moved to new URL structure
    ///     Ok(Response::redirect_moved("/articles/2023/new-article-slug"))
    /// }
    /// ```
    ///
    /// ## Domain Migration
    /// ```
    /// use ignitia::Response;
    ///
    /// fn redirect_to_new_domain() -> Response {
    ///     Response::redirect_moved("https://newdomain.com/same-path")
    /// }
    /// ```
    #[inline]
    pub fn redirect_moved(new_location: impl Into<String>) -> Self {
        Self::permanent_redirect(new_location)
    }
}

pub use builder::ResponseBuilder;

use crate::error::{Error, ErrorResponse};

impl From<Error> for Response {
    /// Converts an Error into an HTTP Response.
    ///
    /// This implementation automatically converts framework errors into proper
    /// HTTP responses with JSON formatting and appropriate status codes.
    ///
    /// # Parameters
    /// - `err`: The error to convert
    ///
    /// # Returns
    /// An HTTP response representing the error
    ///
    /// # Error Response Format
    /// The generated response contains:
    /// - Appropriate HTTP status code
    /// - JSON body with error details
    /// - Proper content-type headers
    /// - Timestamp information
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Error, Response};
    /// use http::StatusCode;
    ///
    /// // Convert a NotFound error
    /// let error = Error::NotFound("User not found".to_string());
    /// let response = Response::from(error);
    /// assert_eq!(response.status, StatusCode::NOT_FOUND);
    ///
    /// // The response body will be JSON:
    /// // {
    /// //   "error": "Not Found",
    /// //   "message": "User not found",
    /// //   "status": 404,
    /// //   "error_type": "not_found",
    /// //   "timestamp": "2023-01-01T12:00:00Z"
    /// // }
    /// ```
    fn from(err: Error) -> Self {
        let status = err.status_code();
        let error_response = err.to_response(true);

        // Try JSON first, fallback to plain text
        match serde_json::to_vec(&error_response) {
            Ok(json_body) => {
                let mut response = Response::new(status);
                response.headers.insert(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("application/json"),
                );
                response.body = Arc::new(bytes::Bytes::from(json_body));
                response
            }
            Err(_) => {
                let mut response = Response::new(status);
                response.headers.insert(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("text/plain; charset=utf-8"),
                );
                response.body = Arc::new(bytes::Bytes::from(err.to_string()));
                response
            }
        }
    }
}

// Enhanced response methods
impl Response {
    /// Creates a JSON error response from an error.
    ///
    /// This method provides more control over error response generation than
    /// the automatic From implementation. It always returns JSON and provides
    /// better error handling for serialization failures.
    ///
    /// # Type Parameters
    /// - `E`: Error type that can be converted to framework Error
    ///
    /// # Parameters
    /// - `error`: The error to convert to a response
    ///
    /// # Returns
    /// - `Ok(Response)`: Successfully created error response
    /// - `Err(Error)`: JSON serialization error
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Response, Error};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let error = Error::Validation("Email format is invalid".to_string());
    /// let response = Response::error_json(error)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn error_json<E: Into<Error>>(error: E) -> crate::Result<Self> {
        let err = error.into();
        let status = err.status_code();
        let error_response = err.to_response(true);

        let mut response = Self::new(status);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        response.body = Arc::new(bytes::Bytes::from(serde_json::to_vec(&error_response)?));
        Ok(response)
    }

    /// Creates a validation error response with multiple error messages.
    ///
    /// This method creates a structured validation error response that can
    /// contain multiple validation failure messages.
    ///
    /// # Parameters
    /// - `messages`: Vector of validation error messages
    ///
    /// # Returns
    /// - `Ok(Response)`: Successfully created validation error response
    /// - `Err(Error)`: JSON serialization error
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let validation_errors = vec![
    ///     "Name is required".to_string(),
    ///     "Email format is invalid".to_string(),
    ///     "Password must be at least 8 characters".to_string(),
    /// ];
    ///
    /// let response = Response::validation_error(validation_errors)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Response Format
    /// The generated response has this structure:
    /// ```
    /// {
    ///   "error": "Validation Failed",
    ///   "message": "Name is required, Email format is invalid, Password must be at least 8 characters",
    ///   "status": 400,
    ///   "error_type": "validation_error",
    ///   "error_code": "VALIDATION_FAILED",
    ///   "metadata": {
    ///     "validation_errors": ["Name is required", "Email format is invalid", "Password must be at least 8 characters"]
    ///   },
    ///   "timestamp": "2023-01-01T12:00:00Z"
    /// }
    /// ```
    ///
    /// ## Usage in Form Validation
    /// ```
    /// use ignitia::{Response, Request, Result};
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct UserForm {
    ///     name: String,
    ///     email: String,
    ///     password: String,
    /// }
    ///
    /// async fn validate_user_form(req: Request) -> Result<Response> {
    ///     let form: UserForm = req.json()?;
    ///     let mut errors = Vec::new();
    ///
    ///     if form.name.trim().is_empty() {
    ///         errors.push("Name is required".to_string());
    ///     }
    ///
    ///     if !form.email.contains('@') {
    ///         errors.push("Invalid email format".to_string());
    ///     }
    ///
    ///     if form.password.len() < 8 {
    ///         errors.push("Password must be at least 8 characters".to_string());
    ///     }
    ///
    ///     if !errors.is_empty() {
    ///         return Response::validation_error(errors);
    ///     }
    ///
    ///     Ok(Response::json(serde_json::json!({
    ///         "message": "User created successfully"
    ///     }))?)
    /// }
    /// ```
    pub fn validation_error(messages: Vec<String>) -> crate::Result<Self> {
        let error_response = ErrorResponse {
            error: "Validation Failed".to_string(),
            message: messages.join(", "),
            status: 400,
            error_type: Some("validation_error".to_string()),
            error_code: Some("VALIDATION_FAILED".to_string()),
            metadata: Some(serde_json::json!({
                "validation_errors": messages
            })),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        };

        let mut response = Self::new(StatusCode::BAD_REQUEST);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        response.body = Arc::new(bytes::Bytes::from(serde_json::to_vec(&error_response)?));
        Ok(response)
    }
}

// Helper function to escape HTML entities
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
