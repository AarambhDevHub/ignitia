pub mod error;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod utils;

pub use error::{Error, Result};
pub use handler::{Handler, HandlerFn, handler_fn};
pub use middleware::Middleware;
pub use request::Request;
pub use response::{Response, ResponseBuilder};
pub use router::{Route, Router};
pub use server::Server;

// Re-export commonly used types
pub use async_trait::async_trait;
pub use http::{HeaderMap, HeaderValue, Method, StatusCode};
