use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

use crate::{Request, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Route not found: {0}")]
    NotFound(String),

    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // Custom application errors
    #[error("Custom error: {0}")]
    Custom(Box<dyn CustomError>),
}

// Trait for custom error types that can be converted to HTTP responses
pub trait CustomError: fmt::Debug + fmt::Display + Send + Sync + 'static {
    fn status_code(&self) -> StatusCode;
    fn error_type(&self) -> &'static str;
    fn error_code(&self) -> Option<String> {
        None
    }
    fn metadata(&self) -> Option<serde_json::Value> {
        None
    }
}

// Standard error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl Error {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            Error::BadRequest(_) | Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::Database(_) | Error::ExternalService(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Custom(custom) => custom.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[inline]
    pub fn error_type(&self) -> &'static str {
        match self {
            Error::NotFound(_) => "not_found",
            Error::MethodNotAllowed(_) => "method_not_allowed",
            Error::BadRequest(_) => "bad_request",
            Error::Unauthorized => "unauthorized",
            Error::Forbidden => "forbidden",
            Error::Validation(_) => "validation_error",
            Error::Database(_) => "database_error",
            Error::ExternalService(_) => "external_service_error",
            Error::Custom(custom) => custom.error_type(),
            _ => "internal_server_error",
        }
    }

    // Fast path constructors for common errors
    #[inline]
    pub fn not_found(path: &str) -> Self {
        Error::NotFound(path.to_string())
    }

    #[inline]
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Error::BadRequest(msg.into())
    }

    #[inline]
    pub fn validation(msg: impl Into<String>) -> Self {
        Error::Validation(msg.into())
    }

    #[inline]
    pub fn unauthorized() -> Self {
        Error::Unauthorized
    }

    #[inline]
    pub fn forbidden() -> Self {
        Error::Forbidden
    }

    #[inline]
    pub fn internal(msg: impl Into<String>) -> Self {
        Error::Internal(msg.into())
    }

    // Convert to error response with optional custom formatting
    pub fn to_response(&self, include_timestamp: bool) -> ErrorResponse {
        let status = self.status_code();
        let status_reason = status.canonical_reason().unwrap_or("Unknown Error");

        ErrorResponse {
            error: status_reason.to_string(),
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

pub type Result<T> = std::result::Result<T, Error>;

// Macro for creating custom errors easily
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

// Helper trait for easy error conversion
pub trait ErrorExt<T> {
    fn bad_request(self) -> Result<T>;
    fn unauthorized(self) -> Result<T>;
    fn forbidden(self) -> Result<T>;
    fn internal_error(self) -> Result<T>;
    fn validation_error(self) -> Result<T>;
}

impl<T, E> ErrorExt<T> for std::result::Result<T, E>
where
    E: fmt::Display,
{
    fn bad_request(self) -> Result<T> {
        self.map_err(|e| Error::bad_request(e.to_string()))
    }

    fn unauthorized(self) -> Result<T> {
        self.map_err(|_| Error::unauthorized())
    }

    fn forbidden(self) -> Result<T> {
        self.map_err(|_| Error::forbidden())
    }

    fn internal_error(self) -> Result<T> {
        self.map_err(|e| Error::internal(e.to_string()))
    }

    fn validation_error(self) -> Result<T> {
        self.map_err(|e| Error::validation(e.to_string()))
    }
}

// Trait for custom error handlers
pub trait ErrorHandler: Send + Sync + 'static {
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

// Implementation for functions that take both error and request
pub trait ErrorHandlerWithRequest: Send + Sync + 'static {
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

// Wrapper to unify both error handler types
#[derive(Clone)]
pub enum ErrorHandlerType {
    Simple(Arc<dyn ErrorHandler>),
    WithRequest(Arc<dyn ErrorHandlerWithRequest>),
}

impl ErrorHandlerType {
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
