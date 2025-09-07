pub mod method;
pub mod route;

use crate::handler::{into_handler, IntoHandler};
use crate::middleware::{self, Middleware};
use crate::{Error, Handler, HandlerFn, Request, Response, Result};
use arc_swap::ArcSwap;
use http::Method;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub use route::Route;

// Helper macros for common HTTP methods
macro_rules! http_method {
    ($name:ident, $method:expr) => {
        pub fn $name<H, T>(self, path: &str, handler: H) -> Self
        where
            H: IntoHandler<T>,
        {
            self.route_with(path, $method, handler)
        }
    };
}

#[derive(Clone)]
struct CompiledRouter {
    routes: HashMap<Method, Vec<Route>>,
    middleware: Vec<Arc<dyn Middleware>>,
    not_found_handler: Option<HandlerFn>,
}

pub struct Router {
    inner: Arc<RwLock<RouterInner>>,
    compiled: ArcSwap<CompiledRouter>,
}

struct RouterInner {
    routes: HashMap<Method, Vec<Route>>,
    middleware: Vec<Arc<dyn Middleware>>,
    not_found_handler: Option<HandlerFn>,
    nested_routers: Vec<(String, Router)>,
    dirty: bool,
    #[cfg(feature = "websocket")]
    websocket_routes: HashMap<String, Arc<dyn crate::websocket::WebSocketHandler>>,
}

impl Router {
    pub fn new() -> Self {
        let inner = RouterInner {
            routes: HashMap::new(),
            middleware: Vec::new(),
            not_found_handler: None,
            nested_routers: Vec::new(),
            dirty: true,
            #[cfg(feature = "websocket")]
            websocket_routes: HashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
            compiled: ArcSwap::new(Arc::new(CompiledRouter {
                routes: HashMap::new(),
                middleware: Vec::new(),
                not_found_handler: None,
            })),
        }
    }

    // Return Self instead of &mut Self for builder pattern
    pub fn route(mut self, path: &str, method: Method, handler: HandlerFn) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;

            let full_path = normalize_path(path);
            let routes = inner.routes.entry(method.clone()).or_insert_with(Vec::new);

            // Pre-compile the route for better performance
            let route = Route::new(&full_path, method, handler);
            routes.push(route);
        }

        self
    }

    pub fn route_with<H, T>(self, path: &str, method: Method, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route(path, method, into_handler(handler))
    }

    http_method!(get, Method::GET);
    http_method!(post, Method::POST);
    http_method!(put, Method::PUT);
    http_method!(delete, Method::DELETE);
    http_method!(patch, Method::PATCH);
    http_method!(head, Method::HEAD);
    http_method!(options, Method::OPTIONS);

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
            inner.nested_routers.push((normalize_path(path), router));
        }
        self
    }

    #[cfg(feature = "websocket")]
    pub fn websocket<H>(mut self, path: &str, handler: H) -> Self
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
    ) -> HashMap<String, Arc<dyn crate::websocket::WebSocketHandler>> {
        let inner = self.inner.read();
        inner.websocket_routes.clone()
    }

    #[cfg(not(feature = "websocket"))]
    pub fn websocket<H>(self, _path: &str, _handler: H) -> Self {
        panic!("WebSocket support is not enabled. Add 'websocket' feature to your Cargo.toml");
    }

    #[cfg(not(feature = "websocket"))]
    pub fn websocket_fn<F>(self, _path: &str, _f: F) -> Self {
        panic!("WebSocket support is not enabled. Add 'websocket' feature to your Cargo.toml");
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
            self.compile_inner(&inner)
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

        // Process nested routers
        for (prefix, nested_router) in &inner.nested_routers {
            let nested_compiled = nested_router.ensure_compiled();

            // Merge routes with prefix
            for (method, nested_routes) in &nested_compiled.routes {
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

        // Sort routes by specificity for faster matching
        for routes in routes.values_mut() {
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

        CompiledRouter {
            routes,
            middleware,
            not_found_handler,
        }
    }

    pub async fn handle(&self, mut req: Request) -> Result<Response> {
        let compiled = self.ensure_compiled();

        // Apply middleware in order
        for mw in &compiled.middleware {
            mw.before(&mut req).await?;
        }

        // Get routes for this method only
        if let Some(routes) = compiled.routes.get(&req.method) {
            for route in routes {
                if let Some(params) = route.matches(&req) {
                    req.params = params;

                    let mut response = route.handler.handle(req).await?;

                    // Apply middleware in reverse order
                    for mw in compiled.middleware.iter().rev() {
                        mw.after(&mut response).await?;
                    }

                    return Ok(response);
                }
            }
        }

        // Handle not found
        if let Some(handler) = &compiled.not_found_handler {
            handler.handle(req).await
        } else {
            Err(Error::NotFound(req.uri.path().to_string()))
        }
    }

    // Helper method for quick route matching (useful for testing)
    pub fn matches(&self, method: &Method, path: &str) -> bool {
        let compiled = self.ensure_compiled();
        if let Some(routes) = compiled.routes.get(method) {
            for route in routes {
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
                routes: inner.routes.clone(),
                middleware: inner.middleware.clone(),
                not_found_handler: inner.not_found_handler.clone(),
                nested_routers: inner.nested_routers.clone(),
                dirty: inner.dirty,
                #[cfg(feature = "websocket")]
                websocket_routes: inner.websocket_routes.clone(),
            })),
            compiled: ArcSwap::new(Arc::new(CompiledRouter {
                routes: HashMap::new(),
                middleware: Vec::new(),
                not_found_handler: None,
            })),
        }
    }
}

// Default implementation
impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for common patterns
impl Router {
    /// Create a router with common middleware already applied
    pub fn with_default_middleware() -> Self {
        Self::new()
            .middleware(middleware::LoggerMiddleware)
            .middleware(middleware::CorsMiddleware::new())
    }

    /// Create an API router with JSON support
    pub fn api_router() -> Self {
        Self::with_default_middleware()
    }

    /// Create a static file serving router
    pub fn static_router(prefix: &str) -> Self {
        // Implementation would go here for serving static files
        Self::new().get(&format!("{}/*", prefix), |_: Request| async {
            Ok::<Response, Error>(Response::text("Static file serving not implemented"))
        })
    }
}
