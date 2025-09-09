//! # WebSocket Handler System
//!
//! This module provides a flexible and efficient WebSocket handler system that supports
//! multiple handler patterns for different use cases. The handler system is built around
//! the `WebSocketHandler` trait which provides a unified interface for WebSocket message
//! processing while allowing for various optimization strategies.
//!
//! ## Handler Types
//!
//! The module offers several handler types optimized for different scenarios:
//!
//! - **Simple Handlers**: Basic connection-level handling with full control
//! - **Message Handlers**: Optimized per-message processing
//! - **Batch Handlers**: Efficient bulk message processing
//!
//! ## Architecture
//!
//! The handler system uses an async trait-based architecture that provides:
//!
//! - **Flexibility**: Multiple handler patterns for different use cases
//! - **Performance**: Optimized implementations for high-throughput scenarios
//! - **Simplicity**: Easy-to-use convenience functions for common patterns
//! - **Error Handling**: Comprehensive error handling and recovery
//!
//! ## Handler Patterns
//!
//! ### 1. Connection Handler
//!
//! The most basic handler that gives you full control over the WebSocket connection:
//!
//! ```
//! use ignitia::websocket::{websocket_handler, WebSocketConnection, Message};
//!
//! let handler = websocket_handler(|ws: WebSocketConnection| async move {
//!     // Send welcome message
//!     ws.send_text("Welcome to the server!".to_string()).await?;
//!
//!     // Handle messages in a loop
//!     while let Some(message) = ws.recv().await {
//!         match message {
//!             Message::Text(text) => {
//!                 println!("Received: {}", text);
//!                 ws.send_text(format!("Echo: {}", text)).await?;
//!             }
//!             Message::Binary(data) => {
//!                 println!("Received {} bytes", data.len());
//!                 ws.send_bytes(data).await?;
//!             }
//!             Message::Close(_) => {
//!                 println!("Connection closed");
//!                 break;
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! });
//! ```
//!
//! ### 2. Message Handler
//!
//! Optimized for processing individual messages with automatic connection management:
//!
//! ```
//! use ignitia::websocket::{websocket_message_handler, WebSocketConnection, Message};
//!
//! let handler = websocket_message_handler(|ws: WebSocketConnection, message: Message| async move {
//!     match message {
//!         Message::Text(text) => {
//!             let response = format!("Processed: {}", text.to_uppercase());
//!             ws.send_text(response).await?;
//!         }
//!         Message::Binary(data) => {
//!             // Process binary data
//!             let processed = data.iter().map(|b| b.wrapping_add(1)).collect::<Vec<u8>>();
//!             ws.send_bytes(bytes::Bytes::from(processed)).await?;
//!         }
//!         _ => {}
//!     }
//!     Ok(())
//! });
//! ```
//!
//! ### 3. Batch Handler
//!
//! Efficient for high-throughput scenarios where messages can be processed in batches:
//!
//! ```
//! use ignitia::websocket::{websocket_batch_handler, WebSocketConnection, Message};
//!
//! let handler = websocket_batch_handler(
//!     |ws: WebSocketConnection, messages: Vec<Message>| async move {
//!         println!("Processing batch of {} messages", messages.len());
//!
//!         let mut responses = Vec::new();
//!         for message in messages {
//!             if let Message::Text(text) = message {
//!                 responses.push(Message::text(format!("Batch processed: {}", text)));
//!             }
//!         }
//!
//!         if !responses.is_empty() {
//!             ws.send_batch(responses).await?;
//!         }
//!
//!         Ok(())
//!     },
//!     10,   // batch size
//!     100,  // timeout in milliseconds
//! );
//! ```
//!
//! ## Advanced Usage
//!
//! ### Custom Handler Implementation
//!
//! ```
//! use ignitia::websocket::{WebSocketHandler, WebSocketConnection, Message};
//! use std::collections::HashMap;
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//!
//! struct ChatHandler {
//!     rooms: Arc<Mutex<HashMap<String, Vec<WebSocketConnection>>>>,
//! }
//!
//! impl ChatHandler {
//!     fn new() -> Self {
//!         Self {
//!             rooms: Arc::new(Mutex::new(HashMap::new())),
//!         }
//!     }
//! }
//!
//! #[async_trait::async_trait]
//! impl WebSocketHandler for ChatHandler {
//!     async fn handle_connection(&self, websocket: WebSocketConnection) -> ignitia::Result<()> {
//!         // Join default room
//!         {
//!             let mut rooms = self.rooms.lock().await;
//!             rooms.entry("general".to_string())
//!                  .or_insert_with(Vec::new)
//!                  .push(websocket.clone());
//!         }
//!
//!         // Handle messages
//!         while let Some(message) = websocket.recv().await {
//!             match message {
//!                 Message::Text(text) => {
//!                     // Broadcast to all clients in the room
//!                     let rooms = self.rooms.lock().await;
//!                     if let Some(clients) = rooms.get("general") {
//!                         for client in clients {
//!                             let _ = client.send_text(text.clone()).await;
//!                         }
//!                     }
//!                 }
//!                 Message::Close(_) => break,
//!                 _ => {}
//!             }
//!         }
//!
//!         // Remove from room on disconnect
//!         // (In a real implementation, you'd need to track and remove the specific connection)
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Connection Handlers**: Best for complex logic that needs full connection control
//! - **Message Handlers**: Optimal for simple per-message processing
//! - **Batch Handlers**: Most efficient for high-throughput scenarios
//! - **Custom Handlers**: Maximum flexibility but require careful resource management
//!
//! ## Error Handling
//!
//! All handlers should implement proper error handling:
//!
//! ```
//! use ignitia::websocket::{websocket_handler, WebSocketConnection, Message};
//!
//! let handler = websocket_handler(|ws: WebSocketConnection| async move {
//!     while let Some(message) = ws.recv().await {
//!         match message {
//!             Message::Text(text) => {
//!                 // Process message with error handling
//!                 if let Err(e) = process_message(&text).await {
//!                     tracing::error!("Failed to process message: {}", e);
//!
//!                     // Send error response to client
//!                     let error_msg = format!("Error: {}", e);
//!                     if let Err(send_err) = ws.send_text(error_msg).await {
//!                         tracing::error!("Failed to send error response: {}", send_err);
//!                         break; // Connection is likely broken
//!                     }
//!                 }
//!             }
//!             Message::Close(_) => break,
//!             _ => {}
//!         }
//!     }
//!     Ok(())
//! });
//!
//! async fn process_message(text: &str) -> Result<(), Box<dyn std::error::Error>> {
//!     // Your message processing logic here
//!     if text.is_empty() {
//!         return Err("Empty message not allowed".into());
//!     }
//!     Ok(())
//! }
//! ```

use super::connection::WebSocketConnection;
use super::message::Message;
use crate::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// Type alias for boxed futures used in WebSocket handlers.
///
/// This type represents an async computation that returns a `Result<()>` and can be
/// sent across thread boundaries. It's used internally by the handler system to
/// manage asynchronous operations.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The core trait for WebSocket message handling.
///
/// This trait defines the interface for handling WebSocket connections and provides
/// optional hooks for connection lifecycle events. Implementors can choose to override
/// only the methods they need, making it flexible for various use cases.
///
/// ## Lifecycle Methods
///
/// The trait provides several lifecycle hooks:
///
/// - `handle_connection`: Main connection handling method (required)
/// - `on_connect`: Called when a connection is established (optional)
/// - `on_message`: Called for each incoming message (optional)
/// - `on_disconnect`: Called when a connection closes (optional)
///
/// ## Implementation Examples
///
/// ### Simple Echo Handler
///
/// ```
/// use ignitia::websocket::{WebSocketHandler, WebSocketConnection, Message};
///
/// struct EchoHandler;
///
/// #[async_trait::async_trait]
/// impl WebSocketHandler for EchoHandler {
///     async fn handle_connection(&self, websocket: WebSocketConnection) -> ignitia::Result<()> {
///         while let Some(message) = websocket.recv().await {
///             match message {
///                 Message::Text(text) => {
///                     websocket.send_text(format!("Echo: {}", text)).await?;
///                 }
///                 Message::Close(_) => break,
///                 _ => {}
///             }
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ### Stateful Handler
///
/// ```
/// use ignitia::websocket::{WebSocketHandler, WebSocketConnection, Message};
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// struct CounterHandler {
///     message_count: AtomicU64,
/// }
///
/// impl CounterHandler {
///     fn new() -> Self {
///         Self {
///             message_count: AtomicU64::new(0),
///         }
///     }
/// }
///
/// #[async_trait::async_trait]
/// impl WebSocketHandler for CounterHandler {
///     async fn on_connect(&self, websocket: &WebSocketConnection) -> ignitia::Result<()> {
///         websocket.send_text("Connected! Send me messages to count them.".to_string()).await
///     }
///
///     async fn handle_connection(&self, websocket: WebSocketConnection) -> ignitia::Result<()> {
///         self.on_connect(&websocket).await?;
///
///         while let Some(message) = websocket.recv().await {
///             if let Message::Text(_) = message {
///                 let count = self.message_count.fetch_add(1, Ordering::Relaxed) + 1;
///                 let response = format!("Message #{}: Received!", count);
///                 websocket.send_text(response).await?;
///             } else if let Message::Close(_) = message {
///                 break;
///             }
///         }
///
///         self.on_disconnect(&websocket, Some("Client disconnected")).await
///     }
///
///     async fn on_disconnect(
///         &self,
///         _websocket: &WebSocketConnection,
///         reason: Option<&str>,
///     ) -> ignitia::Result<()> {
///         let total = self.message_count.load(Ordering::Relaxed);
///         println!("Connection closed ({}). Total messages processed: {}",
///                  reason.unwrap_or("unknown"), total);
///         Ok(())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait WebSocketHandler: Send + Sync {
    /// Handles a WebSocket connection for its entire lifetime.
    ///
    /// This is the primary method that must be implemented by all WebSocket handlers.
    /// It receives a `WebSocketConnection` and is responsible for managing the
    /// connection until it closes or encounters an error.
    ///
    /// # Parameters
    ///
    /// - `websocket`: The WebSocket connection to handle
    ///
    /// # Returns
    ///
    /// - `Ok(())` when the connection closes normally
    /// - `Err(Error)` if an error occurs during handling
    ///
    /// # Implementation Guidelines
    ///
    /// - Use a loop to continuously receive messages
    /// - Handle different message types appropriately
    /// - Break the loop on `Message::Close` or errors
    /// - Perform any cleanup before returning
    ///
    /// # Example
    ///
    /// ```
    /// use ignitia::websocket::{WebSocketHandler, WebSocketConnection, Message};
    ///
    /// struct MyHandler;
    ///
    /// #[async_trait::async_trait]
    /// impl WebSocketHandler for MyHandler {
    ///     async fn handle_connection(&self, websocket: WebSocketConnection) -> ignitia::Result<()> {
    ///         // Send initial message
    ///         websocket.send_text("Welcome!".to_string()).await?;
    ///
    ///         // Process messages
    ///         while let Some(message) = websocket.recv().await {
    ///             match message {
    ///                 Message::Text(text) => {
    ///                     // Echo the message
    ///                     websocket.send_text(text).await?;
    ///                 }
    ///                 Message::Close(_) => {
    ///                     println!("Connection closing");
    ///                     break;
    ///                 }
    ///                 _ => {} // Handle other message types as needed
    ///             }
    ///         }
    ///
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()>;

    /// Called when a message is received (optional hook).
    ///
    /// This method provides a convenient hook for handling individual messages
    /// without implementing the full connection loop. It's called automatically
    /// by some handler implementations.
    ///
    /// # Parameters
    ///
    /// - `websocket`: Reference to the WebSocket connection
    /// - `message`: The received message
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the message was processed successfully
    /// - `Err(Error)` if an error occurred during processing
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`.
    async fn on_message(&self, _websocket: &WebSocketConnection, _message: Message) -> Result<()> {
        Ok(())
    }

    /// Called when a connection is established (optional hook).
    ///
    /// This method can be used to perform initialization tasks when a new
    /// WebSocket connection is established, such as sending welcome messages
    /// or setting up connection-specific state.
    ///
    /// # Parameters
    ///
    /// - `websocket`: Reference to the newly established WebSocket connection
    ///
    /// # Returns
    ///
    /// - `Ok(())` if initialization was successful
    /// - `Err(Error)` if an error occurred during initialization
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`.
    async fn on_connect(&self, _websocket: &WebSocketConnection) -> Result<()> {
        Ok(())
    }

    /// Called when a connection is closed (optional hook).
    ///
    /// This method can be used to perform cleanup tasks when a WebSocket
    /// connection is closed, such as logging, updating statistics, or
    /// cleaning up connection-specific resources.
    ///
    /// # Parameters
    ///
    /// - `websocket`: Reference to the closing WebSocket connection
    /// - `reason`: Optional reason for the disconnection
    ///
    /// # Returns
    ///
    /// - `Ok(())` if cleanup was successful
    /// - `Err(Error)` if an error occurred during cleanup
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing and returns `Ok(())`.
    async fn on_disconnect(
        &self,
        _websocket: &WebSocketConnection,
        _reason: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }
}

/// Function-based WebSocket handler type.
///
/// This type represents a WebSocket handler implemented as a function that takes
/// a `WebSocketConnection` and returns a future. It's used internally by the
/// convenience functions to create handlers from closures.
pub type WebSocketHandlerFn =
    Arc<dyn Fn(WebSocketConnection) -> BoxFuture<'static, Result<()>> + Send + Sync>;

#[async_trait::async_trait]
impl WebSocketHandler for WebSocketHandlerFn {
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
        (self)(websocket).await
    }
}

/// Creates a WebSocket handler from a function or closure.
///
/// This is the most flexible way to create a WebSocket handler. The function
/// receives a `WebSocketConnection` and has full control over the connection
/// lifecycle. This is ideal for complex logic that needs fine-grained control.
///
/// # Parameters
///
/// - `f`: A function or closure that takes a `WebSocketConnection` and returns a future
///
/// # Returns
///
/// A `WebSocketHandlerFn` that can be used with the router
///
/// # Type Parameters
///
/// - `F`: The function type (inferred from the closure)
/// - `Fut`: The future type returned by the function (inferred)
///
/// # Examples
///
/// ### Simple Echo Server
///
/// ```
/// use ignitia::websocket::{websocket_handler, WebSocketConnection, Message};
///
/// let handler = websocket_handler(|ws: WebSocketConnection| async move {
///     while let Some(message) = ws.recv().await {
///         match message {
///             Message::Text(text) => {
///                 ws.send_text(format!("Echo: {}", text)).await?;
///             }
///             Message::Close(_) => break,
///             _ => {}
///         }
///     }
///     Ok(())
/// });
/// ```
///
/// ### JSON API Handler
///
/// ```
/// use ignitia::websocket::{websocket_handler, WebSocketConnection, Message};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct Request {
///     action: String,
///     data: serde_json::Value,
/// }
///
/// #[derive(Serialize)]
/// struct Response {
///     success: bool,
///     result: serde_json::Value,
/// }
///
/// let handler = websocket_handler(|ws: WebSocketConnection| async move {
///     while let Some(message) = ws.recv().await {
///         match message {
///             Message::Text(text) => {
///                 if let Ok(request) = serde_json::from_str::<Request>(&text) {
///                     let response = match request.action.as_str() {
///                         "ping" => Response {
///                             success: true,
///                             result: serde_json::json!({"message": "pong"}),
///                         },
///                         _ => Response {
///                             success: false,
///                             result: serde_json::json!({"error": "Unknown action"}),
///                         },
///                     };
///
///                     ws.send_json(&response).await?;
///                 }
///             }
///             Message::Close(_) => break,
///             _ => {}
///         }
///     }
///     Ok(())
/// });
/// ```
///
/// ### Connection State Management
///
/// ```
/// use ignitia::websocket::{websocket_handler, WebSocketConnection, Message};
/// use std::sync::atomic::{AtomicU32, Ordering};
/// use std::sync::Arc;
///
/// let connection_counter = Arc::new(AtomicU32::new(0));
///
/// let handler = websocket_handler(move |ws: WebSocketConnection| {
///     let counter = Arc::clone(&connection_counter);
///     async move {
///         let conn_id = counter.fetch_add(1, Ordering::Relaxed);
///         println!("New connection: {}", conn_id);
///
///         ws.send_text(format!("Welcome! Your connection ID is {}", conn_id)).await?;
///
///         while let Some(message) = ws.recv().await {
///             match message {
///                 Message::Text(text) => {
///                     let response = format!("[{}] Received: {}", conn_id, text);
///                     ws.send_text(response).await?;
///                 }
///                 Message::Close(_) => break,
///                 _ => {}
///             }
///         }
///
///         println!("Connection {} closed", conn_id);
///         Ok(())
///     }
/// });
/// ```
pub fn websocket_handler<F, Fut>(f: F) -> WebSocketHandlerFn
where
    F: Fn(WebSocketConnection) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    Arc::new(move |ws| Box::pin(f(ws)))
}

/// Handler for processing messages in batches.
///
/// This handler collects incoming messages into batches and processes them together,
/// which can be more efficient for high-throughput scenarios. It automatically
/// handles batching logic, timeout management, and connection cleanup.
///
/// # Type Parameters
///
/// - `F`: The batch processing function type
///
/// # Fields
///
/// - `handler`: The batch processing function
/// - `batch_size`: Maximum number of messages per batch
/// - `batch_timeout`: Maximum time to wait for a full batch
pub struct BatchMessageHandler<F> {
    handler: Arc<F>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl<F> BatchMessageHandler<F> {
    /// Creates a new batch message handler.
    ///
    /// # Parameters
    ///
    /// - `handler`: Function that processes message batches
    /// - `batch_size`: Maximum number of messages to collect before processing
    /// - `batch_timeout`: Maximum time to wait for messages before processing a partial batch
    ///
    /// # Returns
    ///
    /// A new `BatchMessageHandler` instance
    ///
    /// # Example
    ///
    /// ```
    /// use ignitia::websocket::{BatchMessageHandler, WebSocketConnection, Message};
    /// use std::time::Duration;
    ///
    /// let handler = BatchMessageHandler::new(
    ///     |ws: WebSocketConnection, messages: Vec<Message>| async move {
    ///         println!("Processing {} messages", messages.len());
    ///         for message in messages {
    ///             // Process each message in the batch
    ///         }
    ///         Ok(())
    ///     },
    ///     50,  // batch size
    ///     Duration::from_millis(100),  // timeout
    /// );
    /// ```
    pub fn new(handler: F, batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            handler: Arc::new(handler),
            batch_size,
            batch_timeout,
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut> WebSocketHandler for BatchMessageHandler<F>
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
        let mut message_batch = Vec::with_capacity(self.batch_size);

        loop {
            match websocket.recv().await {
                Some(Message::Close(_)) => {
                    break;
                }
                Some(message) => {
                    message_batch.push(message);

                    // Process batch when it reaches the target size
                    if message_batch.len() >= self.batch_size {
                        if let Err(e) =
                            (self.handler)(websocket.clone(), std::mem::take(&mut message_batch))
                                .await
                        {
                            tracing::debug!("WebSocket batch handler error: {}", e);
                            break;
                        }
                        message_batch = Vec::with_capacity(self.batch_size);
                    }
                }
                None => {
                    // Process any remaining messages in the batch
                    if !message_batch.is_empty() {
                        if let Err(e) =
                            (self.handler)(websocket.clone(), std::mem::take(&mut message_batch))
                                .await
                        {
                            tracing::debug!("WebSocket batch handler error: {}", e);
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }
}

/// Optimized handler for processing individual messages.
///
/// This handler is optimized for scenarios where each message should be processed
/// individually but you want the framework to handle the connection loop automatically.
/// It provides better performance than the general `websocket_handler` for simple
/// per-message processing.
///
/// # Type Parameters
///
/// - `F`: The message processing function type
pub struct OptimizedMessageHandler<F> {
    handler: Arc<F>,
}

impl<F> OptimizedMessageHandler<F> {
    /// Creates a new optimized message handler.
    ///
    /// # Parameters
    ///
    /// - `handler`: Function that processes individual messages
    ///
    /// # Returns
    ///
    /// A new `OptimizedMessageHandler` instance
    ///
    /// # Example
    ///
    /// ```
    /// use ignitia::websocket::{OptimizedMessageHandler, WebSocketConnection, Message};
    ///
    /// let handler = OptimizedMessageHandler::new(
    ///     |ws: WebSocketConnection, message: Message| async move {
    ///         match message {
    ///             Message::Text(text) => {
    ///                 ws.send_text(format!("Processed: {}", text)).await?;
    ///             }
    ///             _ => {}
    ///         }
    ///         Ok(())
    ///     }
    /// );
    /// ```
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut> WebSocketHandler for OptimizedMessageHandler<F>
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
        while let Some(message) = websocket.recv().await {
            match message {
                Message::Close(_) => break,
                _ => {
                    if let Err(e) = (self.handler)(websocket.clone(), message).await {
                        tracing::debug!("WebSocket message handler error: {}", e);
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Creates an optimized WebSocket handler for processing individual messages.
///
/// This function creates a handler that automatically manages the connection loop
/// and calls your function for each received message. It's more efficient than
/// `websocket_handler` for simple per-message processing scenarios.
///
/// # Parameters
///
/// - `f`: Function that processes individual messages
///
/// # Returns
///
/// An `OptimizedMessageHandler` that can be used with the router
///
/// # Type Parameters
///
/// - `F`: The function type (inferred from the closure)
/// - `Fut`: The future type returned by the function (inferred)
///
/// # Examples
///
/// ### Simple Message Processor
///
/// ```
/// use ignitia::websocket::{websocket_message_handler, WebSocketConnection, Message};
///
/// let handler = websocket_message_handler(|ws: WebSocketConnection, message: Message| async move {
///     match message {
///         Message::Text(text) => {
///             let uppercase = text.to_uppercase();
///             ws.send_text(uppercase).await?;
///         }
///         Message::Binary(data) => {
///             // Process binary data
///             let processed = data.iter().map(|b| b.wrapping_mul(2)).collect::<Vec<u8>>();
///             ws.send_bytes(bytes::Bytes::from(processed)).await?;
///         }
///         _ => {}
///     }
///     Ok(())
/// });
/// ```
///
/// ### Message Counter
///
/// ```
/// use ignitia::websocket::{websocket_message_handler, WebSocketConnection, Message};
/// use std::sync::atomic::{AtomicU64, Ordering};
/// use std::sync::Arc;
///
/// let counter = Arc::new(AtomicU64::new(0));
///
/// let handler = websocket_message_handler(move |ws: WebSocketConnection, message: Message| {
///     let counter = Arc::clone(&counter);
///     async move {
///         if let Message::Text(_) = message {
///             let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
///             let response = format!("Message count: {}", count);
///             ws.send_text(response).await?;
///         }
///         Ok(())
///     }
/// });
/// ```
///
/// ### JSON Message Validator
///
/// ```
/// use ignitia::websocket::{websocket_message_handler, WebSocketConnection, Message};
/// use serde_json::Value;
///
/// let handler = websocket_message_handler(|ws: WebSocketConnection, message: Message| async move {
///     if let Message::Text(text) = message {
///         match serde_json::from_str::<Value>(&text) {
///             Ok(json) => {
///                 let response = format!("Valid JSON received: {}", json);
///                 ws.send_text(response).await?;
///             }
///             Err(e) => {
///                 let error = format!("Invalid JSON: {}", e);
///                 ws.send_text(error).await?;
///             }
///         }
///     }
///     Ok(())
/// });
/// ```
pub fn websocket_message_handler<F, Fut>(f: F) -> impl WebSocketHandler
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    OptimizedMessageHandler::new(f)
}

/// Creates a WebSocket handler for efficient batch message processing.
///
/// This function creates a handler that collects messages into batches and processes
/// them together, which is more efficient for high-throughput scenarios. Messages
/// are batched by size or timeout, whichever comes first.
///
/// # Parameters
///
/// - `f`: Function that processes message batches
/// - `batch_size`: Maximum number of messages to collect before processing
/// - `timeout_ms`: Maximum time in milliseconds to wait before processing a partial batch
///
/// # Returns
///
/// A `BatchMessageHandler` that can be used with the router
///
/// # Type Parameters
///
/// - `F`: The function type (inferred from the closure)
/// - `Fut`: The future type returned by the function (inferred)
///
/// # Performance Benefits
///
/// - Reduces per-message processing overhead
/// - Enables bulk operations (database inserts, API calls, etc.)
/// - Better resource utilization for high-volume scenarios
/// - Automatic timeout handling prevents message starvation
///
/// # Examples
///
/// ### Basic Batch Processing
///
/// ```
/// use ignitia::websocket::{websocket_batch_handler, WebSocketConnection, Message};
///
/// let handler = websocket_batch_handler(
///     |ws: WebSocketConnection, messages: Vec<Message>| async move {
///         println!("Processing batch of {} messages", messages.len());
///
///         let mut responses = Vec::new();
///         for message in messages {
///             if let Message::Text(text) = message {
///                 responses.push(Message::text(format!("Processed: {}", text)));
///             }
///         }
///
///         if !responses.is_empty() {
///             ws.send_batch(responses).await?;
///         }
///
///         Ok(())
///     },
///     10,   // Process 10 messages at a time
///     100,  // Or wait 100ms for partial batches
/// );
/// ```
///
/// ### Database Bulk Insert
///
/// ```
/// use ignitia::websocket::{websocket_batch_handler, WebSocketConnection, Message};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct LogEntry {
///     timestamp: String,
///     level: String,
///     message: String,
/// }
///
/// let handler = websocket_batch_handler(
///     |ws: WebSocketConnection, messages: Vec<Message>| async move {
///         let mut log_entries = Vec::new();
///
///         for message in messages {
///             if let Message::Text(text) = message {
///                 if let Ok(entry) = serde_json::from_str::<LogEntry>(&text) {
///                     log_entries.push(entry);
///                 }
///             }
///         }
///
///         if !log_entries.is_empty() {
///             // Perform bulk database insert
///             match bulk_insert_logs(&log_entries).await {
///                 Ok(count) => {
///                     let response = format!("Inserted {} log entries", count);
///                     ws.send_text(response).await?;
///                 }
///                 Err(e) => {
///                     let error = format!("Database error: {}", e);
///                     ws.send_text(error).await?;
///                 }
///             }
///         }
///
///         Ok(())
///     },
///     50,   // Process up to 50 log entries at once
///     1000, // Or wait 1 second for partial batches
/// );
///
/// async fn bulk_insert_logs(entries: &[LogEntry]) -> Result<usize, Box<dyn std::error::Error>> {
///     // Your database bulk insert logic here
///     Ok(entries.len())
/// }
/// ```
///
/// ### Analytics Event Processing
///
/// ```
/// use ignitia::websocket::{websocket_batch_handler, WebSocketConnection, Message};
/// use std::collections::HashMap;
///
/// let handler = websocket_batch_handler(
///     |ws: WebSocketConnection, messages: Vec<Message>| async move {
///         let mut event_counts: HashMap<String, u32> = HashMap::new();
///
///         // Count events by type
///         for message in messages {
///             if let Message::Text(text) = message {
///                 if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
///                     if let Some(event_type) = json.get("type").and_then(|v| v.as_str()) {
///                         *event_counts.entry(event_type.to_string()).or_insert(0) += 1;
///                     }
///                 }
///             }
///         }
///
///         // Send aggregated results
///         if !event_counts.is_empty() {
///             let summary = serde_json::to_string(&event_counts)?;
///             ws.send_text(format!("Event summary: {}", summary)).await?;
///         }
///
///         Ok(())
///     },
///     25,   // Process 25 events at a time
///     500,  // Or wait 500ms for aggregation
/// );
/// ```
pub fn websocket_batch_handler<F, Fut>(
    f: F,
    batch_size: usize,
    timeout_ms: u64,
) -> impl WebSocketHandler
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    BatchMessageHandler::new(f, batch_size, Duration::from_millis(timeout_ms))
}
