pub mod method;
pub mod route;

use crate::handler::{into_handler, IntoHandler};
use crate::middleware::Middleware;
use crate::{Error, Handler, HandlerFn, Request, Response, Result};
use http::Method;
use std::sync::Arc;

pub use route::Route;

pub struct Router {
    routes: Vec<Route>,
    middleware: Vec<Arc<dyn Middleware>>,
    not_found_handler: Option<HandlerFn>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            not_found_handler: None,
        }
    }

    pub fn route(mut self, path: &str, method: Method, handler: HandlerFn) -> Self {
        self.routes.push(Route::new(path, method, handler));
        self
    }

    // New method that accepts IntoHandler
    pub fn route_with<H, T>(mut self, path: &str, method: Method, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.routes
            .push(Route::new(path, method, into_handler(handler)));
        self
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

    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub fn not_found<H, T>(mut self, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.not_found_handler = Some(into_handler(handler));
        self
    }

    pub async fn handle(&self, mut req: Request) -> Result<Response> {
        // Apply middleware in order
        for mw in &self.middleware {
            mw.before(&mut req).await?;
        }

        // Find matching route
        for route in &self.routes {
            if let Some(params) = route.matches(&req) {
                req.params = params;

                let mut response = route.handler.handle(req).await?;

                // Apply middleware in reverse order for response
                for mw in self.middleware.iter().rev() {
                    mw.after(&mut response).await?;
                }

                return Ok(response);
            }
        }

        // Handle not found
        if let Some(handler) = &self.not_found_handler {
            handler.handle(req).await
        } else {
            Err(Error::NotFound)
        }
    }
}
