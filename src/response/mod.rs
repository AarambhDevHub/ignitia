pub mod builder;
pub mod status;

use crate::error::Result;
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use serde::Serialize;

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl Response {
    #[inline]
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Bytes::new(),
        }
    }

    #[inline]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    // Add a convenience method for setting status by number
    #[inline]
    pub fn with_status_code(mut self, status_code: u16) -> Self {
        if let Ok(status) = StatusCode::from_u16(status_code) {
            self.status = status;
        }
        self
    }

    #[inline]
    pub fn with_body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = body.into();
        self
    }

    #[inline]
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    #[inline]
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    #[inline]
    pub fn internal_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn json<T: Serialize>(data: T) -> Result<Self> {
        let body = serde_json::to_vec(&data)?;
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        response.body = Bytes::from(body);
        Ok(response)
    }

    #[inline]
    pub fn text(text: impl Into<String>) -> Self {
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.body = Bytes::from(text.into());
        response
    }

    #[inline]
    pub fn html(html: impl Into<String>) -> Self {
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        response.body = Bytes::from(html.into());
        response
    }
}

pub use builder::ResponseBuilder;

use crate::error::{Error, ErrorResponse};

impl From<Error> for Response {
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
                response.body = bytes::Bytes::from(json_body);
                response
            }
            Err(_) => {
                let mut response = Response::new(status);
                response.headers.insert(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("text/plain; charset=utf-8"),
                );
                response.body = bytes::Bytes::from(err.to_string());
                response
            }
        }
    }
}

// Enhanced response methods
impl Response {
    // Error response builders
    pub fn error_json<E: Into<Error>>(error: E) -> crate::Result<Self> {
        let err = error.into();
        let status = err.status_code();
        let error_response = err.to_response(true);

        let mut response = Self::new(status);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        response.body = bytes::Bytes::from(serde_json::to_vec(&error_response)?);
        Ok(response)
    }

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
        response.body = bytes::Bytes::from(serde_json::to_vec(&error_response)?);
        Ok(response)
    }
}
