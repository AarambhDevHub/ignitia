//! # WebSocket Support Module
//!
//! This module provides comprehensive WebSocket functionality for the Ignitia web framework,
//! including connection management, message handling, and protocol upgrades. The WebSocket
//! implementation follows RFC 6455 standards and provides both low-level and high-level
//! APIs for real-time bidirectional communication.
//!
//! ## Features
//!
//! - **Protocol Compliance**: Full RFC 6455 WebSocket protocol implementation
//! - **Connection Management**: Automatic connection lifecycle handling
//! - **Message Types**: Support for text, binary, ping, pong, and close messages
//! - **Handler Patterns**: Multiple handler patterns for different use cases
//! - **Performance Optimized**: Efficient message processing with batching support
//! - **Error Handling**: Comprehensive error handling and recovery
//! - **Graceful Shutdown**: Proper connection cleanup and resource management
//!
//! ## Architecture
//!
//! The WebSocket module is organized into several key components:
//!
//! - **Connection**: Low-level WebSocket connection management
//! - **Handler**: High-level message and event handling
//! - **Message**: Message types and serialization
//! - **Upgrade**: HTTP to WebSocket protocol upgrade handling
//!
//! ## Usage Examples
//!
//! ### Basic WebSocket Server
//!
//! ```
//! use ignitia::{Router, Server, websocket::*};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .websocket("/ws", websocket_handler(|ws: WebSocketConnection| async move {
//!             while let Some(message) = ws.recv().await {
//!                 match message {
//!                     Message::Text(text) => {
//!                         println!("Received: {}", text);
//!                         ws.send_text(format!("Echo: {}", text)).await?;
//!                     }
//!                     Message::Binary(data) => {
//!                         println!("Received {} bytes", data.len());
//!                         ws.send_bytes(data).await?;
//!                     }
//!                     Message::Close(_) => break,
//!                     _ => {}
//!                 }
//!             }
//!             Ok(())
//!         }));
//!
//!     let addr: SocketAddr = "127.0.0.1:3000".parse()?;
//!     let server = Server::new(router, addr);
//!     server.ignitia().await
//! }
//! ```
//!
//! ### JSON Message Handling
//!
//! ```
//! use serde::{Deserialize, Serialize};
//! use ignitia::{Router, websocket::*};
//!
//! #[derive(Serialize, Deserialize)]
//! struct ChatMessage {
//!     user: String,
//!     message: String,
//!     timestamp: u64,
//! }
//!
//! let router = Router::new()
//!     .websocket("/chat", websocket_handler(|ws: WebSocketConnection| async move {
//!         while let Some(message) = ws.recv().await {
//!             if let Message::Text(text) = message {
//!                 if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
//!                     println!("Chat from {}: {}", chat_msg.user, chat_msg.message);
//!
//!                     // Broadcast to other clients (in real app, you'd maintain client list)
//!                     let response = ChatMessage {
//!                         user: "Server".to_string(),
//!                         message: format!("Received: {}", chat_msg.message),
//!                         timestamp: std::time::SystemTime::now()
//!                             .duration_since(std::time::UNIX_EPOCH)
//!                             .unwrap()
//!                             .as_secs(),
//!                     };
//!
//!                     ws.send_json(&response).await?;
//!                 }
//!             }
//!         }
//!         Ok(())
//!     }));
//! ```
//!
//! ### Batch Message Processing
//!
//! ```
//! use ignitia::{Router, websocket::*};
//!
//! let router = Router::new()
//!     .websocket("/batch", websocket_batch_handler(
//!         |ws: WebSocketConnection, messages: Vec<Message>| async move {
//!             println!("Processing batch of {} messages", messages.len());
//!
//!             let mut responses = Vec::new();
//!             for msg in messages {
//!                 if let Message::Text(text) = msg {
//!                     responses.push(Message::text(format!("Processed: {}", text)));
//!                 }
//!             }
//!
//!             ws.send_batch(responses).await?;
//!             Ok(())
//!         },
//!         10,    // batch size
//!         100,   // timeout in milliseconds
//!     ));
//! ```
//!
//! ## Handler Types
//!
//! The module provides several handler types for different use cases:
//!
//! - **Simple Handler**: `websocket_handler` - Basic message-by-message processing
//! - **Message Handler**: `websocket_message_handler` - Optimized single message processing
//! - **Batch Handler**: `websocket_batch_handler` - Efficient batch processing
//!
//! ## Message Types
//!
//! WebSocket messages support various formats:
//!
//! - **Text**: UTF-8 encoded string messages
//! - **Binary**: Raw byte data
//! - **Ping/Pong**: Connection keepalive frames
//! - **Close**: Connection termination with optional reason
//!
//! ## Performance Considerations
//!
//! - Use batch processing for high-throughput scenarios
//! - Implement connection pooling for multiple clients
//! - Consider message size limits to prevent memory exhaustion
//! - Use binary messages for non-text data to avoid encoding overhead
//!
//! ## Security Notes
//!
//! - Validate all incoming messages
//! - Implement rate limiting to prevent abuse
//! - Use secure WebSocket (WSS) in production
//! - Sanitize data before broadcasting to other clients

#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod connection;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod handler;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod message;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod upgrade;

// Export all WebSocket functionality when the feature is enabled
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use connection::WebSocketConnection;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use handler::{
    universal_ws_handler, websocket_batch_handler, websocket_handler, websocket_message_handler,
    BatchMessageHandler, OptimizedMessageHandler, UniversalWebSocketHandler, WebSocketHandler,
};
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use message::{CloseFrame, Message, MessageType};
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use upgrade::{handle_websocket_upgrade, is_websocket_request, upgrade_connection};

// Stub implementations when WebSocket feature is not enabled
// These provide helpful error messages at runtime
#[cfg(not(feature = "websocket"))]
pub struct WebSocketConnection;
#[cfg(not(feature = "websocket"))]
pub struct Message;
#[cfg(not(feature = "websocket"))]
pub struct MessageType;
#[cfg(not(feature = "websocket"))]
pub struct CloseFrame;

#[cfg(not(feature = "websocket"))]
pub trait WebSocketHandler {}

/// Creates a WebSocket handler when the feature is disabled.
///
/// # Panics
///
/// This function will panic with an informative message when called
/// without the `websocket` feature enabled.
#[cfg(not(feature = "websocket"))]
pub fn websocket_handler<F>(_f: F) -> impl WebSocketHandler {
    panic!("WebSocket support is not enabled. Add 'websocket' feature to your Cargo.toml");
}

/// Checks if a request is a WebSocket upgrade request when the feature is disabled.
///
/// # Returns
///
/// Always returns `false` when the WebSocket feature is not enabled.
#[cfg(not(feature = "websocket"))]
pub fn is_websocket_request(_req: &crate::Request) -> bool {
    false
}

/// Attempts to upgrade a connection to WebSocket when the feature is disabled.
///
/// # Returns
///
/// Always returns an error indicating WebSocket support is not available.
#[cfg(not(feature = "websocket"))]
pub fn upgrade_connection(_req: crate::Request) -> crate::Result<crate::Response> {
    Err(crate::Error::Internal("WebSocket not supported".into()))
}
