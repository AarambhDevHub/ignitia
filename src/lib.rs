//! # Ignitia - A Blazing Fast Rust Web Framework
//!
//! **Ignitia** is a high-performance, lightweight web framework for Rust that ignites your web development
//! experience with speed, safety, and simplicity. Built on top of modern async Rust with Tokio and Hyper,
//! Ignitia provides everything you need to build fast, reliable web applications and APIs.
//!
//! ## 🔥 Key Features
//!
//! - **Blazing Fast Performance**: Built with performance in mind using zero-cost abstractions
//! - **Type-Safe Routing**: Compile-time route validation with automatic parameter extraction
//! - **Flexible Middleware**: Composable middleware system for cross-cutting concerns
//! - **WebSocket Support**: Built-in WebSocket support with multiple handler patterns
//! - **Ergonomic APIs**: Clean, intuitive APIs inspired by the best web frameworks
//! - **Production Ready**: Comprehensive error handling, logging, and observability
//!
//! ## 🚀 Quick Start
//!
//! Add Ignitia to your `Cargo.toml`:
//!
//! ```
//! [dependencies]
//! ignitia = "0.1.7"
//! tokio = { version = "1.40", features = ["full"] }
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! Create your first Ignitia application:
//!
//! ```
//! use ignitia::{Router, Server, Response};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(Response::text("Hello, Ignitia! 🔥")) })
//!         .get("/health", || async { Ok(Response::json("OK")?) })
//!         .post("/echo", |body: String| async move {
//!             Ok(Response::text(format!("Echo: {}", body)))
//!         });
//!
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()?;
//!     let server = Server::new(router, addr);
//!
//!     println!("🔥 Ignitia server blazing on http://{}", addr);
//!     server.ignitia().await
//! }
//! ```
//!
//! ## 📚 Core Concepts
//!
//! ### Router and Routes
//!
//! The [`Router`] is the heart of your Ignitia application. It manages route definitions,
//! middleware, and request dispatching:
//!
//! ```
//! use ignitia::{Router, Response, Json};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! let router = Router::new()
//!     .get("/users/:id", |Path(id): Path<u32>| async move {
//!         let user = User {
//!             id,
//!             name: "John Doe".to_string(),
//!             email: "john@example.com".to_string(),
//!         };
//!         Response::json(user)
//!     })
//!     .post("/users", |Json(user): Json<CreateUser>| async move {
//!         println!("Creating user: {}", user.name);
//!         Response::json("User created")
//!     });
//! ```
//!
//! ### Request Extractors
//!
//! Ignitia provides powerful extractors to automatically parse request data:
//!
//! ```
//! use ignitia::{Path, Query, Json, Headers, Cookies};
//! use serde::Deserialize;
//! use std::collections::HashMap;
//!
//! #[derive(Deserialize)]
//! struct UserQuery {
//!     page: Option<u32>,
//!     limit: Option<u32>,
//! }
//!
//! // Extract path parameters, query parameters, JSON body, headers, and cookies
//! async fn complex_handler(
//!     Path(user_id): Path<u32>,
//!     Query(query): Query<UserQuery>,
//!     Json(data): Json<HashMap<String, String>>,
//!     headers: Headers,
//!     cookies: Cookies,
//! ) -> ignitia::Result<Response> {
//!     println!("User ID: {}", user_id);
//!     println!("Page: {:?}, Limit: {:?}", query.page, query.limit);
//!     println!("Headers: {:?}", headers.len());
//!     println!("Cookies: {:?}", cookies.len());
//!
//!     Response::json(data)
//! }
//! ```
//!
//! ### Middleware System
//!
//! Build composable middleware for cross-cutting concerns:
//!
//! ```
//! use ignitia::{Router, LoggerMiddleware, CorsMiddleware, AuthMiddleware};
//!
//! let router = Router::new()
//!     .middleware(LoggerMiddleware)
//!     .middleware(CorsMiddleware::new().allow_origin("https://example.com"))
//!     .middleware(AuthMiddleware::new("secret-token").protect_path("/admin"))
//!     .get("/", || async { Ok(Response::text("Hello, World!")) })
//!     .get("/admin", || async { Ok(Response::text("Admin area")) });
//! ```
//!
//! ### Error Handling
//!
//! Comprehensive error handling with custom error types:
//!
//! ```
//! use ignitia::{Error, Response, define_error};
//! use http::StatusCode;
//!
//! define_error! {
//!     AppError {
//!         UserNotFound(StatusCode::NOT_FOUND, "user_not_found"),
//!         InvalidInput(StatusCode::BAD_REQUEST, "invalid_input", "INVALID_INPUT"),
//!     }
//! }
//!
//! async fn get_user(Path(id): Path<u32>) -> Result<Response, AppError> {
//!     if id == 0 {
//!         return Err(AppError::InvalidInput("User ID cannot be zero".into()));
//!     }
//!
//!     // Simulate user lookup
//!     if id > 1000 {
//!         return Err(AppError::UserNotFound("User not found".into()));
//!     }
//!
//!     Ok(Response::json(format!("User {}", id))?)
//! }
//! ```
//!
//! ## 🌐 WebSocket Support
//!
//! Ignitia provides first-class WebSocket support with multiple handler patterns:
//!
//! ### Enable WebSocket Support
//!
//! Add the WebSocket feature to your `Cargo.toml`:
//!
//! ```
//! [dependencies]
//! ignitia = { version = "0.1.7", features = ["websocket"] }
//! ```
//!
//! ### Simple WebSocket Echo Server
//!
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::websocket::{websocket_handler, Message};
//!
//! #[cfg(feature = "websocket")]
//! let router = Router::new()
//!     .websocket("/ws", websocket_handler(|ws| async move {
//!         while let Some(message) = ws.recv().await {
//!             match message {
//!                 Message::Text(text) => {
//!                     ws.send_text(format!("Echo: {}", text)).await?;
//!                 }
//!                 Message::Close(_) => break,
//!                 _ => {}
//!             }
//!         }
//!         Ok(())
//!     }));
//! ```
//!
//! ### Advanced WebSocket Patterns
//!
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::websocket::{websocket_message_handler, websocket_batch_handler};
//!
//! #[cfg(feature = "websocket")]
//! let router = Router::new()
//!     // Per-message processing
//!     .websocket("/ws/messages", websocket_message_handler(|ws, message| async move {
//!         match message {
//!             Message::Text(text) => {
//!                 let response = process_message(&text).await?;
//!                 ws.send_text(response).await?;
//!             }
//!             _ => {}
//!         }
//!         Ok(())
//!     }))
//!     // Batch message processing
//!     .websocket("/ws/batch", websocket_batch_handler(
//!         |ws, messages| async move {
//!             println!("Processing {} messages", messages.len());
//!             let responses = process_batch(messages).await?;
//!             ws.send_batch(responses).await?;
//!             Ok(())
//!         },
//!         10,   // batch size
//!         100,  // timeout in milliseconds
//!     ));
//! ```
//!
//! ## 🏗️ Architecture Overview
//!
//! Ignitia follows a modular architecture with clear separation of concerns:
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        Application                          │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
//! │  │   Routes    │  │ Middleware  │  │    WebSocket        │ │
//! │  └─────────────┘  └─────────────┘  │    Handlers         │ │
//! │                                     └─────────────────────┘ │
//! ├─────────────────────────────────────────────────────────────┤
//! │                    Ignitia Framework                        │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
//! │  │   Router    │  │   Server    │  │     WebSocket       │ │
//! │  │             │  │             │  │     Support         │ │
//! │  │ ┌─────────┐ │  │ ┌─────────┐ │  │ ┌─────────────────┐ │ │
//! │  │ │ Route   │ │  │ │Request  │ │  │ │   Connection    │ │ │
//! │  │ │Matching │ │  │ │Handling │ │  │ │   Management    │ │ │
//! │  │ └─────────┘ │  │ └─────────┘ │  │ └─────────────────┘ │ │
//! │  │             │  │             │  │                     │ │
//! │  │ ┌─────────┐ │  │ ┌─────────┐ │  │ ┌─────────────────┐ │ │
//! │  │ │Handler  │ │  │ │Response │ │  │ │     Message     │ │ │
//! │  │ │Extract  │ │  │ │Building │ │  │ │    Processing   │ │ │
//! │  │ └─────────┘ │  │ └─────────┘ │  │ └─────────────────┘ │ │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘ │
//! ├─────────────────────────────────────────────────────────────┤
//! │                    Runtime (Tokio)                         │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
//! │  │    HTTP     │  │    TCP      │  │      Async I/O      │ │
//! │  │   (Hyper)   │  │ Listeners   │  │                     │ │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘ │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 🔧 Feature Flags
//!
//! Ignitia uses feature flags to provide optional functionality:
//!
//! - **`websocket`**: Enables WebSocket support with connection management and message handling
//!
//! ```
//! [dependencies]
//! # Enable all features
//! ignitia = { version = "0.1.7", features = ["websocket"] }
//!
//! # Or minimal installation (HTTP only)
//! ignitia = "0.1.7"
//! ```
//!
//! ## 🎯 Performance Optimizations
//!
//! Ignitia is designed for performance from the ground up:
//!
//! - **Zero-Cost Abstractions**: Leverage Rust's zero-cost abstractions for maximum performance
//! - **Compile-Time Route Optimization**: Routes are compiled and optimized at build time
//! - **Efficient Memory Usage**: Minimal allocations with smart memory management
//! - **Async-First Design**: Built on Tokio for excellent concurrency performance
//! - **Request Processing Pipeline**: Optimized request/response processing with minimal overhead
//!
//! ## 🧪 Testing Your Application
//!
//! Ignitia provides utilities to make testing your web applications easy:
//!
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use ignitia::{Router, Response};
//!
//!     #[tokio::test]
//!     async fn test_hello_endpoint() {
//!         let router = Router::new()
//!             .get("/hello", || async { Ok(Response::text("Hello, World!")) });
//!
//!         // Test route matching
//!         assert!(router.matches(&http::Method::GET, "/hello"));
//!         assert!(!router.matches(&http::Method::POST, "/hello"));
//!     }
//! }
//! ```
//!
//! ## 🔍 Examples and Tutorials
//!
//! ### REST API Example
//!
//! ```
//! use ignitia::{Router, Response, Json, Path};
//! use serde::{Deserialize, Serialize};
//! use std::collections::HashMap;
//! use std::sync::{Arc, Mutex};
//!
//! #[derive(Serialize, Deserialize, Clone)]
//! struct Todo {
//!     id: u32,
//!     title: String,
//!     completed: bool,
//! }
//!
//! type TodoStore = Arc<Mutex<HashMap<u32, Todo>>>;
//!
//! fn create_todo_api() -> Router {
//!     let store: TodoStore = Arc::new(Mutex::new(HashMap::new()));
//!
//!     Router::new()
//!         // GET /todos - List all todos
//!         .get("/todos", {
//!             let store = Arc::clone(&store);
//!             move || {
//!                 let store = Arc::clone(&store);
//!                 async move {
//!                     let todos: Vec<Todo> = store.lock().unwrap().values().cloned().collect();
//!                     Response::json(todos)
//!                 }
//!             }
//!         })
//!         // GET /todos/:id - Get specific todo
//!         .get("/todos/:id", {
//!             let store = Arc::clone(&store);
//!             move |Path(id): Path<u32>| {
//!                 let store = Arc::clone(&store);
//!                 async move {
//!                     let todos = store.lock().unwrap();
//!                     match todos.get(&id) {
//!                         Some(todo) => Response::json(todo),
//!                         None => Ok(Response::not_found().with_body("Todo not found")),
//!                     }
//!                 }
//!             }
//!         })
//!         // POST /todos - Create new todo
//!         .post("/todos", {
//!             let store = Arc::clone(&store);
//!             move |Json(mut todo): Json<Todo>| {
//!                 let store = Arc::clone(&store);
//!                 async move {
//!                     let mut todos = store.lock().unwrap();
//!                     let id = todos.len() as u32 + 1;
//!                     todo.id = id;
//!                     todos.insert(id, todo.clone());
//!                     Response::json(todo)
//!                 }
//!             }
//!         });
//! }
//! ```
//!
//! ### File Upload Example
//!
//! ```
//! use ignitia::{Router, Response, Body};
//! use std::path::Path;
//! use tokio::fs;
//!
//! let router = Router::new()
//!     .post("/upload", |body: Body| async move {
//!         let filename = format!("upload_{}.bin", uuid::Uuid::new_v4());
//!         let path = Path::new("uploads").join(&filename);
//!
//!         fs::create_dir_all("uploads").await?;
//!         fs::write(&path, body.bytes()).await?;
//!
//!         Response::json(serde_json::json!({
//!             "filename": filename,
//!             "size": body.len(),
//!             "message": "File uploaded successfully"
//!         }))
//!     });
//! ```
//!
//! ## 📖 Module Documentation
//!
//! - [`cookie`]: HTTP cookie handling and management
//! - [`error`]: Error types, custom error handling, and error response generation
//! - [`extension`]: Type-safe request/response extensions for sharing data
//! - [`handler`]: Request handlers, extractors, and handler trait implementations
//! - [`middleware`]: Middleware system for cross-cutting concerns
//! - [`request`]: HTTP request representation and utilities
//! - [`response`]: HTTP response building and utilities
//! - [`router`]: Route matching, parameter extraction, and request routing
//! - [`server`]: HTTP server implementation and connection handling
//! - [`utils`]: Utility functions for common web development tasks
//! - [`websocket`]: WebSocket protocol support and connection management (feature-gated)
//!
//! ## 🤝 Contributing
//!
//! We welcome contributions! Please see our [contributing guidelines](https://github.com/AarambhDevHub/ignitia/blob/main/CONTRIBUTING.md)
//! for more information.
//!
//! ## 📄 License
//!
//! This project is licensed under the MIT License - see the [LICENSE](https://github.com/AarambhDevHub/ignitia/blob/main/LICENSE)
//! file for details.
//!
//! ## 🔗 Links
//!
//! - [Repository](https://github.com/AarambhDevHub/ignitia)
//! - [Documentation](https://docs.rs/ignitia)
//! - [Examples](https://github.com/AarambhDevHub/ignitia/tree/main/examples)
//! - [Changelog](https://github.com/AarambhDevHub/ignitia/blob/main/CHANGELOG.md)

// Enable documentation features for docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]
// Deny missing docs to ensure comprehensive documentation
#![warn(missing_docs)]
// Enable additional documentation lint rules
#![warn(rustdoc::missing_crate_level_docs)]

// Core framework modules
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

// WebSocket support (feature-gated)
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod websocket;

// Re-export cookie types for easy access
pub use cookie::{Cookie, CookieJar, SameSite};

// Re-export error types and utilities
pub use error::{
    CustomError, Error, ErrorExt, ErrorHandler, ErrorHandlerType, ErrorHandlerWithRequest,
    ErrorResponse, Result,
};

// Re-export extension system
pub use extension::{Extension, Extensions};

// Re-export handler extractors with aliases to avoid naming conflicts
pub use handler::extractor::{
    Body, Cookies, Headers, Json, Method as IgnitiaMethod, Path, Query, Uri,
};

// Re-export handler types and utilities
pub use handler::{
    handler_fn, into_handler, raw_handler, Handler, HandlerFn, IntoHandler, RawRequest,
};

// Re-export middleware types
pub use middleware::{
    AuthMiddleware, CorsMiddleware, ErrorHandlerMiddleware, LoggerMiddleware, Middleware,
};

// Re-export core request and response types
pub use request::Request;
pub use response::{Response, ResponseBuilder};

// Re-export routing components
pub use router::{Route, Router};

// Re-export server components
pub use server::Server;

// Re-export WebSocket functionality when feature is enabled
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use websocket::{
    handle_websocket_upgrade, is_websocket_request, upgrade_connection, websocket_batch_handler,
    websocket_handler, websocket_message_handler, BatchMessageHandler, CloseFrame, Message,
    MessageType, OptimizedMessageHandler, WebSocketConnection, WebSocketHandler,
};

// Re-export commonly used external types for convenience
/// Async trait support for defining async traits
pub use async_trait::async_trait;

/// HTTP types from the `http` crate
pub use http::{HeaderMap, HeaderValue, Method, StatusCode};

// Version information
/// The current version of the Ignitia framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the Ignitia framework
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Framework information and build details
pub mod info {
    //! Framework information and build metadata.

    /// Returns the framework name and version as a formatted string
    pub fn version() -> String {
        format!("{} v{}", crate::NAME, crate::VERSION)
    }

    /// Returns build information
    pub fn build_info() -> BuildInfo {
        BuildInfo {
            name: crate::NAME,
            version: crate::VERSION,
            features: get_enabled_features(),
        }
    }

    /// Build information structure
    #[derive(Debug, Clone)]
    pub struct BuildInfo {
        /// Framework name
        pub name: &'static str,
        /// Framework version
        pub version: &'static str,
        /// Enabled features
        pub features: Vec<&'static str>,
    }

    fn get_enabled_features() -> Vec<&'static str> {
        let mut features = Vec::new();

        #[cfg(feature = "websocket")]
        features.push("websocket");

        if features.is_empty() {
            features.push("default");
        }

        features
    }
}

/// Prelude module for common imports
pub mod prelude {
    //! Common imports for Ignitia applications.
    //!
    //! This module provides a convenient way to import the most commonly used
    //! types and traits from Ignitia. Instead of importing each type individually,
    //! you can use:
    //!
    //! ```
    //! use ignitia::prelude::*;
    //! ```

    // Core framework types
    pub use crate::{Error, Request, Response, Result, Router, Server};

    // Handler and middleware types
    pub use crate::{Handler, HandlerFn, Middleware};

    // Common extractors
    pub use crate::{Body, Cookies, Headers, Json, Path, Query};

    // HTTP types
    pub use crate::{HeaderMap, HeaderValue, Method, StatusCode};

    // Async trait
    pub use crate::async_trait;

    // WebSocket types (when feature is enabled)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub use crate::{Message, MessageType, WebSocketConnection, WebSocketHandler};
}
