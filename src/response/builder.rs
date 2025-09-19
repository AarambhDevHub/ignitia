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

// Pre-allocated common headers for zero-copy serving
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
static CONTENT_TYPE: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("content-type"));
static CONTENT_LENGTH: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("content-length"));
static CACHE_CONTROL: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("cache-control"));
static ACCESS_CONTROL_ALLOW_ORIGIN: Lazy<HeaderName> =
    Lazy::new(|| HeaderName::from_static("access-control-allow-origin"));
static ACCESS_CONTROL_ALLOW_METHODS: Lazy<HeaderName> =
    Lazy::new(|| HeaderName::from_static("access-control-allow-methods"));
static ACCESS_CONTROL_ALLOW_HEADERS: Lazy<HeaderName> =
    Lazy::new(|| HeaderName::from_static("access-control-allow-headers"));

#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
    body: Option<ResponseBody>,
}

// Zero-copy response body variants
#[derive(Debug, Clone)]
enum ResponseBody {
    // Static bytes (zero-copy)
    Static(&'static [u8]),
    // Pre-allocated bytes shared via Arc
    Shared(Arc<Bytes>),
    // Owned bytes for dynamic content
    Owned(Bytes),
    // Borrowed string data with potential zero-copy
    Cow(Cow<'static, str>),
}

impl ResponseBuilder {
    // Create a new response builder with default OK status
    #[inline]
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::with_capacity(8), // Pre-allocate for common headers
            body: None,
        }
    }

    // Create response builder with specific status
    #[inline]
    pub fn with_status(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::with_capacity(8),
            body: None,
        }
    }

    // FIXED: Changed to consume self and return Self instead of &mut Self
    #[inline]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn status_code(mut self, status_code: u16) -> Self {
        if let Ok(status) = StatusCode::from_u16(status_code) {
            self.status = status;
        }
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn body_static(mut self, body: &'static [u8]) -> Self {
        self.body = Some(ResponseBody::Static(body));
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn body_static_str(mut self, body: &'static str) -> Self {
        self.body = Some(ResponseBody::Static(body.as_bytes()));
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn body_bytes(mut self, body: Bytes) -> Self {
        self.body = Some(ResponseBody::Owned(body));
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn body_shared(mut self, body: Arc<Bytes>) -> Self {
        self.body = Some(ResponseBody::Shared(body));
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn body_cow(mut self, body: Cow<'static, str>) -> Self {
        self.body = Some(ResponseBody::Cow(body));
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = Some(ResponseBody::Owned(body.into()));
        self
    }

    // FIXED: Changed to consume self and return Self
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

    // FIXED: Changed to consume self and return Self
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

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn json_cow<T: Into<Cow<'static, str>>>(mut self, text: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
        self.body = Some(ResponseBody::Cow(text.into()));
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn text<T: Into<Cow<'static, str>>>(mut self, text: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["text"].clone());
        self.body = Some(ResponseBody::Cow(text.into()));
        self
    }

    // FIXED: Changed to consume self and return Self
    #[inline]
    pub fn html<T: Into<Cow<'static, str>>>(mut self, html: T) -> Self {
        self.headers
            .insert(CONTENT_TYPE.clone(), COMMON_HEADERS["html"].clone());
        self.body = Some(ResponseBody::Cow(html.into()));
        self
    }

    // FIXED: Changed to consume self and return Self
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

    // FIXED: Changed to consume self and return Result<Self>
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

    // FIXED: Changed to consume self and return Result<Self>
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

    // FIXED: Changed to consume self and return Self
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

    // NEW: Cache control headers - FIXED: consume self
    pub fn cache_control(mut self, value: &str) -> Self {
        if let Ok(header_value) = HeaderValue::from_str(value) {
            self.headers.insert(CACHE_CONTROL.clone(), header_value);
        }
        self
    }

    // NEW: Static cache control for common values - FIXED: consume self
    pub fn cache_control_static(mut self, value: &'static str) -> Self {
        let header_value = HeaderValue::from_static(value);
        self.headers.insert(CACHE_CONTROL.clone(), header_value);
        self
    }

    #[inline]
    pub fn no_cache(self) -> Self {
        self.cache_control_static("no-cache, no-store, must-revalidate")
    }

    #[inline]
    pub fn cache_1_hour(self) -> Self {
        self.cache_control_static("public, max-age=3600")
    }

    #[inline]
    pub fn cache_1_day(self) -> Self {
        self.cache_control_static("public, max-age=86400")
    }

    // NEW: Cache control with dynamic max-age - FIXED: consume self
    pub fn cache_max_age(mut self, seconds: u64) -> Self {
        let cache_value = format!("public, max-age={}", seconds);
        if let Ok(header_value) = HeaderValue::from_str(&cache_value) {
            self.headers.insert(CACHE_CONTROL.clone(), header_value);
        }
        self
    }

    // NEW: Private cache control - FIXED: consume self
    pub fn cache_private(mut self, seconds: u64) -> Self {
        let cache_value = format!("private, max-age={}", seconds);
        if let Ok(header_value) = HeaderValue::from_str(&cache_value) {
            self.headers.insert(CACHE_CONTROL.clone(), header_value);
        }
        self
    }

    // NEW: Cache control with must-revalidate - FIXED: consume self
    pub fn cache_must_revalidate(mut self, seconds: u64) -> Self {
        let cache_value = format!("public, max-age={}, must-revalidate", seconds);
        if let Ok(header_value) = HeaderValue::from_str(&cache_value) {
            self.headers.insert(CACHE_CONTROL.clone(), header_value);
        }
        self
    }

    // Static convenience methods return Response
    #[inline]
    pub fn ok_json_static() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["health_ok"].clone()),
            cache_control: None,
        }
    }

    #[inline]
    pub fn not_found_static() -> Response {
        Response {
            status: StatusCode::NOT_FOUND,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["not_found"].clone()),
            cache_control: None,
        }
    }

    #[inline]
    pub fn server_error_static() -> Response {
        Response {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["server_error"].clone()),
            cache_control: None,
        }
    }

    #[inline]
    pub fn health_check() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["health_ok"].clone()),
            cache_control: None,
        }
    }

    #[inline]
    pub fn pong() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["pong"].clone()),
            cache_control: None,
        }
    }

    #[inline]
    pub fn empty_json() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["empty_json"].clone()),
            cache_control: None,
        }
    }

    // Build the final response with zero-copy optimizations
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
}

// Convenience static constructors for ultra-fast responses
impl ResponseBuilder {
    // Ultra-fast JSON response for APIs
    pub fn api_success() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["success"].clone()),
            cache_control: None,
        }
    }

    // Ultra-fast health check response
    pub fn health() -> Response {
        Response {
            status: StatusCode::OK,
            headers: {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert(CONTENT_TYPE.clone(), COMMON_HEADERS["json"].clone());
                headers
            },
            body: Arc::new(COMMON_RESPONSES["health_ok"].clone()),
            cache_control: None,
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
    // Create response with pre-compiled static content
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
            // Fixed: Use ResponseBuilder's empty_json method
            ResponseBuilder::empty_json()
        }
    }

    // Zero-copy JSON response from static string
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

    // Zero-copy text response from static string
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

    // Zero-copy HTML response from static string
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

    // Share body between responses (zero-copy clone)
    pub fn clone_body(&self) -> Arc<Bytes> {
        Arc::clone(&self.body)
    }

    // Fixed: Add the missing empty_json method
    pub fn empty_json() -> Self {
        ResponseBuilder::empty_json()
    }

    // Common health check response
    pub fn health_check() -> Self {
        ResponseBuilder::health()
    }
}
