//! Handler module for the Ignitia web framework.
//!
//! This module provides the core handler abstractions and implementations for processing HTTP requests.
//! It includes traits for defining request handlers, extractors for parsing request data, and utilities
//! for creating handlers with various signatures.
//!
//! # Overview
//!
//! The handler system is built around several key traits:
//! - [`Handler`] - Core async trait for handling requests
//! - [`UniversalHandler`] - Generic handler trait that works with any `IntoResponse` return type
//! - [`IntoHandler`] - Trait for converting functions into handlers
//! - [`FromRequest`] - Trait for extracting data from requests (defined in `extractor` module)
//!
//! # Examples
//!
//! ## Basic Handler
//!
//! ```
//! use ignitia::prelude::*;
//!
//! async fn hello_handler() -> &'static str {
//!     "Hello, World!"
//! }
//!
//! let router = Router::new()
//!     .get("/hello", hello_handler);
//! ```
//!
//! ## Handler with Extractors
//!
//! ```
//! use ignitia::prelude::*;
//!
//! async fn user_handler(Path(id): Path<String>, Json(data): Json<UserData>) -> Result<Response> {
//!     // Process user data...
//!     Ok(Response::json(data))
//! }
//! ```
//!
//! ## Handler returning custom types
//!
//! Any type implementing `IntoResponse` can be returned from handlers:
//!
//! ```
//! use ignitia::prelude::*;
//!
//! async fn status_handler() -> StatusCode {
//!     StatusCode::OK
//! }
//!
//! async fn text_handler() -> String {
//!     "Hello".to_string()
//! }
//!
//! async fn result_handler() -> Result<Response> {
//!     Ok(Response::json(json!({"status": "ok"})))
//! }
//! ```

pub mod extractor;

use crate::response::IntoResponse;
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
    #[inline]
    async fn handle(&self, req: Request) -> Result<Response> {
        (self)(req).await
    }
}

/// Universal handler trait that works with any return type implementing [`IntoResponse`].
///
/// This trait enables handlers to return various types (String, StatusCode, Result, etc.)
/// which are automatically converted to [`Response`] instances. It's the foundation of
/// Ignitia's flexible handler system.
///
/// The type parameter `T` represents the extractors and their order. This trait is
/// implemented via macro for functions with 0-8 parameters.
///
/// # Type Parameter
///
/// * `T` - Tuple representing the extractor types used by the handler
///
/// # Examples
///
/// ```
/// // Handler with no extractors
/// async fn simple_handler() -> String {
///     "Hello".to_string()
/// }
///
/// // Handler with Path extractor
/// async fn path_handler(Path(id): Path<u32>) -> String {
///     format!("User ID: {}", id)
/// }
///
/// // Handler with multiple extractors
/// async fn multi_handler(
///     Path(id): Path<String>,
///     Query(params): Query<HashMap<String, String>>,
///     Json(body): Json<MyData>
/// ) -> Result<Response> {
///     // Process request...
///     Ok(Response::json(body))
/// }
/// ```
#[async_trait::async_trait]
pub trait UniversalHandler<T>: Clone + Send + Sync + 'static {
    /// Call the handler with the given request and return a [`Response`].
    ///
    /// This method extracts parameters from the request, calls the handler function,
    /// and converts the result to a [`Response`].
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to process
    ///
    /// # Returns
    ///
    /// Returns a [`Response`]. If parameter extraction fails, an error response is returned.
    async fn call(self, req: Request) -> Response;
}

/// Convert a [`UniversalHandler`] to a [`HandlerFn`].
///
/// This function wraps a universal handler in a [`HandlerFn`], making it compatible
/// with the router's internal handler storage. It's used internally by the framework
/// but can be useful for advanced use cases.
///
/// # Type Parameters
///
/// * `H` - The handler type implementing [`UniversalHandler`]
/// * `T` - The extractor tuple type
///
/// # Arguments
///
/// * `handler` - The universal handler to convert
///
/// # Returns
///
/// Returns a [`HandlerFn`] that can be used with the router.
///
/// # Examples
///
/// ```
/// use ignitia::handler::universal_handler;
///
/// async fn my_handler() -> &'static str {
///     "Hello"
/// }
///
/// let handler_fn = universal_handler(my_handler);
/// ```
#[inline]
pub fn universal_handler<H, T>(handler: H) -> HandlerFn
where
    H: UniversalHandler<T>,
{
    Arc::new(move |req| {
        let handler = handler.clone();
        Box::pin(async move {
            let response = handler.call(req).await;
            Ok(response)
        })
    })
}

/// Implementation of [`UniversalHandler`] for functions with no extractors.
///
/// This allows simple async functions that return any type implementing [`IntoResponse`]
/// to be used as handlers.
#[async_trait::async_trait]
impl<F, Fut, R> UniversalHandler<()> for F
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    #[inline]
    async fn call(self, _req: Request) -> Response {
        self().await.into_response()
    }
}

/// # Arguments
///
/// * `f` - The function to convert to a handler
///
/// # Returns
///
/// Returns a [`HandlerFn`] wrapping the provided function.
///
/// # Examples
///
/// ```
/// use ignitia::handler::handler_fn;
/// use ignitia::prelude::*;
///
/// let handler = handler_fn(|req: Request| async move {
///     Ok(Response::text(format!("Method: {}", req.method)))
/// });
/// ```
#[inline]
pub fn handler_fn<F, Fut>(f: F) -> HandlerFn
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    Arc::new(move |req| Box::pin(f(req)))
}

/// Trait for converting functions into handlers.
///
/// This trait is similar to [`UniversalHandler`] but specifically for handlers
/// that return `Result<Response>`. It provides a bridge between regular async
/// functions and the handler system.
///
/// The type parameter `T` represents the extractor types used by the handler.
///
/// # Type Parameter
///
/// * `T` - Tuple representing the extractor types
#[async_trait::async_trait]
pub trait IntoHandler<T>: Clone + Send + Sync + 'static {
    /// Call the handler and return a `Result<Response>`.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to process
    ///
    /// # Returns
    ///
    /// Returns `Result<Response>`, allowing handlers to propagate errors.
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
#[inline]
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
    #[inline]
    async fn call(self, _req: Request) -> Result<Response> {
        self().await
    }
}

// THE MAGIC MACRO: Works with ANY return type that implements IntoResponse
macro_rules! impl_universal_handler {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[async_trait::async_trait]
        impl<F, Fut, $($ty,)* $last, R> UniversalHandler<($($ty,)* $last,)> for F
        where
            F: Fn($($ty,)* $last) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = R> + Send + 'static,
            R: IntoResponse, // ANY type that can become a Response!
            $( $ty: extractor::FromRequest + Send, )*
            $last: extractor::FromRequest + Send,
        {
            #[inline]
            async fn call(self, req: Request) -> Response {
                $(
                    let $ty = match $ty::from_request(&req) {
                        Ok(val) => val,
                        Err(error_response) => return error_response.into_response(),
                    };
                )*
                let $last = match $last::from_request(&req) {
                    Ok(val) => val,
                    Err(error_response) => return error_response.into_response(),
                };

                // THE MAGIC: auto-convert ANY return type to Response!
                let result = self($($ty,)* $last).await;
                result.into_response()
            }
        }
    };
}

// Generate implementations for 1-8 parameters
impl_universal_handler!([], T1);
impl_universal_handler!([T1], T2);
impl_universal_handler!([T1, T2], T3);
impl_universal_handler!([T1, T2, T3], T4);
impl_universal_handler!([T1, T2, T3, T4], T5);
impl_universal_handler!([T1, T2, T3, T4, T5], T6);
impl_universal_handler!([T1, T2, T3, T4, T5, T6], T7);
impl_universal_handler!([T1, T2, T3, T4, T5, T6, T7], T8);

/// Wrapper type for handlers that need raw [`Request`] access.
///
/// This type can be used as an extractor when you want to receive the entire
/// request object without automatic extraction.
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn handler(RawRequest(req): RawRequest) -> Result<Response> {
///     println!("Method: {}", req.method);
///     println!("URI: {}", req.uri);
///     Ok(Response::ok())
/// }
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
    #[inline]
    async fn call(self, req: Request) -> Result<Response> {
        self(req).await
    }
}

#[async_trait::async_trait]
impl<F, Fut, R> UniversalHandler<(RawRequest,)> for F
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    #[inline]
    async fn call(self, req: Request) -> Response {
        let result = self(req).await;
        result.into_response()
    }
}

/// Create a raw request handler that receives the full [`Request`] object.
///
/// This function is useful when you need complete control over request processing
/// and want to return any type implementing [`IntoResponse`].
///
/// # Type Parameters
///
/// * `F` - Function type taking [`Request`]
/// * `Fut` - Future type returned by the function
/// * `R` - Response type implementing [`IntoResponse`]
///
/// # Arguments
///
/// * `f` - The function to convert to a handler
///
/// # Returns
///
/// Returns an implementation of `UniversalHandler<(RawRequest,)>`.
///
/// # Examples
///
/// ```
/// use ignitia::handler::raw_handler;
/// use ignitia::prelude::*;
///
/// let handler = raw_handler(|req: Request| async move {
///     if req.method == Method::POST {
///         "POST request"
///     } else {
///         "Other request"
///     }
/// });
/// ```
#[inline]
pub fn raw_handler<F, Fut, R>(f: F) -> impl UniversalHandler<(RawRequest,)>
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    f
}

//----Old Macro--------

// Macro to generate IntoHandler implementations for different numbers of extractors.

// This macro generates implementations for functions that take 1-8 extractor parameters,
// automatically handling the extraction of each parameter type from the request.

// The macro generates code that:
// 1. Extracts each parameter using its `FromRequest` implementation
// 2. Calls the handler function with the extracted parameters
// 3. Returns the result from the handler

// # Generated Implementations
// For each number of parameters (1-8), the macro generates an implementation like:
// ```
// impl<F, Fut, T1, T2> IntoHandler<(T1, T2)> for F
// where
//     F: Fn(T1, T2) -> Fut + Clone + Send + Sync + 'static,
//     Fut: Future<Output = Result<Response>> + Send + 'static,
//     T1: FromRequest + Send,
//     T2: FromRequest + Send,
// {
//     async fn call(self, req: Request) -> Result<Response> {
//         let t1 = T1::from_request(&req)?;
//         let t2 = T2::from_request(&req)?;
//         self(t1, t2).await
//     }
// }
// ```
// macro_rules! impl_handler {
//     (
//         [$($ty:ident),*], $last:ident
//     ) => {
//         #[async_trait::async_trait]
//         impl<F, Fut, $($ty,)* $last> IntoHandler<($($ty,)* $last,)> for F
//         where
//             F: Fn($($ty,)* $last) -> Fut + Clone + Send + Sync + 'static,
//             Fut: Future<Output = Result<Response>> + Send + 'static,
//             $( $ty: extractor::FromRequest + Send, )*
//             $last: extractor::FromRequest + Send,
//         {
//             async fn call(self, req: Request) -> Result<Response> {
//                 $(
//                     let $ty = $ty::from_request(&req)?;
//                 )*
//                 let $last = $last::from_request(&req)?;

//                 self($($ty,)* $last).await
//             }
//         }
//     };
// }

// // Generate implementations for 1-8 parameters
// impl_handler!([], T1);
// impl_handler!([T1], T2);
// impl_handler!([T1, T2], T3);
// impl_handler!([T1, T2, T3], T4);
// impl_handler!([T1, T2, T3, T4], T5);
// impl_handler!([T1, T2, T3, T4, T5], T6);
// impl_handler!([T1, T2, T3, T4, T5, T6], T7);
// impl_handler!([T1, T2, T3, T4, T5, T6, T7], T8);
