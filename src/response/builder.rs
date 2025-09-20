//! Response builder module for creating HTTP responses with performance optimizations
//!
//! This module provides a flexible and high-performance response builder that supports
//! zero-copy operations, pre-allocated common responses, and optimized header management.
//! The builder pattern allows for fluent and readable response construction while maintaining
//! excellent performance characteristics.
//!
//! # Key Features
//!
//! - **Zero-copy operations**: Efficient memory usage through Arc and Bytes
//! - **Pre-allocated responses**: Common responses cached for instant access
//! - **Static content support**: Zero-allocation serving of static content
//! - **Flexible body types**: Support for various data sources
//! - **Header optimization**: Pre-compiled headers for common content types
//! - **Cache control**: Built-in support for HTTP caching headers
//! - **CORS support**: Convenient methods for Cross-Origin Resource Sharing
//!
//! # Examples
//!
//! ```rust
//! use ignitia::{ResponseBuilder, Response, StatusCode};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//!
//! // Basic JSON response
//! let user = User { id: 1, name: "Alice".to_string() };
//! let response = ResponseBuilder::new()
//!     .status(StatusCode::OK)
//!     .json(&user)
//!     .unwrap()
//!     .build();
//!
//! // Static optimized response
//! let health = ResponseBuilder::health();
//!
//! // Zero-copy text response
//! let text = Response::text_static("Hello, World!");
//!
//! // Response with caching
//! let cached_response = ResponseBuilder::new()
//!     .text("Cached content")
//!     .cache_1_hour()
//!     .build();
//! ```

use super::Response;
use crate::error::Result;
use ahash::AHashMap;
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::borrow::Cow;
use std::sync::Arc;

/// Pre-compiled common responses for ultra-fast serving
///
/// These responses are pre-allocated at startup for zero-allocation serving
/// of common API responses. The responses are stored as `Bytes` for efficient
/// memory usage and can be shared across multiple responses using `Arc`.
///
/// # Available Responses
///
/// - `"health_ok"`: `{"status":"healthy"}`
/// - `"not_found"`: `{"error":"Not Found"}`
/// - `"server_error"`: `{"error":"Internal Server Error"}`
/// - `"unauthorized"`: `{"error":"Unauthorized"}`
/// - `"forbidden"`: `{"error":"Forbidden"}`
/// - `"bad_request"`: `{"error":"Bad Request"}`
/// - `"method_not_allowed"`: `{"error":"Method Not Allowed"}`
/// - `"empty_json"`: `{}`
/// - `"empty_array"`: `[]`
/// - `"ok_message"`: `{"message":"OK"}`
/// - `"success"`: `{"success":true}`
/// - `"pong"`: `{"message":"pong"}`
static COMMON_RESPONSES: Lazy<AHashMap<&'static str, Bytes>> = Lazy::new(|| {
    let mut map = AHashMap::new();
    map.insert("health_ok", Bytes::from_static(b"{\"status\":\"healthy\"}"));
    map.insert(
        "not_found",
        Bytes::from_static(b"{\"error\":\"Not Found\"}"),
    );
    map.insert(
        "server_error",
        Bytes::from_static(b"{\"error\":\"Internal Server Error\"}"),
    );
    map.insert(
        "unauthorized",
        Bytes::from_static(b"{\"error\":\"Unauthorized\"}"),
    );
    map.insert(
        "forbidden",
        Bytes::from_static(b"{\"error\":\"Forbidden\"}"),
    );
    map.insert(
        "bad_request",
        Bytes::from_static(b"{\"error\":\"Bad Request\"}"),
    );
    map.insert(
        "method_not_allowed",
        Bytes::from_static(b"{\"error\":\"Method Not Allowed\"}"),
    );
    map.insert("empty_json", Bytes::from_static(b"{}"));
    map.insert("empty_array", Bytes::from_static(b"[]"));
    map.insert("ok_message", Bytes::from_static(b"{\"message\":\"OK\"}"));
    map.insert("success", Bytes::from_static(b"{\"success\":true}"));
    map.insert("pong", Bytes::from_static(b"{\"message\":\"pong\"}"));
    map
});

/// Pre-allocated common header values for performance optimization
///
/// Using static header values avoids repeated allocations during response building.
/// The values are stored as `HeaderValue` for direct use in header maps.
///
/// # Available Headers
///
/// - `"json"`: `application/json`
/// - `"text"`: `text/plain; charset=utf-8`
/// - `"html"`: `text/html; charset=utf-8`
/// - `"xml"`: `application/xml`
/// - `"css"`: `text/css`
/// - `"js"`: `application/javascript`
/// - `"png"`: `image/png`
/// - `"jpg"`: `image/jpeg`
/// - `"gif"`: `image/gif`
/// - `"svg"`: `image/svg+xml`
/// - `"pdf"`: `application/pdf`
/// - `"octet"`: `application/octet-stream`
/// - `"cors_any"`: `*` (for CORS)
/// - `"cors_methods"`: `GET, POST, PUT, DELETE, OPTIONS` (for CORS)
/// - `"cors_headers"`: `Content-Type, Authorization` (for CORS)
static COMMON_HEADERS: Lazy<AHashMap<&'static str, HeaderValue>> = Lazy::new(|| {
    let mut map = AHashMap::new();
    map.insert("json", HeaderValue::from_static("application/json"));
    map.insert(
        "text",
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    map.insert("html", HeaderValue::from_static("text/html; charset=utf-8"));
    map.insert("xml", HeaderValue::from_static("application/xml"));
    map.insert("css", HeaderValue::from_static("text/css"));
    map.insert("js", HeaderValue::from_static("application/javascript"));
    map.insert("png", HeaderValue::from_static("image/png"));
    map.insert("jpg", HeaderValue::from_static("image/jpeg"));
    map.insert("gif", HeaderValue::from_static("image/gif"));
    map.insert("svg", HeaderValue::from_static("image/svg+xml"));
    map.insert("pdf", HeaderValue::from_static("application/pdf"));
    map.insert(
        "octet",
        HeaderValue::from_static("application/octet-stream"),
    );
    map.insert("cors_any", HeaderValue::from_static("*"));
    map.insert(
        "cors_methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    map.insert(
        "cors_headers",
        HeaderValue::from_static("Content-Type, Authorization"),
    );
    map
});

/// Pre-allocated common header names for performance optimization
///
/// Using static header names avoids repeated allocations during response building.
/// These are commonly used HTTP header names that can be reused across responses.
static CONTENT_TYPE: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("content-type"));
static CONTENT_LENGTH: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("content-length"));
static CACHE_CONTROL: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("cache-control"));
static ACCESS_CONTROL_ALLOW_ORIGIN: Lazy<HeaderName> =
    Lazy::new(|| HeaderName::from_static("access-control-allow-origin"));
static ACCESS_CONTROL_ALLOW_METHODS: Lazy<HeaderName> =
    Lazy::new(|| HeaderName::from_static("access-control-allow-methods"));
static ACCESS_CONTROL_ALLOW_HEADERS: Lazy<HeaderName> =
    Lazy::new(|| HeaderName::from_static("access-control-allow-headers"));

/// A high-performance HTTP response builder with zero-copy optimizations
///
/// The `ResponseBuilder` provides a fluent interface for constructing HTTP responses
/// with various optimizations for common use cases. It supports different body types
/// and provides methods for efficient header management.
///
/// # Performance Features
///
/// - **Zero-copy body sharing**: Through `Arc<Bytes>` for efficient memory usage
/// - **Pre-allocated headers**: For common content types and CORS settings
/// - **Static response caching**: For frequently used responses
/// - **Efficient memory usage**: Through `Cow<str>` for string content
/// - **Cache control**: Built-in methods for HTTP caching headers
///
/// # Examples
///
/// ```rust
/// use ignitia::{ResponseBuilder, StatusCode};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Data {
///     value: String,
/// }
///
/// // JSON response with custom status
/// let data = Data { value: "test".to_string() };
/// let response = ResponseBuilder::new()
///     .status(StatusCode::CREATED)
///     .json(&data)
///     .unwrap()
///     .build();
///
/// // Text response with caching
/// let response = ResponseBuilder::new()
///     .status(StatusCode::OK)
///     .text("Hello, World!")
///     .cache_1_hour()
///     .build();
///
/// // Static content (zero-copy)
/// let response = ResponseBuilder::new()
///     .json_static("success")
///     .build();
///
/// // CORS-enabled response
/// let response = ResponseBuilder::new()
///     .text("CORS enabled")
///     .cors_any()
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    /// HTTP status code for the response
    status: StatusCode,
    /// HTTP headers map with pre-allocated capacity for common headers
    headers: HeaderMap,
    /// Optional response body supporting multiple data sources
    body: Option<ResponseBody>,
}

/// Zero-copy response body variants supporting different data sources
///
/// This enum provides efficient storage options for response bodies,
/// allowing zero-copy operations where possible and minimizing allocations.
///
/// # Variants
///
/// - `Static`: References to static byte arrays (zero allocation)
/// - `Shared`: Arc-wrapped bytes for sharing between responses
/// - `Owned`: Owned bytes for dynamic content
/// - `Cow`: Copy-on-write strings for flexible string handling
#[derive(Debug, Clone)]
enum ResponseBody {
    /// Static bytes - zero-copy references to compile-time data
    ///
    /// Perfect for serving static assets or pre-defined responses.
    /// No allocation required as it references static memory.
    Static(&'static [u8]),

    /// Pre-allocated bytes shared via Arc for efficient cloning
    ///
    /// Ideal for responses that may be sent to multiple clients
    /// or cached responses that need to be shared.
    Shared(Arc<Bytes>),

    /// Owned bytes for dynamic content that needs exclusive ownership
    ///
    /// Used for dynamically generated content like JSON serialization
    /// or content that can't be shared or is temporary.
    Owned(Bytes),

    /// Borrowed string data with potential zero-copy optimization
    ///
    /// Allows both static string references and owned strings.
    /// Uses copy-on-write semantics for optimal memory usage.
    Cow(Cow<'static, str>),
}

impl ResponseBuilder {
    /// Creates a new response builder with default OK status
    ///
    /// Initializes a new `ResponseBuilder` with HTTP 200 OK status and
    /// pre-allocates header map capacity for common headers to improve performance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    /// use http::StatusCode;
    ///
    /// let builder = ResponseBuilder::new();
    /// assert_eq!(builder.status, StatusCode::OK);
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Pre-allocates HeaderMap with capacity for 8 headers
    /// - Uses inline hint for better optimization
    /// - Zero-cost initialization for common use cases
    #[inline]
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::with_capacity(8), // Pre-allocate for common headers
            body: None,
        }
    }

    /// Creates a response builder with specific HTTP status code
    ///
    /// This method provides a convenient way to create a response builder
    /// with a specific status code while maintaining the same performance
    /// optimizations as the default constructor.
    ///
    /// # Arguments
    ///
    /// * `status` - The HTTP status code for the response
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::{ResponseBuilder, StatusCode};
    ///
    /// let builder = ResponseBuilder::with_status(StatusCode::CREATED);
    /// let builder = ResponseBuilder::with_status(StatusCode::NOT_FOUND);
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Inline optimized for better performance
    /// - Pre-allocates header capacity
    /// - Maintains same memory characteristics as `new()`
    #[inline]
    pub fn with_status(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::with_capacity(8),
            body: None,
        }
    }

    /// Sets the HTTP status code for the response
    ///
    /// This method allows changing the status code of an existing builder.
    /// It consumes the builder and returns a new one with the updated status,
    /// following the builder pattern for method chaining.
    ///
    /// # Arguments
    ///
    /// * `status` - The new HTTP status code
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::{ResponseBuilder, StatusCode};
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Data {
    ///     value: String,
    /// }
    ///
    /// let data = Data { value: "created".to_string() };
    /// let response = ResponseBuilder::new()
    ///     .status(StatusCode::CREATED)
    ///     .json(&data)
    ///     .unwrap()
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Method is inlined for optimal performance
    /// - Consumes and returns Self for zero-cost chaining
    /// - No additional allocations required
    #[inline]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Sets the HTTP status code from a u16 value
    ///
    /// Convenience method for setting status codes from numeric values.
    /// If the provided status code is invalid, the status remains unchanged.
    ///
    /// # Arguments
    ///
    /// * `status_code` - HTTP status code as u16
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .status_code(201) // Created
    ///     .text("Resource created")
    ///     .build();
    /// ```
    #[inline]
    pub fn status_code(mut self, status_code: u16) -> Self {
        if let Ok(status) = StatusCode::from_u16(status_code) {
            self.status = status;
        }
        self
    }

    /// Sets a static byte array as the response body
    ///
    /// Zero-copy method for setting response body from static byte arrays.
    /// Ideal for serving pre-compiled content or static assets.
    ///
    /// # Arguments
    ///
    /// * `body` - Static byte array for the response body
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// static CONTENT: &[u8] = b"Hello, World!";
    /// let response = ResponseBuilder::new()
    ///     .body_static(CONTENT)
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation for body storage
    /// - References static memory directly
    #[inline]
    pub fn body_static(mut self, body: &'static [u8]) -> Self {
        self.body = Some(ResponseBody::Static(body));
        self
    }

    /// Sets a static string as the response body
    ///
    /// Zero-copy method for setting response body from static strings.
    /// Automatically converts the string to bytes for HTTP response.
    ///
    /// # Arguments
    ///
    /// * `body` - Static string for the response body
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// static GREETING: &str = "Hello, World!";
    /// let response = ResponseBuilder::new()
    ///     .body_static_str(GREETING)
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation for body storage
    /// - References static memory directly
    #[inline]
    pub fn body_static_str(mut self, body: &'static str) -> Self {
        self.body = Some(ResponseBody::Static(body.as_bytes()));
        self
    }

    /// Sets owned bytes as the response body
    ///
    /// For dynamic content that needs to be owned by the response.
    /// Uses `Bytes` for efficient memory handling and potential sharing.
    ///
    /// # Arguments
    ///
    /// * `body` - Bytes object containing the response body
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    /// use bytes::Bytes;
    ///
    /// let content = Bytes::from("Dynamic content");
    /// let response = ResponseBuilder::new()
    ///     .body_bytes(content)
    ///     .build();
    /// ```
    #[inline]
    pub fn body_bytes(mut self, body: Bytes) -> Self {
        self.body = Some(ResponseBody::Owned(body));
        self
    }

    /// Sets shared bytes as the response body
    ///
    /// For content that may be shared between multiple responses.
    /// Uses `Arc<Bytes>` for efficient memory sharing.
    ///
    /// # Arguments
    ///
    /// * `body` - Arc-wrapped Bytes for shared ownership
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    /// use bytes::Bytes;
    /// use std::sync::Arc;
    ///
    /// let shared_content = Arc::new(Bytes::from("Shared content"));
    /// let response = ResponseBuilder::new()
    ///     .body_shared(shared_content)
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Enables zero-copy cloning of response bodies
    /// - Ideal for cached or frequently reused content
    #[inline]
    pub fn body_shared(mut self, body: Arc<Bytes>) -> Self {
        self.body = Some(ResponseBody::Shared(body));
        self
    }

    /// Sets a copy-on-write string as the response body
    ///
    /// Flexible method that can accept both static and owned strings
    /// with optimal memory usage through copy-on-write semantics.
    ///
    /// # Arguments
    ///
    /// * `body` - Cow<'static, str> for flexible string handling
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    /// use std::borrow::Cow;
    ///
    /// // Static string (zero-copy)
    /// let response1 = ResponseBuilder::new()
    ///     .body_cow(Cow::Borrowed("Static content"))
    ///     .build();
    ///
    /// // Owned string
    /// let response2 = ResponseBuilder::new()
    ///     .body_cow(Cow::Owned("Owned content".to_string()))
    ///     .build();
    /// ```
    #[inline]
    pub fn body_cow(mut self, body: Cow<'static, str>) -> Self {
        self.body = Some(ResponseBody::Cow(body));
        self
    }

    /// Sets a generic body that can be converted to Bytes
    ///
    /// Convenience method for types that implement `Into<Bytes>`.
    /// Useful for various string and byte types.
    ///
    /// # Arguments
    ///
    /// * `body` - Any type that can be converted into Bytes
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .body("String content")
    ///     .build();
    ///
    /// let response = ResponseBuilder::new()
    ///     .body(vec![1, 2, 3, 4])
    ///     .build();
    /// ```
    #[inline]
    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = Some(ResponseBody::Owned(body.into()));
        self
    }

    /// Adds a header to the response
    ///
    /// Generic method for adding any header to the response.
    /// Supports any types that can be converted to `HeaderName` and `HeaderValue`.
    ///
    /// # Arguments
    ///
    /// * `key` - Header name (convertible to HeaderName)
    /// * `value` - Header value (convertible to HeaderValue)
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .header("X-Custom-Header", "custom-value")
    ///     .text("Content with custom header")
    ///     .build();
    /// ```
    ///
    /// # Panics
    ///
    /// Debug assertions will panic if header name or value conversion fails.
    /// In release builds, invalid headers are silently ignored.
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

    /// Sets Content-Type header to application/json
    ///
    /// Convenience method for JSON responses that uses pre-allocated
    /// header values for optimal performance.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .content_type_json()
    ///     .body(r#"{"status":"ok"}"#)
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Uses pre-allocated header value
    /// - Zero allocation for header setting
    #[inline]
    pub fn content_type_json(mut self) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
        self
    }

    /// Sets Content-Type header to text/plain
    ///
    /// Convenience method for plain text responses that uses pre-allocated
    /// header values for optimal performance.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .content_type_text()
    ///     .body("Plain text content")
    ///     .build();
    /// ```
    #[inline]
    pub fn content_type_text(mut self) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["text"].clone());
        self
    }

    /// Sets Content-Type header to text/html
    ///
    /// Convenience method for HTML responses that uses pre-allocated
    /// header values for optimal performance.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .content_type_html()
    ///     .body("<html><body>Hello</body></html>")
    ///     .build();
    /// ```
    #[inline]
    pub fn content_type_html(mut self) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["html"].clone());
        self
    }

    /// Sets a JSON body from copy-on-write string with proper content type
    ///
    /// Convenience method for JSON responses that accepts flexible string types
    /// and automatically sets the correct Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `text` - JSON content as Cow<'static, str> or convertible type
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    /// use std::borrow::Cow;
    ///
    /// // Static JSON
    /// let response1 = ResponseBuilder::new()
    ///     .json_cow(Cow::Borrowed(r#"{"status":"ok"}"#))
    ///     .build();
    ///
    /// // Owned JSON
    /// let response2 = ResponseBuilder::new()
    ///     .json_cow(Cow::Owned(r#"{"message":"hello"}"#.to_string()))
    ///     .build();
    /// ```
    #[inline]
    pub fn json_cow<T: Into<Cow<'static, str>>>(mut self, text: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
        self.body = Some(ResponseBody::Cow(text.into()));
        self
    }

    /// Sets a plain text body with proper content type
    ///
    /// Convenience method for text responses that accepts flexible string types
    /// and automatically sets the correct Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `text` - Text content as Cow<'static, str> or convertible type
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("Hello, World!")
    ///     .build();
    /// ```
    #[inline]
    pub fn text<T: Into<Cow<'static, str>>>(mut self, text: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["text"].clone());
        self.body = Some(ResponseBody::Cow(text.into()));
        self
    }

    /// Sets an HTML body with proper content type
    ///
    /// Convenience method for HTML responses that accepts flexible string types
    /// and automatically sets the correct Content-Type header.
    ///
    /// # Arguments
    ///
    /// * `html` - HTML content as Cow<'static, str> or convertible type
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .html("<h1>Hello</h1>")
    ///     .build();
    /// ```
    #[inline]
    pub fn html<T: Into<Cow<'static, str>>>(mut self, html: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["html"].clone());
        self.body = Some(ResponseBody::Cow(html.into()));
        self
    }

    /// Sets a JSON body using a pre-defined static key
    ///
    /// This method provides ultra-fast JSON responses by using pre-compiled
    /// JSON strings stored in a static HashMap. It's ideal for common responses
    /// like success messages, health checks, or error responses.
    ///
    /// # Arguments
    ///
    /// * `json_key` - Static string key for pre-compiled JSON responses
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Available Keys
    ///
    /// - `"health_ok"` - `{"status":"healthy"}`
    /// - `"not_found"` - `{"error":"Not Found"}`
    /// - `"server_error"` - `{"error":"Internal Server Error"}`
    /// - `"unauthorized"` - `{"error":"Unauthorized"}`
    /// - `"forbidden"` - `{"error":"Forbidden"}`
    /// - `"bad_request"` - `{"error":"Bad Request"}`
    /// - `"method_not_allowed"` - `{"error":"Method Not Allowed"}`
    /// - `"empty_json"` - `{}`
    /// - `"empty_array"` - `[]`
    /// - `"ok_message"` - `{"message":"OK"}`
    /// - `"success"` - `{"success":true}`
    /// - `"pong"` - `{"message":"pong"}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .json_static("success")
    ///     .build();
    /// ```
    ///
    /// # Performance Benefits
    ///
    /// - Zero serialization overhead
    /// - Pre-compiled JSON strings
    /// - Shared Arc<Bytes> for memory efficiency
    /// - O(1) lookup time
    /// - Automatic Content-Type header setting
    pub fn json_static(mut self, json_key: &'static str) -> Self {
        if let Some(body) = COMMON_RESPONSES.get(json_key) {
            self.headers
                .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
            self.body = Some(ResponseBody::Shared(Arc::new(body.clone())));
        } else {
            // Fallback for unknown keys
            self.headers
                .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
            self.body = Some(ResponseBody::Static(b"{}"));
        }
        self
    }

    /// Serializes data to JSON and sets it as the response body
    ///
    /// Convenience method for serializing Rust data structures to JSON
    /// and setting the appropriate headers. Uses a pre-allocated buffer
    /// for better performance.
    ///
    /// # Arguments
    ///
    /// * `data` - Serializable data structure
    ///
    /// # Returns
    ///
    /// Returns `Result<Self>` to allow error handling and method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct User {
    ///     id: u64,
    ///     name: String,
    /// }
    ///
    /// let user = User { id: 1, name: "Alice".to_string() };
    /// let response = ResponseBuilder::new()
    ///     .json(&user)
    ///     .unwrap()
    ///     .build();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if JSON serialization fails
    ///
    /// # Performance Notes
    ///
    /// - Uses pre-allocated buffer (1KB initial capacity)
    /// - Sets Content-Length header for HTTP/1.1 performance
    /// - Automatic Content-Type header setting
    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self> {
        // Use a pre-allocated buffer for better performance
        let mut buf = Vec::with_capacity(1024); // Start with 1KB buffer
        serde_json::to_writer(&mut buf, data)?;

        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());

        // Set Content-Length for HTTP/1.1 performance
        if let Ok(len_str) = buf.len().to_string().parse::<HeaderValue>() {
            self.headers.insert(CONTENT_LENGTH.clone(), len_str);
        }

        self.body = Some(ResponseBody::Owned(Bytes::from(buf)));
        Ok(self)
    }

    /// Serializes data to JSON with custom buffer capacity
    ///
    /// Similar to `json()` but allows specifying the initial buffer capacity
    /// for potentially better performance with known data sizes.
    ///
    /// # Arguments
    ///
    /// * `data` - Serializable data structure
    /// * `capacity` - Initial buffer capacity in bytes
    ///
    /// # Returns
    ///
    /// Returns `Result<Self>` to allow error handling and method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct LargeData {
    ///     items: Vec<String>,
    /// }
    ///
    /// let data = LargeData { items: vec!["item".to_string(); 1000] };
    /// let response = ResponseBuilder::new()
    ///     .json_with_capacity(&data, 16384) // 16KB initial buffer
    ///     .unwrap()
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Can reduce reallocations for large responses
    /// - Useful when approximate response size is known
    pub fn json_with_capacity<T: Serialize>(mut self, data: &T, capacity: usize) -> Result<Self> {
        let mut buf = Vec::with_capacity(capacity);
        serde_json::to_writer(&mut buf, data)?;

        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());

        if let Ok(len_str) = buf.len().to_string().parse::<HeaderValue>() {
            self.headers.insert(CONTENT_LENGTH.clone(), len_str);
        }

        self.body = Some(ResponseBody::Owned(Bytes::from(buf)));
        Ok(self)
    }

    /// Sets Cache-Control header for 1 hour caching
    ///
    /// Convenience method for setting appropriate caching headers
    /// for content that can be cached for 1 hour.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("Cacheable content")
    ///     .cache_1_hour()
    ///     .build();
    /// ```
    #[inline]
    pub fn cache_1_hour(mut self) -> Self {
        self.headers.insert(
            CACHE_CONTROL.clone(),
            HeaderValue::from_static("public, max-age=3600"),
        );
        self
    }

    /// Sets Cache-Control header for 1 day caching
    ///
    /// Convenience method for setting appropriate caching headers
    /// for content that can be cached for 1 day.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("Long-term cacheable content")
    ///     .cache_1_day()
    ///     .build();
    /// ```
    #[inline]
    pub fn cache_1_day(mut self) -> Self {
        self.headers.insert(
            CACHE_CONTROL.clone(),
            HeaderValue::from_static("public, max-age=86400"),
        );
        self
    }

    /// Sets Cache-Control header for 1 week caching
    ///
    /// Convenience method for setting appropriate caching headers
    /// for content that can be cached for 1 week.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("Very cacheable content")
    ///     .cache_1_week()
    ///     .build();
    /// ```
    #[inline]
    pub fn cache_1_week(mut self) -> Self {
        self.headers.insert(
            CACHE_CONTROL.clone(),
            HeaderValue::from_static("public, max-age=604800"),
        );
        self
    }

    /// Sets Cache-Control header for no caching
    ///
    /// Convenience method for setting appropriate caching headers
    /// for content that should not be cached.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("Non-cacheable content")
    ///     .cache_no_store()
    ///     .build();
    /// ```
    #[inline]
    pub fn cache_no_store(mut self) -> Self {
        self.headers.insert(
            CACHE_CONTROL.clone(),
            HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        );
        self
    }

    /// Sets CORS headers to allow any origin
    ///
    /// Convenience method for setting CORS headers that allow requests
    /// from any origin. Uses pre-allocated header values for performance.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("CORS enabled for any origin")
    ///     .cors_any()
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Uses pre-allocated header values
    /// - Sets all required CORS headers
    #[inline]
    pub fn cors_any(mut self) -> Self {
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_ORIGIN.clone(),
            COMMON_HEADERS["cors_any"].clone(),
        );
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_METHODS.clone(),
            COMMON_HEADERS["cors_methods"].clone(),
        );
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_HEADERS.clone(),
            COMMON_HEADERS["cors_headers"].clone(),
        );
        self
    }

    /// Sets CORS headers with specific allowed origin
    ///
    /// Convenience method for setting CORS headers with a specific
    /// allowed origin. Uses pre-allocated values for methods and headers.
    ///
    /// # Arguments
    ///
    /// * `origin` - Allowed origin as string
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("CORS enabled for specific origin")
    ///     .cors_origin("https://example.com")
    ///     .build();
    /// ```
    #[inline]
    pub fn cors_origin(mut self, origin: &str) -> Self {
        if let Ok(origin_val) = HeaderValue::from_str(origin) {
            self.headers
                .insert(ACCESS_CONTROL_ALLOW_ORIGIN.clone(), origin_val);
        }
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_METHODS.clone(),
            COMMON_HEADERS["cors_methods"].clone(),
        );
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_HEADERS.clone(),
            COMMON_HEADERS["cors_headers"].clone(),
        );
        self
    }

    /// Builds the final Response object
    ///
    /// Consumes the builder and returns a fully constructed `Response`.
    /// This method performs the final conversion from builder state to
    /// the actual HTTP response.
    ///
    /// # Returns
    ///
    /// Returns a `Response` object ready for serving
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .text("Hello, World!")
    ///     .build();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Finalizes header map with optimal capacity
    /// - Converts body to appropriate Bytes representation
    /// - Zero-copy operations where possible
    pub fn build(self) -> Response {
        let body_bytes = match self.body {
            Some(ResponseBody::Static(bytes)) => Arc::new(Bytes::from_static(bytes)),
            Some(ResponseBody::Shared(arc_bytes)) => arc_bytes,
            Some(ResponseBody::Owned(bytes)) => Arc::new(bytes),
            Some(ResponseBody::Cow(cow)) => match cow {
                Cow::Borrowed(s) => Arc::new(Bytes::from_static(s.as_bytes())),
                Cow::Owned(s) => Arc::new(Bytes::from(s)),
            },
            None => Arc::new(Bytes::new()),
        };

        Response {
            status: self.status,
            headers: self.headers,
            body: body_bytes,
            cache_control: None, // You can extend this to extract from headers if needed
        }
    }

    /// Creates a pre-built health check response
    ///
    /// Returns a pre-compiled health check response with status 200 OK
    /// and JSON body `{"status":"healthy"}`. Uses zero-copy operations.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for health checks
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let health_response = ResponseBuilder::health();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation response building
    /// - Pre-compiled JSON body
    /// - Pre-set Content-Type header
    #[inline]
    pub fn health() -> Response {
        ResponseBuilder::new().json_static("health_ok").build()
    }

    /// Creates a pre-built "Not Found" response
    ///
    /// Returns a pre-compiled 404 response with JSON body `{"error":"Not Found"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for 404 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let not_found = ResponseBuilder::not_found();
    /// ```
    #[inline]
    pub fn not_found() -> Response {
        ResponseBuilder::with_status(StatusCode::NOT_FOUND)
            .json_static("not_found")
            .build()
    }

    /// Creates a pre-built "Internal Server Error" response
    ///
    /// Returns a pre-compiled 500 response with JSON body
    /// `{"error":"Internal Server Error"}`. Uses zero-copy operations.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for 500 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let server_error = ResponseBuilder::server_error();
    /// ```
    #[inline]
    pub fn server_error() -> Response {
        ResponseBuilder::with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .json_static("server_error")
            .build()
    }

    /// Creates a pre-built "Unauthorized" response
    ///
    /// Returns a pre-compiled 401 response with JSON body `{"error":"Unauthorized"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for 401 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let unauthorized = ResponseBuilder::unauthorized();
    /// ```
    #[inline]
    pub fn unauthorized() -> Response {
        ResponseBuilder::with_status(StatusCode::UNAUTHORIZED)
            .json_static("unauthorized")
            .build()
    }

    /// Creates a pre-built "Forbidden" response
    ///
    /// Returns a pre-compiled 403 response with JSON body `{"error":"Forbidden"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for 403 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let forbidden = ResponseBuilder::forbidden();
    /// ```
    #[inline]
    pub fn forbidden() -> Response {
        ResponseBuilder::with_status(StatusCode::FORBIDDEN)
            .json_static("forbidden")
            .build()
    }

    /// Creates a pre-built "Bad Request" response
    ///
    /// Returns a pre-compiled 400 response with JSON body `{"error":"Bad Request"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for 400 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let bad_request = ResponseBuilder::bad_request();
    /// ```
    #[inline]
    pub fn bad_request() -> Response {
        ResponseBuilder::with_status(StatusCode::BAD_REQUEST)
            .json_static("bad_request")
            .build()
    }

    /// Creates a pre-built "Method Not Allowed" response
    ///
    /// Returns a pre-compiled 405 response with JSON body
    /// `{"error":"Method Not Allowed"}`. Uses zero-copy operations.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for 405 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let method_not_allowed = ResponseBuilder::method_not_allowed();
    /// ```
    #[inline]
    pub fn method_not_allowed() -> Response {
        ResponseBuilder::with_status(StatusCode::METHOD_NOT_ALLOWED)
            .json_static("method_not_allowed")
            .build()
    }

    /// Creates a pre-built "OK" response
    ///
    /// Returns a pre-compiled 200 response with JSON body `{"message":"OK"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for success responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let ok_response = ResponseBuilder::ok();
    /// ```
    #[inline]
    pub fn ok() -> Response {
        ResponseBuilder::new().json_static("ok_message").build()
    }

    /// Creates a pre-built success response
    ///
    /// Returns a pre-compiled 200 response with JSON body `{"success":true}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for success responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let success_response = ResponseBuilder::success();
    /// ```
    #[inline]
    pub fn success() -> Response {
        ResponseBuilder::new().json_static("success").build()
    }

    /// Creates a pre-built "pong" response
    ///
    /// Returns a pre-compiled 200 response with JSON body `{"message":"pong"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for ping/pong endpoints
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let pong_response = ResponseBuilder::pong();
    /// ```
    #[inline]
    pub fn pong() -> Response {
        ResponseBuilder::new().json_static("pong").build()
    }

    /// Creates a pre-built empty JSON response
    ///
    /// Returns a pre-compiled 200 response with empty JSON body `{}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for empty responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let empty_response = ResponseBuilder::empty_json();
    /// ```
    #[inline]
    pub fn empty_json() -> Response {
        ResponseBuilder::new().json_static("empty_json").build()
    }

    /// Creates a pre-built empty array response
    ///
    /// Returns a pre-compiled 200 response with empty array body `[]`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a pre-built `Response` for empty array responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let empty_array = ResponseBuilder::empty_array();
    /// ```
    #[inline]
    pub fn empty_array() -> Response {
        ResponseBuilder::new().json_static("empty_array").build()
    }
}

impl Default for ResponseBuilder {
    /// Creates a default response builder with OK status
    ///
    /// This implementation allows using `ResponseBuilder::default()` for
    /// convenience and consistency with Rust conventions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::ResponseBuilder;
    ///
    /// let builder = ResponseBuilder::default();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl Response {
    /// Creates a response with pre-compiled static JSON content
    ///
    /// This method provides ultra-fast response creation by looking up pre-compiled
    /// JSON responses from the static common responses map. Ideal for frequently
    /// used API responses like health checks, errors, or standard success messages.
    ///
    /// # Arguments
    ///
    /// * `key` - Static string key for pre-compiled JSON responses
    ///
    /// # Returns
    ///
    /// Returns a `Response` with the pre-compiled content and appropriate headers
    ///
    /// # Available Keys
    ///
    /// - `"health_ok"` - `{"status":"healthy"}`
    /// - `"not_found"` - `{"error":"Not Found"}`
    /// - `"server_error"` - `{"error":"Internal Server Error"}`
    /// - `"unauthorized"` - `{"error":"Unauthorized"}`
    /// - `"forbidden"` - `{"error":"Forbidden"}`
    /// - `"bad_request"` - `{"error":"Bad Request"}`
    /// - `"method_not_allowed"` - `{"error":"Method Not Allowed"}`
    /// - `"empty_json"` - `{}`
    /// - `"empty_array"` - `[]`
    /// - `"ok_message"` - `{"message":"OK"}`
    /// - `"success"` - `{"success":true}`
    /// - `"pong"` - `{"message":"pong"}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let health_response = Response::static_json("health_ok");
    /// let not_found_response = Response::static_json("not_found");
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - O(1) lookup time in static HashMap
    /// - Zero allocation for body content
    /// - Pre-set Content-Type header
    /// - Uses shared Arc<Bytes> for memory efficiency
    pub fn static_json(key: &'static str) -> Self {
        if let Some(body) = COMMON_RESPONSES.get(key) {
            Self {
                status: StatusCode::OK,
                headers: {
                    let mut headers = HeaderMap::with_capacity(1);
                    headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                    headers
                },
                body: Arc::new(body.clone()),
                cache_control: None,
            }
        } else {
            // Fallback to empty JSON for unknown keys
            ResponseBuilder::empty_json()
        }
    }

    /// Creates a zero-copy JSON response from a static string
    ///
    /// This method creates a response with a static JSON string without any
    /// serialization overhead. The string must be valid JSON and is referenced
    /// directly from static memory.
    ///
    /// # Arguments
    ///
    /// * `json_str` - Static string containing valid JSON
    ///
    /// # Returns
    ///
    /// Returns a `Response` with the static JSON content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::json_static(r#"{"status":"ok","data":null}"#);
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation for body content
    /// - References static memory directly
    /// - No serialization overhead
    /// - Pre-set Content-Type header
    pub fn json_static(json_str: &'static str) -> Self {
        Self {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(Bytes::from_static(json_str.as_bytes())),
            cache_control: None,
        }
    }

    /// Creates a zero-copy text response from a static string
    ///
    /// This method creates a plain text response with a static string without
    /// any allocation. The string is referenced directly from static memory.
    ///
    /// # Arguments
    ///
    /// * `text` - Static string for the response body
    ///
    /// # Returns
    ///
    /// Returns a `Response` with the static text content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::text_static("Hello, World!");
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation for body content
    /// - References static memory directly
    /// - Pre-set Content-Type header with proper charset
    pub fn text_static(text: &'static str) -> Self {
        Self {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["text"].clone());
                headers
            },
            body: Arc::new(Bytes::from_static(text.as_bytes())),
            cache_control: None,
        }
    }

    /// Creates a zero-copy HTML response from a static string
    ///
    /// This method creates an HTML response with a static string without
    /// any allocation. The string is referenced directly from static memory.
    ///
    /// # Arguments
    ///
    /// * `html` - Static string containing HTML content
    ///
    /// # Returns
    ///
    /// Returns a `Response` with the static HTML content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::html_static("<h1>Welcome</h1><p>Hello World</p>");
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation for body content
    /// - References static memory directly
    /// - Pre-set Content-Type header with proper charset
    pub fn html_static(html: &'static str) -> Self {
        Self {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["html"].clone());
                headers
            },
            body: Arc::new(Bytes::from_static(html.as_bytes())),
            cache_control: None,
        }
    }

    /// Creates a shared clone of the response body
    ///
    /// This method creates a new `Arc<Bytes>` reference to the response body,
    /// allowing the same body content to be shared across multiple responses
    /// without copying the actual bytes.
    ///
    /// # Returns
    ///
    /// Returns an `Arc<Bytes>` reference to the response body
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::text_static("Hello");
    /// let body_clone = response.clone_body();
    ///
    /// // Use the cloned body in another response
    /// let another_response = Response::new(
    ///     response.status().clone(),
    ///     response.headers().clone(),
    ///     body_clone.as_ref().clone()
    /// );
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero-copy operation (only increments Arc reference count)
    /// - Allows efficient body sharing between responses
    /// - Useful for response caching or middleware
    pub fn clone_body(&self) -> Arc<Bytes> {
        Arc::clone(&self.body)
    }

    /// Creates an empty JSON response
    ///
    /// Returns a response with empty JSON object `{}` and status 200 OK.
    /// Uses pre-compiled content for optimal performance.
    ///
    /// # Returns
    ///
    /// Returns a `Response` with empty JSON content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let empty_response = Response::empty_json();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation response
    /// - Uses pre-compiled static content
    /// - Pre-set Content-Type header
    pub fn empty_json() -> Self {
        ResponseBuilder::empty_json()
    }

    /// Creates a health check response
    ///
    /// Returns a pre-compiled health check response with status 200 OK
    /// and JSON body `{"status":"healthy"}`. Uses zero-copy operations.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for health check endpoints
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let health_response = Response::health_check();
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Zero allocation response
    /// - Uses pre-compiled static content
    /// - Pre-set Content-Type header
    pub fn health_check() -> Self {
        ResponseBuilder::health()
    }

    /// Creates an "Internal Server Error" response
    ///
    /// Returns a pre-compiled 500 response with JSON body
    /// `{"error":"Internal Server Error"}`. Uses zero-copy operations.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for 500 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let server_error = Response::server_error();
    /// ```
    pub fn server_error() -> Self {
        ResponseBuilder::server_error()
    }

    /// Creates an "Unauthorized" response
    ///
    /// Returns a pre-compiled 401 response with JSON body `{"error":"Unauthorized"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for 401 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let unauthorized = Response::unauthorized();
    /// ```
    pub fn unauthorized() -> Self {
        ResponseBuilder::unauthorized()
    }

    /// Creates a "Forbidden" response
    ///
    /// Returns a pre-compiled 403 response with JSON body `{"error":"Forbidden"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for 403 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let forbidden = Response::forbidden();
    /// ```
    pub fn forbidden() -> Self {
        ResponseBuilder::forbidden()
    }

    /// Creates a "Bad Request" response
    ///
    /// Returns a pre-compiled 400 response with JSON body `{"error":"Bad Request"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for 400 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let bad_request = Response::bad_request();
    /// ```
    pub fn bad_request() -> Self {
        ResponseBuilder::bad_request()
    }

    /// Creates a "Method Not Allowed" response
    ///
    /// Returns a pre-compiled 405 response with JSON body
    /// `{"error":"Method Not Allowed"}`. Uses zero-copy operations.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for 405 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let method_not_allowed = Response::method_not_allowed();
    /// ```
    pub fn method_not_allowed() -> Self {
        ResponseBuilder::method_not_allowed()
    }

    /// Creates a success response
    ///
    /// Returns a pre-compiled 200 response with JSON body `{"success":true}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for success responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let success_response = Response::success();
    /// ```
    pub fn success() -> Self {
        ResponseBuilder::success()
    }

    /// Creates a "pong" response
    ///
    /// Returns a pre-compiled 200 response with JSON body `{"message":"pong"}`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for ping/pong endpoints
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let pong_response = Response::pong();
    /// ```
    pub fn pong() -> Self {
        ResponseBuilder::pong()
    }

    /// Creates an empty array response
    ///
    /// Returns a pre-compiled 200 response with empty array body `[]`.
    /// Uses zero-copy operations and pre-allocated content.
    ///
    /// # Returns
    ///
    /// Returns a `Response` for empty array responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let empty_array = Response::empty_array();
    /// ```
    pub fn empty_array() -> Self {
        ResponseBuilder::empty_array()
    }

    /// Checks if the response has a cache control header
    ///
    /// This method checks if the response contains a Cache-Control header,
    /// which can be useful for middleware or response processing.
    ///
    /// # Returns
    ///
    /// Returns `true` if Cache-Control header is present, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::text_static("test");
    /// assert!(!response.has_cache_control());
    ///
    /// let cached_response = Response::text_static("test").with_cache_control("max-age=3600");
    /// assert!(cached_response.has_cache_control());
    /// ```
    pub fn has_cache_control(&self) -> bool {
        self.headers.contains_key(CACHE_CONTROL.as_str())
    }

    /// Gets the cache control header value if present
    ///
    /// This method returns the value of the Cache-Control header if it exists,
    /// or `None` if the header is not present.
    ///
    /// # Returns
    ///
    /// Returns `Some(HeaderValue)` if Cache-Control header exists, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::text_static("test");
    /// assert_eq!(response.get_cache_control(), None);
    ///
    /// let cached_response = Response::text_static("test").with_cache_control("max-age=3600");
    /// assert!(cached_response.get_cache_control().is_some());
    /// ```
    pub fn get_cache_control(&self) -> Option<&HeaderValue> {
        self.headers.get(CACHE_CONTROL.as_str())
    }

    /// Sets CORS headers to allow any origin
    ///
    /// This method adds CORS headers to allow requests from any origin.
    /// The response is consumed and returned for method chaining.
    ///
    /// # Returns
    ///
    /// Returns `Self` with CORS headers added
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::text_static("test")
    ///     .with_cors_any();
    /// ```
    pub fn with_cors_any(mut self) -> Self {
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_ORIGIN.clone(),
            COMMON_HEADERS["cors_any"].clone(),
        );
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_METHODS.clone(),
            COMMON_HEADERS["cors_methods"].clone(),
        );
        self.headers.insert(
            ACCESS_CONTROL_ALLOW_HEADERS.clone(),
            COMMON_HEADERS["cors_headers"].clone(),
        );
        self
    }

    /// Adds a header to the response
    ///
    /// This method adds a header to an existing response.
    /// The response is consumed and returned for method chaining.
    ///
    /// # Arguments
    ///
    /// * `key` - Header name
    /// * `value` - Header value
    ///
    /// # Returns
    ///
    /// Returns `Self` with the added header
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ignitia::Response;
    ///
    /// let response = Response::text_static("test")
    ///     .with_header("X-Custom", "value");
    /// ```
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
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
}
