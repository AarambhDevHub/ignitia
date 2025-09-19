pub mod method;
pub mod radix;
pub mod route; // New radix tree module

use crate::handler::{into_handler, IntoHandler};
use crate::middleware::Middleware;
use crate::{Error, Extensions, Handler, HandlerFn, Request, Response, Result};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use http::Method;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

pub use radix::RadixRouter;
pub use route::Route;

// Router mode configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouterMode {
    Base,  // Original regex-based router
    Radix, // New radix tree router
}

impl Default for RouterMode {
    fn default() -> Self {
        RouterMode::Radix // Default to radix for better performance
    }
}

macro_rules! define_http_method {
    ($name:ident, $method:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name<H, T>(self, path: &str, handler: H) -> Self
        where
            H: IntoHandler<T>,
        {
            self.route_with(path, $method, handler)
        }
    };
}

#[derive(Clone)]
pub struct LayeredHandler {
    /// The core handler function to execute
    handler: HandlerFn,
    /// Stack of middleware to apply to this handler
    middleware: Vec<Arc<dyn Middleware>>,
}

impl LayeredHandler {
    pub fn new<H, T>(handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        Self {
            handler: into_handler(handler),
            middleware: Vec::new(),
        }
    }

    pub fn layer<M: Middleware + 'static>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    pub fn into_handler(self) -> HandlerFn {
        let handler = self.handler;
        let middleware = self.middleware;

        Arc::new(move |mut req: Request| {
            let middleware = middleware.clone();
            let handler = handler.clone();
            Box::pin(async move {
                // Run middleware.before in order
                for mw in &middleware {
                    mw.before(&mut req).await?;
                }

                let mut res = handler.handle(req.clone()).await?;

                // Run middleware.after in reverse order
                for mw in middleware.iter().rev() {
                    mw.after(&req, &mut res).await?;
                }
                Ok(res)
            })
        })
    }
}

// Compiled router structures for both modes
#[derive(Clone)]
struct CompiledRouter {
    mode: RouterMode,
    // Base router data
    routes: DashMap<Method, Vec<Route>>,
    // Radix router data
    radix_router: Option<RadixRouter>,
    // Shared data
    middleware: Vec<Arc<dyn Middleware>>,
    not_found_handler: Option<HandlerFn>,
    route_cache: DashMap<String, Option<Arc<Route>>>,
}

pub struct Router {
    /// Internal router state protected by RwLock
    inner: Arc<RwLock<RouterInner>>,
    /// Atomically updated compiled router for fast read access
    compiled: ArcSwap<CompiledRouter>,
}

/// Internal router state that can be modified during route building.
struct RouterInner {
    mode: RouterMode,
    /// Routes organized by HTTP method (for base mode)
    routes: DashMap<Method, Vec<Route>>,
    /// Radix router (for radix mode)
    radix_router: RadixRouter,
    /// Middleware stack
    middleware: Vec<Arc<dyn Middleware>>,
    /// Custom 404 handler
    not_found_handler: Option<HandlerFn>,
    /// Nested routers with their path prefixes
    nested_routers: Vec<(String, Router)>,
    /// Flag indicating if router needs recompilation
    dirty: bool,
    /// Extensions for state management
    extensions: Extensions,
    /// WebSocket route handlers (when feature is enabled)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    websocket_routes: DashMap<String, Arc<dyn crate::websocket::WebSocketHandler>>,
}

impl Router {
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
            #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
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

    /// Set the router mode
    pub fn with_mode(self, mode: RouterMode) -> Self {
        let mut inner = self.inner.write();
        inner.mode = mode;
        inner.dirty = true;
        drop(inner);
        self
    }

    /// Get the current router mode
    pub fn mode(&self) -> RouterMode {
        let inner = self.inner.read();
        inner.mode
    }

    pub fn route(self, path: &str, method: Method, handler: HandlerFn) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            let full_path = normalize_path(path);

            match inner.mode {
                RouterMode::Base => {
                    let mut routes = inner.routes.entry(method.clone()).or_insert_with(Vec::new);
                    // Pre-compile the route for better performance
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

    pub fn route_with<H, T>(self, path: &str, method: Method, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route(path, method, into_handler(handler))
    }

    pub fn route_with_layered(self, path: &str, method: http::Method, lh: LayeredHandler) -> Self {
        self.route(path, method, lh.into_handler())
    }

    // HTTP method convenience functions
    define_http_method!(
        get,
        Method::GET,
        "Adds a GET route.\n\n# Parameters\n- `path`: The route path pattern\n- `handler`: The handler function for GET requests\n\n# Examples\n``````"
    );

    define_http_method!(
        post,
        Method::POST,
        "Adds a POST route.\n\n# Parameters\n- `path`: The route path pattern\n- `handler`: The handler function for POST requests\n\n# Examples\n``````"
    );

    define_http_method!(
        put,
        Method::PUT,
        "Adds a PUT route.\n\n# Parameters\n- `path`: The route path pattern\n- `handler`: The handler function for PUT requests"
    );

    define_http_method!(
        delete,
        Method::DELETE,
        "Adds a DELETE route.\n\n# Parameters\n- `path`: The route path pattern\n- `handler`: The handler function for DELETE requests"
    );

    define_http_method!(
        patch,
        Method::PATCH,
        "Adds a PATCH route.\n\n# Parameters\n- `path`: The route path pattern\n- `handler`: The handler function for PATCH requests"
    );

    define_http_method!(
        head,
        Method::HEAD,
        "Adds a HEAD route.\n\n# Parameters\n- `path`: The route path pattern\n- `handler`: The handler function for HEAD requests"
    );

    define_http_method!(
        options,
        Method::OPTIONS,
        "Adds an OPTIONS route.\n\n# Parameters\n- `path`: The route path pattern\n- `handler`: The handler function for OPTIONS requests"
    );

    pub fn middleware<M: Middleware + 'static>(self, middleware: M) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.middleware.push(Arc::new(middleware));
        }
        self
    }

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

    pub fn nest(self, path: &str, router: Router) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            let prefix = normalize_path(path);

            match inner.mode {
                RouterMode::Radix => {
                    // For radix mode, extract the nested router's radix tree and merge it
                    let nested_inner = router.inner.read();
                    inner
                        .radix_router
                        .insert_nested(&prefix, &nested_inner.radix_router);
                    tracing::debug!("Nested radix router at prefix: {}", prefix);
                }
                RouterMode::Base => {
                    // For base mode, use the original nested router approach
                    inner.nested_routers.push((prefix.clone(), router));
                    tracing::debug!("Nested base router at prefix: {}", prefix);
                }
            }
        }
        self
    }

    // WebSocket support (existing code)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub fn websocket<H>(self, path: &str, handler: H) -> Self
    where
        H: crate::websocket::WebSocketHandler + 'static,
    {
        let normalized_path = normalize_path(path);
        tracing::debug!("Storing WebSocket handler for path: {}", normalized_path);

        let ws_handler: Arc<dyn crate::websocket::WebSocketHandler> = Arc::new(handler);

        // Store the WebSocket handler
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner
                .websocket_routes
                .insert(normalized_path.clone(), Arc::clone(&ws_handler));
        }
        // Create a regular HTTP handler that handles WebSocket upgrades
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

    // Rest of WebSocket methods remain the same...
    #[cfg(feature = "websocket")]
    pub fn websocket_fn<F, Fut>(self, path: &str, f: F) -> Self
    where
        F: Fn(crate::websocket::WebSocketConnection) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = crate::Result<()>> + Send + 'static,
    {
        use crate::websocket::websocket_handler;
        self.websocket(path, websocket_handler(f))
    }

    #[cfg(feature = "websocket")]
    pub fn get_websocket_handlers(
        &self,
    ) -> DashMap<String, Arc<dyn crate::websocket::WebSocketHandler>> {
        let inner = self.inner.read();
        inner.websocket_routes.clone()
    }

    // State management methods
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

    pub fn has_state<T: Send + Sync + Clone + 'static>(&self) -> bool {
        let inner = self.inner.read();
        inner.extensions.get::<T>().is_some()
    }

    pub fn get_state<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let inner = self.inner.read();
        // Extensions store Arc<T>, so we get the Arc and then clone the inner value
        inner
            .extensions
            .get::<T>()
            .map(|arc_t| arc_t.as_ref().clone())
    }

    fn ensure_compiled(&self) -> Arc<CompiledRouter> {
        // Fast path: check if compilation is needed without holding the lock
        {
            let inner = self.inner.read();
            if !inner.dirty {
                return self.compiled.load_full();
            }
        }

        // Now get write lock for compilation
        let compiled = {
            let inner = self.inner.read();
            self.compile_inner(&*inner)
        };

        // Store the compiled version
        let compiled_arc = Arc::new(compiled);
        self.compiled.store(Arc::clone(&compiled_arc));

        // Mark as clean
        {
            let mut inner = self.inner.write();
            inner.dirty = false;
        }

        compiled_arc
    }

    fn compile_inner(&self, inner: &RouterInner) -> CompiledRouter {
        let mut routes = inner.routes.clone();
        let mut middleware = inner.middleware.clone();
        let mut not_found_handler = inner.not_found_handler.clone();
        let route_cache = DashMap::new();

        // Compile radix router if in radix mode
        let radix_router = match inner.mode {
            RouterMode::Radix => Some(inner.radix_router.clone()),
            RouterMode::Base => None,
        };

        // Process nested routers (only for base mode, radix handles nesting during insertion)
        if matches!(inner.mode, RouterMode::Base) {
            for (prefix, nested_router) in &inner.nested_routers {
                let nested_compiled = nested_router.ensure_compiled();

                // Merge routes with prefix for base mode
                for entry in nested_compiled.routes.iter() {
                    let method = entry.key().clone();
                    let nested_routes = entry.value();

                    for route in nested_routes {
                        let full_path = if route.path == "/" {
                            prefix.clone()
                        } else {
                            format!("{}{}", prefix, route.path)
                        };

                        let mut new_route = route.clone();
                        new_route.path = full_path.clone();
                        new_route.regex = Route::compile_regex(&full_path);

                        routes
                            .entry(method.clone())
                            .or_insert_with(Vec::new)
                            .push(new_route);
                    }
                }

                // Merge middleware (nested first)
                let mut combined = nested_compiled.middleware.clone();
                combined.extend(middleware.drain(..));
                middleware = combined;

                // Use nested not found handler if we don't have one
                if not_found_handler.is_none() {
                    not_found_handler = nested_compiled.not_found_handler.clone();
                }
            }
        }

        // Sort routes by specificity for faster matching (base mode only)
        if matches!(inner.mode, RouterMode::Base) {
            for mut entry in routes.iter_mut() {
                let routes = entry.value_mut();
                routes.sort_by(|a, b| {
                    // Sort by number of path segments (more specific first)
                    let a_segments = a.path.matches('/').count();
                    let b_segments = b.path.matches('/').count();

                    // Then by number of parameters (fewer parameters first)
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

    pub async fn handle(&self, mut req: Request) -> Result<Response> {
        let compiled = self.ensure_compiled();

        println!("middleware: {}", compiled.middleware.len());

        // Apply global middleware in order
        for mw in &compiled.middleware {
            mw.before(&mut req).await?;
        }

        let response = match compiled.mode {
            RouterMode::Base => self.handle_base_route(&compiled, &mut req).await,
            RouterMode::Radix => self.handle_radix_route(&compiled, &mut req).await,
        };

        let mut response = response?;

        // Apply global middleware after handler in reverse order
        for mw in compiled.middleware.iter().rev() {
            mw.after(&req, &mut response).await?;
        }

        Ok(response)
    }

    async fn handle_base_route(
        &self,
        compiled: &CompiledRouter,
        req: &mut Request,
    ) -> Result<Response> {
        // Generate cache key for route matching
        let cache_key = format!("{}:{}", req.method, req.uri.path());

        // Try to get cached route first
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

        // Get routes for this method only
        if let Some(routes) = compiled.routes.get(&req.method) {
            for route in routes.value() {
                if let Some(params) = route.matches(req) {
                    // Cache this successful match
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

        // Cache the miss
        compiled.route_cache.insert(cache_key, None);

        // Handle not found
        if let Some(handler) = &compiled.not_found_handler {
            handler.handle(req.clone()).await
        } else {
            Err(Error::NotFound(req.uri.path().to_string()))
        }
    }

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

        // Handle not found
        if let Some(handler) = &compiled.not_found_handler {
            handler.handle(req.clone()).await
        } else {
            Err(Error::NotFound(req.uri.path().to_string()))
        }
    }

    async fn execute_route(
        &self,
        route: &Route,
        mut req: Request,
        global_middleware: &[Arc<dyn Middleware>],
    ) -> Result<Response> {
        // Apply route middleware before handler
        for mw in &route.middleware {
            mw.before(&mut req).await?;
        }

        let mut response = route.handler.handle(req.clone()).await?;

        // Apply route middleware after handler (in reverse order)
        for mw in route.middleware.iter().rev() {
            mw.after(&req, &mut response).await?;
        }

        // Apply global middleware after handler (in reverse order)
        for mw in global_middleware.iter().rev() {
            mw.after(&req, &mut response).await?;
        }

        Ok(response)
    }

    pub fn matches(&self, method: &Method, path: &str) -> bool {
        let compiled = self.ensure_compiled();

        match compiled.mode {
            RouterMode::Base => {
                if let Some(routes) = compiled.routes.get(method) {
                    for route in routes.value() {
                        // Create a mock request for matching
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

    /// Clear the route cache (useful for testing or when routes change dynamically)
    pub fn clear_cache(&self) {
        let compiled = self.ensure_compiled();
        compiled.route_cache.clear();
    }

    /// Get router statistics (only available in radix mode)
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

    /// Print router tree structure (debug only, radix mode)
    pub fn print_tree(&self) {
        let compiled = self.ensure_compiled();
        if let Some(radix_router) = &compiled.radix_router {
            radix_router.print_tree();
        } else {
            println!("Tree printing only available in radix mode");
        }
    }
}

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

// Implement Clone for Router
impl Clone for Router {
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
                #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
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

/// Default implementation
impl Default for Router {
    /// Creates a new empty router (same as `Router::new()`).
    fn default() -> Self {
        Self::new()
    }
}
