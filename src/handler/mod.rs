//! # Request Handler System
//!
//! This module provides the core request handling system for the Ignitia web framework.
//! It defines traits and types for processing HTTP requests and generating responses,
//! with support for automatic parameter extraction and type-safe handler functions.
//!
//! ## Features
//!
//! - **Type-Safe Handlers**: Compile-time guarantees for handler function signatures
//! - **Automatic Extraction**: Extract typed data from requests automatically
//! - **Flexible Handler Types**: Support for various handler function patterns
//! - **Zero-Cost Abstractions**: Minimal runtime overhead for handler dispatch
//! - **Async/Await Support**: Full support for asynchronous request processing
//! - **Builder Pattern**: Fluent API for handler composition
//!
//! ## Handler Types
//!
//! The framework supports several types of handlers:
//!
//! ### 1. Handler Trait
//! The fundamental trait that all handlers must implement:
//! ```
//! use ignitia::{Handler, Request, Response, Result};
//!
//! struct MyHandler;
//!
//! #[async_trait::async_trait]
//! impl Handler for MyHandler {
//!     async fn handle(&self, req: Request) -> Result<Response> {
//!         Ok(Response::text("Hello from custom handler!"))
//!     }
//! }
//! ```
//!
//! ### 2. Handler Functions
//! Simple functions that can be converted to handlers:
//! ```
//! use ignitia::{handler_fn, Request, Response, Result};
//!
//! let handler = handler_fn(|req: Request| async move {
//!     Ok(Response::text("Hello from function handler!"))
//! });
//! ```
//!
//! ### 3. IntoHandler Functions
//! Functions with automatic parameter extraction:
//! ```
//! use ignitia::{Path, Query, Json, Response, Result};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserParams {
//!     id: u64,
//! }
//!
//! #[derive(Deserialize)]
//! struct QueryParams {
//!     format: Option<String>,
//! }
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//! }
//!
//! // Handler with path and query extraction
//! async fn get_user(
//!     Path(params): Path<UserParams>,
//!     Query(query): Query<QueryParams>,
//! ) -> Result<Response> {
//!     let format = query.format.unwrap_or_else(|| "json".to_string());
//!     Ok(Response::text(format!("User {} in {} format", params.id, format)))
//! }
//!
//! // Handler with JSON body extraction
//! async fn create_user(Json(user): Json<CreateUserRequest>) -> Result<Response> {
//!     // Create user logic here
//!     Ok(Response::text(format!("Created user: {}", user.name)))
//! }
//! ```
//!
//! ## Usage in Router
//!
//! Handlers are typically used with the router:
//! ```
//! use ignitia::{Router, Response, Result};
//!
//! let router = Router::new()
//!     .get("/", || async { Ok(Response::text("Hello World!")) })
//!     .get("/users/:id", get_user)
//!     .post("/users", create_user);
//! ```
//!
//! ## Error Handling
//!
//! Handlers can return errors that are automatically converted to HTTP responses:
//! ```
//! use ignitia::{Error, Response, Result};
//!
//! async fn fallible_handler() -> Result<Response> {
//!     if some_condition() {
//!         return Err(Error::BadRequest("Invalid input".into()));
//!     }
//!     Ok(Response::text("Success"))
//! }
//!
//! # fn some_condition() -> bool { false }
//! ```

pub mod extractor;

use crate::{Request, Response, Result};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A boxed future representing an asynchronous operation that returns a Result<Response>.
///
/// This type alias is used internally to represent handler functions that return
/// futures. The 'a lifetime parameter allows the future to borrow from the request.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The core trait for handling HTTP requests.
///
/// This trait defines the fundamental interface that all request handlers must implement.
/// Handlers receive a `Request` and asynchronously produce a `Result<Response>`.
///
/// # Examples
///
/// ## Simple Handler Implementation
/// ```
/// use ignitia::{Handler, Request, Response, Result};
/// use async_trait::async_trait;
///
/// struct GreetingHandler {
///     name: String,
/// }
///
/// #[async_trait]
/// impl Handler for GreetingHandler {
///     async fn handle(&self, _req: Request) -> Result<Response> {
///         Ok(Response::text(format!("Hello, {}!", self.name)))
///     }
/// }
/// ```
///
/// ## Stateful Handler
/// ```
/// use ignitia::{Handler, Request, Response, Result};
/// use async_trait::async_trait;
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// struct CounterHandler {
///     count: AtomicU64,
/// }
///
/// impl CounterHandler {
///     fn new() -> Self {
///         Self {
///             count: AtomicU64::new(0),
///         }
///     }
/// }
///
/// #[async_trait]
/// impl Handler for CounterHandler {
///     async fn handle(&self, _req: Request) -> Result<Response> {
///         let current = self.count.fetch_add(1, Ordering::SeqCst);
///         Ok(Response::text(format!("Request count: {}", current + 1)))
///     }
/// }
/// ```
///
/// ## Database Handler
/// ```
/// use ignitia::{Handler, Request, Response, Result};
/// use async_trait::async_trait;
///
/// struct DatabaseHandler {
///     // Your database connection pool
///     // db_pool: DatabasePool,
/// }
///
/// #[async_trait]
/// impl Handler for DatabaseHandler {
///     async fn handle(&self, req: Request) -> Result<Response> {
///         // Extract user ID from path
///         let user_id = req.param("id")
///             .ok_or_else(|| ignitia::Error::BadRequest("Missing user ID".into()))?;
///
///         // Query database
///         // let user = self.db_pool.get_user(user_id).await?;
///
///         Ok(Response::json(serde_json::json!({
///             "user_id": user_id,
///             "message": "User data would be here"
///         }))?)
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    /// Handle an HTTP request and return a response.
    ///
    /// This method processes the incoming request and produces an HTTP response.
    /// It's called by the framework's routing system when a matching route is found.
    ///
    /// # Parameters
    /// - `req`: The incoming HTTP request with all its data
    ///
    /// # Returns
    /// A `Result<Response>` where:
    /// - `Ok(Response)` represents a successful response
    /// - `Err(Error)` represents an error that will be converted to an error response
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Handler, Request, Response, Result};
    /// use async_trait::async_trait;
    ///
    /// struct EchoHandler;
    ///
    /// #[async_trait]
    /// impl Handler for EchoHandler {
    ///     async fn handle(&self, req: Request) -> Result<Response> {
    ///         let path = req.uri.path();
    ///         Ok(Response::text(format!("You requested: {}", path)))
    ///     }
    /// }
    /// ```
    async fn handle(&self, req: Request) -> Result<Response>;
}

/// A type alias for handler functions stored as trait objects.
///
/// This represents a handler function that:
/// - Takes a `Request` as input
/// - Returns a boxed future that resolves to `Result<Response>`
/// - Is thread-safe (`Send + Sync`)
/// - Has a static lifetime
///
/// # Usage
/// This type is primarily used internally by the framework, but can be useful
/// when you need to store handler functions in data structures.
///
/// # Examples
/// ```
/// use ignitia::{HandlerFn, Request, Response, Result, handler_fn};
/// use std::collections::HashMap;
///
/// let mut handlers: HashMap<String, HandlerFn> = HashMap::new();
///
/// handlers.insert(
///     "greeting".to_string(),
///     handler_fn(|_req: Request| async {
///         Ok(Response::text("Hello!"))
///     })
/// );
/// ```
pub type HandlerFn = Arc<dyn Fn(Request) -> BoxFuture<'static, Result<Response>> + Send + Sync>;

#[async_trait::async_trait]
impl Handler for HandlerFn {
    async fn handle(&self, req: Request) -> Result<Response> {
        (self)(req).await
    }
}

/// Converts a closure into a HandlerFn for legacy support.
///
/// This function allows you to convert async closures or functions into
/// the `HandlerFn` type that can be stored and called later.
///
/// # Type Parameters
/// - `F`: The function type that takes a Request and returns a Future
/// - `Fut`: The Future type returned by the function
///
/// # Parameters
/// - `f`: The function to convert into a handler
///
/// # Returns
/// A `HandlerFn` that can be used with the routing system
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::{handler_fn, Request, Response, Result};
///
/// let my_handler = handler_fn(|req: Request| async move {
///     let path = req.uri.path();
///     Ok(Response::text(format!("Path: {}", path)))
/// });
/// ```
///
/// ## With Request Processing
/// ```
/// use ignitia::{handler_fn, Request, Response, Result};
///
/// let json_handler = handler_fn(|req: Request| async move {
///     let method = req.method.as_str();
///     let content_type = req.header("content-type").unwrap_or("unknown");
///
///     Ok(Response::json(serde_json::json!({
///         "method": method,
///         "content_type": content_type
///     }))?)
/// });
/// ```
///
/// ## Error Handling
/// ```
/// use ignitia::{handler_fn, Request, Response, Result, Error};
///
/// let validated_handler = handler_fn(|req: Request| async move {
///     // Validate request
///     if req.method != http::Method::POST {
///         return Err(Error::BadRequest("Only POST allowed".into()));
///     }
///
///     if req.body.is_empty() {
///         return Err(Error::BadRequest("Body required".into()));
///     }
///
///     Ok(Response::text("Valid request!"))
/// });
/// ```
pub fn handler_fn<F, Fut>(f: F) -> HandlerFn
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    Arc::new(move |req| Box::pin(f(req)))
}

/// Trait for types that can be converted into handlers with automatic extraction.
///
/// This trait enables the framework to automatically extract typed data from requests
/// and pass them as parameters to handler functions. It's the foundation of the
/// type-safe parameter extraction system.
///
/// # Type Parameters
/// - `T`: A tuple representing the extracted parameter types
///
/// # Examples
///
/// The trait is automatically implemented for functions with compatible signatures:
/// ```
/// use ignitia::{IntoHandler, Path, Query, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UserParams { id: u64 }
///
/// #[derive(Deserialize)]
/// struct QueryParams { format: Option<String> }
///
/// // This function automatically implements IntoHandler<(Path<UserParams>, Query<QueryParams>)>
/// async fn get_user(
///     Path(params): Path<UserParams>,
///     Query(query): Query<QueryParams>,
/// ) -> Result<Response> {
///     Ok(Response::text(format!("User {}, format: {:?}", params.id, query.format)))
/// }
/// ```
///
/// # Implementation Details
/// The trait is implemented using a macro system that generates implementations
/// for functions with 0-8 extractor parameters, providing compile-time type safety
/// while maintaining runtime efficiency.
#[async_trait::async_trait]
pub trait IntoHandler<T>: Clone + Send + Sync + 'static {
    /// Calls the handler with the given request.
    ///
    /// This method is responsible for extracting the required parameters from
    /// the request and calling the actual handler function.
    ///
    /// # Parameters
    /// - `req`: The incoming HTTP request
    ///
    /// # Returns
    /// A `Result<Response>` from the handler execution
    ///
    /// # Errors
    /// Returns an error if:
    /// - Parameter extraction fails
    /// - The handler function returns an error
    async fn call(self, req: Request) -> Result<Response>;
}

/// Converts an IntoHandler into a HandlerFn.
///
/// This function bridges the gap between the type-safe `IntoHandler` system
/// and the runtime `HandlerFn` type used by the routing system.
///
/// # Type Parameters
/// - `H`: The handler type that implements IntoHandler
/// - `T`: The tuple of extractor types
///
/// # Parameters
/// - `handler`: The handler to convert
///
/// # Returns
/// A `HandlerFn` that can be used with the router
///
/// # Examples
/// ```
/// use ignitia::{into_handler, Router, Response, Result};
///
/// async fn hello_handler() -> Result<Response> {
///     Ok(Response::text("Hello World!"))
/// }
///
/// let router = Router::new()
///     .route("/hello", http::Method::GET, into_handler(hello_handler));
/// ```
///
/// Note: In most cases, you won't need to call this function directly as the
/// router methods (`.get()`, `.post()`, etc.) handle the conversion automatically.
pub fn into_handler<H, T>(handler: H) -> HandlerFn
where
    H: IntoHandler<T>,
{
    Arc::new(move |req| {
        let handler = handler.clone();
        Box::pin(async move { handler.call(req).await })
    })
}

/// Implementation for functions with no extractors that just return Response.
///
/// This allows simple functions that don't need request data to be used as handlers.
///
/// # Examples
/// ```
/// use ignitia::{Response, Result};
///
/// // This function implements IntoHandler<()>
/// async fn health_check() -> Result<Response> {
///     Ok(Response::json(serde_json::json!({
///         "status": "healthy",
///         "timestamp": chrono::Utc::now().to_rfc3339()
///     }))?)
/// }
/// ```
#[async_trait::async_trait]
impl<F, Fut> IntoHandler<()> for F
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    async fn call(self, _req: Request) -> Result<Response> {
        self().await
    }
}

/// Macro to generate IntoHandler implementations for different numbers of extractors.
///
/// This macro generates implementations for functions that take 1-8 extractor parameters,
/// automatically handling the extraction of each parameter type from the request.
///
/// The macro generates code that:
/// 1. Extracts each parameter using its `FromRequest` implementation
/// 2. Calls the handler function with the extracted parameters
/// 3. Returns the result from the handler
///
/// # Generated Implementations
/// For each number of parameters (1-8), the macro generates an implementation like:
/// ```
/// impl<F, Fut, T1, T2> IntoHandler<(T1, T2)> for F
/// where
///     F: Fn(T1, T2) -> Fut + Clone + Send + Sync + 'static,
///     Fut: Future<Output = Result<Response>> + Send + 'static,
///     T1: FromRequest + Send,
///     T2: FromRequest + Send,
/// {
///     async fn call(self, req: Request) -> Result<Response> {
///         let t1 = T1::from_request(&req)?;
///         let t2 = T2::from_request(&req)?;
///         self(t1, t2).await
///     }
/// }
/// ```
macro_rules! impl_handler {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[async_trait::async_trait]
        impl<F, Fut, $($ty,)* $last> IntoHandler<($($ty,)* $last,)> for F
        where
            F: Fn($($ty,)* $last) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<Response>> + Send + 'static,
            $( $ty: extractor::FromRequest + Send, )*
            $last: extractor::FromRequest + Send,
        {
            async fn call(self, req: Request) -> Result<Response> {
                $(
                    let $ty = $ty::from_request(&req)?;
                )*
                let $last = $last::from_request(&req)?;

                self($($ty,)* $last).await
            }
        }
    };
}

// Generate implementations for 1-8 parameters
impl_handler!([], T1);
impl_handler!([T1], T2);
impl_handler!([T1, T2], T3);
impl_handler!([T1, T2, T3], T4);
impl_handler!([T1, T2, T3, T4], T5);
impl_handler!([T1, T2, T3, T4, T5], T6);
impl_handler!([T1, T2, T3, T4, T5, T6], T7);
impl_handler!([T1, T2, T3, T4, T5, T6, T7], T8);

/// A marker type to distinguish raw Request handlers from extractors.
///
/// This type is used when you want to access the raw `Request` object directly
/// in your handler, bypassing the extraction system.
///
/// # Examples
/// ```
/// use ignitia::{raw_handler, Request, Response, Result};
///
/// let handler = raw_handler(|req: Request| async move {
///     let method = req.method.as_str();
///     let path = req.uri.path();
///     let headers_count = req.headers.len();
///
///     Ok(Response::json(serde_json::json!({
///         "method": method,
///         "path": path,
///         "headers_count": headers_count
///     }))?)
/// });
/// ```
pub struct RawRequest(pub Request);

/// Implementation for functions that take RawRequest directly.
///
/// This allows handlers to receive the complete Request object without
/// going through the extraction system.
#[async_trait::async_trait]
impl<F, Fut> IntoHandler<(RawRequest,)> for F
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    async fn call(self, req: Request) -> Result<Response> {
        self(req).await
    }
}

/// Convenience function for creating raw request handlers.
///
/// This function creates a handler that receives the raw `Request` object,
/// giving you full access to all request data without automatic extraction.
///
/// # Type Parameters
/// - `F`: The function type that takes a Request and returns a Future
/// - `Fut`: The Future type returned by the function
///
/// # Parameters
/// - `f`: The function to convert into a raw handler
///
/// # Returns
/// A handler that implements `IntoHandler<(RawRequest,)>`
///
/// # When to Use
/// Use raw handlers when you need:
/// - Access to raw request data
/// - Custom parsing logic
/// - Performance-critical code that wants to avoid extraction overhead
/// - Complex request processing that doesn't fit the extraction pattern
///
/// # Examples
///
/// ## Basic Raw Handler
/// ```
/// use ignitia::{raw_handler, Request, Response, Result};
///
/// let handler = raw_handler(|req: Request| async move {
///     Ok(Response::text(format!("Method: {}", req.method)))
/// });
/// ```
///
/// ## Custom Header Processing
/// ```
/// use ignitia::{raw_handler, Request, Response, Result};
///
/// let custom_auth_handler = raw_handler(|req: Request| async move {
///     // Custom authentication logic
///     let auth_header = req.header("authorization")
///         .ok_or_else(|| ignitia::Error::Unauthorized)?;
///
///     if !auth_header.starts_with("Bearer ") {
///         return Err(ignitia::Error::Unauthorized);
///     }
///
///     let token = &auth_header[7..];
///
///     // Validate token (custom logic)
///     if !is_valid_token(token) {
///         return Err(ignitia::Error::Unauthorized);
///     }
///
///     Ok(Response::text("Access granted"))
/// });
///
/// # fn is_valid_token(_token: &str) -> bool { true }
/// ```
///
/// ## File Upload Handler
/// ```
/// use ignitia::{raw_handler, Request, Response, Result, Error};
///
/// let upload_handler = raw_handler(|req: Request| async move {
///     // Check content type
///     let content_type = req.header("content-type")
///         .ok_or_else(|| Error::BadRequest("Content-Type required".into()))?;
///
///     if !content_type.starts_with("multipart/form-data") {
///         return Err(Error::BadRequest("Expected multipart/form-data".into()));
///     }
///
///     // Process raw body for file upload
///     let body_size = req.body.len();
///
///     // Custom multipart parsing logic would go here
///
///     Ok(Response::text(format!("Received {} bytes", body_size)))
/// });
/// ```
pub fn raw_handler<F, Fut>(f: F) -> impl IntoHandler<(RawRequest,)>
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    f
}
