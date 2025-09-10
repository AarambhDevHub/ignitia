//! # Middleware System Module
//!
//! This module provides a comprehensive middleware system for the Ignitia web framework.
//! Middleware allows you to process requests and responses at various stages of the request
//! lifecycle, enabling cross-cutting concerns like authentication, logging, CORS handling,
//! and error processing.
//!
//! ## Features
//!
//! - **Request Processing**: Intercept and modify requests before they reach handlers
//! - **Response Processing**: Modify responses after handlers complete
//! - **Composable**: Chain multiple middleware together for complex processing pipelines
//! - **Async Support**: Full support for asynchronous middleware operations
//! - **Type Safety**: Compile-time guarantees for middleware behavior
//! - **Performance**: Minimal overhead middleware execution
//!
//! ## How Middleware Works
//!
//! Middleware in Ignitia follows a two-phase execution model:
//!
//! 1. **Before Phase**: Executed before the request reaches the handler
//! 2. **After Phase**: Executed after the handler produces a response
//!
//! ```
//! Request -> Middleware::before -> Handler -> Middleware::after -> Response
//! ```
//!
//! Multiple middleware are executed in order for the before phase, and in reverse
//! order for the after phase:
//!
//! ```
//! Request -> MW1::before -> MW2::before -> Handler -> MW2::after -> MW1::after -> Response
//! ```
//!
//! ## Available Middleware
//!
//! - **LoggerMiddleware**: HTTP request and response logging
//! - **CorsMiddleware**: Cross-Origin Resource Sharing handling
//! - **BodySizeLimitMiddleware**: Limit the size of incoming request bodies
//! - **CompressionMiddleware**: Compress response bodies using gzip or brotli
//! - **AuthMiddleware**: Token-based authentication for protected routes
//! - **ErrorHandlerMiddleware**: Advanced error handling and logging
//!
//! ## Quick Start
//!
//! ### Basic Usage
//! ```
//! use ignitia::{Router, LoggerMiddleware, CorsMiddleware};
//!
//! let router = Router::new()
//!     .middleware(LoggerMiddleware)
//!     .middleware(CorsMiddleware::new())
//!     .get("/", || async { Ok(ignitia::Response::text("Hello World!")) });
//! ```
//!
//! ### Multiple Middleware
//! ```
//! use ignitia::{Router, LoggerMiddleware, CorsMiddleware, AuthMiddleware};
//!
//! let router = Router::new()
//!     .middleware(LoggerMiddleware)
//!     .middleware(CorsMiddleware::new().allow_origin("https://myapp.com"))
//!     .middleware(AuthMiddleware::new("secret-token").protect_path("/api"))
//!     .get("/", || async { Ok(ignitia::Response::text("Public endpoint")) })
//!     .get("/api/data", || async { Ok(ignitia::Response::text("Protected data")) });
//! ```
//!
//! ## Custom Middleware
//!
//! ### Simple Custom Middleware
//! ```
//! use ignitia::{Middleware, Request, Response, Result};
//! use async_trait::async_trait;
//!
//! struct CustomHeaderMiddleware;
//!
//! #[async_trait]
//! impl Middleware for CustomHeaderMiddleware {
//!     async fn after(&self, _req: &Request, res: &mut Response) -> Result<()> {
//!         res.headers.insert(
//!             "X-Custom-Header",
//!             "Added by middleware".parse().unwrap()
//!         );
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ### Request Processing Middleware
//! ```
//! use ignitia::{Middleware, Request, Response, Result, Error};
//! use async_trait::async_trait;
//!
//! struct RateLimitMiddleware {
//!     max_requests: u32,
//! }
//!
//! #[async_trait]
//! impl Middleware for RateLimitMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         // Check rate limit logic here
//!         let client_ip = req.header("x-forwarded-for")
//!             .or_else(|| req.header("x-real-ip"))
//!             .unwrap_or("unknown");
//!
//!         // Rate limiting logic would go here
//!         if should_rate_limit(client_ip) {
//!             return Err(Error::BadRequest("Rate limit exceeded".into()));
//!         }
//!
//!         Ok(())
//!     }
//! }
//!
//! # fn should_rate_limit(_ip: &str) -> bool { false }
//! ```
//!
//! ## Error Handling in Middleware
//!
//! Middleware can return errors to short-circuit request processing:
//!
//! ```
//! use ignitia::{Middleware, Request, Result, Error};
//! use async_trait::async_trait;
//!
//! struct ValidationMiddleware;
//!
//! #[async_trait]
//! impl Middleware for ValidationMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         if req.header("content-type").is_none() {
//!             return Err(Error::BadRequest("Content-Type header required".into()));
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - Middleware is executed for every request - keep operations lightweight
//! - Expensive operations should be cached or optimized
//! - Consider async operations that don't block the event loop
//! - Use early returns to avoid unnecessary processing

pub mod auth;
pub mod body_limit;
pub mod compression;
pub mod cors;
pub mod error_handler;
pub mod logger;
pub mod request_id;

use crate::{Request, Response, Result};

/// The core trait for implementing middleware in the Ignitia web framework.
///
/// Middleware provides hooks to process requests before they reach handlers and
/// responses after handlers complete. This enables cross-cutting concerns like
/// authentication, logging, CORS handling, and error processing.
///
/// ## Lifecycle
///
/// Middleware has two phases:
/// 1. **Before**: Called before the request reaches the handler
/// 2. **After**: Called after the handler produces a response
///
/// Both phases are optional - implement only the phases you need.
///
/// ## Execution Order
///
/// - Before phases execute in the order middleware was added
/// - After phases execute in reverse order (last added, first executed)
///
/// ## Error Handling
///
/// If any middleware returns an error during the before phase:
/// - Request processing stops immediately
/// - The error is converted to an HTTP response
/// - After phases of already-executed middleware still run
///
/// ## Examples
///
/// ### Request Processing Only
/// ```
/// use ignitia::{Middleware, Request, Result, Error};
/// use async_trait::async_trait;
///
/// struct RequestValidationMiddleware;
///
/// #[async_trait]
/// impl Middleware for RequestValidationMiddleware {
///     async fn before(&self, req: &mut Request) -> Result<()> {
///         if req.method == http::Method::POST && req.body.is_empty() {
///             return Err(Error::BadRequest("POST requests must have a body".into()));
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ### Response Processing Only
/// ```
/// use ignitia::{Middleware, Request, Response, Result};
/// use async_trait::async_trait;
///
/// struct SecurityHeadersMiddleware;
///
/// #[async_trait]
/// impl Middleware for SecurityHeadersMiddleware {
///     async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
///         res.headers.insert("X-Frame-Options", "DENY".parse().unwrap());
///         res.headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
///         Ok(())
///     }
/// }
/// ```
///
/// ### Both Phases
/// ```
/// use ignitia::{Middleware, Request, Response, Result};
/// use async_trait::async_trait;
/// use std::time::Instant;
///
/// struct TimingMiddleware;
///
/// #[async_trait]
/// impl Middleware for TimingMiddleware {
///     async fn before(&self, req: &mut Request) -> Result<()> {
///         req.insert_extension(Instant::now());
///         Ok(())
///     }
///
///     async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
///         // Note: We can't access the request in after phase
///         // This is a limitation of the current design
///         res.headers.insert("X-Processing-Time", "calculated".parse().unwrap());
///         Ok(())
///     }
/// }
/// ```
///
/// ### Conditional Processing
/// ```
/// use ignitia::{Middleware, Request, Response, Result};
/// use async_trait::async_trait;
///
/// struct ConditionalMiddleware {
///     enabled: bool,
/// }
///
/// #[async_trait]
/// impl Middleware for ConditionalMiddleware {
///     async fn before(&self, req: &mut Request) -> Result<()> {
///         if !self.enabled {
///             return Ok(());
///         }
///
///         // Process request only if enabled
///         println!("Processing request to: {}", req.uri.path());
///         Ok(())
///     }
///
///     async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
///         if !self.enabled {
///             return Ok(());
///         }
///
///         // Process response only if enabled
///         println!("Response status: {}", res.status);
///         Ok(())
///     }
/// }
/// ```
///
/// ### Async Operations
/// ```
/// use ignitia::{Middleware, Request, Result};
/// use async_trait::async_trait;
///
/// struct AsyncMiddleware;
///
/// #[async_trait]
/// impl Middleware for AsyncMiddleware {
///     async fn before(&self, req: &mut Request) -> Result<()> {
///         // Simulate async operation (database lookup, external API call, etc.)
///         tokio::time::sleep(std::time::Duration::from_millis(10)).await;
///
///         // Add result to request extensions
///         req.insert_extension(String::from("async_result"));
///         Ok(())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// Called before the request is processed by the handler.
    ///
    /// This method allows middleware to:
    /// - Validate or modify the incoming request
    /// - Add data to request extensions for use by handlers
    /// - Short-circuit processing by returning an error
    /// - Perform authentication or authorization checks
    ///
    /// # Parameters
    /// - `req`: Mutable reference to the request being processed
    ///
    /// # Returns
    /// - `Ok(())`: Continue processing with the next middleware or handler
    /// - `Err(Error)`: Stop processing and return the error as a response
    ///
    /// # Default Implementation
    /// The default implementation does nothing and allows processing to continue.
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Middleware, Request, Result, Error};
    /// use async_trait::async_trait;
    ///
    /// struct AuthCheckMiddleware;
    ///
    /// #[async_trait]
    /// impl Middleware for AuthCheckMiddleware {
    ///     async fn before(&self, req: &mut Request) -> Result<()> {
    ///         if req.uri.path().starts_with("/admin") {
    ///             let auth_header = req.header("authorization")
    ///                 .ok_or_else(|| Error::Unauthorized)?;
    ///
    ///             if !auth_header.starts_with("Bearer ") {
    ///                 return Err(Error::Unauthorized);
    ///             }
    ///         }
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn before(&self, _req: &mut Request) -> Result<()> {
        Ok(())
    }

    /// Called after the handler has processed the request and generated a response.
    ///
    /// This method allows middleware to:
    /// - Modify the response before it's sent to the client
    /// - Add headers (security, CORS, caching, etc.)
    /// - Log response information
    /// - Transform response data
    ///
    /// # Parameters
    /// - `req`: Immutable reference to the request
    /// - `res`: Mutable reference to the response from the handler
    ///
    /// # Returns
    /// - `Ok(())`: Continue processing with the next middleware
    /// - `Err(Error)`: Replace the current response with the error response
    ///
    /// # Default Implementation
    /// The default implementation does nothing and allows the response to pass through.
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Middleware, Response, Result, Request};
    /// use async_trait::async_trait;
    ///
    /// struct CompressionMiddleware;
    ///
    /// #[async_trait]
    /// impl Middleware for CompressionMiddleware {
    ///     async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
    ///         // Add compression headers if body is large enough
    ///         if res.body.len() > 1024 {
    ///             res.headers.insert(
    ///                 "content-encoding",
    ///                 "gzip".parse().unwrap()
    ///             );
    ///         }
    ///         Ok(())
    ///     }
    /// }
    /// ```
    ///
    /// # Note
    /// Currently, the after phase doesn't have access to the original request.
    /// If you need request data in the after phase, store it in response
    /// extensions or headers during the before phase.
    async fn after(&self, _req: &Request, _res: &mut Response) -> Result<()> {
        Ok(())
    }
}

pub use self::auth::AuthMiddleware;
pub use self::body_limit::{BodySizeLimitBuilder, BodySizeLimitMiddleware};
pub use self::compression::CompressionMiddleware;
pub use self::cors::Cors as CorsMiddleware;
pub use self::error_handler::ErrorHandlerMiddleware;
pub use self::logger::LoggerMiddleware;
pub use self::request_id::{IdGenerator, RequestIdMiddleware};
