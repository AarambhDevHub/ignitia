pub mod extractor;

use crate::{Request, Response, Result};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, req: Request) -> Result<Response>;
}

pub type HandlerFn = Arc<dyn Fn(Request) -> BoxFuture<'static, Result<Response>> + Send + Sync>;

#[async_trait::async_trait]
impl Handler for HandlerFn {
    async fn handle(&self, req: Request) -> Result<Response> {
        (self)(req).await
    }
}

// Helper function to convert closures to handlers
pub fn handler_fn<F, Fut>(f: F) -> HandlerFn
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    Arc::new(move |req| Box::pin(f(req)))
}
