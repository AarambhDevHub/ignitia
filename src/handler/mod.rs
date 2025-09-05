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

// Helper function to convert closures to handlers (legacy support)
pub fn handler_fn<F, Fut>(f: F) -> HandlerFn
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    Arc::new(move |req| Box::pin(f(req)))
}

// New trait for handlers with automatic extraction
#[async_trait::async_trait]
pub trait IntoHandler<T>: Clone + Send + Sync + 'static {
    async fn call(self, req: Request) -> Result<Response>;
}

// Convert IntoHandler to Handler
pub fn into_handler<H, T>(handler: H) -> HandlerFn
where
    H: IntoHandler<T>,
{
    Arc::new(move |req| {
        let handler = handler.clone();
        Box::pin(async move { handler.call(req).await })
    })
}

// Implementation for functions with no extractors (just returns Response)
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

// Macro to generate implementations for different numbers of extractors
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

// Special marker type to distinguish raw Request handlers from extractors
pub struct RawRequest(pub Request);

// Implementation for functions that take RawRequest directly
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

// Convenience function for raw request handlers
pub fn raw_handler<F, Fut>(f: F) -> impl IntoHandler<(RawRequest,)>
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send + 'static,
{
    f
}
