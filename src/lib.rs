#![cfg_attr(docsrs, feature(doc_cfg))]

//! # ignitia Web Framework
//!
//! A blazing fast, lightweight web framework for Rust that ignitias your development.
//! Built by Aarambh Dev Hub.
//!
//! ## Quick Start
//!
//! ```
//! use ignitia::{Router, Server, Request, Response, Result};
//!
//! async fn hello() -> Result<Response> {
//!     Ok(Response::text("Hello from ignitia! 🔥"))
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let router = Router::new()
//!         .get("/", hello);
//!
//!     let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
//!     server.ignitia().await?;
//!     Ok(())
//! }
//! ```

pub mod cookie;
pub mod error;
pub mod extension;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod utils;

#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod websocket;

pub use cookie::{Cookie, CookieJar, SameSite};
pub use error::{
    CustomError, Error, ErrorExt, ErrorHandler, ErrorHandlerType, ErrorHandlerWithRequest,
    ErrorResponse, Result,
};
pub use extension::{Extension, Extensions};
pub use handler::extractor::{
    Body, Cookies, Headers, Json, Method as IgnitiaMethod, Path, Query, Uri,
};
pub use handler::{
    handler_fn, into_handler, raw_handler, Handler, HandlerFn, IntoHandler, RawRequest,
};
pub use middleware::{
    AuthMiddleware, CorsMiddleware, ErrorHandlerMiddleware, LoggerMiddleware, Middleware,
};
pub use request::Request;
pub use response::{Response, ResponseBuilder};
pub use router::{Route, Router};
pub use server::Server;

#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use websocket::{
    is_websocket_request, upgrade_connection, websocket_batch_handler, websocket_handler,
    websocket_message_handler, BatchMessageHandler, CloseFrame, Message, MessageType,
    OptimizedMessageHandler, WebSocketConnection, WebSocketHandler,
};

// Re-export commonly used types
pub use async_trait::async_trait;
pub use http::{HeaderMap, HeaderValue, Method, StatusCode};
