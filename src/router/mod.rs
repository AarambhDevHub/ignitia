//! # Router Module - High-Performance Request Routing
//!
//! This module provides the core routing functionality for the Ignitia web framework.
//! It implements an efficient radix tree-based router with support for:
//!
//! - Path parameters (`{id}`, `{name}`)
//! - Wildcard routes (`*filepath`)
//! - Nested routers with prefix mounting
//! - Global and route-specific middleware
//! - HTTP method-based routing (GET, POST, PUT, DELETE, etc.)
//! - State management across routes
//! - WebSocket endpoint registration (with feature flag)
//! - Compile-time route optimization with atomic swapping
//!
//! ## Architecture
//!
//! The router uses a two-phase design:
//! 1. **Build Phase**: Routes are registered and middleware is configured
//! 2. **Compile Phase**: Routes are optimized and middleware is pre-wrapped
//!
//! This approach allows for zero-cost route matching at runtime with all
//! middleware composition done once during compilation.
//!
//! ## Performance Features
//!
//! - Lock-free compiled route access using `ArcSwap`
//! - Radix tree for O(log n) route lookups
//! - Pre-compiled middleware chains
//! - Zero-allocation route matching
//! - Efficient parameter extraction
//!
//! ## Usage Example
//!
//! ```
//! use ignitia::Router;
//!
//! let router = Router::new()
//!     .get("/users", list_users)
//!     .post("/users", create_user)
//!     .get("/users/{id}", get_user)
//!     .middleware(LoggerMiddleware::new());
//! ```

pub mod method;
pub mod radix;
pub mod route;

use crate::handler::universal_handler;
use crate::middleware::{BoxFuture, Middleware, Next};
use crate::response::IntoResponse;
use crate::{Error, Extensions, Handler, HandlerFn, Request, Response, Result};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use http::Method;
use parking_lot::RwLock;
use std::sync::Arc;

pub use radix::{RadixNode, RadixRouter};
pub use route::Route;

/// Helper macro to define HTTP method routing functions
///
/// This macro generates convenience methods for each HTTP method,
/// reducing boilerplate when defining routes.
macro_rules! define_http_method {
    ($name:ident, $method:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name<H, T>(self, path: &str, handler: H) -> Self
        where
            H: crate::handler::UniversalHandler<T>,
        {
            self.route_with(path, $method, handler)
        }
    };
}

/// A handler with attached middleware layers
///
/// This struct represents a handler function along with its middleware chain.
/// It allows building up middleware layers before converting to a final handler.
///
/// # Example
///
/// ```
/// let layered = LayeredHandler::new(my_handler)
///     .layer(AuthMiddleware::new())
///     .layer(RateLimitMiddleware::new());
/// ```
#[derive(Clone)]
pub struct LayeredHandler {
    /// The core handler function
    handler: HandlerFn,
    /// Middleware layers to apply to this handler
    middleware: Vec<Arc<dyn Middleware>>,
}

impl LayeredHandler {
    /// Create a new layered handler from any handler type
    ///
    /// # Example
    ///
    /// ```
    /// async fn handler() -> &'static str { "Hello" }
    /// let layered = LayeredHandler::new(handler);
    /// ```
    pub fn new<H, T>(handler: H) -> Self
    where
        H: crate::handler::UniversalHandler<T>,
    {
        Self {
            handler: universal_handler(handler),
            middleware: Vec::new(),
        }
    }

    /// Add a middleware layer to this handler
    ///
    /// Middleware is applied in the order it's added.
    ///
    /// # Example
    ///
    /// ```
    /// layered
    ///     .layer(LoggerMiddleware::new())
    ///     .layer(AuthMiddleware::new());
    /// ```
    pub fn layer<M: Middleware + 'static>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    /// Convert this layered handler into a final HandlerFn
    ///
    /// This wraps the handler with all its middleware layers.
    pub fn into_handler(self) -> HandlerFn {
        wrap_handler_with_middleware(self.handler, self.middleware)
    }
}

/// Compiled router state for lock-free access
///
/// This struct represents the optimized, ready-to-use router state.
/// All middleware is pre-wrapped and routes are optimized for fast lookups.
#[derive(Clone)]
struct CompiledRouter {
    /// The radix tree with optimized routes
    radix_router: RadixRouter,
    /// Global middleware (empty after compilation)
    _middleware: Vec<Arc<dyn Middleware>>,
    /// Handler for 404 Not Found responses
    not_found_handler: Option<HandlerFn>,
}

/// High-performance HTTP router with radix tree routing
///
/// The `Router` is the core routing component of Ignitia. It uses a radix tree
/// for efficient path matching and supports advanced features like nested routers,
/// middleware composition, and state management.
///
/// ## Features
///
/// - **Radix Tree Routing**: O(log n) route lookups
/// - **Path Parameters**: Extract dynamic segments from URLs
/// - **Wildcard Routes**: Match arbitrary path suffixes
/// - **Nested Routers**: Compose routers with path prefixes
/// - **Middleware**: Apply middleware globally or per-route
/// - **State Management**: Share state across handlers
/// - **Atomic Compilation**: Lock-free route access after compilation
///
/// ## Example
///
/// ```
/// let router = Router::new()
///     .get("/", index_handler)
///     .get("/users/{id}", get_user)
///     .post("/users", create_user)
///     .middleware(LoggerMiddleware::new())
///     .state(db_pool);
/// ```
pub struct Router {
    /// Mutable router configuration (read/write locked)
    pub inner: Arc<RwLock<RouterInner>>,
    /// Compiled, optimized router state (lock-free read access)
    compiled: ArcSwap<CompiledRouter>,
}

/// Internal mutable router state
///
/// This struct holds the router configuration during the build phase.
/// It's protected by a RwLock for safe concurrent access.
pub struct RouterInner {
    /// The radix tree router
    radix_router: RadixRouter,
    /// Global middleware stack
    middleware: Vec<Arc<dyn Middleware>>,
    /// Custom 404 handler
    not_found_handler: Option<HandlerFn>,
    /// Nested routers for composition
    nested_routers: Vec<(String, Router)>,
    /// Dirty flag to trigger recompilation
    dirty: bool,
    /// Shared state extensions
    pub extensions: Extensions,

    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    /// WebSocket endpoint handlers
    websocket_routes: DashMap<String, Arc<dyn crate::websocket::WebSocketHandler>>,
}
impl Router {
    /// Create a new empty router
    ///
    /// # Example
    ///
    /// ```
    /// let router = Router::new();
    /// ```
    pub fn new() -> Self {
        let inner = RouterInner {
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
                radix_router: RadixRouter::new(),
                _middleware: Vec::new(),
                not_found_handler: None,
            })),
        }
    }

    /// Extract all routes from a radix router
    ///
    /// Used internally for router merging and inspection.
    fn extract_radix_routes(radix_router: &RadixRouter) -> DashMap<Method, Vec<Route>> {
        let routes = DashMap::new();
        Self::extract_node_routes(&radix_router.root, "", &routes);
        routes
    }

    /// Recursively extract routes from radix tree nodes
    ///
    /// Traverses the radix tree and collects all registered routes.
    fn extract_node_routes(
        node: &RadixNode,
        path_prefix: &str,
        routes: &DashMap<Method, Vec<Route>>,
    ) {
        let current_path = if path_prefix.is_empty() {
            if let Some(param_name) = &node.param_name {
                if node.is_wildcard {
                    format!("/{{*{}}}", param_name)
                } else {
                    format!("/{{{}}}", param_name)
                }
            } else {
                node.path.clone()
            }
        } else {
            if let Some(param_name) = &node.param_name {
                let param_syntax = if node.is_wildcard {
                    format!("/{{*{}}}", param_name)
                } else {
                    format!("/{{{}}}", param_name)
                };
                format!("{}{}", path_prefix.trim_end_matches('/'), param_syntax)
            } else if node.path.is_empty() {
                path_prefix.to_string()
            } else if node.path.starts_with('/') {
                format!("{}{}", path_prefix.trim_end_matches('/'), node.path)
            } else {
                format!("{}/{}", path_prefix.trim_end_matches('/'), node.path)
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

    /// Register a route with a handler
    ///
    /// This is the low-level route registration method. Most users should use
    /// the convenience methods like `get()`, `post()`, etc.
    ///
    /// # Arguments
    ///
    /// * `path` - The URL path pattern (e.g., "/users/{id}")
    /// * `method` - The HTTP method
    /// * `handler` - The handler function
    ///
    /// # Example
    ///
    /// ```
    /// router.route("/api/users", Method::GET, list_users);
    /// ```
    pub fn route(self, path: &str, method: Method, handler: HandlerFn) -> Self {
        let full_path = normalize_path(path);
        let mut inner = self.inner.write();
        inner.dirty = true;
        inner.radix_router.insert(&full_path, method, handler);
        drop(inner);
        self
    }

    /// Register a route with a universal handler
    ///
    /// Accepts any handler that implements `UniversalHandler<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// router.route_with("/users", Method::GET, list_users_handler);
    /// ```
    pub fn route_with<H, T>(self, path: &str, method: Method, handler: H) -> Self
    where
        H: crate::handler::UniversalHandler<T>,
    {
        self.route(path, method, universal_handler(handler))
    }

    /// Register a route with a layered handler
    ///
    /// Allows attaching route-specific middleware.
    ///
    /// # Example
    ///
    /// ```
    /// let layered = LayeredHandler::new(handler)
    ///     .layer(AuthMiddleware::new());
    /// router.route_with_layered("/admin", Method::GET, layered);
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

    /// Register a handler for all HTTP methods
    ///
    /// Convenient for routes that handle multiple methods the same way.
    ///
    /// # Example
    ///
    /// ```
    /// router.any("/health", health_check);
    /// ```
    pub fn any<H, T>(self, path: &str, handler: H) -> Self
    where
        H: crate::handler::UniversalHandler<T>,
    {
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

    /// Add global middleware to the router
    ///
    /// Global middleware applies to all routes registered on this router.
    /// Middleware is applied in the order it's added.
    ///
    /// # Example
    ///
    /// ```
    /// router
    ///     .middleware(LoggerMiddleware::new())
    ///     .middleware(CorsMiddleware::new());
    /// ```
    pub fn middleware<M: Middleware + 'static>(self, middleware: M) -> Self {
        let mut inner = self.inner.write();
        inner.dirty = true;
        inner.middleware.push(Arc::new(middleware));
        drop(inner);
        self
    }

    /// Set a custom 404 Not Found handler
    ///
    /// By default, returns a simple "Not Found" response.
    ///
    /// # Example
    ///
    /// ```
    /// router.not_found(custom_404_handler);
    /// ```
    pub fn not_found<H, T>(self, handler: H) -> Self
    where
        H: crate::handler::UniversalHandler<T>,
    {
        let mut inner = self.inner.write();
        inner.dirty = true;
        inner.not_found_handler = Some(universal_handler(handler));
        drop(inner);
        self
    }

    /// Nest a router under a path prefix
    ///
    /// All routes from the nested router are mounted under the specified prefix.
    /// The nested router's middleware is preserved.
    ///
    /// # Example
    ///
    /// ```
    /// let api_router = Router::new()
    ///     .get("/users", list_users)
    ///     .post("/users", create_user);
    ///
    /// router.nest("/api", api_router);
    /// // Now accessible at /api/users
    /// ```
    pub fn nest(self, path: &str, router: Router) -> Self {
        let prefix = normalize_path(path);
        let mut inner = self.inner.write();
        inner.dirty = true;

        let nested_inner = router.inner.read();

        // Take nested radix tree, wrap its handlers with nested middleware, and insert under prefix
        let mut wrapped_root = nested_inner.radix_router.root.clone();
        wrap_tree_handlers(&mut wrapped_root, nested_inner.middleware.clone());
        inner
            .radix_router
            .insert_nested(&prefix, &RadixRouter { root: wrapped_root });

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

        Self::merge_extensions(&mut inner.extensions, &nested_inner.extensions);
        drop(nested_inner);
        drop(inner);
        self
    }

    /// Merge another router into this one
    ///
    /// Combines routes, middleware, and state from both routers.
    /// Useful for modular application composition.
    ///
    /// # Example
    ///
    /// ```
    /// router
    ///     .merge(user_routes)
    ///     .merge(product_routes);
    /// ```
    pub fn merge(self, other: Router) -> Self {
        let mut inner = self.inner.write();
        let other_inner = other.inner.read();

        inner.dirty = true;

        // Merge other radix routes into this router
        let extracted_routes = Self::extract_radix_routes(&other_inner.radix_router);
        for entry in extracted_routes.iter() {
            let method = entry.key().clone();
            let routes = entry.value();
            for route in routes.iter() {
                inner
                    .radix_router
                    .insert(&route.path, method.clone(), route.handler.clone());
            }
        }

        inner
            .middleware
            .extend(other_inner.middleware.iter().cloned());
        inner
            .nested_routers
            .extend(other_inner.nested_routers.iter().cloned());

        if inner.not_found_handler.is_none() && other_inner.not_found_handler.is_some() {
            inner.not_found_handler = other_inner.not_found_handler.clone();
        }

        #[cfg(feature = "websocket")]
        {
            for entry in other_inner.websocket_routes.iter() {
                let path = entry.key();
                let handler = entry.value();
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

    /// Merge extensions from source into target
    ///
    /// Only inserts extensions that don't already exist in target.
    fn merge_extensions(target_extensions: &mut Extensions, source_extensions: &Extensions) {
        for entry in source_extensions.map.iter() {
            let type_id = entry.key();
            let extension = entry.value();
            target_extensions.insert_if_not_exists_typeid(*type_id, extension.clone());
        }
    }

    /// Register a WebSocket endpoint
    ///
    /// WebSocket handlers implement the `WebSocketHandler` trait.
    ///
    /// # Example
    ///
    /// ```
    /// router.websocket("/ws", MyWebSocketHandler::new());
    /// ```
    #[cfg(feature = "websocket")]
    pub fn websocket<H, T>(self, path: &str, handler: H) -> Self
    where
        H: crate::websocket::UniversalWebSocketHandler<T>,
        T: Send + Sync + 'static,
    {
        let normalized_path = normalize_path(path);
        let ws_handler = crate::websocket::universal_ws_handler(handler);

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
                    Ok(crate::websocket::upgrade_connection(&req)?)
                } else {
                    Ok(Response::bad_request("WebSocket upgrade required"))
                }
            }) as crate::handler::BoxFuture<'static, crate::Result<Response>>
        });

        self.route(&normalized_path, Method::GET, http_handler)
    }

    /// Register a WebSocket endpoint with a closure
    ///
    /// Convenience method for simple WebSocket handlers.
    ///
    /// # Example
    ///
    /// ```
    /// router.websocket_fn("/chat", |conn| async move {
    ///     // Handle WebSocket connection
    ///     Ok(())
    /// });
    /// ```
    #[cfg(feature = "websocket")]
    pub fn websocket_fn<F, Fut, R>(self, path: &str, f: F) -> Self
    where
        F: Fn(crate::websocket::WebSocketConnection) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
        R: crate::response::IntoResponse,
    {
        use crate::websocket::websocket_handler;
        self.websocket(path, websocket_handler(f))
    }

    /// Get all registered WebSocket handlers
    ///
    /// Used internally by the server to handle WebSocket upgrades.
    #[cfg(feature = "websocket")]
    pub fn get_websocket_handlers(
        &self,
    ) -> DashMap<String, Arc<dyn crate::websocket::WebSocketHandler>> {
        self.inner.read().websocket_routes.clone()
    }

    /// Add shared state to the router
    ///
    /// State can be extracted in handlers using the `State` extractor.
    ///
    /// # Example
    ///
    /// ```
    /// let db_pool = create_db_pool();
    /// router.state(db_pool);
    /// ```
    pub fn state<T>(self, state: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        let mut inner = self.inner.write();
        inner.dirty = true;
        inner.extensions.insert(state);
        drop(inner);
        self
    }

    /// Add Arc-wrapped state to the router
    ///
    /// For state that's already in an Arc, avoiding double-wrapping.
    ///
    /// # Example
    ///
    /// ```
    /// let db = Arc::new(Database::new());
    /// router.state_arc(db);
    /// ```
    pub fn state_arc<T>(self, state: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        let mut inner = self.inner.write();
        inner.dirty = true;
        inner.extensions.insert(state);
        drop(inner);
        self
    }

    /// Add state created by a factory function
    ///
    /// The factory is called once when the state is registered.
    ///
    /// # Example
    ///
    /// ```
    /// router.state_factory(|| Database::connect());
    /// ```
    pub fn state_factory<T, F>(self, factory: F) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let state = factory();
        let mut inner = self.inner.write();
        inner.dirty = true;
        inner.extensions.insert(state);
        drop(inner);
        self
    }

    /// Check if state of type T exists
    ///
    /// # Example
    ///
    /// ```
    /// if router.has_state::<Database>() {
    ///     // Database is available
    /// }
    /// ```
    pub fn has_state<T: Send + Sync + Clone + 'static>(&self) -> bool {
        self.inner.read().extensions.get::<T>().is_some()
    }

    /// Get a clone of the state if it exists
    ///
    /// Returns None if the state hasn't been registered.
    ///
    /// # Example
    ///
    /// ```
    /// if let Some(db) = router.get_state::<Database>() {
    ///     // Use database
    /// }
    /// ```
    pub fn get_state<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.inner
            .read()
            .extensions
            .get::<T>()
            .map(|arc_t| arc_t.as_ref().clone())
    }

    /// Ensure the router is compiled
    ///
    /// Returns the compiled router, recompiling if necessary.
    /// Uses atomic swap for lock-free access after compilation.
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

    /// Compile the router into optimized state
    ///
    /// Pre-wraps all handlers with middleware for zero-cost runtime execution.
    fn compile_inner(&self, inner: &RouterInner) -> CompiledRouter {
        let mut middleware = inner.middleware.clone();
        let mut not_found_handler = inner.not_found_handler.clone();

        // Start with a copy of the radix tree
        let mut radix_router = inner.radix_router.clone();

        // Wrap nested routers captured earlier are already merged into inner.radix_router
        // Pre-wrap global middleware into all handlers once at compile time, then clear it
        if !middleware.is_empty() {
            // Wrap not_found handler
            if let Some(h) = &mut not_found_handler {
                let wrapped = wrap_handler_with_middleware(h.clone(), middleware.clone());
                not_found_handler = Some(wrapped);
            }

            // Wrap the whole tree
            let mut root = radix_router.root.clone();
            wrap_tree_handlers(&mut root, middleware.clone());
            radix_router = RadixRouter { root };

            // Clear middleware for runtime fast path
            middleware.clear();
        }

        CompiledRouter {
            radix_router,
            _middleware: middleware,
            not_found_handler,
        }
    }

    /// Handle an incoming HTTP request
    ///
    /// This is the main entry point for request processing.
    /// Routes are looked up in the compiled radix tree and handlers are executed.
    ///
    /// # Performance
    ///
    /// - Lock-free route lookup using ArcSwap
    /// - O(log n) path matching with radix tree
    /// - Zero allocation for route matching
    /// - Pre-compiled middleware chains
    #[inline]
    pub async fn handle(&self, req: Request) -> Result<Response> {
        let compiled = self.ensure_compiled();
        let mut req = req;
        {
            let inner = self.inner.read();
            Self::merge_extensions(&mut req.extensions, &inner.extensions);
        }
        self.handle_radix_route(&compiled, req).await
    }

    /// Handle a request using radix tree routing
    ///
    /// Performs path matching and parameter extraction.
    async fn handle_radix_route(
        &self,
        compiled: &CompiledRouter,
        req: Request,
    ) -> Result<Response> {
        if let Some((handler, params)) = compiled.radix_router.lookup(&req.method, req.uri.path()) {
            let mut req = req;
            req.params = params;
            return handler.handle(req).await;
        }

        if let Some(handler) = &compiled.not_found_handler {
            handler.handle(req.clone()).await
        } else {
            Err(Error::NotFound(req.uri.path().to_string()))
        }
    }

    /// Check if a route exists for the given method and path
    ///
    /// Useful for OPTIONS requests or route introspection.
    ///
    /// # Example
    ///
    /// ```
    /// if router.matches(&Method::GET, "/users") {
    ///     // Route exists
    /// }
    /// ```
    pub fn matches(&self, method: &Method, path: &str) -> bool {
        let compiled = self.ensure_compiled();
        compiled.radix_router.lookup(method, path).is_some()
    }

    /// Get routing statistics
    ///
    /// Returns information about the radix tree structure.
    ///
    /// # Example
    ///
    /// ```
    /// if let Some(stats) = router.stats() {
    ///     println!("Total routes: {}", stats.handler_count);
    /// }
    pub fn stats(&self) -> Option<crate::router::radix::RadixStats> {
        let compiled = self.ensure_compiled();
        Some(compiled.radix_router.stats())
    }

    /// Print the routing tree for debugging
    ///
    /// Outputs a human-readable representation of all registered routes.
    ///
    /// # Example
    ///
    /// ```
    /// router.print_tree();
    /// ```
    pub fn print_tree(&self) {
        let compiled = self.ensure_compiled();
        compiled.radix_router.print_tree();
    }
}

/// Wrap a handler with middleware chain
///
/// Pre-composes the middleware chain once, creating a single optimized handler.
/// This is done at compile time, not on every request.
///
/// # Performance
///
/// The middleware chain is built once and reused for all requests,
/// eliminating per-request allocation and function composition overhead.
fn wrap_handler_with_middleware(
    handler: HandlerFn,
    middleware: Vec<Arc<dyn Middleware>>,
) -> HandlerFn {
    if middleware.is_empty() {
        return handler;
    }

    // Terminal handler
    let terminal = Arc::new(move |req: Request| -> BoxFuture<'static, Response> {
        let handler = handler.clone();
        Box::pin(async move {
            match handler.handle(req).await {
                Ok(resp) => resp,
                Err(err) => err.into_response(),
            }
        })
    }) as Arc<dyn Fn(Request) -> BoxFuture<'static, Response> + Send + Sync>;

    // Build chain by folding middleware in reverse
    let chain = middleware.iter().rev().fold(terminal, |next, mw| {
        let mw = mw.clone();
        Arc::new(move |req: Request| -> BoxFuture<'static, Response> {
            let mw = mw.clone();
            let next = next.clone();

            Box::pin(async move {
                let nxt = Next::new(move |r: Request| -> BoxFuture<'static, Response> { next(r) });
                // Middleware returns Response directly, no need for match
                mw.handle(req, nxt).await
            })
        }) as Arc<dyn Fn(Request) -> BoxFuture<'static, Response> + Send + Sync>
    });

    // Return final handler
    Arc::new(
        move |req: Request| -> BoxFuture<'static, crate::Result<Response>> {
            let chain = chain.clone();
            Box::pin(async move { Ok(chain(req).await) })
        },
    )
}

/// Recursively wrap all handlers in a radix tree with middleware
///
/// Applies middleware to every handler in the tree, used for global
/// middleware and nested router middleware.
fn wrap_tree_handlers(node: &mut RadixNode, middleware: Vec<Arc<dyn Middleware>>) {
    let new_handlers = DashMap::new();
    for entry in &node.handlers {
        let method = entry.key().clone();
        let handler = entry.value().clone();
        let wrapped = wrap_handler_with_middleware(handler.clone(), middleware.clone());
        new_handlers.insert(method.clone(), wrapped);
    }
    node.handlers = new_handlers;

    for child in &mut node.children {
        wrap_tree_handlers(child, middleware.clone());
    }
}

/// Normalize a path for routing
///
/// Ensures paths start with `/` and don't end with `/` (except root).
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
    /// Clone the router
    ///
    /// Creates a new router with the same configuration.
    /// The compiled state is shared using Arc.
    fn clone(&self) -> Self {
        let inner = self.inner.read();
        Self {
            inner: Arc::new(RwLock::new(RouterInner {
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
                radix_router: RadixRouter::new(),
                _middleware: Vec::new(),
                not_found_handler: None,
            })),
        }
    }
}

impl Default for Router {
    /// Create a default router
    ///
    /// Equivalent to `Router::new()`.
    fn default() -> Self {
        Self::new()
    }
}
