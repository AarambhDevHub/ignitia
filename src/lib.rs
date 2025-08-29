//! # Ignite Web Framework
//!
//! A blazing fast, lightweight web framework for Rust that ignites your development.
//! Built by Aarambh Dev Hub.
//!
//! ## Quick Start
//!
//! ```
//! use ignite::{Router, Server, Request, Response, Result, handler_fn};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let router = Router::new()
//!         .get("/", handler_fn(|_req| async {
//!             Ok(Response::text("Hello from Ignite! 🔥"))
//!         }));
//!
//!     let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
//!     server.ignite().await.unwrap();
//!     Ok(())
//! }
//! ```

pub mod cookie;
pub mod error;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod utils; // Add this line

pub use cookie::{Cookie, CookieJar, SameSite};
pub use error::{Error, Result};
pub use handler::{handler_fn, Handler, HandlerFn};
pub use middleware::Middleware;
pub use request::Request;
pub use response::{Response, ResponseBuilder};
pub use router::{Route, Router};
pub use server::Server; // Add this line

// Re-export commonly used types
pub use async_trait::async_trait;
pub use http::{HeaderMap, HeaderValue, Method, StatusCode};
