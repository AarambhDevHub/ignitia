//! # HTTP Request and Response Logger Middleware
//!
//! This module provides comprehensive HTTP logging middleware for the Ignitia web framework.
//! It logs incoming requests and outgoing responses with detailed information about
//! HTTP methods, paths, versions, and status codes using the `tracing` crate.
//!
//! ## Features
//!
//! - **Request Logging**: Logs incoming HTTP requests with method, path, and HTTP version
//! - **Response Logging**: Logs outgoing responses with status codes
//! - **Structured Logging**: Uses the `tracing` crate for structured, level-based logging
//! - **Zero Configuration**: Works out of the box with sensible defaults
//! - **Performance Optimized**: Minimal overhead logging implementation
//!
//! ## Usage
//!
//! ### Basic Usage
//! ```
//! use ignitia::{Router, LoggerMiddleware};
//!
//! let router = Router::new()
//!     .middleware(LoggerMiddleware)
//!     .get("/", || async { Ok(ignitia::Response::text("Hello World!")) });
//! ```
//!
//! ### With Custom Logging Configuration
//! ```
//! use ignitia::{Router, LoggerMiddleware};
//! use tracing_subscriber;
//!
//! // Initialize tracing subscriber with custom format
//! tracing_subscriber::fmt()
//!     .with_target(false)
//!     .with_thread_ids(true)
//!     .with_level(true)
//!     .init();
//!
//! let router = Router::new()
//!     .middleware(LoggerMiddleware)
//!     .get("/api/users", || async { Ok(ignitia::Response::text("Users")) });
//! ```
//!
//! ## Log Format
//!
//! The middleware produces logs in the following format:
//!
//! ### Request Logs
//! ```
//! GET /api/users HTTP/1.1
//! POST /api/users HTTP/2.0
//! DELETE /api/users/123 HTTP/1.1
//! ```
//!
//! ### Response Logs
//! ```
//! Response: 200
//! Response: 404
//! Response: 500
//! ```
//!
//! ## HTTP Version Support
//!
//! The middleware correctly identifies and logs all HTTP versions:
//! - HTTP/0.9 (rarely used)
//! - HTTP/1.0
//! - HTTP/1.1 (most common)
//! - HTTP/2.0 (modern browsers and servers)
//! - HTTP/3.0 (latest standard)
//!
//! ## Integration with Tracing
//!
//! This middleware uses the `tracing` crate's `info!` macro, which means:
//! - Logs are structured and can be filtered by level
//! - Integration with distributed tracing systems is possible
//! - Custom formatting and output targets are supported
//!
//! ### Configuring Log Levels
//! ```
//! use tracing_subscriber;
//!
//! // Only show warnings and errors (hide info logs)
//! std::env::set_var("RUST_LOG", "warn");
//! tracing_subscriber::fmt::init();
//!
//! // Show all logs including debug
//! std::env::set_var("RUST_LOG", "debug");
//! tracing_subscriber::fmt::init();
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Minimal Overhead**: Logging operations are very fast
//! - **Non-Blocking**: Uses async logging that doesn't block request processing
//! - **Memory Efficient**: No significant memory allocation per request
//! - **CPU Efficient**: Simple string formatting with minimal processing
//!
//! ## Custom Logging Middleware
//!
//! If you need more advanced logging, you can create custom middleware:
//!
//! ```
//! use ignitia::{Middleware, Request, Response, Result};
//! use async_trait::async_trait;
//! use tracing::{info, warn};
//! use std::time::Instant;
//!
//! pub struct CustomLoggerMiddleware;
//!
//! #[async_trait]
//! impl Middleware for CustomLoggerMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         let start_time = Instant::now();
//!         req.insert_extension(start_time);
//!
//!         info!(
//!             method = %req.method,
//!             path = req.uri.path(),
//!             query = req.uri.query(),
//!             user_agent = req.header("user-agent").unwrap_or("unknown"),
//!             "Request started"
//!         );
//!         Ok(())
//!     }
//!
//!     async fn after(&self, res: &mut Response) -> Result<()> {
//!         let status = res.status.as_u16();
//!
//!         if status >= 400 {
//!             warn!(status = status, "Request completed with error");
//!         } else {
//!             info!(status = status, "Request completed successfully");
//!         }
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Production Considerations
//!
//! ### Log Rotation
//! For production use, consider implementing log rotation:
//! ```
//! use tracing_appender::rolling::{RollingFileAppender, Rotation};
//! use tracing_subscriber::fmt::writer::MakeWriterExt;
//!
//! let file_appender = RollingFileAppender::new(Rotation::DAILY, "/var/log/myapp", "app.log");
//! let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);
//!
//! tracing_subscriber::fmt()
//!     .with_writer(non_blocking_writer)
//!     .init();
//! ```
//!
//! ### Security Considerations
//! - Be careful not to log sensitive data like authentication tokens
//! - Consider filtering or masking sensitive query parameters
//! - Ensure log files have appropriate permissions in production

use crate::middleware::Middleware;
use crate::{Request, Response, Result};
use tracing::info;

/// A simple HTTP request and response logging middleware.
///
/// This middleware logs incoming HTTP requests with their method, path, and HTTP version,
/// and outgoing responses with their status codes. It uses the `tracing` crate for
/// structured logging.
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::{Router, LoggerMiddleware};
///
/// let router = Router::new()
///     .middleware(LoggerMiddleware)
///     .get("/", || async { Ok(ignitia::Response::text("Hello!")) });
/// ```
///
/// ## With Multiple Routes
/// ```
/// use ignitia::{Router, LoggerMiddleware, Response, Result};
///
/// let router = Router::new()
///     .middleware(LoggerMiddleware)
///     .get("/users", || async { Ok(Response::text("Users list")) })
///     .post("/users", || async { Ok(Response::text("User created")) })
///     .get("/health", || async { Ok(Response::text("OK")) });
/// ```
///
/// ## Expected Log Output
/// ```
/// INFO GET /users HTTP/1.1
/// INFO Response: 200
/// INFO POST /users HTTP/1.1
/// INFO Response: 201
/// INFO GET /health HTTP/1.1
/// INFO Response: 200
/// ```
pub struct LoggerMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggerMiddleware {
    /// Logs the incoming HTTP request.
    ///
    /// This method is called before the request reaches the handler. It logs:
    /// - HTTP method (GET, POST, PUT, DELETE, etc.)
    /// - Request path from the URI
    /// - HTTP version (1.0, 1.1, 2.0, 3.0, etc.)
    ///
    /// The log entry is created using the `info!` macro from the `tracing` crate,
    /// which means it will be visible when the log level is set to INFO or lower.
    ///
    /// # Parameters
    /// - `req`: Mutable reference to the incoming request
    ///
    /// # Returns
    /// - `Ok(())`: Always succeeds, allowing the request to continue
    ///
    /// # Examples
    /// Given a request `GET /api/users HTTP/1.1`, this will log:
    /// ```
    /// GET /api/users HTTP/1.1
    /// ```
    async fn before(&self, req: &mut Request) -> Result<()> {
        let version_str = match req.version {
            http::Version::HTTP_09 => "HTTP/0.9",
            http::Version::HTTP_10 => "HTTP/1.0",
            http::Version::HTTP_11 => "HTTP/1.1",
            http::Version::HTTP_2 => "HTTP/2.0",
            http::Version::HTTP_3 => "HTTP/3.0",
            _ => "UNKNOWN",
        };

        info!("{} {} {}", req.method, req.uri.path(), version_str);
        Ok(())
    }

    /// Logs the outgoing HTTP response.
    ///
    /// This method is called after the handler has processed the request and generated
    /// a response. It logs the HTTP status code of the response.
    ///
    /// # Parameters
    /// - `res`: Mutable reference to the outgoing response
    ///
    /// # Returns
    /// - `Ok(())`: Always succeeds, allowing the response to continue
    ///
    /// # Examples
    /// Given a response with status 200, this will log:
    /// ```
    /// Response: 200
    /// ```
    ///
    /// For error responses:
    /// ```
    /// Response: 404
    /// Response: 500
    /// ```
    async fn after(&self, res: &mut Response) -> Result<()> {
        info!("Response: {}", res.status.as_u16());
        Ok(())
    }
}
