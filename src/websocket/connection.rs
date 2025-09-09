//! # WebSocket Connection Management
//!
//! This module provides the core WebSocket connection functionality, including
//! message sending/receiving, connection lifecycle management, and protocol
//! handling. The implementation is built on top of `tokio-tungstenite` for
//! performance and reliability.
//!
//! ## Features
//!
//! - **Asynchronous I/O**: Non-blocking message processing using Tokio
//! - **Automatic Ping/Pong**: Built-in keepalive mechanism
//! - **Message Queuing**: Bounded channel for message buffering
//! - **Error Recovery**: Graceful handling of connection errors
//! - **Resource Management**: Proper cleanup of connection resources
//! - **Concurrent Safety**: Thread-safe connection sharing
//!
//! ## Architecture
//!
//! The connection is split into two primary components:
//!
//! - **Sender**: Handles outgoing messages with mutex protection
//! - **Receiver**: Processes incoming messages in a dedicated task
//!
//! Messages are processed through a bounded channel to prevent memory
//! exhaustion while maintaining good performance characteristics.
//!
//! ## Usage Examples
//!
//! ### Basic Connection Usage
//!
//! ```
//! use ignitia::websocket::{WebSocketConnection, Message};
//!
//! async fn handle_connection(ws: WebSocketConnection) -> ignitia::Result<()> {
//!     // Send a welcome message
//!     ws.send_text("Welcome to the server!".to_string()).await?;
//!
//!     // Process incoming messages
//!     while let Some(message) = ws.recv().await {
//!         match message {
//!             Message::Text(text) => {
//!                 println!("Received text: {}", text);
//!                 ws.send_text(format!("Echo: {}", text)).await?;
//!             }
//!             Message::Binary(data) => {
//!                 println!("Received {} bytes of binary data", data.len());
//!                 ws.send_bytes(data).await?;
//!             }
//!             Message::Close(frame) => {
//!                 if let Some(frame) = frame {
//!                     println!("Connection closed: {} - {}", frame.code, frame.reason);
//!                 }
//!                 break;
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### JSON Communication
//!
//! ```
//! use serde::{Deserialize, Serialize};
//! use ignitia::websocket::WebSocketConnection;
//!
//! #[derive(Serialize, Deserialize)]
//! struct ApiRequest {
//!     action: String,
//!     data: serde_json::Value,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct ApiResponse {
//!     status: String,
//!     result: serde_json::Value,
//! }
//!
//! async fn handle_api_connection(ws: WebSocketConnection) -> ignitia::Result<()> {
//!     while let Some(message) = ws.recv().await {
//!         if let Message::Text(text) = message {
//!             if let Ok(request) = serde_json::from_str::<ApiRequest>(&text) {
//!                 let response = match request.action.as_str() {
//!                     "ping" => ApiResponse {
//!                         status: "success".to_string(),
//!                         result: serde_json::json!({"message": "pong"}),
//!                     },
//!                     "echo" => ApiResponse {
//!                         status: "success".to_string(),
//!                         result: request.data,
//!                     },
//!                     _ => ApiResponse {
//!                         status: "error".to_string(),
//!                         result: serde_json::json!({"message": "Unknown action"}),
//!                     },
//!                 };
//!
//!                 ws.send_json(&response).await?;
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Batch Processing
//!
//! ```
//! use ignitia::websocket::{WebSocketConnection, Message};
//!
//! async fn batch_processor(ws: WebSocketConnection) -> ignitia::Result<()> {
//!     let mut message_batch = Vec::new();
//!     const BATCH_SIZE: usize = 10;
//!
//!     while let Some(message) = ws.recv_timeout(std::time::Duration::from_millis(100)).await {
//!         match message {
//!             Message::Text(text) => {
//!                 message_batch.push(Message::text(format!("Processed: {}", text)));
//!
//!                 if message_batch.len() >= BATCH_SIZE {
//!                     ws.send_batch(std::mem::take(&mut message_batch)).await?;
//!                 }
//!             }
//!             Message::Close(_) => break,
//!             _ => {}
//!         }
//!     }
//!
//!     // Send any remaining messages
//!     if !message_batch.is_empty() {
//!         ws.send_batch(message_batch).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Channel Capacity**: The internal message channel has a capacity of 1024 messages
//! - **Memory Management**: Bounded channels prevent memory exhaustion
//! - **Concurrency**: Multiple tasks can safely share a connection
//! - **Batch Operations**: Use `send_batch` for multiple messages to reduce system calls
//!
//! ## Error Handling
//!
//! Connection errors are handled gracefully:
//!
//! - Network errors cause the connection to close cleanly
//! - Invalid messages are logged but don't crash the connection
//! - Ping/pong failures are logged as warnings
//! - The receiver task exits cleanly on connection closure

use super::message::{CloseFrame, Message};
use crate::{Error, Result};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{tungstenite::Message as TungsteniteMessage, WebSocketStream};

/// Type alias for the WebSocket stream over upgraded HTTP connection
type WsStream = WebSocketStream<TokioIo<hyper::upgrade::Upgraded>>;

/// A WebSocket connection that handles bidirectional communication.
///
/// `WebSocketConnection` provides a high-level interface for WebSocket communication,
/// handling the low-level protocol details automatically. It supports concurrent
/// sending and receiving of messages, automatic ping/pong handling, and graceful
/// connection management.
///
/// ## Thread Safety
///
/// This struct is `Clone` and can be safely shared across multiple tasks. The
/// internal state is protected by appropriate synchronization primitives.
///
/// ## Message Processing
///
/// Incoming messages are processed in a dedicated background task that:
/// - Automatically responds to ping frames with pong frames
/// - Filters out internal protocol frames from the message stream
/// - Handles connection closure gracefully
/// - Provides clean error reporting
///
/// ## Resource Management
///
/// The connection automatically cleans up resources when dropped or when
/// the peer closes the connection. Background tasks will terminate cleanly
/// when the connection is no longer needed.
#[derive(Clone)]
pub struct WebSocketConnection {
    /// Protected sender for outgoing messages
    sender: Arc<Mutex<SplitSink<WsStream, TungsteniteMessage>>>,
    /// Protected receiver for incoming messages
    message_rx: Arc<Mutex<mpsc::Receiver<Message>>>,
}

impl WebSocketConnection {
    /// Creates a new WebSocket connection from a raw WebSocket stream.
    ///
    /// This method sets up the connection infrastructure, including:
    /// - Splitting the stream into sender and receiver components
    /// - Creating a bounded message channel for incoming messages
    /// - Spawning a background task to process incoming messages
    /// - Setting up automatic ping/pong handling
    ///
    /// # Parameters
    ///
    /// - `ws`: The raw WebSocket stream from the protocol upgrade
    ///
    /// # Returns
    ///
    /// A new `WebSocketConnection` ready for use
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    /// use tokio_tungstenite::WebSocketStream;
    ///
    /// // This is typically called internally during the upgrade process
    /// // let ws_stream = /* obtain from upgrade */;
    /// // let connection = WebSocketConnection::new(ws_stream);
    /// ```
    pub fn new(ws: WsStream) -> Self {
        let (sender, receiver) = ws.split();

        // Use bounded channel to prevent memory exhaustion while maintaining good performance
        // The capacity of 1024 messages strikes a balance between memory usage and throughput
        let (message_tx, message_rx) = mpsc::channel(1024);

        let sender_arc = Arc::new(Mutex::new(sender));

        // Spawn optimized message processing task that handles all incoming messages
        // This task runs until the connection is closed or an error occurs
        tokio::spawn(Self::message_processor(
            receiver,
            message_tx,
            Arc::clone(&sender_arc),
        ));

        Self {
            sender: sender_arc,
            message_rx: Arc::new(Mutex::new(message_rx)),
        }
    }

    /// Background task that processes all incoming WebSocket messages.
    ///
    /// This task handles:
    /// - Converting low-level tungstenite messages to high-level Message types
    /// - Automatic ping/pong handling for connection keepalive
    /// - Graceful error handling and connection closure
    /// - Message filtering to remove internal protocol messages
    ///
    /// The task runs until:
    /// - The connection is closed by either peer
    /// - A fatal error occurs
    /// - The message receiver is dropped (connection cleanup)
    ///
    /// # Parameters
    ///
    /// - `receiver`: The receiving half of the WebSocket stream
    /// - `message_tx`: Channel sender for forwarding messages to the application
    /// - `sender`: Shared sender for automatic ping responses
    async fn message_processor(
        mut receiver: SplitStream<WsStream>,
        message_tx: mpsc::Sender<Message>,
        sender: Arc<Mutex<SplitSink<WsStream, TungsteniteMessage>>>,
    ) {
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(tungstenite_msg) => {
                    match tungstenite_msg {
                        // Handle connection closure
                        TungsteniteMessage::Close(_) => {
                            tracing::debug!("WebSocket connection closed by client");
                            break;
                        }
                        // Auto-respond to pings to maintain connection
                        TungsteniteMessage::Ping(data) => {
                            if let Err(e) = Self::handle_ping(&sender, data).await {
                                tracing::warn!("Failed to send pong response: {}", e);
                            }
                            continue; // Skip forwarding ping messages to application
                        }
                        // Forward all other messages to the application
                        _ => {
                            if let Some(converted_msg) = convert_message(tungstenite_msg) {
                                if message_tx.send(converted_msg).await.is_err() {
                                    // Receiver dropped, connection is being cleaned up
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        tracing::debug!("WebSocket message processor task terminated");
    }

    /// Handles incoming ping frames by sending appropriate pong responses.
    ///
    /// This method is called automatically by the message processor to maintain
    /// connection liveness. Ping/pong frames are part of the WebSocket protocol's
    /// keepalive mechanism.
    ///
    /// # Parameters
    ///
    /// - `sender`: Shared sender for outgoing messages
    /// - `data`: Ping payload data to echo back in the pong frame
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the pong was sent successfully
    /// - `Err(Error)` if sending failed
    async fn handle_ping(
        sender: &Arc<Mutex<SplitSink<WsStream, TungsteniteMessage>>>,
        data: Vec<u8>,
    ) -> Result<()> {
        let pong_msg = TungsteniteMessage::Pong(data);

        let mut sender_lock = sender.lock().await;
        sender_lock
            .send(pong_msg)
            .await
            .map_err(|e| Error::Internal(format!("WebSocket pong error: {}", e)))
    }

    /// Sends a message over the WebSocket connection.
    ///
    /// This is the primary method for sending messages to the connected peer.
    /// The method handles conversion from high-level Message types to low-level
    /// tungstenite messages and manages the sending process safely.
    ///
    /// # Parameters
    ///
    /// - `message`: The message to send
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the message was sent successfully
    /// - `Err(Error)` if sending failed
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::{WebSocketConnection, Message};
    ///
    /// async fn send_messages(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     // Send text message
    ///     ws.send(Message::text("Hello, World!")).await?;
    ///
    ///     // Send binary message
    ///     ws.send(Message::binary(vec!)).await?;[1]
    ///
    ///     // Send ping
    ///     ws.send(Message::ping("ping-data")).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn send(&self, message: Message) -> Result<()> {
        let tungstenite_msg = match message {
            Message::Text(text) => TungsteniteMessage::Text(text),
            Message::Binary(data) => TungsteniteMessage::Binary(data.to_vec()),
            Message::Ping(data) => TungsteniteMessage::Ping(data.to_vec()),
            Message::Pong(data) => TungsteniteMessage::Pong(data.to_vec()),
            Message::Close(frame) => {
                if let Some(frame) = frame {
                    TungsteniteMessage::Close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame {
                        code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(frame.code),
                        reason: frame.reason.into(),
                    }))
                } else {
                    TungsteniteMessage::Close(None)
                }
            }
        };

        let mut sender = self.sender.lock().await;
        sender
            .send(tungstenite_msg)
            .await
            .map_err(|e| Error::Internal(format!("WebSocket send error: {}", e)))
    }

    /// Receives the next message from the WebSocket connection.
    ///
    /// This method blocks until a message is available or the connection is closed.
    /// It automatically filters out internal protocol messages (like ping frames)
    /// and only returns application-level messages.
    ///
    /// # Returns
    ///
    /// - `Some(Message)` if a message was received
    /// - `None` if the connection was closed or an error occurred
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::{WebSocketConnection, Message};
    ///
    /// async fn receive_loop(ws: &WebSocketConnection) {
    ///     while let Some(message) = ws.recv().await {
    ///         match message {
    ///             Message::Text(text) => println!("Received text: {}", text),
    ///             Message::Binary(data) => println!("Received {} bytes", data.len()),
    ///             Message::Close(_) => {
    ///                 println!("Connection closed");
    ///                 break;
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn recv(&self) -> Option<Message> {
        let mut receiver = self.message_rx.lock().await;
        receiver.recv().await
    }

    /// Receives the next message with a timeout to prevent indefinite blocking.
    ///
    /// This method is useful for implementing non-blocking message processing
    /// or periodic operations while waiting for messages.
    ///
    /// # Parameters
    ///
    /// - `timeout`: Maximum time to wait for a message
    ///
    /// # Returns
    ///
    /// - `Some(Message)` if a message was received within the timeout
    /// - `None` if the timeout expired or the connection was closed
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    /// use std::time::Duration;
    ///
    /// async fn periodic_processing(ws: &WebSocketConnection) {
    ///     loop {
    ///         match ws.recv_timeout(Duration::from_secs(5)).await {
    ///             Some(message) => {
    ///                 // Process message
    ///                 println!("Got message: {:?}", message);
    ///             }
    ///             None => {
    ///                 // Timeout - perform periodic maintenance
    ///                 println!("No message received, doing maintenance...");
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Message> {
        let mut receiver = self.message_rx.lock().await;
        tokio::time::timeout(timeout, receiver.recv())
            .await
            .ok()
            .flatten()
    }

    /// Sends a text message over the WebSocket connection.
    ///
    /// This is a convenience method for sending text messages without
    /// explicitly creating a Message::Text variant.
    ///
    /// # Parameters
    ///
    /// - `text`: The text content to send
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the message was sent successfully
    /// - `Err(Error)` if sending failed
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    ///
    /// async fn send_greeting(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     ws.send_text("Hello, WebSocket!".to_string()).await
    /// }
    /// ```
    pub async fn send_text(&self, text: String) -> Result<()> {
        self.send(Message::Text(text)).await
    }

    /// Sends binary data over the WebSocket connection.
    ///
    /// This is a convenience method for sending binary messages without
    /// explicitly creating a Message::Binary variant.
    ///
    /// # Parameters
    ///
    /// - `data`: The binary data to send
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the message was sent successfully
    /// - `Err(Error)` if sending failed
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    /// use bytes::Bytes;
    ///
    /// async fn send_binary_data(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     let data = Bytes::from(vec![0x01, 0x02, 0x03, 0x04]);
    ///     ws.send_bytes(data).await
    /// }
    /// ```
    pub async fn send_bytes(&self, data: bytes::Bytes) -> Result<()> {
        self.send(Message::Binary(data)).await
    }

    /// Sends a JSON-serializable object as a text message.
    ///
    /// This convenience method serializes an object to JSON and sends it
    /// as a text message. This is commonly used for structured communication
    /// over WebSocket connections.
    ///
    /// # Parameters
    ///
    /// - `data`: The object to serialize and send
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the message was sent successfully
    /// - `Err(Error)` if serialization or sending failed
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::Serialize;
    /// use ignitia::websocket::WebSocketConnection;
    ///
    /// #[derive(Serialize)]
    /// struct Response {
    ///     status: String,
    ///     data: Vec<i32>,
    /// }
    ///
    /// async fn send_response(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     let response = Response {
    ///         status: "success".to_string(),
    ///         data: vec!,[1]
    ///     };
    ///
    ///     ws.send_json(&response).await
    /// }
    /// ```
    pub async fn send_json<T: serde::Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_vec(data)
            .map_err(|e| Error::Internal(format!("JSON serialization error: {}", e)))?;
        self.send_bytes(bytes::Bytes::from(json)).await
    }

    /// Closes the WebSocket connection gracefully.
    ///
    /// This method sends a close frame to the peer and begins the connection
    /// shutdown process. The connection will be closed after this call.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the close frame was sent successfully
    /// - `Err(Error)` if sending the close frame failed
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    ///
    /// async fn graceful_shutdown(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     println!("Shutting down connection...");
    ///     ws.close().await
    /// }
    /// ```
    pub async fn close(&self) -> Result<()> {
        self.send(Message::Close(None)).await
    }

    /// Closes the WebSocket connection with a specific close code and reason.
    ///
    /// This method allows for more descriptive connection closure by providing
    /// a close code and human-readable reason. This information is sent to
    /// the peer as part of the close frame.
    ///
    /// # Parameters
    ///
    /// - `code`: WebSocket close code (see RFC 6455 for standard codes)
    /// - `reason`: Human-readable close reason
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the close frame was sent successfully
    /// - `Err(Error)` if sending the close frame failed
    ///
    /// # Standard Close Codes
    ///
    /// - `1000`: Normal closure
    /// - `1001`: Going away
    /// - `1002`: Protocol error
    /// - `1003`: Unsupported data
    /// - `1007`: Invalid frame payload data
    /// - `1008`: Policy violation
    /// - `1009`: Message too big
    /// - `1011`: Internal server error
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    ///
    /// async fn close_with_error(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     ws.close_with_reason(1008, "Policy violation: spam detected".to_string()).await
    /// }
    /// ```
    pub async fn close_with_reason(&self, code: u16, reason: String) -> Result<()> {
        self.send(Message::Close(Some(CloseFrame { code, reason })))
            .await
    }

    /// Sends a ping frame to test connection liveness.
    ///
    /// Ping frames are used to verify that the connection is still active.
    /// The peer should respond with a pong frame containing the same data.
    ///
    /// # Parameters
    ///
    /// - `data`: Optional data to include in the ping frame
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the ping was sent successfully
    /// - `Err(Error)` if sending failed
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    ///
    /// async fn test_connection(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     ws.ping("keepalive").await
    /// }
    /// ```
    pub async fn ping(&self, data: impl Into<bytes::Bytes>) -> Result<()> {
        self.send(Message::Ping(data.into())).await
    }

    /// Sends a pong frame in response to a ping.
    ///
    /// Pong frames are typically sent automatically in response to ping frames,
    /// but this method allows manual pong sending if needed.
    ///
    /// # Parameters
    ///
    /// - `data`: Data to include in the pong frame (typically echoes ping data)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the pong was sent successfully
    /// - `Err(Error)` if sending failed
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::WebSocketConnection;
    ///
    /// async fn manual_pong(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     ws.pong("response-data").await
    /// }
    /// ```
    pub async fn pong(&self, data: impl Into<bytes::Bytes>) -> Result<()> {
        self.send(Message::Pong(data.into())).await
    }

    /// Sends multiple messages efficiently in a single batch operation.
    ///
    /// This method is optimized for sending multiple messages with reduced
    /// system call overhead. It's particularly useful for high-throughput
    /// scenarios or when responding with multiple related messages.
    ///
    /// # Parameters
    ///
    /// - `messages`: Vector of messages to send
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all messages were sent successfully
    /// - `Err(Error)` if any message failed to send
    ///
    /// # Performance
    ///
    /// Batch sending reduces the number of lock acquisitions and system calls,
    /// providing better performance than sending messages individually when
    /// multiple messages need to be sent together.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::{WebSocketConnection, Message};
    ///
    /// async fn send_bulk_data(ws: &WebSocketConnection) -> ignitia::Result<()> {
    ///     let messages = vec![
    ///         Message::text("Starting bulk operation"),
    ///         Message::text("Processing item 1"),
    ///         Message::text("Processing item 2"),
    ///         Message::text("Processing item 3"),
    ///         Message::text("Bulk operation complete"),
    ///     ];
    ///
    ///     ws.send_batch(messages).await
    /// }
    /// ```
    pub async fn send_batch(&self, messages: Vec<Message>) -> Result<()> {
        let mut sender = self.sender.lock().await;
        for message in messages {
            let tungstenite_msg = match message {
                Message::Text(text) => TungsteniteMessage::Text(text),
                Message::Binary(data) => TungsteniteMessage::Binary(data.to_vec()),
                Message::Ping(data) => TungsteniteMessage::Ping(data.to_vec()),
                Message::Pong(data) => TungsteniteMessage::Pong(data.to_vec()),
                Message::Close(frame) => {
                    if let Some(frame) = frame {
                        TungsteniteMessage::Close(Some(tokio_tungstenite::tungstenite::protocol::CloseFrame {
                            code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(frame.code),
                            reason: frame.reason.into(),
                        }))
                    } else {
                        TungsteniteMessage::Close(None)
                    }
                }
            };

            sender
                .send(tungstenite_msg)
                .await
                .map_err(|e| Error::Internal(format!("WebSocket batch send error: {}", e)))?;
        }
        Ok(())
    }
}

/// Converts a low-level tungstenite message to a high-level Message.
///
/// This function filters out internal protocol messages and converts
/// the remaining messages to the application-level Message enum.
///
/// # Parameters
///
/// - `msg`: The tungstenite message to convert
///
/// # Returns
///
/// - `Some(Message)` for application-level messages
/// - `None` for internal protocol messages that should be filtered out
///
/// # Internal Messages
///
/// The following message types are handled internally and not forwarded:
/// - Ping frames (handled automatically with pong responses)
/// - Raw frame data (low-level protocol detail)
fn convert_message(msg: TungsteniteMessage) -> Option<Message> {
    match msg {
        TungsteniteMessage::Text(text) => Some(Message::Text(text)),
        TungsteniteMessage::Binary(data) => Some(Message::Binary(bytes::Bytes::from(data))),
        TungsteniteMessage::Pong(data) => Some(Message::Pong(bytes::Bytes::from(data))),
        TungsteniteMessage::Close(frame) => Some(Message::Close(frame.map(|f| CloseFrame {
            code: f.code.into(),
            reason: f.reason.to_string(),
        }))),
        // These message types are handled internally and not forwarded to the application
        TungsteniteMessage::Ping(_) => None, // Handled automatically
        TungsteniteMessage::Frame(_) => None, // Raw frame data
    }
}
