pub mod method;
pub mod route;

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

    pub fn get(self, path: &str, handler: HandlerFn) -> Self {
        self.route(path, Method::GET, handler)
    }

    pub fn post(self, path: &str, handler: HandlerFn) -> Self {
        self.route(path, Method::POST, handler)
    }

    pub fn put(self, path: &str, handler: HandlerFn) -> Self {
        self.route(path, Method::PUT, handler)
    }

    pub fn delete(self, path: &str, handler: HandlerFn) -> Self {
        self.route(path, Method::DELETE, handler)
    }

    pub fn middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub fn not_found(mut self, handler: HandlerFn) -> Self {
        self.not_found_handler = Some(handler);
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
