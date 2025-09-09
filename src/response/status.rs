//! # HTTP Status Code Extensions Module
//!
//! This module provides extension traits for HTTP status codes in the Ignitia web framework.
//! It extends the standard `http::StatusCode` with additional utility methods for categorizing
//! and working with status codes in a more intuitive way.
//!
//! ## Features
//!
//! - **Status Code Categories**: Methods to check if status codes belong to specific categories
//! - **Intuitive API**: Easy-to-use methods with descriptive names
//! - **Performance Optimized**: Efficient range checks using numeric comparisons
//! - **Standard Compliant**: Based on RFC 7231 HTTP status code definitions
//!
//! ## Status Code Categories
//!
//! HTTP status codes are divided into five categories:
//!
//! ### 1xx - Informational Responses
//! - **100 Continue**: The server has received the request headers
//! - **101 Switching Protocols**: The server is switching protocols
//! - **102 Processing**: The server is processing the request (WebDAV)
//!
//! ### 2xx - Success Responses
//! - **200 OK**: The request succeeded
//! - **201 Created**: The request succeeded and created a new resource
//! - **202 Accepted**: The request has been received but not yet acted upon
//! - **204 No Content**: The request succeeded but returns no content
//!
//! ### 3xx - Redirection Messages
//! - **301 Moved Permanently**: The resource has permanently moved
//! - **302 Found**: The resource has temporarily moved
//! - **304 Not Modified**: The resource has not been modified
//! - **307 Temporary Redirect**: Temporary redirect with method preservation
//!
//! ### 4xx - Client Error Responses
//! - **400 Bad Request**: The request was malformed
//! - **401 Unauthorized**: Authentication is required
//! - **403 Forbidden**: The server refuses to authorize the request
//! - **404 Not Found**: The requested resource was not found
//! - **422 Unprocessable Entity**: The request was well-formed but semantically invalid
//!
//! ### 5xx - Server Error Responses
//! - **500 Internal Server Error**: The server encountered an unexpected error
//! - **501 Not Implemented**: The server does not support the functionality
//! - **502 Bad Gateway**: The server received an invalid response from upstream
//! - **503 Service Unavailable**: The server is temporarily unavailable
//!
//! ## Usage Examples
//!
//! ### Basic Status Code Checking
//! ```
//! use ignitia::response::status::StatusCodeExt;
//! use http::StatusCode;
//!
//! let status = StatusCode::OK;
//! assert!(status.is_success());
//! assert!(!status.is_client_error());
//!
//! let status = StatusCode::NOT_FOUND;
//! assert!(status.is_client_error());
//! assert!(!status.is_success());
//!
//! let status = StatusCode::INTERNAL_SERVER_ERROR;
//! assert!(status.is_server_error());
//! assert!(!status.is_success());
//! ```
//!
//! ### Response Processing Based on Status Category
//! ```
//! use ignitia::response::status::StatusCodeExt;
//! use ignitia::{Request, Response, Result};
//! use http::StatusCode;
//!
//! async fn process_api_response(status: StatusCode, body: &str) -> String {
//!     match status {
//!         s if s.is_success() => {
//!             format!("✅ Success: {}", body)
//!         }
//!         s if s.is_client_error() => {
//!             format!("❌ Client Error ({}): {}", s.as_u16(), body)
//!         }
//!         s if s.is_server_error() => {
//!             format!("💥 Server Error ({}): {}", s.as_u16(), body)
//!         }
//!         s if s.is_redirection() => {
//!             format!("🔄 Redirect ({}): {}", s.as_u16(), body)
//!         }
//!         s if s.is_informational() => {
//!             format!("ℹ️ Info ({}): {}", s.as_u16(), body)
//!         }
//!         _ => {
//!             format!("❓ Unknown status ({}): {}", status.as_u16(), body)
//!         }
//!     }
//! }
//! ```
//!
//! ## Advanced Usage Patterns
//!
//! ### Error Response Categorization
//! ```
//! use ignitia::response::status::StatusCodeExt;
//! use ignitia::{Response, Error};
//! use http::StatusCode;
//!
//! fn categorize_error_response(response: &Response) -> &'static str {
//!     match response.status {
//!         s if s.is_client_error() => match s {
//!             StatusCode::BAD_REQUEST => "validation_error",
//!             StatusCode::UNAUTHORIZED => "authentication_error",
//!             StatusCode::FORBIDDEN => "authorization_error",
//!             StatusCode::NOT_FOUND => "resource_not_found",
//!             StatusCode::CONFLICT => "conflict_error",
//!             StatusCode::UNPROCESSABLE_ENTITY => "business_logic_error",
//!             _ => "client_error"
//!         },
//!         s if s.is_server_error() => match s {
//!             StatusCode::INTERNAL_SERVER_ERROR => "internal_error",
//!             StatusCode::BAD_GATEWAY => "upstream_error",
//!             StatusCode::SERVICE_UNAVAILABLE => "service_unavailable",
//!             StatusCode::GATEWAY_TIMEOUT => "timeout_error",
//!             _ => "server_error"
//!         },
//!         _ => "unknown_error"
//!     }
//! }
//! ```
//!
//! ### Retry Logic Based on Status Code
//! ```
//! use ignitia::response::status::StatusCodeExt;
//! use http::StatusCode;
//!
//! fn should_retry_request(status: StatusCode, attempt: u32) -> bool {
//!     const MAX_RETRIES: u32 = 3;
//!
//!     if attempt >= MAX_RETRIES {
//!         return false;
//!     }
//!
//!     match status {
//!         // Retry on server errors
//!         s if s.is_server_error() => true,
//!
//!         // Retry on specific client errors
//!         StatusCode::REQUEST_TIMEOUT => true,
//!         StatusCode::TOO_MANY_REQUESTS => true,
//!
//!         // Don't retry on other client errors or success
//!         _ => false
//!     }
//! }
//!
//! async fn make_request_with_retry() -> Result<Response, Box<dyn std::error::Error>> {
//!     let mut attempt = 0;
//!
//!     loop {
//!         // Simulated request
//!         let status = StatusCode::SERVICE_UNAVAILABLE;
//!
//!         if status.is_success() {
//!             // Return successful response
//!             break Ok(ignitia::Response::new(status));
//!         }
//!
//!         attempt += 1;
//!         if !should_retry_request(status, attempt) {
//!             break Err(format!("Request failed with status: {}", status.as_u16()).into());
//!         }
//!
//!         // Wait before retry (exponential backoff)
//!         let delay = std::time::Duration::from_millis(100 * 2_u64.pow(attempt));
//!         tokio::time::sleep(delay).await;
//!     }
//! }
//! ```
//!
//! ### Logging Based on Status Code Category
//! ```
//! use ignitia::response::status::StatusCodeExt;
//! use ignitia::Response;
//! use tracing::{info, warn, error, debug};
//!
//! fn log_response(response: &Response, request_path: &str) {
//!     let status = response.status;
//!     let status_code = status.as_u16();
//!
//!     match status {
//!         s if s.is_success() => {
//!             info!("✅ {} - {} - Success", status_code, request_path);
//!         }
//!         s if s.is_redirection() => {
//!             info!("🔄 {} - {} - Redirect", status_code, request_path);
//!         }
//!         s if s.is_client_error() => {
//!             warn!("⚠️ {} - {} - Client Error", status_code, request_path);
//!         }
//!         s if s.is_server_error() => {
//!             error!("💥 {} - {} - Server Error", status_code, request_path);
//!         }
//!         s if s.is_informational() => {
//!             debug!("ℹ️ {} - {} - Informational", status_code, request_path);
//!         }
//!         _ => {
//!             warn!("❓ {} - {} - Unknown Status", status_code, request_path);
//!         }
//!     }
//! }
//! ```
//!
//! ### HTTP Client Response Handling
//! ```
//! use ignitia::response::status::StatusCodeExt;
//! use http::StatusCode;
//!
//! #[derive(Debug)]
//! enum ApiResult<T> {
//!     Success(T),
//!     ClientError(String),
//!     ServerError(String),
//!     Redirect(String),
//! }
//!
//! fn handle_api_response<T>(
//!     status: StatusCode,
//!     data: T,
//!     error_message: Option<String>
//! ) -> ApiResult<T> {
//!     match status {
//!         s if s.is_success() => ApiResult::Success(data),
//!         s if s.is_redirection() => {
//!             ApiResult::Redirect(error_message.unwrap_or_else(||
//!                 format!("Redirect required: {}", s.as_u16())
//!             ))
//!         }
//!         s if s.is_client_error() => {
//!             ApiResult::ClientError(error_message.unwrap_or_else(||
//!                 format!("Client error: {}", s.as_u16())
//!             ))
//!         }
//!         s if s.is_server_error() => {
//!             ApiResult::ServerError(error_message.unwrap_or_else(||
//!                 format!("Server error: {}", s.as_u16())
//!             ))
//!         }
//!         _ => {
//!             ApiResult::ClientError(format!("Unexpected status: {}", status.as_u16()))
//!         }
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Efficient Range Checking
//! The extension methods use simple numeric range comparisons which are highly optimized:
//! - Status codes are converted to u16 once
//! - Range checks use simple integer comparisons
//! - No string parsing or complex matching
//!
//! ### Inlined Methods
//! All methods are marked with `#[inline]` for optimal performance in hot paths.
//!
//! ## Testing Status Code Extensions
//!
//! ### Unit Tests
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use http::StatusCode;
//!
//!     #[test]
//!     fn test_informational_status_codes() {
//!         assert!(StatusCode::CONTINUE.is_informational());
//!         assert!(StatusCode::SWITCHING_PROTOCOLS.is_informational());
//!         assert!(!StatusCode::OK.is_informational());
//!     }
//!
//!     #[test]
//!     fn test_success_status_codes() {
//!         assert!(StatusCode::OK.is_success());
//!         assert!(StatusCode::CREATED.is_success());
//!         assert!(StatusCode::NO_CONTENT.is_success());
//!         assert!(!StatusCode::NOT_FOUND.is_success());
//!     }
//!
//!     #[test]
//!     fn test_client_error_status_codes() {
//!         assert!(StatusCode::BAD_REQUEST.is_client_error());
//!         assert!(StatusCode::NOT_FOUND.is_client_error());
//!         assert!(StatusCode::UNPROCESSABLE_ENTITY.is_client_error());
//!         assert!(!StatusCode::OK.is_client_error());
//!     }
//!
//!     #[test]
//!     fn test_server_error_status_codes() {
//!         assert!(StatusCode::INTERNAL_SERVER_ERROR.is_server_error());
//!         assert!(StatusCode::BAD_GATEWAY.is_server_error());
//!         assert!(StatusCode::SERVICE_UNAVAILABLE.is_server_error());
//!         assert!(!StatusCode::NOT_FOUND.is_server_error());
//!     }
//!
//!     #[test]
//!     fn test_redirection_status_codes() {
//!         assert!(StatusCode::MOVED_PERMANENTLY.is_redirection());
//!         assert!(StatusCode::FOUND.is_redirection());
//!         assert!(StatusCode::NOT_MODIFIED.is_redirection());
//!         assert!(!StatusCode::OK.is_redirection());
//!     }
//! }
//! ```

use http::StatusCode;

/// Extension trait for HTTP status codes providing category checking methods.
///
/// This trait extends `http::StatusCode` with convenient methods to check which
/// category a status code belongs to, based on RFC 7231 definitions.
///
/// # Categories
/// - **Informational (1xx)**: Request received, continuing process
/// - **Success (2xx)**: Request successfully received, understood, and accepted
/// - **Redirection (3xx)**: Further action needs to be taken to complete the request
/// - **Client Error (4xx)**: Request contains bad syntax or cannot be fulfilled
/// - **Server Error (5xx)**: Server failed to fulfill an apparently valid request
///
/// # Examples
/// ```
/// use ignitia::response::status::StatusCodeExt;
/// use http::StatusCode;
///
/// let status = StatusCode::OK;
/// assert!(status.is_success());
///
/// let status = StatusCode::NOT_FOUND;
/// assert!(status.is_client_error());
/// ```
pub trait StatusCodeExt {
    /// Returns true if the status code is informational (1xx).
    ///
    /// Informational responses indicate that the request was received and understood.
    /// These interim responses consist only of the status line and optional headers.
    ///
    /// # Range
    /// Status codes from 100 to 199 (inclusive)
    ///
    /// # Examples
    /// ```
    /// use ignitia::response::status::StatusCodeExt;
    /// use http::StatusCode;
    ///
    /// assert!(StatusCode::CONTINUE.is_informational());           // 100
    /// assert!(StatusCode::SWITCHING_PROTOCOLS.is_informational()); // 101
    /// assert!(!StatusCode::OK.is_informational());                // 200
    /// ```
    fn is_informational(&self) -> bool;

    /// Returns true if the status code indicates success (2xx).
    ///
    /// Success responses indicate that the client's request was successfully
    /// received, understood, and accepted.
    ///
    /// # Range
    /// Status codes from 200 to 299 (inclusive)
    ///
    /// # Examples
    /// ```
    /// use ignitia::response::status::StatusCodeExt;
    /// use http::StatusCode;
    ///
    /// assert!(StatusCode::OK.is_success());              // 200
    /// assert!(StatusCode::CREATED.is_success());         // 201
    /// assert!(StatusCode::ACCEPTED.is_success());        // 202
    /// assert!(StatusCode::NO_CONTENT.is_success());      // 204
    /// assert!(!StatusCode::NOT_FOUND.is_success());      // 404
    /// ```
    fn is_success(&self) -> bool;

    /// Returns true if the status code indicates redirection (3xx).
    ///
    /// Redirection responses indicate that further action needs to be taken
    /// by the user agent in order to fulfill the request.
    ///
    /// # Range
    /// Status codes from 300 to 399 (inclusive)
    ///
    /// # Examples
    /// ```
    /// use ignitia::response::status::StatusCodeExt;
    /// use http::StatusCode;
    ///
    /// assert!(StatusCode::MOVED_PERMANENTLY.is_redirection());  // 301
    /// assert!(StatusCode::FOUND.is_redirection());             // 302
    /// assert!(StatusCode::NOT_MODIFIED.is_redirection());      // 304
    /// assert!(StatusCode::TEMPORARY_REDIRECT.is_redirection()); // 307
    /// assert!(!StatusCode::OK.is_redirection());               // 200
    /// ```
    fn is_redirection(&self) -> bool;

    /// Returns true if the status code indicates a client error (4xx).
    ///
    /// Client error responses indicate that the client seems to have made an error.
    /// These status codes are applicable to any request method.
    ///
    /// # Range
    /// Status codes from 400 to 499 (inclusive)
    ///
    /// # Examples
    /// ```
    /// use ignitia::response::status::StatusCodeExt;
    /// use http::StatusCode;
    ///
    /// assert!(StatusCode::BAD_REQUEST.is_client_error());         // 400
    /// assert!(StatusCode::UNAUTHORIZED.is_client_error());        // 401
    /// assert!(StatusCode::FORBIDDEN.is_client_error());           // 403
    /// assert!(StatusCode::NOT_FOUND.is_client_error());           // 404
    /// assert!(StatusCode::UNPROCESSABLE_ENTITY.is_client_error()); // 422
    /// assert!(!StatusCode::OK.is_client_error());                 // 200
    /// ```
    fn is_client_error(&self) -> bool;

    /// Returns true if the status code indicates a server error (5xx).
    ///
    /// Server error responses indicate that the server is aware that it has
    /// made an error or is incapable of performing the requested method.
    ///
    /// # Range
    /// Status codes from 500 to 599 (inclusive)
    ///
    /// # Examples
    /// ```
    /// use ignitia::response::status::StatusCodeExt;
    /// use http::StatusCode;
    ///
    /// assert!(StatusCode::INTERNAL_SERVER_ERROR.is_server_error()); // 500
    /// assert!(StatusCode::NOT_IMPLEMENTED.is_server_error());       // 501
    /// assert!(StatusCode::BAD_GATEWAY.is_server_error());           // 502
    /// assert!(StatusCode::SERVICE_UNAVAILABLE.is_server_error());   // 503
    /// assert!(StatusCode::GATEWAY_TIMEOUT.is_server_error());       // 504
    /// assert!(!StatusCode::NOT_FOUND.is_server_error());            // 404
    /// ```
    fn is_server_error(&self) -> bool;
}

impl StatusCodeExt for StatusCode {
    #[inline]
    fn is_informational(&self) -> bool {
        self.as_u16() >= 100 && self.as_u16() < 200
    }

    #[inline]
    fn is_success(&self) -> bool {
        self.as_u16() >= 200 && self.as_u16() < 300
    }

    #[inline]
    fn is_redirection(&self) -> bool {
        self.as_u16() >= 300 && self.as_u16() < 400
    }

    #[inline]
    fn is_client_error(&self) -> bool {
        self.as_u16() >= 400 && self.as_u16() < 500
    }

    #[inline]
    fn is_server_error(&self) -> bool {
        self.as_u16() >= 500 && self.as_u16() < 600
    }
}
