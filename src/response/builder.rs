use super::Response;
use crate::error::Result;
use ahash::AHashMap;
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

// Pre-allocated common responses for zero-copy serving
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

// Pre-allocated common headers for zero-copy header setting
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

// Pre-allocated common header names for performance
static CONTENT_TYPE: HeaderName = HeaderName::from_static("content-type");
static CONTENT_LENGTH: HeaderName = HeaderName::from_static("content-length");
static CACHE_CONTROL: HeaderName = HeaderName::from_static("cache-control");
static ACCESS_CONTROL_ALLOW_ORIGIN: HeaderName =
    HeaderName::from_static("access-control-allow-origin");
static ACCESS_CONTROL_ALLOW_METHODS: HeaderName =
    HeaderName::from_static("access-control-allow-methods");
static ACCESS_CONTROL_ALLOW_HEADERS: HeaderName =
    HeaderName::from_static("access-control-allow-headers");

#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
    body: Option<ResponseBody>,
}

/// Zero-copy response body variants
#[derive(Debug, Clone)]
enum ResponseBody {
    /// Static bytes (zero-copy)
    Static(&'static [u8]),
    /// Pre-allocated bytes (shared via Arc)
    Shared(Arc<Bytes>),
    /// Owned bytes (for dynamic content)
    Owned(Bytes),
    /// Borrowed string data with potential zero-copy
    Cow(Cow<'static, str>),
}

impl ResponseBuilder {
    /// Create a new response builder with default OK status
    #[inline]
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::with_capacity(8), // Pre-allocate for common headers
            body: None,
        }
    }

    /// Create response builder with specific status
    #[inline]
    pub fn with_status(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::with_capacity(8),
            body: None,
        }
    }

    /// Set status code (builder pattern)
    #[inline]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Set status by numeric code
    #[inline]
    pub fn status_code(mut self, status_code: u16) -> Self {
        if let Ok(status) = StatusCode::from_u16(status_code) {
            self.status = status;
        }
        self
    }

    /// Zero-copy body setting using static bytes
    #[inline]
    pub fn body_static(mut self, body: &'static [u8]) -> Self {
        self.body = Some(ResponseBody::Static(body));
        self
    }

    /// Zero-copy body setting using static string
    #[inline]
    pub fn body_static_str(mut self, body: &'static str) -> Self {
        self.body = Some(ResponseBody::Static(body.as_bytes()));
        self
    }

    /// Zero-copy body setting using Bytes
    #[inline]
    pub fn body_bytes(mut self, body: Bytes) -> Self {
        self.body = Some(ResponseBody::Owned(body));
        self
    }

    /// Zero-copy body setting using shared Arc<Bytes>
    #[inline]
    pub fn body_shared(mut self, body: Arc<Bytes>) -> Self {
        self.body = Some(ResponseBody::Shared(body));
        self
    }

    /// Use Cow for potentially borrowed string data
    #[inline]
    pub fn body_cow(mut self, body: Cow<'static, str>) -> Self {
        self.body = Some(ResponseBody::Cow(body));
        self
    }

    /// Generic body method (for compatibility)
    #[inline]
    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = Some(ResponseBody::Owned(body.into()));
        self
    }

    /// Set header with zero-copy optimization for common headers
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

    /// Fast header setting for common content types
    #[inline]
    pub fn content_type_json(mut self) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
        self
    }

    #[inline]
    pub fn content_type_text(mut self) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["text"].clone());
        self
    }

    #[inline]
    pub fn content_type_html(mut self) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["html"].clone());
        self
    }

    /// Pre-compiled JSON response from static data
    pub fn json_static(self, json_key: &'static str) -> Self {
        if let Some(body) = COMMON_RESPONSES.get(json_key) {
            self.content_type_json().body_shared(Arc::new(body.clone()))
        } else {
            // Fallback for unknown keys - use the builder pattern
            self.content_type_json().body_static_str("{}")
        }
    }

    /// High-performance JSON serialization with pre-allocated buffer
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

    /// High-performance JSON with pre-allocated capacity hint
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

    /// Zero-copy text response
    #[inline]
    pub fn text<T: Into<Cow<'static, str>>>(mut self, text: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["text"].clone());
        self.body = Some(ResponseBody::Cow(text.into()));
        self
    }

    /// Zero-copy HTML response
    #[inline]
    pub fn html<T: Into<Cow<'static, str>>>(mut self, html: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["html"].clone());
        self.body = Some(ResponseBody::Cow(html.into()));
        self
    }

    /// Pre-compiled common responses for ultra-fast serving
    #[inline]
    pub fn ok_json_static() -> Self {
        Self::with_status(StatusCode::OK)
            .content_type_json()
            .body_shared(Arc::new(COMMON_RESPONSES["health_ok"].clone()))
    }

    #[inline]
    pub fn not_found_static() -> Self {
        Self::with_status(StatusCode::NOT_FOUND)
            .content_type_json()
            .body_shared(Arc::new(COMMON_RESPONSES["not_found"].clone()))
    }

    #[inline]
    pub fn server_error_static() -> Self {
        Self::with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type_json()
            .body_shared(Arc::new(COMMON_RESPONSES["server_error"].clone()))
    }

    #[inline]
    pub fn health_check() -> Self {
        Self::with_status(StatusCode::OK)
            .content_type_json()
            .body_shared(Arc::new(COMMON_RESPONSES["health_ok"].clone()))
    }

    #[inline]
    pub fn pong() -> Self {
        Self::with_status(StatusCode::OK)
            .content_type_json()
            .body_shared(Arc::new(COMMON_RESPONSES["pong"].clone()))
    }

    /// Fast CORS headers
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

    /// Cache control headers - Fixed the HeaderValue::from_static issue
    pub fn cache_control(mut self, value: &'static str) -> Self {
        let header_value = HeaderValue::from_static(value); // Direct assignment, not Result
        self.headers.insert(CACHE_CONTROL.clone(), header_value);
        self
    }

    #[inline]
    pub fn no_cache(mut self) -> Self {
        self.cache_control("no-cache, no-store, must-revalidate")
    }

    #[inline]
    pub fn cache_1_hour(mut self) -> Self {
        self.cache_control("public, max-age=3600")
    }

    #[inline]
    pub fn cache_1_day(mut self) -> Self {
        self.cache_control("public, max-age=86400")
    }

    /// Build the final response with zero-copy optimizations
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
        }
    }
}

// Convenience static constructors for ultra-fast responses
impl ResponseBuilder {
    /// Ultra-fast JSON response for APIs
    pub fn api_success() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["success"].clone()),
        }
    }

    /// Ultra-fast health check response
    pub fn health() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["health_ok"].clone()),
        }
    }

    /// Ultra-fast empty JSON object - Fixed the missing method
    pub fn empty_json() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["empty_json"].clone()),
        }
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Enhanced Response methods for zero-copy operations
impl Response {
    /// Create response with pre-compiled static content
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
            }
        } else {
            // Fixed: Use ResponseBuilder's empty_json method
            ResponseBuilder::empty_json()
        }
    }

    /// Zero-copy JSON response from static string
    pub fn json_static(json_str: &'static str) -> Self {
        Self {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(Bytes::from_static(json_str.as_bytes())),
        }
    }

    /// Zero-copy text response from static string
    pub fn text_static(text: &'static str) -> Self {
        Self {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["text"].clone());
                headers
            },
            body: Arc::new(Bytes::from_static(text.as_bytes())),
        }
    }

    /// Zero-copy HTML response from static string
    pub fn html_static(html: &'static str) -> Self {
        Self {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["html"].clone());
                headers
            },
            body: Arc::new(Bytes::from_static(html.as_bytes())),
        }
    }

    /// Share body between responses (zero-copy clone)
    pub fn clone_body(&self) -> Arc<Bytes> {
        Arc::clone(&self.body)
    }

    /// Fixed: Add the missing empty_json method
    pub fn empty_json() -> Self {
        ResponseBuilder::empty_json()
    }

    /// Common health check response
    pub fn health_check() -> Self {
        ResponseBuilder::health()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_responses() {
        let response = ResponseBuilder::health();
        assert_eq!(response.status, StatusCode::OK);
        assert!(!response.body.is_empty());
    }

    #[test]
    fn test_zero_copy_text() {
        let response = ResponseBuilder::new()
            .text(Cow::Borrowed("Hello, World!"))
            .build();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_pre_compiled_json() {
        let response = ResponseBuilder::new().json_static("health_ok").build();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[test]
    fn test_cache_control() {
        let response = ResponseBuilder::new().cache_1_hour().build();
        assert!(response.headers.contains_key(&CACHE_CONTROL));
    }
}
