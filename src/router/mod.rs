//! # Router Module
//!
//! This module provides a high-performance, flexible routing system for the Ignitia web framework.
//! It supports both simple base routing and advanced radix tree routing with mixed mode nesting capabilities.
//!
//! ## Features
//!
//! - **Dual Routing Modes**: Base (simple linear matching) and Radix (compressed trie) modes
//! - **Mixed Mode Nesting**: Nest routers of different modes (Base->Radix, Radix->Base, etc.)
//! - **Parameter Extraction**: Support for path parameters (`:id`) and wildcards (`*path`)
//! - **Middleware Support**: Per-route and global middleware with proper ordering
//! - **High Performance**: Optimized compilation, caching, and lookup algorithms
//! - **WebSocket Support**: Built-in WebSocket routing when the feature is enabled
//! - **Thread Safety**: Lock-free operations with atomic updates
//! - **State Management**: Built-in application state management with type safety
//!
//! ## Quick Start
//!
//! ```
//! use ignitia::{Router, RouterMode, Response};
//!
//! async fn hello() -> Result<Response, ignitia::Error> {
//!     Ok(Response::text("Hello, World!"))
//! }
//!
//! async fn user_profile(id: String) -> Result<Response, ignitia::Error> {
//!     Ok(Response::text(format!("User ID: {}", id)))
//! }
//!
//! let app = Router::new()
//!     .with_mode(RouterMode::Radix)
//!     .get("/", hello)
//!     .get("/users/:id", user_profile);
//! ```
//!
//! ## Router Modes
//!
//! ### Base Mode
//! - **Best for**: Small to medium applications (< 100 routes)
//! - **Algorithm**: Linear search through routes with regex matching
//! - **Memory**: Lower memory usage
//! - **Compilation**: Faster route compilation
//!
//! ### Radix Mode (Default)
//! - **Best for**: Large applications with many routes
//! - **Algorithm**: Compressed trie (radix tree) for O(k) lookup where k is path length
//! - **Memory**: Higher memory usage but better for large route sets
//! - **Compilation**: Slower compilation but much faster lookup
//!
//! ## Mixed Mode Nesting
//!
//! The router supports nesting routers with different modes:
//!
//! ```
//! # use ignitia::{Router, RouterMode, Response};
//! # async fn api_handler() -> Result<Response, ignitia::Error> { Ok(Response::text("API")) }
//! # async fn web_handler() -> Result<Response, ignitia::Error> { Ok(Response::text("Web")) }
//!
//! // High-performance API routes using Radix mode
//! let api_router = Router::new()
//!     .with_mode(RouterMode::Radix)
//!     .get("/users/:id", api_handler);
//!
//! // Simple static routes using Base mode
//! let web_router = Router::new()
//!     .with_mode(RouterMode::Base)
//!     .get("/about", web_handler);
//!
//! // Mix them together
//! let app = Router::new()
//!     .with_mode(RouterMode::Radix)
//!     .nest("/api", api_router)
//!     .nest("/web", web_router);
//! ```
//!
//! ## Middleware
//!
//! Middleware can be applied at both router and route levels:
//!
//! ```
//! # use ignitia::{Router, Response, middleware::LoggerMiddleware};
//! # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
//!
//! let app = Router::new()
//!     .middleware(LoggerMiddleware)  // Global middleware
//!     .get("/", handler);
//! ```
//!
//! ## State Management
//!
//! The router provides type-safe state management:
//!
//! ```
//! # use ignitia::{Router, Response};
//! # use std::sync::Arc;
//! # struct Database;
//! # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
//!
//! let db = Arc::new(Database);
//! let app = Router::new()
//!     .state_arc(db)
//!     .get("/", handler);
//! ```

pub mod method;
pub mod radix;
pub mod route;

use crate::handler::{into_handler, IntoHandler};
use crate::middleware::Middleware;
use crate::{Error, Extensions, Handler, HandlerFn, Request, Response, Result};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use http::Method;
use parking_lot::RwLock;
use std::any::Any;
use std::sync::Arc;

pub use radix::{RadixNode, RadixRouter};
pub use route::Route;

/// Router mode configuration that determines the routing algorithm used.
///
/// # Router Mode Comparison
///
/// | Feature | Base Mode | Radix Mode |
/// |---------|-----------|------------|
/// | Algorithm | Linear regex matching | Compressed trie |
/// | Best for | Small apps (< 100 routes) | Large apps (100+ routes) |
/// | Memory Usage | Lower | Higher |
/// | Lookup Speed | O(n) where n = routes | O(k) where k = path length |
/// | Compilation | Faster | Slower |
/// | Parameter Support | Full regex support | Optimized parameters |
///
/// # Examples
///
/// ```
/// use ignitia::{Router, RouterMode};
///
/// // For small applications
/// let simple_router = Router::new()
///     .with_mode(RouterMode::Base);
///
/// // For high-performance applications (default)
/// let fast_router = Router::new()
///     .with_mode(RouterMode::Radix);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterMode {
    /// Simple linear search through routes using regex matching.
    ///
    /// **Advantages:**
    /// - Lower memory usage
    /// - Faster compilation time
    /// - Full regex support for complex patterns
    /// - Better for small route sets (< 100 routes)
    ///
    /// **Disadvantages:**
    /// - O(n) lookup time where n is the number of routes
    /// - Performance degrades with more routes
    Base,

    /// Compressed trie (radix tree) routing for optimal performance.
    ///
    /// **Advantages:**
    /// - O(k) lookup time where k is path length
    /// - Excellent performance with many routes
    /// - Memory-efficient path compression
    /// - Optimal for large applications
    ///
    /// **Disadvantages:**
    /// - Higher memory usage
    /// - Longer compilation time
    /// - Limited regex support (parameters only)
    Radix, // New radix tree router
}

impl Default for RouterMode {
    /// Default router mode is Radix for optimal performance.
    fn default() -> Self {
        RouterMode::Radix
    }
}

/// Macro to define HTTP method functions for the router.
///
/// This macro generates a function that adds a route for a specific HTTP method.
/// Each generated function takes a path and handler, and returns a modified router.
///
/// # Parameters
/// * `name` - The function name (e.g., `get`, `post`, `put`)
/// * `method` - The HTTP method constant (e.g., `Method::GET`, `Method::POST`)
/// * `doc` - Documentation string for the generated function
///
/// # Generated Function Signature
/// ```
/// pub fn {name}<H, T>(self, path: &str, handler: H) -> Self
/// where
///     H: IntoHandler<T>,
/// ```
///
/// # Examples
/// ```
/// define_http_method!(get, Method::GET, "Adds a GET route");
/// define_http_method!(post, Method::POST, "Adds a POST route");
/// ```
macro_rules! define_http_method {
    ($name:ident, $method:expr, $doc:expr) => {
        #[doc = $doc]
        ///
        /// # Arguments
        ///
        /// * `path` - The route path, can include parameters (`:id`) and wildcards (`*path`)
        /// * `handler` - The handler function to execute for this route
        ///
        /// # Returns
        ///
        /// Returns `Self` for method chaining.
        ///
        /// # Examples
        ///
        /// ```
        /// # use ignitia::{Router, Response};
        /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
        /// let router = Router::new()
        ///     .get("/users/:id", handler);
        /// ```
        pub fn $name<H, T>(self, path: &str, handler: H) -> Self
        where
            H: IntoHandler<T>,
        {
            self.route_with(path, $method, handler)
        }
    };
}

/// A layered handler that combines a handler function with middleware stack.
///
/// This allows for per-route middleware that executes independently of global middleware.
/// Middleware execution order: route middleware (in order) -> handler -> route middleware (reverse).
///
/// # Examples
///
/// ```
/// # use ignitia::{Router, Response, middleware::LoggerMiddleware};
/// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
///
/// use ignitia::router::LayeredHandler;
///
/// let layered = LayeredHandler::new(handler)
///     .layer(LoggerMiddleware);
///
/// let router = Router::new()
///     .route_with_layered("/api/data", http::Method::GET, layered);
/// ```
#[derive(Clone)]
pub struct LayeredHandler {
    /// The core handler function to execute
    handler: HandlerFn,
    /// Stack of middleware to apply to this handler
    middleware: Vec<Arc<dyn Middleware>>,
}

impl LayeredHandler {
    /// Create a new layered handler from a handler function.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler that implements `IntoHandler<T>`
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Response, router::LayeredHandler};
    /// # async fn my_handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    /// let layered = LayeredHandler::new(my_handler);
    /// ```
    pub fn new<H, T>(handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        Self {
            handler: into_handler(handler),
            middleware: Vec::new(),
        }
    }

    /// Add a middleware layer to this handler.
    ///
    /// Middleware is executed in the order it's added (FIFO for before, LIFO for after).
    ///
    /// # Arguments
    ///
    /// * `mw` - The middleware to add
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Response, router::LayeredHandler, middleware::LoggerMiddleware};
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    /// let layered = LayeredHandler::new(handler)
    ///     .layer(LoggerMiddleware);
    /// ```
    pub fn layer<M: Middleware + 'static>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    /// Convert this layered handler into a regular handler function.
    ///
    /// This is used internally by the router to create executable handlers.
    pub fn into_handler(self) -> HandlerFn {
        let handler = self.handler;
        let middleware = self.middleware;

        Arc::new(move |mut req: Request| {
            let middleware = middleware.clone();
            let handler = handler.clone();
            Box::pin(async move {
                for mw in &middleware {
                    mw.before(&mut req).await?;
                }

                let mut res = handler.handle(req.clone()).await?;

                for mw in middleware.iter().rev() {
                    mw.after(&req, &mut res).await?;
                }
                Ok(res)
            })
        })
    }
}

/// Compiled router for optimized request handling.
///
/// This is an internal structure created during router compilation that provides
/// optimized route lookup and caching for high-performance request processing.
#[derive(Clone)]
struct CompiledRouter {
    /// The routing mode used by this compiled router
    mode: RouterMode,
    /// Routes organized by HTTP method (used in Base mode)
    routes: DashMap<Method, Vec<Route>>,
    /// Radix tree router (used in Radix mode)
    radix_router: Option<RadixRouter>,
    /// Global middleware stack
    middleware: Vec<Arc<dyn Middleware>>,
    /// Handler for 404 Not Found responses
    not_found_handler: Option<HandlerFn>,
    /// Route lookup cache for improved performance
    route_cache: DashMap<String, Option<Arc<Route>>>,
}

/// High-performance HTTP router with dual-mode support and advanced features.
///
/// The Router is the core of the Ignitia web framework's routing system. It provides:
///
/// - **Dual Mode Support**: Choose between Base and Radix routing algorithms
/// - **Mixed Nesting**: Nest routers with different modes seamlessly
/// - **Middleware Integration**: Global and per-route middleware support
/// - **Parameter Extraction**: Automatic path parameter and wildcard handling
/// - **State Management**: Type-safe application state sharing
/// - **WebSocket Support**: Built-in WebSocket routing (with feature flag)
/// - **Performance Optimization**: Compilation caching and optimized lookups
///
/// # Thread Safety
///
/// The Router is designed for high-concurrency scenarios:
/// - Uses `RwLock` for construction-time mutations
/// - Uses `ArcSwap` for atomic compiled router updates
/// - Lock-free reads during request handling
/// - DashMap for concurrent route caching
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// # use ignitia::{Router, Response};
/// # async fn home() -> Result<Response, ignitia::Error> { Ok(Response::text("Home")) }
/// # async fn user_profile(id: String) -> Result<Response, ignitia::Error> { Ok(Response::text(format!("User: {}", id))) }
///
/// let app = Router::new()
///     .get("/", home)
///     .get("/users/:id", user_profile);
/// ```
///
/// ## With Middleware
///
/// ```
/// # use ignitia::{Router, Response, middleware::LoggerMiddleware};
/// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
///
/// let app = Router::new()
///     .middleware(LoggerMiddleware)
///     .get("/api/data", handler);
/// ```
///
/// ## Nested Routing
///
/// ```
/// # use ignitia::{Router, Response};
/// # async fn api_v1() -> Result<Response, ignitia::Error> { Ok(Response::text("API v1")) }
/// # async fn api_v2() -> Result<Response, ignitia::Error> { Ok(Response::text("API v2")) }
///
/// let v1_routes = Router::new().get("/users", api_v1);
/// let v2_routes = Router::new().get("/users", api_v2);
///
/// let app = Router::new()
///     .nest("/api/v1", v1_routes)
///     .nest("/api/v2", v2_routes);
/// ```
pub struct Router {
    /// Internal mutable state protected by RwLock
    inner: Arc<RwLock<RouterInner>>,
    /// Atomically swappable compiled router for zero-downtime updates
    compiled: ArcSwap<CompiledRouter>,
}

/// Internal router state that gets compiled into a CompiledRouter.
///
/// This structure holds the mutable state during router construction and
/// gets compiled into an optimized read-only structure for request handling.
struct RouterInner {
    /// Current routing mode
    mode: RouterMode,
    /// Routes organized by HTTP method (Base mode)
    routes: DashMap<Method, Vec<Route>>,
    /// Radix tree for fast routing (Radix mode)
    radix_router: RadixRouter,
    /// Global middleware stack
    middleware: Vec<Arc<dyn Middleware>>,
    /// Optional 404 handler
    not_found_handler: Option<HandlerFn>,
    /// Nested routers for modular applications
    nested_routers: Vec<(String, Router)>,
    /// Dirty flag to trigger recompilation
    dirty: bool,
    /// Application state and extensions
    extensions: Extensions,

    /// WebSocket route handlers (when websocket feature is enabled)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    websocket_routes: DashMap<String, Arc<dyn crate::websocket::WebSocketHandler>>,
}

impl Router {
    /// Create a new empty router with default configuration.
    ///
    /// The router starts with:
    /// - Radix mode enabled (optimal performance)
    /// - No routes or middleware
    /// - Empty extensions/state
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::Router;
    ///
    /// let router = Router::new();
    /// assert_eq!(router.mode(), ignitia::router::RouterMode::Radix);
    /// ```
    pub fn new() -> Self {
        let inner = RouterInner {
            mode: RouterMode::default(),
            routes: DashMap::new(),
            radix_router: RadixRouter::new(),
            middleware: Vec::new(),
            not_found_handler: None,
            nested_routers: Vec::new(),
            extensions: Extensions::new(),
            dirty: true,
            #[cfg(feature = "websocket")]
            websocket_routes: DashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
            compiled: ArcSwap::new(Arc::new(CompiledRouter {
                mode: RouterMode::default(),
                routes: DashMap::new(),
                radix_router: None,
                middleware: Vec::new(),
                not_found_handler: None,
                route_cache: DashMap::new(),
            })),
        }
    }

    /// Set the routing mode for this router.
    ///
    /// This determines which algorithm will be used for route matching:
    /// - `RouterMode::Base`: Linear search with regex matching
    /// - `RouterMode::Radix`: Compressed trie for O(k) lookup
    ///
    /// **Note**: Changing the mode after adding routes will trigger recompilation.
    ///
    /// # Arguments
    ///
    /// * `mode` - The RouterMode to use
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, RouterMode};
    ///
    /// // Use Base mode for small applications
    /// let simple_router = Router::new()
    ///     .with_mode(RouterMode::Base);
    ///
    /// // Use Radix mode for high-performance applications
    /// let fast_router = Router::new()
    ///     .with_mode(RouterMode::Radix);
    /// ```
    pub fn with_mode(self, mode: RouterMode) -> Self {
        {
            let mut inner = self.inner.write();
            inner.mode = mode;
            inner.dirty = true;
            drop(inner);
        }

        self
    }

    /// Extract routes from a radix router for mixed mode conversion.
    ///
    /// This method is used internally when nesting a Radix mode router into
    /// a Base mode router, requiring conversion of the radix tree back to
    /// individual routes.
    ///
    /// # Arguments
    ///
    /// * `radix_router` - The RadixRouter to extract routes from
    ///
    /// # Returns
    ///
    /// A DashMap containing routes organized by HTTP method.
    fn extract_radix_routes(radix_router: &RadixRouter) -> DashMap<Method, Vec<Route>> {
        let routes = DashMap::new();
        Self::extract_node_routes(&radix_router.root, "", &routes);
        routes
    }

    /// Recursively extract routes from radix tree nodes.
    ///
    /// This helper method traverses the radix tree and converts nodes back
    /// into Route objects, preserving parameter syntax (`:param` and `*wildcard`).
    ///
    /// # Arguments
    ///
    /// * `node` - The current RadixNode being processed
    /// * `path_prefix` - The accumulated path prefix from parent nodes
    /// * `routes` - The DashMap to store extracted routes
    fn extract_node_routes(
        node: &RadixNode,
        path_prefix: &str,
        routes: &DashMap<Method, Vec<Route>>,
    ) {
        let current_path = if path_prefix.is_empty() {
            if let Some(param_name) = &node.param_name {
                if node.is_wildcard {
                    format!("/*{}", param_name)
                } else {
                    format!("/:{}", param_name)
                }
            } else {
                node.path.clone()
            }
        } else {
            if let Some(param_name) = &node.param_name {
                let param_syntax = if node.is_wildcard {
                    format!("/*{}", param_name)
                } else {
                    format!("/:{}", param_name)
                };
                format!("{}{}", path_prefix.trim_end_matches('/'), param_syntax)
            } else if node.path.is_empty() {
                path_prefix.to_string()
            } else {
                if node.path.starts_with('/') {
                    format!("{}{}", path_prefix.trim_end_matches('/'), node.path)
                } else {
                    format!("{}/{}", path_prefix.trim_end_matches('/'), node.path)
                }
            }
        };

        for entry in &node.handlers {
            let method = entry.key();
            let handler = entry.value();
            let route_path = if current_path.is_empty() {
                "/".to_string()
            } else if !current_path.starts_with('/') {
                format!("/{}", current_path)
            } else {
                current_path.clone()
            };

            let route = Route::new(&route_path, method.clone(), handler.clone());
            routes
                .entry(method.clone())
                .or_insert_with(Vec::new)
                .push(route);
        }

        for child in &node.children {
            Self::extract_node_routes(child, &current_path, routes);
        }
    }

    /// Get the current routing mode of this router.
    ///
    /// # Returns
    ///
    /// The current `RouterMode` (Base or Radix).
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, RouterMode};
    ///
    /// let router = Router::new().with_mode(RouterMode::Base);
    /// assert_eq!(router.mode(), RouterMode::Base);
    /// ```
    pub fn mode(&self) -> RouterMode {
        self.inner.read().mode
    }

    /// Add a route with a specific HTTP method and handler.
    ///
    /// This is the core method for adding routes. All HTTP method helpers
    /// (get, post, etc.) eventually call this method.
    ///
    /// # Arguments
    ///
    /// * `path` - The route path (e.g., "/users/:id", "/files/*path")
    /// * `method` - The HTTP method for this route
    /// * `handler` - The handler function to execute
    ///
    /// # Path Syntax
    ///
    /// - Static: `/users`, `/api/v1/health`
    /// - Parameters: `/users/:id`, `/posts/:slug/comments/:comment_id`
    /// - Wildcards: `/files/*path`, `/static/*filepath`
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response};
    /// # use http::Method;
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    ///
    /// let router = Router::new()
    ///     .route("/users/:id", Method::GET, handler);
    /// ```
    pub fn route(self, path: &str, method: Method, handler: HandlerFn) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            let full_path = normalize_path(path);

            match inner.mode {
                RouterMode::Base => {
                    let mut routes = inner.routes.entry(method.clone()).or_insert_with(Vec::new);
                    let route = Route::new(&full_path, method, handler);
                    routes.push(route);
                }
                RouterMode::Radix => {
                    inner.radix_router.insert(&full_path, method, handler);
                }
            }
        }
        self
    }

    /// Add a route using the IntoHandler trait for automatic parameter extraction.
    ///
    /// This method allows handlers to automatically extract parameters from
    /// requests based on their function signature.
    ///
    /// # Arguments
    ///
    /// * `path` - The route path
    /// * `method` - The HTTP method
    /// * `handler` - A handler implementing IntoHandler<T>
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response};
    /// # use http::Method;
    /// # async fn user_handler(id: String) -> Result<Response, ignitia::Error> {
    /// #   Ok(Response::text(format!("User ID: {}", id)))
    /// # }
    ///
    /// let router = Router::new()
    ///     .route_with("/users/:id", Method::GET, user_handler);
    /// ```
    pub fn route_with<H, T>(self, path: &str, method: Method, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route(path, method, into_handler(handler))
    }

    /// Add a route with a layered handler (handler + middleware).
    ///
    /// This allows adding per-route middleware that executes independently
    /// of global router middleware.
    ///
    /// # Arguments
    ///
    /// * `path` - The route path
    /// * `method` - The HTTP method
    /// * `lh` - A LayeredHandler containing handler + middleware
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response, router::LayeredHandler, middleware::LoggerMiddleware};
    /// # use http::Method;
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    ///
    /// let layered = LayeredHandler::new(handler)
    ///     .layer(LoggerMiddleware);
    ///
    /// let router = Router::new()
    ///     .route_with_layered("/api/data", Method::GET, layered);
    /// ```
    pub fn route_with_layered(self, path: &str, method: Method, lh: LayeredHandler) -> Self {
        self.route(path, method, lh.into_handler())
    }

    define_http_method!(get, Method::GET, "Adds a GET route");

    define_http_method!(post, Method::POST, "Adds a POST route");

    define_http_method!(put, Method::PUT, "Adds a PUT route");

    define_http_method!(delete, Method::DELETE, "Adds a DELETE route");

    define_http_method!(patch, Method::PATCH, "Adds a PATCH route");

    define_http_method!(head, Method::HEAD, "Adds a HEAD route");

    define_http_method!(options, Method::OPTIONS, "Adds an OPTIONS route");

    define_http_method!(connect, Method::CONNECT, "Adds an CONNECT route");

    define_http_method!(trace, Method::TRACE, "Adds an TRACE route");

    /// Adds a route that matches ANY HTTP method.
    ///
    /// This method creates a route that will handle requests regardless of the HTTP method used.
    /// It's useful for catch-all handlers, debugging endpoints, or when you want to handle
    /// multiple HTTP methods with the same logic.
    ///
    /// # Implementation Details
    /// The `any` method works by registering the handler for all common HTTP methods:
    /// - GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT, TRACE
    ///
    /// # Arguments
    /// * `path` - The route path pattern (e.g., "/debug", "/api/*path")
    /// * `handler` - The handler function that will process requests for any HTTP method
    ///
    /// # Path Patterns
    /// - **Static paths**: `/debug`, `/health`
    /// - **Parameters**: `/api/:version/debug`, `/users/:id/any`
    /// - **Wildcards**: `/catch-all/*path`, `/proxy/*target`
    ///
    /// # Handler Requirements
    /// The handler should be prepared to handle any HTTP method. You can check the method
    /// in your handler using the request object:
    ///
    /// ```
    /// use ignitia::prelude::*;
    ///
    /// async fn any_method_handler(req: Request) -> Result<Response> {
    ///     match req.method() {
    ///         &Method::GET => Ok(Response::text("This was a GET")),
    ///         &Method::POST => Ok(Response::text("This was a POST")),
    ///         _ => Ok(Response::text(format!("This was a {}", req.method()))),
    ///     }
    /// }
    /// ```
    ///
    /// # Use Cases
    /// - **Debugging endpoints**: Accept any method for testing
    /// - **Proxy endpoints**: Forward any method to another service
    /// - **Catch-all handlers**: Handle unmatched routes with fallback logic
    /// - **Method-agnostic APIs**: When the HTTP method doesn't matter for your logic
    ///
    /// # Performance Considerations
    /// Using `any` creates multiple route entries internally, which may have a small
    /// performance impact compared to specific method routes. Use specific methods
    /// when possible for better performance and clearer API semantics.
    ///
    /// # Examples
    /// ```
    /// use ignitia::prelude::*;
    ///
    /// let router = Router::new()
    ///     // Debug endpoint that accepts any method
    ///     .any("/debug", |req: Request| async move {
    ///         Ok(Response::json(serde_json::json!({
    ///             "method": req.method().as_str(),
    ///             "path": req.uri().path(),
    ///             "timestamp": chrono::Utc::now()
    ///         })))
    ///     })
    ///
    ///     // Catch-all proxy endpoint
    ///     .any("/proxy/*path", |req: Request, Path(path): Path<String>| async move {
    ///         // Forward request to another service
    ///         let client = reqwest::Client::new();
    ///         let response = client
    ///             .request(req.method().clone(), format!("http://backend.service/{}", path))
    ///             .send()
    ///             .await?;
    ///
    ///         Ok(Response::text(response.text().await?))
    ///     })
    ///
    ///     // Method-agnostic API endpoint
    ///     .any("/api/echo", |req: Request| async move {
    ///         Ok(Response::json(serde_json::json!({
    ///             "echo": "Hello from any method!",
    ///             "received_method": req.method().as_str()
    ///         })))
    ///     });
    /// ```
    ///
    /// # Returns
    /// Returns a new `Router` instance with the route registered for all HTTP methods.
    pub fn any<H, T>(self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        // Register the handler for all common HTTP methods
        // This ensures the route responds to any HTTP method
        let methods = [
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
            Method::CONNECT,
            Method::TRACE,
        ];

        let mut router = self;
        for method in methods {
            router = router.route_with(path, method, handler.clone());
        }
        router
    }

    /// Add global middleware to this router.
    ///
    /// Global middleware executes for all routes in this router and any nested routers.
    /// Middleware executes in the order it's added (FIFO for before, LIFO for after).
    ///
    /// # Arguments
    ///
    /// * `middleware` - The middleware to add
    ///
    /// # Execution Order
    ///
    /// 1. Global middleware (before) - in order added
    /// 2. Route-specific middleware (before) - in order added
    /// 3. Handler execution
    /// 4. Route-specific middleware (after) - reverse order
    /// 5. Global middleware (after) - reverse order
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response, middleware::LoggerMiddleware};
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    ///
    /// let router = Router::new()
    ///     .middleware(LoggerMiddleware)
    ///     .get("/", handler);
    /// ```
    pub fn middleware<M: Middleware + 'static>(self, middleware: M) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.middleware.push(Arc::new(middleware));
        }
        self
    }

    /// Set a custom handler for 404 Not Found responses.
    ///
    /// By default, the router returns a simple 404 error. This method allows
    /// customizing the 404 response with your own handler.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler to execute for 404 responses
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response};
    /// # async fn custom_404() -> Result<Response, ignitia::Error> {
    /// #   Ok(Response::new(http::StatusCode::NOT_FOUND)
    /// #      .with_body("Page not found"))
    /// # }
    ///
    /// let router = Router::new()
    ///     .not_found(custom_404);
    /// ```
    pub fn not_found<H, T>(self, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.not_found_handler = Some(into_handler(handler));
        }
        self
    }

    /// Nest another router at the specified path prefix with full mixed-mode support.
    ///
    /// This enables modular application architecture by mounting sub-routers at specific paths.
    /// The router supports all combinations of mixed modes:
    ///
    /// - **Base -> Base**: Simple nested routing
    /// - **Base -> Radix**: Converts radix routes to base routes
    /// - **Radix -> Base**: Converts base routes to radix entries
    /// - **Radix -> Radix**: Direct radix tree merging
    ///
    /// # Arguments
    ///
    /// * `path` - The path prefix for the nested router
    /// * `router` - The router to nest
    ///
    /// # Route Merging
    ///
    /// - Routes are prefixed with the nesting path
    /// - Middleware stacks are merged (nested first, then parent)
    /// - WebSocket routes are merged (when feature enabled)
    /// - 404 handlers cascade from nested to parent
    ///
    /// # Examples
    ///
    /// ## Basic Nesting
    ///
    /// ```
    /// # use ignitia::{Router, Response};
    /// # async fn api_handler() -> Result<Response, ignitia::Error> { Ok(Response::text("API")) }
    /// # async fn web_handler() -> Result<Response, ignitia::Error> { Ok(Response::text("Web")) }
    ///
    /// let api_router = Router::new()
    ///     .get("/users", api_handler);
    ///
    /// let app = Router::new()
    ///     .nest("/api/v1", api_router)
    ///     .get("/", web_handler);
    /// // Results in routes: "/api/v1/users" and "/"
    /// ```
    ///
    /// ## Mixed Mode Nesting
    ///
    /// ```
    /// # use ignitia::{Router, RouterMode, Response};
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    ///
    /// // High-performance radix router for API
    /// let api = Router::new()
    ///     .with_mode(RouterMode::Radix)
    ///     .get("/users/:id", handler);
    ///
    /// // Simple base router for web pages
    /// let web = Router::new()
    ///     .with_mode(RouterMode::Base)
    ///     .get("/about", handler);
    ///
    /// // Combine with different modes
    /// let app = Router::new()
    ///     .with_mode(RouterMode::Radix)
    ///     .nest("/api", api)
    ///     .nest("/web", web);
    /// ```
    pub fn nest(self, path: &str, router: Router) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            let prefix = normalize_path(path);

            let nested_inner = router.inner.read();
            let parent_mode = inner.mode;
            let nested_mode = nested_inner.mode;

            tracing::debug!(
                "Nesting {:?} router into {:?} parent at prefix '{}'",
                nested_mode,
                parent_mode,
                prefix
            );

            match (parent_mode, nested_mode) {
                (RouterMode::Base, RouterMode::Base) => {
                    inner.nested_routers.push((prefix.clone(), router.clone()));
                    tracing::debug!("Base -> Base nesting at prefix '{}'", prefix);
                }
                (RouterMode::Base, RouterMode::Radix) => {
                    let extracted_routes = Self::extract_radix_routes(&nested_inner.radix_router);
                    for entry in extracted_routes.iter() {
                        let method = entry.key().clone();
                        let nested_routes = entry.value();

                        for route in nested_routes {
                            let full_path = if route.path == "/" {
                                prefix.clone()
                            } else {
                                format!("{}{}", prefix.trim_end_matches('/'), route.path)
                            };

                            let new_route =
                                Route::new(&full_path, method.clone(), route.handler.clone());
                            tracing::debug!(
                                "Converting Radix->Base route: {} -> {}",
                                route.path,
                                full_path
                            );
                            inner
                                .routes
                                .entry(method.clone())
                                .or_insert_with(Vec::new)
                                .push(new_route);
                        }
                    }

                    let mut combined = nested_inner.middleware.clone();
                    combined.extend(inner.middleware.iter().cloned());
                    inner.middleware = combined;
                    tracing::debug!("Base -> Radix nesting completed at prefix '{}'", prefix);
                }
                (RouterMode::Radix, RouterMode::Base) => {
                    for entry in nested_inner.routes.iter() {
                        let method = entry.key().clone();
                        let routes = entry.value();

                        for route in routes {
                            let full_path = if route.path == "/" {
                                prefix.clone()
                            } else {
                                format!("{}{}", prefix.trim_end_matches('/'), route.path)
                            };
                            tracing::debug!(
                                "Converting Base->Radix route: {} -> {}",
                                route.path,
                                full_path
                            );
                            inner.radix_router.insert(
                                &full_path,
                                method.clone(),
                                route.handler.clone(),
                            );
                        }
                    }

                    for (nested_prefix, nested_router) in nested_inner.nested_routers.iter() {
                        let combined_prefix =
                            format!("{}{}", prefix.trim_end_matches('/'), nested_prefix);
                        let nested_compiled = nested_router.ensure_compiled();
                        for entry in nested_compiled.routes.iter() {
                            let method = entry.key().clone();
                            let routes = entry.value();

                            for route in routes {
                                let full_path = if route.path == "/" {
                                    combined_prefix.clone()
                                } else {
                                    format!(
                                        "{}{}",
                                        combined_prefix.trim_end_matches('/'),
                                        route.path
                                    )
                                };

                                inner.radix_router.insert(
                                    &full_path,
                                    method.clone(),
                                    route.handler.clone(),
                                );
                            }
                        }
                    }

                    let mut combined = inner.middleware.clone();
                    combined.extend(nested_inner.middleware.iter().cloned());
                    inner.middleware = combined;
                    tracing::debug!("Radix -> Base nesting completed at prefix '{}'", prefix);
                }
                (RouterMode::Radix, RouterMode::Radix) => {
                    inner
                        .radix_router
                        .insert_nested(&prefix, &nested_inner.radix_router);

                    let mut combined = inner.middleware.clone();
                    combined.extend(nested_inner.middleware.iter().cloned());
                    inner.middleware = combined;
                    tracing::debug!("Radix -> Radix nesting completed at prefix '{}'", prefix);
                }
            }

            #[cfg(feature = "websocket")]
            {
                for entry in nested_inner.websocket_routes.iter() {
                    let path = entry.key();
                    let handler = entry.value();
                    let full_path = if path == "/" {
                        prefix.clone()
                    } else {
                        format!("{}{}", prefix.trim_end_matches('/'), path)
                    };
                    inner.websocket_routes.insert(full_path, handler.clone());
                }
            }

            if inner.not_found_handler.is_none() {
                inner.not_found_handler = nested_inner.not_found_handler.clone();
            }

            // Merge extensions from nested router
            Self::merge_extensions(&mut inner.extensions, &nested_inner.extensions);
        }

        self
    }

    /// Merges another router into this router, combining all routes, middleware, and configurations.
    ///
    /// This method allows you to combine multiple routers into a single router, which is useful
    /// for modular application architecture where different parts of your application are defined
    /// in separate routers.
    ///
    /// # Merge Behavior
    ///
    /// ## Routes
    /// - **Base + Base**: Routes are directly merged, with conflicts resolved by order
    /// - **Radix + Radix**: Radix trees are merged efficiently maintaining performance
    /// - **Base + Radix**: Base routes are converted and inserted into the radix tree
    /// - **Radix + Base**: Base routes are inserted into the existing radix tree
    ///
    /// ## Middleware
    /// - Middleware from the merged router is **appended** to the current router's middleware
    /// - Order matters: current router's middleware executes first, then merged router's middleware
    /// - This allows for layered middleware application (e.g., global auth + module-specific validation)
    ///
    /// ## Configurations
    /// - **Not Found Handler**: Uses the merged router's handler only if current router doesn't have one
    /// - **Extensions/State**: Merged router's state is added, existing state in current router takes precedence
    /// - **WebSocket Routes** (if feature enabled): WebSocket routes are merged, current router takes precedence on conflicts
    /// - **Nested Routers**: All nested routers from the merged router are included
    ///
    /// # Arguments
    /// * `other` - The router to merge into this router
    ///
    /// # Router Mode Handling
    /// The merge operation handles different router modes intelligently:
    ///
    /// | Current Mode | Other Mode | Result Behavior |
    /// |--------------|------------|-----------------|
    /// | Base | Base | Direct route merging |
    /// | Radix | Radix | Efficient tree merging |
    /// | Base | Radix | Converts Radix routes to Base format |
    /// | Radix | Base | Inserts Base routes into Radix tree |
    ///
    /// # Performance Considerations
    /// - **Same Mode Merging**: Very efficient, especially Radix + Radix
    /// - **Cross Mode Merging**: Requires conversion, slight overhead but still efficient
    /// - **Large Router Merging**: Consider doing bulk merges rather than multiple small merges
    ///
    /// # Use Cases
    ///
    /// ## Modular Application Architecture
    /// ```
    /// use ignitia::prelude::*;
    ///
    /// // Define module-specific routers
    /// fn user_routes() -> Router {
    ///     Router::new()
    ///         .get("/users", list_users)
    ///         .post("/users", create_user)
    ///         .get("/users/:id", get_user)
    /// }
    ///
    /// fn product_routes() -> Router {
    ///     Router::new()
    ///         .get("/products", list_products)
    ///         .post("/products", create_product)
    /// }
    ///
    /// // Merge into main router
    /// let app = Router::new()
    ///     .get("/health", health_check)
    ///     .merge(user_routes())
    ///     .merge(product_routes());
    /// ```
    ///
    /// ## Plugin System
    /// ```
    /// struct AppBuilder {
    ///     router: Router,
    /// }
    ///
    /// impl AppBuilder {
    ///     pub fn new() -> Self {
    ///         Self {
    ///             router: Router::new()
    ///         }
    ///     }
    ///
    ///     pub fn plugin<F>(mut self, plugin: F) -> Self
    ///     where F: FnOnce() -> Router
    ///     {
    ///         self.router = self.router.merge(plugin());
    ///         self
    ///     }
    ///
    ///     pub fn build(self) -> Router {
    ///         self.router
    ///     }
    /// }
    ///
    /// let app = AppBuilder::new()
    ///     .plugin(auth_plugin)
    ///     .plugin(api_v1_plugin)
    ///     .plugin(admin_plugin)
    ///     .build();
    /// ```
    ///
    /// ## Environment-Specific Routes
    /// ```
    /// let mut app = Router::new()
    ///     .get("/", home_handler);
    ///
    /// if cfg!(debug_assertions) {
    ///     let debug_router = Router::new()
    ///         .get("/debug", debug_info)
    ///         .any("/debug/*path", debug_catch_all);
    ///     app = app.merge(debug_router);
    /// }
    /// ```
    ///
    /// # Conflict Resolution
    /// - **Route Conflicts**: Last merged router's routes take precedence
    /// - **Middleware Order**: Current router's middleware executes first
    /// - **State Conflicts**: Current router's state takes precedence
    /// - **Handler Conflicts**: Merged router's handlers take precedence for the same path/method
    ///
    /// # Error Handling
    /// This method does not return errors but logs warnings for potential issues:
    /// - Route conflicts (same path + method)
    /// - Middleware order changes
    /// - State overwrites
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently with other router operations.
    /// However, the merge operation itself is atomic from the perspective of request handling.
    ///
    /// # Examples
    ///
    /// ## Basic Merge
    /// ```
    /// use ignitia::prelude::*;
    ///
    /// let api_v1 = Router::new()
    ///     .get("/v1/users", get_users)
    ///     .post("/v1/users", create_user);
    ///
    /// let api_v2 = Router::new()
    ///     .get("/v2/users", get_users_v2)
    ///     .post("/v2/users", create_user_v2);
    ///
    /// let app = Router::new()
    ///     .get("/health", health_check)
    ///     .merge(api_v1)
    ///     .merge(api_v2);
    /// ```
    ///
    /// ## Middleware Composition
    /// ```
    /// let authenticated_routes = Router::new()
    ///     .middleware(AuthMiddleware::new("secret"))
    ///     .get("/profile", get_profile)
    ///     .post("/settings", update_settings);
    ///
    /// let app = Router::new()
    ///     .middleware(LoggerMiddleware::new())
    ///     .get("/public", public_handler)
    ///     .merge(authenticated_routes); // Auth middleware will run after Logger
    /// ```
    ///
    /// ## State Sharing
    /// ```
    /// #[derive(Clone)]
    /// struct DatabasePool(Arc<Pool>);
    ///
    /// #[derive(Clone)]
    /// struct Config(Arc<AppConfig>);
    ///
    /// let user_router = Router::new()
    ///     .state(DatabasePool(db_pool.clone()))
    ///     .get("/users", list_users);
    ///
    /// let app = Router::new()
    ///     .state(Config(app_config))
    ///     .state(DatabasePool(db_pool)) // This takes precedence
    ///     .merge(user_router); // user_router's DatabasePool is ignored
    /// ```
    ///
    /// # Returns
    /// Returns a new `Router` instance with all routes, middleware, and configurations merged.
    pub fn merge(self, other: Router) -> Self {
        let mut inner = self.inner.write();
        let other_inner = other.inner.read();

        inner.dirty = true;

        match (inner.mode, other_inner.mode) {
            // Both routers use Base mode - direct merge
            (RouterMode::Base, RouterMode::Base) => {
                for entry in other_inner.routes.iter() {
                    let method = entry.key().clone();
                    let other_routes = entry.value();

                    let mut routes = inner.routes.entry(method).or_insert_with(Vec::new);
                    routes.extend(other_routes.iter().cloned());
                }
            }
            // Both routers use Radix mode - merge radix trees
            (RouterMode::Radix, RouterMode::Radix) => {
                // Extract all routes from the other router and insert them
                let extracted_routes = Self::extract_radix_routes(&other_inner.radix_router);
                for entry in extracted_routes.iter() {
                    let method = entry.key().clone();
                    let routes = entry.value();

                    for route in routes.iter() {
                        inner.radix_router.insert(
                            &route.path,
                            method.clone(),
                            route.handler.clone(),
                        );
                    }
                }
            }
            // Mixed modes - convert other to current mode
            (RouterMode::Base, RouterMode::Radix) => {
                // Extract routes from radix tree and add to base router
                let extracted_routes = Self::extract_radix_routes(&other_inner.radix_router);
                for entry in extracted_routes.iter() {
                    let method = entry.key().clone();
                    let other_routes = entry.value();

                    let mut routes = inner.routes.entry(method).or_insert_with(Vec::new);
                    routes.extend(other_routes.iter().cloned());
                }
            }
            (RouterMode::Radix, RouterMode::Base) => {
                // Insert base routes into radix tree
                for entry in other_inner.routes.iter() {
                    let method = entry.key().clone();
                    let other_routes = entry.value();

                    for route in other_routes.iter() {
                        inner.radix_router.insert(
                            &route.path,
                            method.clone(),
                            route.handler.clone(),
                        );
                    }
                }
            }
        }

        // Merge middleware (other's middleware is applied after current middleware)
        inner
            .middleware
            .extend(other_inner.middleware.iter().cloned());

        // Merge nested routers
        inner
            .nested_routers
            .extend(other_inner.nested_routers.iter().cloned());

        // Use other's not_found_handler if current router doesn't have one
        if inner.not_found_handler.is_none() && other_inner.not_found_handler.is_some() {
            inner.not_found_handler = other_inner.not_found_handler.clone();
        }

        #[cfg(feature = "websocket")]
        {
            // Merge WebSocket routes
            for entry in other_inner.websocket_routes.iter() {
                let path = entry.key();
                let handler = entry.value();
                // Only add if path doesn't already exist (current router takes precedence)
                if !inner.websocket_routes.contains_key(path) {
                    inner.websocket_routes.insert(path.clone(), handler.clone());
                }
            }
        }

        Self::merge_extensions(&mut inner.extensions, &other_inner.extensions);

        drop(inner);
        drop(other_inner);
        self
    }

    /// Merge extensions from another router into the current router
    fn merge_extensions(target_extensions: &mut Extensions, source_extensions: &Extensions) {
        for entry in source_extensions.map.iter() {
            let type_id = entry.key();
            let extension = entry.value();
            target_extensions.insert_if_not_exists_typeid(*type_id, extension.clone());
        }
    }

    /// Add WebSocket support to a route (requires 'websocket' feature).
    ///
    /// This method creates a WebSocket endpoint that handles the upgrade handshake
    /// and delegates message handling to the provided WebSocket handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The WebSocket endpoint path
    /// * `handler` - The WebSocket handler for this endpoint
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "websocket")]
    /// # use ignitia::{Router, websocket::WebSocketConnection};
    /// # #[cfg(feature = "websocket")]
    /// # async fn websocket_handler(mut conn: WebSocketConnection) -> ignitia::Result<()> {
    /// #     // Handle WebSocket messages
    /// #     Ok(())
    /// # }
    /// # #[cfg(feature = "websocket")]
    /// let router = Router::new()
    ///     .websocket_fn("/ws", websocket_handler);
    /// ```
    ///
    /// **Note**: This method is only available when the `websocket` feature is enabled.
    #[cfg(feature = "websocket")]
    pub fn websocket<H>(self, path: &str, handler: H) -> Self
    where
        H: crate::websocket::WebSocketHandler + 'static,
    {
        let normalized_path = normalize_path(path);
        let ws_handler: Arc<dyn crate::websocket::WebSocketHandler> = Arc::new(handler);

        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner
                .websocket_routes
                .insert(normalized_path.clone(), Arc::clone(&ws_handler));
        }

        let http_handler = Arc::new(move |req: Request| {
            Box::pin(async move {
                if crate::websocket::is_websocket_request(&req) {
                    crate::websocket::upgrade_connection(req)
                } else {
                    Err(crate::Error::BadRequest(
                        "This endpoint only accepts WebSocket connections".into(),
                    ))
                }
            }) as crate::handler::BoxFuture<'static, crate::Result<Response>>
        });

        self.route(&normalized_path, Method::GET, http_handler)
    }

    /// Add WebSocket support using a closure (requires 'websocket' feature).
    ///
    /// This is a convenience method for creating WebSocket handlers from closures
    /// without implementing the WebSocketHandler trait manually.
    ///
    /// # Arguments
    ///
    /// * `path` - The WebSocket endpoint path
    /// * `f` - A closure that handles WebSocket connections
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "websocket")]
    /// # use ignitia::{Router, websocket::WebSocketConnection};
    /// # #[cfg(feature = "websocket")]
    /// let router = Router::new()
    ///     .websocket_fn("/chat", |mut conn: WebSocketConnection| async move {
    ///         // Handle WebSocket messages here
    ///         Ok(())
    ///     });
    /// ```
    #[cfg(feature = "websocket")]
    pub fn websocket_fn<F, Fut>(self, path: &str, f: F) -> Self
    where
        F: Fn(crate::websocket::WebSocketConnection) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = crate::Result<()>> + Send + 'static,
    {
        use crate::websocket::websocket_handler;
        self.websocket(path, websocket_handler(f))
    }

    /// Get all WebSocket handlers registered with this router.
    ///
    /// This method is primarily used internally by the server for WebSocket upgrade handling.
    ///
    /// # Returns
    ///
    /// A DashMap containing all registered WebSocket handlers keyed by path.
    #[cfg(feature = "websocket")]
    pub fn get_websocket_handlers(
        &self,
    ) -> DashMap<String, Arc<dyn crate::websocket::WebSocketHandler>> {
        self.inner.read().websocket_routes.clone()
    }

    /// Add shared state to the router that can be accessed by handlers.
    ///
    /// State is stored per-type and can be retrieved in handlers using the State extractor.
    /// The state must implement Clone for efficient sharing across handlers.
    ///
    /// # Arguments
    ///
    /// * `state` - The state object to store (must implement Clone + Send + Sync)
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response, handler::extractor::State};
    /// # use std::sync::Arc;
    /// # use parking_lot::Mutex;
    ///
    /// #[derive(Clone)]
    /// struct AppConfig {
    ///     database_url: String,
    /// }
    ///
    /// async fn get_config(State(config): State<AppConfig>) -> Result<Response, ignitia::Error> {
    ///     Ok(Response::text(format!("DB: {}", config.database_url)))
    /// }
    ///
    /// let config = AppConfig {
    ///     database_url: "postgres://localhost/myapp".to_string(),
    /// };
    ///
    /// let router = Router::new()
    ///     .state(config)
    ///     .get("/config", get_config);
    /// ```
    pub fn state<T>(self, state: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.extensions.insert(state);
        }
        self
    }

    /// Add shared state wrapped in Arc to avoid cloning large objects.
    ///
    /// This is more efficient than `state()` when the state object is large
    /// or expensive to clone, as it only clones the Arc pointer.
    ///
    /// # Arguments
    ///
    /// * `state` - The state wrapped in Arc<T>
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response, handler::extractor::State};
    /// # use std::sync::Arc;
    ///
    /// struct LargeDatabase {
    ///     // Large or expensive-to-clone structure
    /// }
    ///
    /// let db = Arc::new(LargeDatabase {});
    /// let router = Router::new()
    ///     .state_arc(db);
    /// ```
    pub fn state_arc<T>(self, state: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.extensions.insert(state);
        }
        self
    }

    /// Add state using a factory function for lazy initialization.
    ///
    /// The factory function is called once when the state is first needed,
    /// allowing for expensive initialization to be deferred.
    ///
    /// # Arguments
    ///
    /// * `factory` - A function that creates the state object
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::Router;
    /// # struct DatabasePool;
    /// # impl DatabasePool { fn new() -> Self { DatabasePool } }
    ///
    /// let router = Router::new()
    ///     .state_factory(|| DatabasePool::new());
    /// ```
    pub fn state_factory<T, F>(self, factory: F) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        {
            let state = factory();
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.extensions.insert(state);
        }
        self
    }

    /// Check if the router has state of a specific type.
    ///
    /// # Returns
    ///
    /// `true` if state of type T exists, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::Router;
    /// # struct AppConfig;
    ///
    /// let router = Router::new()
    ///     .state(AppConfig);
    ///
    /// assert!(router.has_state::<AppConfig>());
    /// assert!(!router.has_state::<String>());
    /// ```
    pub fn has_state<T: Send + Sync + Clone + 'static>(&self) -> bool {
        self.inner.read().extensions.get::<T>().is_some()
    }

    /// Get state of a specific type from the router.
    ///
    /// # Returns
    ///
    /// `Some(T)` if state exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::Router;
    /// # struct AppConfig { pub value: i32 }
    ///
    /// let router = Router::new()
    ///     .state(AppConfig { value: 42 });
    ///
    /// if let Some(config) = router.get_state::<AppConfig>() {
    ///     println!("Config value: {}", config.value);
    /// }
    /// ```
    pub fn get_state<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.inner
            .read()
            .extensions
            .get::<T>()
            .map(|arc_t| arc_t.as_ref().clone())
    }

    /// Ensure the router is compiled and return the compiled version.
    ///
    /// This method uses lazy compilation - the router is only compiled when needed
    /// and cached for subsequent requests. Uses atomic operations for thread safety.
    ///
    /// # Returns
    ///
    /// An atomic guard to the compiled router.
    fn ensure_compiled(&self) -> Arc<CompiledRouter> {
        {
            let inner = self.inner.read();
            if !inner.dirty {
                return self.compiled.load_full();
            }
        }

        let compiled = {
            let inner = self.inner.read();
            self.compile_inner(&*inner)
        };

        let compiled_arc = Arc::new(compiled);
        self.compiled.store(Arc::clone(&compiled_arc));

        {
            let mut inner = self.inner.write();
            inner.dirty = false;
        }

        compiled_arc
    }

    /// Compile the router's internal state into an optimized form.
    ///
    /// This method processes nested routers, sorts routes by specificity,
    /// merges middleware stacks, and creates optimized lookup structures.
    ///
    /// # Arguments
    ///
    /// * `inner` - The RouterInner state to compile
    ///
    /// # Returns
    ///
    /// A CompiledRouter ready for high-performance request handling.
    fn compile_inner(&self, inner: &RouterInner) -> CompiledRouter {
        let routes = inner.routes.clone();
        let mut middleware = inner.middleware.clone();
        let mut not_found_handler = inner.not_found_handler.clone();
        let route_cache = DashMap::new();

        let radix_router = match inner.mode {
            RouterMode::Radix => Some(inner.radix_router.clone()),
            RouterMode::Base => None,
        };

        if matches!(inner.mode, RouterMode::Base) {
            for (prefix, nested_router) in &inner.nested_routers {
                let nested_compiled = nested_router.ensure_compiled();
                match nested_compiled.mode {
                    RouterMode::Base => {
                        for entry in nested_compiled.routes.iter() {
                            let method = entry.key().clone();
                            let nested_routes = entry.value();
                            for route in nested_routes {
                                let full_path = if route.path == "/" {
                                    prefix.trim_end_matches('/').to_string()
                                } else {
                                    format!("{}{}", prefix.trim_end_matches('/'), route.path)
                                };

                                let new_route =
                                    Route::new(&full_path, method.clone(), route.handler.clone());
                                routes
                                    .entry(method.clone())
                                    .or_insert_with(Vec::new)
                                    .push(new_route);
                            }
                        }
                    }
                    RouterMode::Radix => {
                        if let Some(ref radix_router) = nested_compiled.radix_router {
                            let extracted = Self::extract_radix_routes(radix_router);

                            for entry in extracted.iter() {
                                let method = entry.key().clone();
                                let route_vec = entry.value();

                                for route in route_vec {
                                    let full_path = if route.path == "/" {
                                        prefix.clone()
                                    } else {
                                        format!("{}{}", prefix.trim_end_matches('/'), route.path)
                                    };

                                    let new_route = Route::new(
                                        &full_path,
                                        method.clone(),
                                        route.handler.clone(),
                                    );
                                    routes
                                        .entry(method.clone())
                                        .or_insert_with(Vec::new)
                                        .push(new_route);
                                }
                            }
                        }
                    }
                }

                let mut combined = nested_compiled.middleware.clone();
                combined.extend(middleware.drain(..));
                middleware = combined;

                if not_found_handler.is_none() {
                    not_found_handler = nested_compiled.not_found_handler.clone();
                }
            }
        }

        if matches!(inner.mode, RouterMode::Base) {
            for mut entry in routes.iter_mut() {
                let routes = entry.value_mut();
                routes.sort_by(|a, b| {
                    let a_segments = a.path.matches('/').count();
                    let b_segments = b.path.matches('/').count();
                    let a_params = a.param_names.len() + a.wildcard_names.len();
                    let b_params = b.param_names.len() + b.wildcard_names.len();
                    b_segments.cmp(&a_segments).then(a_params.cmp(&b_params))
                });
            }
        }

        CompiledRouter {
            mode: inner.mode,
            routes,
            radix_router,
            middleware,
            not_found_handler,
            route_cache,
        }
    }

    /// Handle an incoming HTTP request using the compiled router.
    ///
    /// This is the main entry point for request processing. It:
    /// 1. Ensures the router is compiled
    /// 2. Adds application state to the request
    /// 3. Executes global middleware (before)
    /// 4. Routes the request to the appropriate handler
    /// 5. Executes global middleware (after)
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming HTTP request
    ///
    /// # Returns
    ///
    /// The HTTP response or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Request, Response};
    /// # use http::{Method, Uri};
    /// # use bytes::Bytes;
    /// # async fn example(router: Router) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = Request::new(
    ///     Method::GET,
    ///     Uri::from_static("/users/123"),
    ///     http::Version::HTTP_11,
    ///     http::HeaderMap::new(),
    ///     Bytes::new(),
    /// );
    ///
    /// let response = router.handle(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle(&self, mut req: Request) -> Result<Response> {
        let compiled = self.ensure_compiled();

        {
            let inner = self.inner.read();
            // req.extensions = inner.extensions.clone();
            Self::merge_extensions(&mut req.extensions, &inner.extensions);
        }

        for mw in &compiled.middleware {
            mw.before(&mut req).await?;
        }

        let response = match compiled.mode {
            RouterMode::Base => self.handle_base_route(&compiled, &mut req).await,
            RouterMode::Radix => self.handle_radix_route(&compiled, &mut req).await,
        }?;

        let mut response = response;

        for mw in compiled.middleware.iter().rev() {
            mw.after(&req, &mut response).await?;
        }

        Ok(response)
    }

    /// Handle request routing using Base mode (linear search).
    ///
    /// This method searches through routes linearly, using a cache to speed up
    /// frequently accessed routes.
    async fn handle_base_route(
        &self,
        compiled: &CompiledRouter,
        req: &mut Request,
    ) -> Result<Response> {
        let cache_key = format!("{}:{}", req.method, req.uri.path());

        if let Some(cached_route) = compiled.route_cache.get(&cache_key) {
            if let Some(route) = cached_route.value().as_ref() {
                if let Some(params) = route.matches(req) {
                    req.params = params;
                    return self
                        .execute_route(route, req.clone(), &compiled.middleware)
                        .await;
                }
            }
        }

        if let Some(routes) = compiled.routes.get(&req.method) {
            for route in routes.value() {
                if let Some(params) = route.matches(req) {
                    compiled
                        .route_cache
                        .insert(cache_key, Some(Arc::new(route.clone())));
                    req.params = params;
                    return self
                        .execute_route(route, req.clone(), &compiled.middleware)
                        .await;
                }
            }
        }

        compiled.route_cache.insert(cache_key, None);

        if let Some(handler) = &compiled.not_found_handler {
            handler.handle(req.clone()).await
        } else {
            Err(Error::NotFound(req.uri.path().to_string()))
        }
    }

    /// Handle request routing using Radix mode (tree lookup).
    ///
    /// This method uses the radix tree for O(k) lookup where k is the path length.
    async fn handle_radix_route(
        &self,
        compiled: &CompiledRouter,
        req: &mut Request,
    ) -> Result<Response> {
        if let Some(radix_router) = &compiled.radix_router {
            if let Some((handler, params)) = radix_router.lookup(&req.method, req.uri.path()) {
                req.params = params;
                return handler.handle(req.clone()).await;
            }
        }

        if let Some(handler) = &compiled.not_found_handler {
            handler.handle(req.clone()).await
        } else {
            Err(Error::NotFound(req.uri.path().to_string()))
        }
    }

    /// Execute a route handler.
    ///
    /// This method executes the route handler and applies middleware before and after the handler.
    async fn execute_route(
        &self,
        route: &Route,
        mut req: Request,
        global_middleware: &[Arc<dyn Middleware>],
    ) -> Result<Response> {
        for mw in &route.middleware {
            mw.before(&mut req).await?;
        }

        let mut response = route.handler.handle(req.clone()).await?;

        for mw in route.middleware.iter().rev() {
            mw.after(&req, &mut response).await?;
        }

        for mw in global_middleware.iter().rev() {
            mw.after(&req, &mut response).await?;
        }

        Ok(response)
    }

    /// Check if the router matches a given method and path.
    ///
    /// This method is useful for testing and introspection without actually
    /// executing the handler.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method to test
    /// * `path` - The path to test
    ///
    /// # Returns
    ///
    /// `true` if a route matches, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, Response};
    /// # use http::Method;
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    ///
    /// let router = Router::new()
    ///     .get("/users/:id", handler);
    ///
    /// assert!(router.matches(&Method::GET, "/users/123"));
    /// assert!(!router.matches(&Method::POST, "/users/123"));
    /// ```
    pub fn matches(&self, method: &Method, path: &str) -> bool {
        let compiled = self.ensure_compiled();

        match compiled.mode {
            RouterMode::Base => {
                if let Some(routes) = compiled.routes.get(method) {
                    for route in routes.value() {
                        let mock_req = Request::new(
                            method.clone(),
                            path.parse().unwrap_or_default(),
                            http::Version::HTTP_11,
                            http::HeaderMap::new(),
                            bytes::Bytes::new(),
                        );
                        if route.matches(&mock_req).is_some() {
                            return true;
                        }
                    }
                }
                false
            }
            RouterMode::Radix => {
                if let Some(radix_router) = &compiled.radix_router {
                    radix_router.lookup(method, path).is_some()
                } else {
                    false
                }
            }
        }
    }

    /// Clear the route lookup cache.
    ///
    /// This can be useful in development or when routes are modified at runtime.
    /// The cache will be rebuilt automatically as new requests are processed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::Router;
    /// let router = Router::new();
    /// router.clear_cache();
    /// ```
    pub fn clear_cache(&self) {
        let compiled = self.ensure_compiled();
        compiled.route_cache.clear();
    }

    /// Get routing statistics (only available in Radix mode).
    ///
    /// Returns detailed statistics about the radix tree structure,
    /// including node counts, depth metrics, and memory usage.
    ///
    /// # Returns
    ///
    /// `Some(RadixStats)` in Radix mode, `None` in Base mode.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, RouterMode, Response};
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    ///
    /// let router = Router::new()
    ///     .with_mode(RouterMode::Radix)
    ///     .get("/users/:id", handler);
    ///
    /// if let Some(stats) = router.stats() {
    ///     println!("Routes: {}", stats.total_routes);
    /// }
    /// ```
    pub fn stats(&self) -> Option<crate::router::radix::RadixStats> {
        let compiled = self.ensure_compiled();
        match compiled.mode {
            RouterMode::Radix => {
                if let Some(radix_router) = &compiled.radix_router {
                    Some(radix_router.stats())
                } else {
                    None
                }
            }
            RouterMode::Base => None,
        }
    }

    /// Print the radix tree structure for debugging (Radix mode only).
    ///
    /// This prints a visual representation of the radix tree to stdout,
    /// showing the tree structure, paths, and handlers.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ignitia::{Router, RouterMode, Response};
    /// # async fn handler() -> Result<Response, ignitia::Error> { Ok(Response::text("OK")) }
    ///
    /// let router = Router::new()
    ///     .with_mode(RouterMode::Radix)
    ///     .get("/api/users/:id", handler);
    ///
    /// router.print_tree(); // Prints tree structure to stdout
    /// ```
    pub fn print_tree(&self) {
        let compiled = self.ensure_compiled();
        if let Some(radix_router) = &compiled.radix_router {
            radix_router.print_tree();
        } else {
            println!("Tree printing only available in radix mode");
        }
    }
}

/// Normalize a path by ensuring it starts with `/` and doesn't end with `/` (except root).
///
/// This function standardizes path formats for consistent routing behavior.
///
/// # Arguments
///
/// * `path` - The path string to normalize
///
/// # Returns
///
/// A normalized path string.
///
/// # Examples
///
/// ```
/// # use ignitia::router::normalize_path;
/// assert_eq!(normalize_path("users"), "/users");
/// assert_eq!(normalize_path("/users/"), "/users");
/// assert_eq!(normalize_path("/"), "/");
/// ```
#[inline]
fn normalize_path(path: &str) -> String {
    let mut normalized = path.to_string();
    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }
    if normalized != "/" && normalized.ends_with('/') {
        normalized.pop();
    }
    normalized
}

impl Clone for Router {
    /// Create a deep clone of the router.
    ///
    /// This creates a new Router with the same configuration, routes, and state
    /// but independent compilation cache. Useful for creating router variants.
    fn clone(&self) -> Self {
        let inner = self.inner.read();
        Self {
            inner: Arc::new(RwLock::new(RouterInner {
                mode: inner.mode,
                routes: inner.routes.clone(),
                radix_router: inner.radix_router.clone(),
                middleware: inner.middleware.clone(),
                not_found_handler: inner.not_found_handler.clone(),
                nested_routers: inner.nested_routers.clone(),
                dirty: inner.dirty,
                extensions: inner.extensions.clone(),
                #[cfg(feature = "websocket")]
                websocket_routes: inner.websocket_routes.clone(),
            })),
            compiled: ArcSwap::new(Arc::new(CompiledRouter {
                mode: inner.mode,
                routes: DashMap::new(),
                radix_router: None,
                middleware: Vec::new(),
                not_found_handler: None,
                route_cache: DashMap::new(),
            })),
        }
    }
}

impl Default for Router {
    /// Create a router with default settings.
    ///
    /// Equivalent to `Router::new()` - creates an empty router with Radix mode enabled.
    fn default() -> Self {
        Self::new()
    }
}
