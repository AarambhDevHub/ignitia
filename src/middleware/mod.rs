//! Middleware system for the Ignitia web framework.
//!
//! This module provides the core middleware infrastructure and built-in middleware implementations
//! for common HTTP server functionality. Middleware can intercept and modify requests and responses,
//! enabling cross-cutting concerns like logging, rate limiting, compression, and security.
//!
//! # Overview
//!
//! Middleware in Ignitia follows a chain-of-responsibility pattern where each middleware can:
//! - Inspect and modify incoming requests
//! - Pass control to the next middleware or handler in the chain
//! - Inspect and modify outgoing responses
//! - Short-circuit the chain by returning early
//!
//! # Core Components
//!
//! - [`Middleware`] - Trait for implementing custom middleware
//! - [`Next`] - Represents the next step in the middleware chain
//! - [`BoxFuture`] - Type alias for boxed async futures used internally
//!
//! # Built-in Middleware
//!
//! The framework provides several production-ready middleware implementations:
//!
//! - [`LoggerMiddleware`] - Request/response logging
//! - [`CorsMiddleware`] - Cross-Origin Resource Sharing (CORS)
//! - [`RateLimitingMiddleware`] - Rate limiting and throttling
//! - [`SecurityMiddleware`] - Security headers and protections
//! - [`CompressionMiddleware`] - Response compression (gzip, brotli)
//! - [`BodySizeLimitMiddleware`] - Request body size limits
//! - [`RequestIdMiddleware`] - Request ID generation and tracking
//!
//! # Examples
//!
//! ## Using Built-in Middleware
//!
//! ```
//! use ignitia::prelude::*;
//!
//! let router = Router::new()
//!     .middleware(LoggerMiddleware::new())
//!     .middleware(CorsMiddleware::permissive())
//!     .middleware(RateLimitingMiddleware::per_minute(100))
//!     .get("/", || async { "Hello, World!" });
//! ```
//!
//! ## Creating Custom Middleware
//!
//! ```
//! use ignitia::prelude::*;
//! use ignitia::middleware::{Middleware, Next};
//!
//! #[derive(Clone)]
//! struct AuthMiddleware {
//!     api_key: String,
//! }
//!
//! #[async_trait::async_trait]
//! impl Middleware for AuthMiddleware {
//!     async fn handle(&self, req: Request, next: Next) -> Response {
//!         // Check for API key in headers
//!         if let Some(key) = req.header("x-api-key") {
//!             if key == self.api_key {
//!                 return next.run(req).await;
//!             }
//!         }
//!
//!         // Return unauthorized if no valid key
//!         Response::new(StatusCode::UNAUTHORIZED)
//!             .with_body("Invalid API key")
//!     }
//! }
//!
//! let router = Router::new()
//!     .middleware(AuthMiddleware {
//!         api_key: "secret123".to_string(),
//!     })
//!     .get("/protected", || async { "Protected resource" });
//! ```
//!
//! ## Conditional Middleware
//!
//! ```
//! use ignitia::prelude::*;
//! use ignitia::middleware::{Middleware, Next};
//!
//! #[derive(Clone)]
//! struct ConditionalLogger {
//!     verbose: bool,
//! }
//!
//! #[async_trait::async_trait]
//! impl Middleware for ConditionalLogger {
//!     async fn handle(&self, req: Request, next: Next) -> Response {
//!         if self.verbose {
//!             println!("Request: {} {}", req.method, req.uri.path());
//!         }
//!
//!         let response = next.run(req).await;
//!
//!         if self.verbose {
//!             println!("Response: {}", response.status);
//!         }
//!
//!         response
//!     }
//! }
//! ```
//!
//! ## Middleware with State
//!
//! ```
//! use ignitia::prelude::*;
//! use ignitia::middleware::{Middleware, Next};
//! use std::sync::Arc;
//! use parking_lot::Mutex;
//!
//! #[derive(Clone)]
//! struct RequestCounterMiddleware {
//!     counter: Arc<Mutex<u64>>,
//! }
//!
//! impl RequestCounterMiddleware {
//!     fn new() -> Self {
//!         Self {
//!             counter: Arc::new(Mutex::new(0)),
//!         }
//!     }
//!
//!     fn count(&self) -> u64 {
//!         *self.counter.lock()
//!     }
//! }
//!
//! #[async_trait::async_trait]
//! impl Middleware for RequestCounterMiddleware {
//!     async fn handle(&self, req: Request, next: Next) -> Response {
//!         let count = {
//!             let mut counter = self.counter.lock();
//!             *counter += 1;
//!             *counter
//!         };
//!
//!         println!("Request #{}", count);
//!         next.run(req).await
//!     }
//! }
//! ```

pub mod body_limit;
pub mod compression;
pub mod cors;
pub mod logger;
pub mod rate_limit;
pub mod request_id;
pub mod security;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::{Request, Response};

pub use self::body_limit::{BodySizeLimitBuilder, BodySizeLimitMiddleware};
pub use self::compression::CompressionMiddleware;
pub use self::cors::Cors as CorsMiddleware;
pub use self::logger::LoggerMiddleware;
pub use self::rate_limit::{
    RateLimitConfig, RateLimitInfo, RateLimitStats, RateLimitingMiddleware,
};
pub use self::request_id::{IdGenerator, RequestIdMiddleware};
pub use self::security::SecurityMiddleware;

// #[async_trait::async_trait]
// pub trait Middleware: Send + Sync {
//     async fn before(&self, _req: &mut Request) -> Result<()> {
//         Ok(())
//     }
//     async fn after(&self, _req: &Request, _res: &mut Response) -> Result<()> {
//         Ok(())
//     }
// }

/// Type alias for boxed async futures.
///
/// Used internally by the middleware system for async execution. This type represents
/// a pinned, boxed future that resolves to type `T` and can be sent across threads.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the future
/// * `T` - The type the future resolves to
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Represents the next step in the middleware chain.
///
/// `Next` is a continuation that encapsulates the remaining middleware and the final
/// handler. When called via `run()`, it executes the next middleware (or handler) in
/// the chain and returns the response.
///
/// # Cloning
///
/// `Next` is cheap to clone as it uses `Arc` internally for shared ownership.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use ignitia::middleware::{Middleware, Next};
/// use ignitia::{Request, Response};
///
/// #[derive(Clone)]
/// struct LoggingMiddleware;
///
/// #[async_trait::async_trait]
/// impl Middleware for LoggingMiddleware {
///     async fn handle(&self, req: Request, next: Next) -> Response {
///         println!("Processing request...");
///
///         // Call next middleware/handler
///         let response = next.run(req).await;
///
///         println!("Request processed");
///         response
///     }
/// }
/// ```
///
/// ## Conditional Next Execution
///
/// ```
/// use ignitia::middleware::{Middleware, Next};
/// use ignitia::{Request, Response, StatusCode};
///
/// #[derive(Clone)]
/// struct AuthMiddleware;
///
/// #[async_trait::async_trait]
/// impl Middleware for AuthMiddleware {
///     async fn handle(&self, req: Request, next: Next) -> Response {
///         if req.header("authorization").is_some() {
///             // Authorized - continue chain
///             next.run(req).await
///         } else {
///             // Not authorized - don't call next
///             Response::new(StatusCode::UNAUTHORIZED)
///         }
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Next {
    /// The function representing the next step in the middleware chain
    inner: Arc<dyn Fn(Request) -> BoxFuture<'static, Response> + Send + Sync>,
}

impl Next {
    /// Create a new `Next` continuation.
    ///
    /// This method is used internally by the framework to build the middleware chain.
    /// It wraps a function that takes a request and returns a future resolving to a response.
    ///
    /// # Arguments
    ///
    /// * `func` - Function representing the next step in the chain
    ///
    /// # Returns
    ///
    /// Returns a new `Next` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::Next;
    /// use ignitia::{Request, Response};
    ///
    /// let next = Next::new(|req: Request| {
    ///     Box::pin(async move {
    ///         Response::text("Hello from next")
    ///     })
    /// });
    /// ```
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Request) -> BoxFuture<'static, Response> + Send + Sync + 'static,
    {
        Self { inner: Arc::new(f) }
    }

    /// Execute the next step in the middleware chain.
    ///
    /// This method invokes the next middleware or handler in the chain with the given request
    /// and awaits its completion, returning the resulting response.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to pass to the next step
    ///
    /// # Returns
    ///
    /// Returns the HTTP [`Response`] from the next step in the chain.
    ///
    /// # Examples
    ///
    /// ## Simple Pass-through
    ///
    /// ```
    /// use ignitia::middleware::{Middleware, Next};
    /// use ignitia::{Request, Response};
    ///
    /// #[derive(Clone)]
    /// struct PassThroughMiddleware;
    ///
    /// #[async_trait::async_trait]
    /// impl Middleware for PassThroughMiddleware {
    ///     async fn handle(&self, req: Request, next: Next) -> Response {
    ///         // Simply pass request to next step
    ///         next.run(req).await
    ///     }
    /// }
    /// ```
    ///
    /// ## Measuring Execution Time
    ///
    /// ```
    /// use ignitia::middleware::{Middleware, Next};
    /// use ignitia::{Request, Response};
    /// use std::time::Instant;
    ///
    /// #[derive(Clone)]
    /// struct TimerMiddleware;
    ///
    /// #[async_trait::async_trait]
    /// impl Middleware for TimerMiddleware {
    ///     async fn handle(&self, req: Request, next: Next) -> Response {
    ///         let start = Instant::now();
    ///
    ///         // Run next middleware/handler
    ///         let response = next.run(req).await;
    ///
    ///         let duration = start.elapsed();
    ///         println!("Request took: {:?}", duration);
    ///
    ///         response
    ///     }
    /// }
    /// ```
    #[inline]
    pub async fn run(self, req: Request) -> Response {
        (self.inner)(req).await
    }
}

/// Core middleware trait for intercepting and modifying HTTP requests and responses.
///
/// Middleware implementations must be `Clone + Send + Sync` to support concurrent request
/// processing. The `handle` method receives a request and a [`Next`] continuation that
/// represents the rest of the middleware chain.
///
/// # Examples
///
/// ## Basic Middleware
///
/// ```
/// use ignitia::prelude::*;
/// use ignitia::middleware::{Middleware, Next};
///
/// #[derive(Clone)]
/// struct TimingMiddleware;
///
/// #[async_trait::async_trait]
/// impl Middleware for TimingMiddleware {
///     async fn handle(&self, req: Request, next: Next) -> Response {
///         let start = std::time::Instant::now();
///         let response = next.run(req).await;
///         let duration = start.elapsed();
///
///         println!("Request took {:?}", duration);
///         response
///     }
/// }
/// ```
///
/// ## Modifying Requests
///
/// ```
/// use ignitia::prelude::*;
/// use ignitia::middleware::{Middleware, Next};
///
/// #[derive(Clone)]
/// struct HeaderInjector;
///
/// #[async_trait::async_trait]
/// impl Middleware for HeaderInjector {
///     async fn handle(&self, mut req: Request, next: Next) -> Response {
///         // Add custom header to request
///         req.headers.insert(
///             http::header::HeaderName::from_static("x-custom-header"),
///             http::HeaderValue::from_static("custom-value"),
///         );
///
///         next.run(req).await
///     }
/// }
/// ```
///
/// ## Modifying Responses
///
/// ```
/// use ignitia::prelude::*;
/// use ignitia::middleware::{Middleware, Next};
///
/// #[derive(Clone)]
/// struct CacheHeaderMiddleware;
///
/// #[async_trait::async_trait]
/// impl Middleware for CacheHeaderMiddleware {
///     async fn handle(&self, req: Request, next: Next) -> Response {
///         let mut response = next.run(req).await;
///
///         // Add cache control header
///         response.headers.insert(
///             http::header::CACHE_CONTROL,
///             http::HeaderValue::from_static("public, max-age=3600"),
///         );
///
///         response
///     }
/// }
/// ```
///
/// ## Short-circuiting
///
/// ```
/// use ignitia::prelude::*;
/// use ignitia::middleware::{Middleware, Next};
///
/// #[derive(Clone)]
/// struct MaintenanceMode {
///     enabled: bool,
/// }
///
/// #[async_trait::async_trait]
/// impl Middleware for MaintenanceMode {
///     async fn handle(&self, req: Request, next: Next) -> Response {
///         if self.enabled {
///             // Short-circuit and return maintenance response
///             return Response::new(StatusCode::SERVICE_UNAVAILABLE)
///                 .with_body("Site is under maintenance");
///         }
///
///         next.run(req).await
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// Process a request and optionally pass it to the next middleware.
    ///
    /// This method receives the current request and a [`Next`] continuation. The middleware
    /// can:
    /// - Inspect and modify the request before calling `next.run(req)`
    /// - Call `next.run(req)` to continue the middleware chain
    /// - Return early without calling `next.run(req)` to short-circuit
    /// - Inspect and modify the response after calling `next.run(req)`
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request being processed
    /// * `next` - Continuation representing the rest of the middleware chain
    ///
    /// # Returns
    ///
    /// Returns the HTTP [`Response`] to send to the client.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::{Middleware, Next};
    /// use ignitia::{Request, Response};
    ///
    /// #[derive(Clone)]
    /// struct MyMiddleware;
    ///
    /// #[async_trait::async_trait]
    /// impl Middleware for MyMiddleware {
    ///     async fn handle(&self, req: Request, next: Next) -> Response {
    ///         // Before request processing
    ///         println!("Before: {} {}", req.method, req.uri.path());
    ///
    ///         // Process request through chain
    ///         let response = next.run(req).await;
    ///
    ///         // After request processing
    ///         println!("After: {}", response.status);
    ///
    ///         response
    ///     }
    /// }
    /// ```
    async fn handle(&self, req: Request, next: Next) -> Response;
}

/// Helper trait for function-based middleware
/// This allows any async function with signature `async fn(Request, Next) -> Response` to be middleware
#[async_trait::async_trait]
impl<F> Middleware for F
where
    F: Fn(Request, Next) -> BoxFuture<'static, Response> + Send + Sync,
{
    async fn handle(&self, req: Request, next: Next) -> Response {
        self(req, next).await
    }
}

/// Convenience function to create middleware from a closure.
///
/// This function allows creating simple middleware without implementing the [`Middleware`] trait.
/// It's useful for one-off middleware or prototyping.
///
/// # Type Parameters
///
/// * `F` - The closure type
/// * `Fut` - The future type returned by the closure
///
/// # Arguments
///
/// * `f` - Async closure that takes `(Request, Next)` and returns `Response`
///
/// # Returns
///
/// Returns an implementation of [`Middleware`].
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
/// use ignitia::middleware::from_fn;
///
/// let logger = from_fn(|req, next| async move {
///     println!("Request: {} {}", req.method, req.uri.path());
///     next.run(req).await
/// });
///
/// let router = Router::new()
///     .middleware(logger)
///     .get("/", || async { "Hello" });
/// ```
pub fn from_fn<F, Fut, T>(f: F) -> MiddlewareFn<F>
where
    F: Fn(Request, Next) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: crate::response::IntoResponse,
{
    MiddlewareFn { f }
}

/// Wrapper for function-based middleware that converts Result<Response, E> to Response
#[derive(Clone)]
pub struct MiddlewareFn<F> {
    f: F,
}

#[async_trait::async_trait]
impl<F, Fut, T> Middleware for MiddlewareFn<F>
where
    F: Fn(Request, Next) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: crate::response::IntoResponse,
{
    async fn handle(&self, req: Request, next: Next) -> Response {
        let result = (self.f)(req, next).await;
        result.into_response()
    }
}
