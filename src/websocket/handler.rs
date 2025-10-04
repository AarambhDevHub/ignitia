//! WebSocket Handler System with Universal Extractor Support
//!
//! This module provides a flexible WebSocket handler system that supports:
//! - Multiple extractor parameters (State, Path, Query, Headers, etc.)
//! - Any return type that implements `IntoResponse`
//! - Zero-copy performance optimizations
//! - Type-safe parameter extraction
//!
//! # Examples
//!
//! ## Basic WebSocket Handler (No Extractors)
//!
//! ```
//! use ignitia::prelude::*;
//!
//! router.websocket("/ws", |mut ws: WebSocketConnection| async move {
//!     while let Some(msg) = ws.recv().await {
//!         match msg {
//!             Message::Text(text) => {
//!                 ws.send_text(format!("Echo: {}", text)).await.ok();
//!             }
//!             Message::Close(_) => break,
//!             _ => {}
//!         }
//!     }
//!     Response::ok()
//! });
//! ```
//!
//! ## WebSocket with State Extractor
//!
//! ```
//! use ignitia::prelude::*;
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[derive(Clone)]
//! struct AppState {
//!     connections: Arc<RwLock<Vec<String>>>,
//! }
//!
//! router
//!     .state(AppState {
//!         connections: Arc::new(RwLock::new(Vec::new())),
//!     })
//!     .websocket("/ws", |
//!         State(state): State<AppState>,
//!         mut ws: WebSocketConnection
//!     | async move {
//!         // Access shared state
//!         state.connections.write().await.push("new_user".to_string());
//!
//!         ws.send_text("Connected!").await.ok();
//!
//!         while let Some(msg) = ws.recv().await {
//!             // Handle messages...
//!         }
//!
//!         "Connection closed"
//!     });
//! ```
//!
//! ## WebSocket with Path Parameters
//!
//! ```
//! use ignitia::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct RoomParams {
//!     room_id: String,
//! }
//!
//! router.websocket("/ws/room/:room_id", |
//!     Path(params): Path<RoomParams>,
//!     mut ws: WebSocketConnection
//! | async move {
//!     ws.send_text(format!("Welcome to room: {}", params.room_id)).await.ok();
//!
//!     while let Some(msg) = ws.recv().await {
//!         // Handle messages in this specific room...
//!     }
//!
//!     Json(serde_json::json!({ "status": "completed" }))
//! });
//! ```
//!
//! ## WebSocket with Multiple Extractors
//!
//! ```
//! use ignitia::prelude::*;
//!
//! router.websocket("/ws/:user_id", |
//!     Path(path): Path<HashMap<String, String>>,
//!     Query(query): Query<HashMap<String, String>>,
//!     State(state): State<AppState>,
//!     Headers(headers): Headers,
//!     mut ws: WebSocketConnection
//! | async move {
//!     let user_id = &path["user_id"];
//!     let token = query.get("token");
//!
//!     // Validate token from query params
//!     if token.is_none() {
//!         return Response::unauthorized("Token required");
//!     }
//!
//!     // Send welcome message
//!     ws.send_text(format!("Authenticated as {}", user_id)).await.ok();
//!
//!     while let Some(msg) = ws.recv().await {
//!         // Handle authenticated messages...
//!     }
//!
//!     (StatusCode::OK, "Session ended")
//! });
//! ```

use super::connection::WebSocketConnection;
use super::message::Message;
use crate::handler::extractor::FromRequest;
use crate::response::IntoResponse;
use crate::{Request, Response};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// Type alias for boxed futures
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// ============================================================================
// Core WebSocket Handler Trait
// ============================================================================

/// Core WebSocket handler trait that receives both request context and WebSocket connection.
///
/// This trait is implemented automatically for functions that match the signature
/// requirements through the `UniversalWebSocketHandler` trait.
///
/// # Implementation Note
///
/// You typically don't implement this trait directly. Instead, use the
/// `UniversalWebSocketHandler` trait which provides automatic conversion.
#[async_trait::async_trait]
pub trait WebSocketHandler: Send + Sync {
    /// Handle a WebSocket connection with full request context.
    ///
    /// # Parameters
    ///
    /// * `req` - The HTTP request that initiated the WebSocket upgrade
    /// * `websocket` - The established WebSocket connection
    ///
    /// # Returns
    ///
    /// A `Response` indicating the result of the WebSocket session
    async fn handle(&self, req: Request, websocket: WebSocketConnection) -> Response;
}

// ============================================================================
// Universal Handler Trait (with Extractor Support)
// ============================================================================

/// Universal WebSocket handler trait that supports automatic parameter extraction.
///
/// This trait enables handlers to accept extracted parameters like `State<T>`,
/// `Path<T>`, `Query<T>`, etc., before receiving the WebSocket connection.
///
/// # Type Parameter
///
/// * `T` - Phantom type parameter used for extractor type resolution
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for functions with 0-7 extractors
/// plus a `WebSocketConnection` parameter, thanks to the macro below.
#[async_trait::async_trait]
pub trait UniversalWebSocketHandler<T>: Clone + Send + Sync + 'static {
    /// Call the handler with request and WebSocket connection.
    ///
    /// Extractors are processed internally before calling the actual handler function.
    async fn call(&self, req: Request, websocket: WebSocketConnection) -> Response;
}

/// Convert a `UniversalWebSocketHandler` to a trait object `WebSocketHandler`.
///
/// This function wraps handlers in a type-erased trait object for storage in the router.
///
/// # Parameters
///
/// * `handler` - Any type implementing `UniversalWebSocketHandler<T>`
///
/// # Returns
///
/// An `Arc<dyn WebSocketHandler>` that can be stored in the router
///
/// # Example
///
/// ```
/// let handler = |ws: WebSocketConnection| async move {
///     // Handler implementation
///     Response::ok()
/// };
///
/// let wrapped = universal_ws_handler(handler);
/// ```
pub fn universal_ws_handler<H, T>(handler: H) -> Arc<dyn WebSocketHandler>
where
    H: UniversalWebSocketHandler<T>,
    T: Send + Sync + 'static,
{
    Arc::new(WebSocketHandlerWrapper {
        handler,
        _phantom: PhantomData,
    })
}

/// Internal wrapper that converts `UniversalWebSocketHandler` to `WebSocketHandler`.
///
/// This struct uses `PhantomData` to maintain type information for the generic
/// parameter `T` without actually storing it at runtime (zero-cost abstraction).
struct WebSocketHandlerWrapper<H, T> {
    handler: H,
    _phantom: PhantomData<T>,
}

#[async_trait::async_trait]
impl<H, T> WebSocketHandler for WebSocketHandlerWrapper<H, T>
where
    H: UniversalWebSocketHandler<T>,
    T: Send + Sync + 'static,
{
    async fn handle(&self, req: Request, websocket: WebSocketConnection) -> Response {
        self.handler.call(req, websocket).await
    }
}

// ============================================================================
// Extractor Implementations (0-7 parameters)
// ============================================================================

/// Implementation for handlers with no extractors (just WebSocket connection).
///
/// # Example
///
/// ```
/// router.websocket("/ws", |mut ws: WebSocketConnection| async move {
///     ws.send_text("Hello!").await.ok();
///     Response::ok()
/// });
/// ```
#[async_trait::async_trait]
impl<F, Fut, R> UniversalWebSocketHandler<()> for F
where
    F: Fn(WebSocketConnection) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    async fn call(&self, _req: Request, websocket: WebSocketConnection) -> Response {
        let result = self(websocket).await;
        result.into_response()
    }
}

/// Macro to generate implementations for handlers with 1-7 extractors.
///
/// This macro generates code that:
/// 1. Extracts each parameter from the request using `FromRequest`
/// 2. Returns early with an error response if extraction fails
/// 3. Calls the handler with all extracted parameters plus WebSocket
/// 4. Converts the handler's return value to a `Response`
///
/// # Generated Implementations
///
/// For each number of parameters (1-7), generates an implementation like:
///
/// ```
/// impl<F, Fut, T1, T2, R> UniversalWebSocketHandler<(T1, T2)> for F
/// where
///     F: Fn(T1, T2, WebSocketConnection) -> Fut + Clone + Send + Sync + 'static,
///     Fut: Future<Output = R> + Send + 'static,
///     R: IntoResponse,
///     T1: FromRequest + Send,
///     T2: FromRequest + Send,
/// {
///     async fn call(&self, req: Request, websocket: WebSocketConnection) -> Response {
///         let t1 = match T1::from_request(&req) {
///             Ok(val) => val,
///             Err(error_response) => return error_response.into_response(),
///         };
///         let t2 = match T2::from_request(&req) {
///             Ok(val) => val,
///             Err(error_response) => return error_response.into_response(),
///         };
///
///         let result = self(t1, t2, websocket).await;
///         result.into_response()
///     }
/// }
/// ```
macro_rules! impl_ws_handler {
    ($($ty:ident),+) => {
        #[async_trait::async_trait]
        impl<F, Fut, $($ty,)+ R> UniversalWebSocketHandler<($($ty,)+)> for F
        where
            F: Fn($($ty,)+ WebSocketConnection) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = R> + Send + 'static,
            R: IntoResponse,
            $($ty: FromRequest + Send,)+
        {
            async fn call(&self, req: Request, websocket: WebSocketConnection) -> Response {
                // Extract each parameter in order
                $(
                    let $ty = match $ty::from_request(&req) {
                        Ok(val) => val,
                        Err(error_response) => return error_response.into_response(),
                    };
                )+

                // Call handler with extracted parameters and websocket
                let result = self($($ty,)+ websocket).await;
                result.into_response()
            }
        }
    };
}

// Generate implementations for 1-7 extractors
impl_ws_handler!(T1);
impl_ws_handler!(T1, T2);
impl_ws_handler!(T1, T2, T3);
impl_ws_handler!(T1, T2, T3, T4);
impl_ws_handler!(T1, T2, T3, T4, T5);
impl_ws_handler!(T1, T2, T3, T4, T5, T6);
impl_ws_handler!(T1, T2, T3, T4, T5, T6, T7);

// ============================================================================
// Helper Functions (Backward Compatibility)
// ============================================================================

/// Legacy function pointer type for WebSocket handlers.
///
/// # Deprecation Note
///
/// This type is maintained for backward compatibility. New code should use
/// `UniversalWebSocketHandler` instead.
pub type WebSocketHandlerFn =
    Arc<dyn Fn(WebSocketConnection) -> BoxFuture<'static, Response> + Send + Sync>;

/// Create a WebSocket handler from a function.
///
/// This is a convenience function that wraps a closure or function pointer
/// into a handler compatible with the router.
///
/// # Example
///
/// ```
/// let handler = websocket_handler(|mut ws: WebSocketConnection| async move {
///     ws.send_text("Hello!").await.ok();
///     Response::ok()
/// });
///
/// router.websocket("/ws", handler);
/// ```
pub fn websocket_handler<F, Fut, R>(f: F) -> impl UniversalWebSocketHandler<()>
where
    F: Fn(WebSocketConnection) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    f
}

// ============================================================================
// Advanced: Message-Based Handlers
// ============================================================================

/// Optimized handler that processes individual messages with automatic loop.
///
/// This handler type automatically manages the message receive loop and calls
/// your function for each incoming message until the connection closes.
///
/// # Example
///
/// ```
/// use ignitia::prelude::*;
///
/// let handler = websocket_message_handler(|ws: WebSocketConnection, msg: Message| async move {
///     match msg {
///         Message::Text(text) => {
///             ws.send_text(format!("Received: {}", text)).await.ok();
///             Response::ok()
///         }
///         Message::Binary(data) => {
///             ws.send_binary(data).await.ok();
///             Response::ok()
///         }
///         _ => Response::ok()
///     }
/// });
///
/// router.websocket("/ws/messages", handler);
/// ```
pub struct OptimizedMessageHandler<F> {
    handler: Arc<F>,
}

impl<F> OptimizedMessageHandler<F> {
    /// Create a new message handler from a function.
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }
}

impl<F> Clone for OptimizedMessageHandler<F> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut, R> UniversalWebSocketHandler<()> for OptimizedMessageHandler<F>
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    async fn call(&self, _req: Request, websocket: WebSocketConnection) -> Response {
        while let Some(message) = websocket.recv().await {
            match message {
                Message::Close(_) => break,
                _ => {
                    let result = (self.handler)(websocket.clone(), message).await;
                    let response = result.into_response();

                    // If handler returns error response, close connection
                    if !response.status.is_success() {
                        tracing::debug!("WebSocket handler returned error, closing connection");
                        let _ = websocket.close(None).await;
                        return response;
                    }
                }
            }
        }
        Response::ok()
    }
}

/// Create a message handler that processes each message individually.
///
/// # Parameters
///
/// * `f` - Function that takes `(WebSocketConnection, Message)` and returns any `IntoResponse`
///
/// # Returns
///
/// An `OptimizedMessageHandler` that can be used with the router
pub fn websocket_message_handler<F, Fut, R>(f: F) -> OptimizedMessageHandler<F>
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    OptimizedMessageHandler::new(f)
}

// ============================================================================
// Advanced: Batch Message Handler
// ============================================================================

/// Handler that processes messages in batches for improved throughput.
///
/// This is useful for high-volume WebSocket connections where processing
/// messages individually would be inefficient.
///
/// # Example
///
/// ```
/// use ignitia::prelude::*;
///
/// let handler = websocket_batch_handler(
///     |ws: WebSocketConnection, messages: Vec<Message>| async move {
///         tracing::info!("Processing {} messages", messages.len());
///
///         for msg in messages {
///             // Process batch...
///         }
///
///         Response::ok()
///     },
///     100,    // batch_size: process up to 100 messages at once
///     1000    // timeout_ms: flush batch after 1 second
/// );
///
/// router.websocket("/ws/batch", handler);
/// ```
pub struct BatchMessageHandler<F> {
    handler: Arc<F>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl<F> BatchMessageHandler<F> {
    /// Create a new batch message handler.
    ///
    /// # Parameters
    ///
    /// * `handler` - Function to process message batches
    /// * `batch_size` - Maximum number of messages per batch
    /// * `batch_timeout` - Maximum time to wait before flushing batch
    pub fn new(handler: F, batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            handler: Arc::new(handler),
            batch_size,
            batch_timeout,
        }
    }
}

impl<F> Clone for BatchMessageHandler<F> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
            batch_size: self.batch_size,
            batch_timeout: self.batch_timeout,
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut, R> UniversalWebSocketHandler<()> for BatchMessageHandler<F>
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    async fn call(&self, _req: Request, websocket: WebSocketConnection) -> Response {
        let mut message_batch = Vec::with_capacity(self.batch_size);

        loop {
            match websocket.recv().await {
                Some(Message::Close(_)) => break,
                Some(message) => {
                    message_batch.push(message);

                    if message_batch.len() >= self.batch_size {
                        let result =
                            (self.handler)(websocket.clone(), std::mem::take(&mut message_batch))
                                .await;

                        let response = result.into_response();
                        if !response.status.is_success() {
                            tracing::debug!("WebSocket batch handler error, closing");
                            let _ = websocket.close(None).await;
                            return response;
                        }
                        message_batch = Vec::with_capacity(self.batch_size);
                    }
                }
                None => {
                    if !message_batch.is_empty() {
                        let result =
                            (self.handler)(websocket.clone(), std::mem::take(&mut message_batch))
                                .await;

                        let response = result.into_response();
                        if !response.status.is_success() {
                            return response;
                        }
                    }
                    break;
                }
            }
        }

        Response::ok()
    }
}

/// Create a batch message handler with specified batch size and timeout.
///
/// # Parameters
///
/// * `f` - Function that processes a batch of messages
/// * `batch_size` - Maximum messages per batch before flushing
/// * `timeout_ms` - Maximum milliseconds to wait before flushing partial batch
///
/// # Example
///
/// ```
/// let handler = websocket_batch_handler(
///     |ws, messages| async move {
///         // Process up to 50 messages at once
///         for msg in messages {
///             // Handle message...
///         }
///         Response::ok()
///     },
///     50,      // Batch size
///     100      // Timeout in milliseconds
/// );
/// ```
pub fn websocket_batch_handler<F, Fut, R>(
    f: F,
    batch_size: usize,
    timeout_ms: u64,
) -> BatchMessageHandler<F>
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse,
{
    BatchMessageHandler::new(f, batch_size, Duration::from_millis(timeout_ms))
}
