pub mod method;
pub mod route;

use crate::handler::{into_handler, IntoHandler};
use crate::middleware::Middleware;
use crate::{Error, Handler, HandlerFn, Request, Response, Result};
use arc_swap::ArcSwap;
use http::Method;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub use route::Route;

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
}

impl Router {
    pub fn new() -> Self {
        let inner = RouterInner {
            routes: HashMap::new(),
            middleware: Vec::new(),
            not_found_handler: None,
            nested_routers: Vec::new(),
            dirty: true,
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
    pub fn route(self, path: &str, method: Method, handler: HandlerFn) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;

            let full_path = normalize_path(path);
            inner
                .routes
                .entry(method.clone())
                .or_insert_with(Vec::new)
                .push(Route::new(&full_path, method, handler));
        }

        self
    }

    pub fn route_with<H, T>(self, path: &str, method: Method, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route(path, method, into_handler(handler))
    }

    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route_with(path, Method::GET, handler)
    }

    pub fn post<H, T>(self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route_with(path, Method::POST, handler)
    }

    pub fn put<H, T>(self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route_with(path, Method::PUT, handler)
    }

    pub fn delete<H, T>(self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route_with(path, Method::DELETE, handler)
    }

    pub fn patch<H, T>(self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route_with(path, Method::PATCH, handler)
    }

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

    fn ensure_compiled(&self) -> Arc<CompiledRouter> {
        // Check if compilation is needed without holding the lock too long
        let needs_compilation = {
            let inner = self.inner.read();
            inner.dirty
        };

        if !needs_compilation {
            return self.compiled.load_full();
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

        // Sort routes by specificity
        for routes in routes.values_mut() {
            routes.sort_by(|a, b| {
                let a_segments = a.path.matches('/').count();
                let b_segments = b.path.matches('/').count();
                b_segments.cmp(&a_segments).then_with(|| {
                    let a_params = a.param_names.len() + a.wildcard_names.len();
                    let b_params = b.param_names.len() + b.wildcard_names.len();
                    a_params.cmp(&b_params)
                })
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
            })),
            compiled: ArcSwap::new(Arc::new(CompiledRouter {
                routes: HashMap::new(),
                middleware: Vec::new(),
                not_found_handler: None,
            })),
        }
    }
}
