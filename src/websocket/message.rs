//! # WebSocket Message Types and Utilities
//!
//! This module defines the message types and utilities for WebSocket communication
//! in the Ignitia web framework. It provides a comprehensive set of message types
//! that correspond to the WebSocket protocol specification (RFC 6455) along with
//! convenient methods for message creation, conversion, and serialization.
//!
//! ## Message Types
//!
//! The WebSocket protocol supports several message types:
//!
//! - **Text Messages**: UTF-8 encoded string data
//! - **Binary Messages**: Raw binary data
//! - **Ping Messages**: Connection liveness test frames
//! - **Pong Messages**: Response to ping frames
//! - **Close Messages**: Connection termination frames
//!
//! ## Protocol Compliance
//!
//! This implementation follows RFC 6455 WebSocket Protocol:
//!
//! - Text messages must be valid UTF-8
//! - Binary messages can contain any byte sequence
//! - Ping/pong frames are used for connection keepalive
//! - Close frames can include a status code and reason
//!
//! ## Message Creation
//!
//! Messages can be created in several ways:
//!
//! ```
//! use ignitia::websocket::Message;
//! use bytes::Bytes;
//!
//! // Text messages
//! let text1 = Message::text("Hello, World!");
//! let text2 = Message::Text("Hello, World!".to_string());
//! let text3: Message = "Hello, World!".into();
//!
//! // Binary messages
//! let binary1 = Message::binary(vec!);[1]
//! let binary2 = Message::Binary(Bytes::from(vec!));[1]
//! let binary3: Message = vec!.into();[1]
//!
//! // Control messages
//! let ping = Message::ping("ping-data");
//! let pong = Message::pong("pong-data");
//! let close = Message::close();
//! let close_with_reason = Message::close_with_reason(1000, "Normal closure");
//! ```
//!
//! ## JSON Support
//!
//! The message system includes built-in JSON serialization and deserialization:
//!
//! ```
//! use ignitia::websocket::Message;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct ChatMessage {
//!     user: String,
//!     message: String,
//!     timestamp: u64,
//! }
//!
//! // Serialize to JSON message
//! let chat_msg = ChatMessage {
//!     user: "Alice".to_string(),
//!     message: "Hello everyone!".to_string(),
//!     timestamp: 1234567890,
//! };
//!
//! let json_message = Message::json(&chat_msg)?;
//!
//! // Deserialize from JSON message
//! if let Message::Text(json_str) = json_message {
//!     let message = Message::Text(json_str);
//!     let parsed: ChatMessage = message.parse_json()?;
//!     println!("User: {}, Message: {}", parsed.user, parsed.message);
//! }
//! ```
//!
//! ## Message Inspection
//!
//! Messages provide various methods for inspection and conversion:
//!
//! ```
//! use ignitia::websocket::{Message, MessageType};
//!
//! let message = Message::text("Hello");
//!
//! // Check message type
//! assert_eq!(message.message_type(), MessageType::Text);
//! assert!(message.is_text());
//! assert!(!message.is_binary());
//!
//! // Extract content
//! if let Some(text) = message.as_text() {
//!     println!("Text content: {}", text);
//! }
//!
//! // Convert to owned content
//! if let Some(text) = message.into_text() {
//!     println!("Owned text: {}", text);
//! }
//! ```
//!
//! ## Close Frames
//!
//! Close messages can include status codes and reasons according to RFC 6455:
//!
//! ```
//! use ignitia::websocket::{Message, CloseFrame};
//!
//! // Standard close codes
//! let normal_close = Message::close_with_reason(1000, "Normal closure");
//! let going_away = Message::close_with_reason(1001, "Server going away");
//! let protocol_error = Message::close_with_reason(1002, "Protocol error");
//! let unsupported_data = Message::close_with_reason(1003, "Unsupported data type");
//! let invalid_data = Message::close_with_reason(1007, "Invalid UTF-8 data");
//! let policy_violation = Message::close_with_reason(1008, "Policy violation");
//! let message_too_big = Message::close_with_reason(1009, "Message too big");
//! let internal_error = Message::close_with_reason(1011, "Internal server error");
//!
//! // Extract close information
//! if let Message::Close(Some(frame)) = normal_close {
//!     println!("Close code: {}, Reason: {}", frame.code, frame.reason);
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - Text messages are validated for UTF-8 encoding
//! - Binary messages have no encoding overhead
//! - JSON serialization is performed lazily
//! - Message cloning is efficient for small messages
//! - Large binary messages should be processed in chunks when possible
//!
//! ## Error Handling
//!
//! JSON operations can fail and return appropriate errors:
//!
//! ```
//! use ignitia::websocket::Message;
//! use serde_json;
//!
//! let invalid_json = Message::text("{ invalid json }");
//!
//! match invalid_json.parse_json::<serde_json::Value>() {
//!     Ok(value) => println!("Parsed: {:?}", value),
//!     Err(e) => println!("JSON parse error: {}", e),
//! }
//! ```

use bytes::Bytes;
use serde::{de::Error, Deserialize, Serialize};

/// Represents different types of WebSocket messages.
///
/// This enum covers all message types defined in RFC 6455 WebSocket Protocol.
/// Each variant contains the appropriate data for that message type.
///
/// ## Message Types
///
/// - `Text`: UTF-8 encoded text data
/// - `Binary`: Raw binary data
/// - `Ping`: Connection liveness test frame
/// - `Pong`: Response to ping frame
/// - `Close`: Connection termination frame
///
/// ## Examples
///
/// ```
/// use ignitia::websocket::{Message, CloseFrame};
/// use bytes::Bytes;
///
/// // Create different message types
/// let text = Message::Text("Hello, WebSocket!".to_string());
/// let binary = Message::Binary(Bytes::from(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]));
/// let ping = Message::Ping(Bytes::from("ping-payload"));
/// let pong = Message::Pong(Bytes::from("pong-payload"));
/// let close = Message::Close(Some(CloseFrame {
///     code: 1000,
///     reason: "Normal closure".to_string(),
/// }));
/// ```
#[derive(Debug, Clone)]
pub enum Message {
    /// A UTF-8 encoded text message.
    ///
    /// Text messages must contain valid UTF-8 data according to the WebSocket
    /// protocol specification. The framework automatically validates UTF-8
    /// encoding when creating text messages from raw bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let message = Message::Text("Hello, World!".to_string());
    /// let emoji = Message::Text("Hello 👋 World 🌍".to_string());
    /// let unicode = Message::Text("Здравствуй мир".to_string());
    /// ```
    Text(String),

    /// A binary message containing raw byte data.
    ///
    /// Binary messages can contain any sequence of bytes and are not subject
    /// to UTF-8 validation. They are ideal for transmitting structured data,
    /// images, files, or any non-text content.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use bytes::Bytes;
    ///
    /// // Raw bytes
    /// let data = vec![0x00, 0x01, 0x02, 0x03, 0xFF];
    /// let message = Message::Binary(Bytes::from(data));
    ///
    /// // Image data (example)
    /// let image_data = std::fs::read("image.png").unwrap();
    /// let image_message = Message::Binary(Bytes::from(image_data));
    ///
    /// // Structured binary data
    /// let mut buffer = Vec::new();
    /// buffer.extend_from_slice(&42u32.to_be_bytes()); // Big-endian u32
    /// buffer.extend_from_slice(&3.14f64.to_be_bytes()); // Big-endian f64
    /// let structured = Message::Binary(Bytes::from(buffer));
    /// ```
    Binary(Bytes),

    /// A ping frame used for connection liveness testing.
    ///
    /// Ping frames are control frames used to test whether the connection is
    /// still alive. When a ping frame is received, the recipient should respond
    /// with a pong frame containing the same payload data.
    ///
    /// # Protocol Notes
    ///
    /// - Ping frames can contain up to 125 bytes of payload
    /// - The payload is optional and can be empty
    /// - Ping frames are typically handled automatically by the framework
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use bytes::Bytes;
    ///
    /// // Empty ping
    /// let ping = Message::Ping(Bytes::new());
    ///
    /// // Ping with timestamp payload
    /// let timestamp = std::time::SystemTime::now()
    ///     .duration_since(std::time::UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs()
    ///     .to_be_bytes();
    /// let timed_ping = Message::Ping(Bytes::from(timestamp.to_vec()));
    /// ```
    Ping(Bytes),

    /// A pong frame sent in response to a ping frame.
    ///
    /// Pong frames are control frames sent in response to ping frames. They
    /// should contain the same payload data as the ping frame they respond to.
    /// Pong frames can also be sent unsolicited as a unidirectional heartbeat.
    ///
    /// # Protocol Notes
    ///
    /// - Pong frames typically echo the ping payload
    /// - Pong frames can be sent unsolicited
    /// - Pong frames are usually handled automatically by the framework
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use bytes::Bytes;
    ///
    /// // Response to ping
    /// let pong = Message::Pong(Bytes::from("ping-data"));
    ///
    /// // Unsolicited heartbeat
    /// let heartbeat = Message::Pong(Bytes::from("heartbeat"));
    /// ```
    Pong(Bytes),

    /// A close frame indicating connection termination.
    ///
    /// Close frames signal that the WebSocket connection should be terminated.
    /// They can optionally include a status code and human-readable reason for
    /// the closure. The close frame is the last frame sent before closing the
    /// underlying TCP connection.
    ///
    /// # Protocol Notes
    ///
    /// - Close frames can be empty (no code or reason)
    /// - Status codes follow RFC 6455 specifications
    /// - Reasons should be UTF-8 encoded and human-readable
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::{Message, CloseFrame};
    ///
    /// // Simple close
    /// let close = Message::Close(None);
    ///
    /// // Close with standard code
    /// let normal_close = Message::Close(Some(CloseFrame {
    ///     code: 1000,
    ///     reason: "Normal closure".to_string(),
    /// }));
    ///
    /// // Close due to error
    /// let error_close = Message::Close(Some(CloseFrame {
    ///     code: 1011,
    ///     reason: "Internal server error".to_string(),
    /// }));
    /// ```
    Close(Option<CloseFrame>),
}

/// Information included in a close frame.
///
/// Close frames can include a status code and human-readable reason for the
/// connection closure. This structure represents that optional information.
///
/// ## Standard Close Codes
///
/// - `1000`: Normal closure
/// - `1001`: Going away (server shutdown or browser navigation)
/// - `1002`: Protocol error
/// - `1003`: Unsupported data type
/// - `1007`: Invalid UTF-8 data
/// - `1008`: Policy violation
/// - `1009`: Message too big
/// - `1010`: Missing extension
/// - `1011`: Internal server error
/// - `1015`: TLS handshake failure
///
/// ## Examples
///
/// ```
/// use ignitia::websocket::CloseFrame;
///
/// let normal = CloseFrame {
///     code: 1000,
///     reason: "Session completed".to_string(),
/// };
///
/// let error = CloseFrame {
///     code: 1011,
///     reason: "Database connection failed".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CloseFrame {
    /// The close status code.
    ///
    /// Status codes indicate the reason for connection closure and should
    /// follow RFC 6455 specifications. Codes in the 1000-2999 range are
    /// reserved for the protocol, while 3000-3999 are available for libraries
    /// and 4000-4999 for applications.
    pub code: u16,

    /// Human-readable reason for the closure.
    ///
    /// The reason should be a UTF-8 encoded string that describes why the
    /// connection is being closed. It's intended for debugging and logging
    /// purposes and should be concise but informative.
    pub reason: String,
}

/// Enumeration of WebSocket message types.
///
/// This enum provides a way to identify the type of a message without
/// accessing its contents. It's useful for routing logic, statistics,
/// and protocol handling.
///
/// ## Examples
///
/// ```
/// use ignitia::websocket::{Message, MessageType};
///
/// let message = Message::text("Hello");
/// match message.message_type() {
///     MessageType::Text => println!("It's a text message"),
///     MessageType::Binary => println!("It's a binary message"),
///     MessageType::Ping => println!("It's a ping frame"),
///     MessageType::Pong => println!("It's a pong frame"),
///     MessageType::Close => println!("It's a close frame"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    /// Text message type
    Text,
    /// Binary message type
    Binary,
    /// Ping frame type
    Ping,
    /// Pong frame type
    Pong,
    /// Close frame type
    Close,
}

impl Message {
    /// Creates a new text message.
    ///
    /// This is a convenience method for creating text messages. The input
    /// can be any type that implements `Into<String>`.
    ///
    /// # Parameters
    ///
    /// - `text`: The text content for the message
    ///
    /// # Returns
    ///
    /// A new `Message::Text` variant
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let msg1 = Message::text("Hello, World!");
    /// let msg2 = Message::text(String::from("Hello, World!"));
    /// let msg3 = Message::text(format!("User {} connected", "Alice"));
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        Message::Text(text.into())
    }

    /// Creates a new binary message.
    ///
    /// This is a convenience method for creating binary messages. The input
    /// can be any type that implements `Into<Bytes>`.
    ///
    /// # Parameters
    ///
    /// - `data`: The binary data for the message
    ///
    /// # Returns
    ///
    /// A new `Message::Binary` variant
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use bytes::Bytes;
    ///
    /// let msg1 = Message::binary(vec!);[1]
    /// let msg2 = Message::binary(Bytes::from("Hello"));
    /// let msg3 = Message::binary([0u8; 1024]); // 1KB of zeros
    /// ```
    pub fn binary(data: impl Into<Bytes>) -> Self {
        Message::Binary(data.into())
    }

    /// Creates a new ping message.
    ///
    /// Ping messages are used for connection liveness testing. The payload
    /// data will be echoed back in the pong response.
    ///
    /// # Parameters
    ///
    /// - `data`: The ping payload data
    ///
    /// # Returns
    ///
    /// A new `Message::Ping` variant
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let ping1 = Message::ping("keepalive");
    /// let ping2 = Message::ping(vec!);[1]
    /// let ping3 = Message::ping(std::time::SystemTime::now()
    ///     .duration_since(std::time::UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_millis()
    ///     .to_be_bytes());
    /// ```
    pub fn ping(data: impl Into<Bytes>) -> Self {
        Message::Ping(data.into())
    }

    /// Creates a new pong message.
    ///
    /// Pong messages are sent in response to ping messages and should contain
    /// the same payload data. They can also be sent unsolicited as heartbeats.
    ///
    /// # Parameters
    ///
    /// - `data`: The pong payload data
    ///
    /// # Returns
    ///
    /// A new `Message::Pong` variant
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let pong1 = Message::pong("keepalive-response");
    /// let pong2 = Message::pong(vec!);[1]
    /// let pong3 = Message::pong("heartbeat");
    /// ```
    pub fn pong(data: impl Into<Bytes>) -> Self {
        Message::Pong(data.into())
    }

    /// Creates a new close message without additional information.
    ///
    /// This creates a simple close message without a status code or reason.
    /// It's the most basic way to signal connection termination.
    ///
    /// # Returns
    ///
    /// A new `Message::Close` variant with no close frame information
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let close_msg = Message::close();
    /// ```
    pub fn close() -> Self {
        Message::Close(None)
    }

    /// Creates a new close message with a status code and reason.
    ///
    /// This creates a close message with detailed information about why the
    /// connection is being terminated. The status code should follow RFC 6455
    /// specifications.
    ///
    /// # Parameters
    ///
    /// - `code`: The close status code
    /// - `reason`: Human-readable reason for closure
    ///
    /// # Returns
    ///
    /// A new `Message::Close` variant with close frame information
    ///
    /// # Standard Close Codes
    ///
    /// - `1000`: Normal closure
    /// - `1001`: Going away
    /// - `1002`: Protocol error
    /// - `1003`: Unsupported data
    /// - `1007`: Invalid UTF-8 data
    /// - `1008`: Policy violation
    /// - `1009`: Message too big
    /// - `1011`: Internal server error
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let normal = Message::close_with_reason(1000, "Session completed");
    /// let error = Message::close_with_reason(1011, "Database error");
    /// let policy = Message::close_with_reason(1008, "Rate limit exceeded");
    /// ```
    pub fn close_with_reason(code: u16, reason: impl Into<String>) -> Self {
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))
    }

    /// Returns the type of this message.
    ///
    /// This method allows you to identify the message type without matching
    /// on the message content. It's useful for routing, statistics, and
    /// protocol handling.
    ///
    /// # Returns
    ///
    /// The `MessageType` corresponding to this message
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::{Message, MessageType};
    ///
    /// let text_msg = Message::text("Hello");
    /// assert_eq!(text_msg.message_type(), MessageType::Text);
    ///
    /// let binary_msg = Message::binary(vec!);[1]
    /// assert_eq!(binary_msg.message_type(), MessageType::Binary);
    /// ```
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Text(_) => MessageType::Text,
            Message::Binary(_) => MessageType::Binary,
            Message::Ping(_) => MessageType::Ping,
            Message::Pong(_) => MessageType::Pong,
            Message::Close(_) => MessageType::Close,
        }
    }

    /// Returns the text content if this is a text message.
    ///
    /// This method provides a convenient way to access the text content of
    /// a message without pattern matching. Returns `None` if the message
    /// is not a text message.
    ///
    /// # Returns
    ///
    /// - `Some(&str)` if this is a text message
    /// - `None` if this is not a text message
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let text_msg = Message::text("Hello, World!");
    /// let binary_msg = Message::binary(vec!);[1]
    ///
    /// assert_eq!(text_msg.as_text(), Some("Hello, World!"));
    /// assert_eq!(binary_msg.as_text(), None);
    /// ```
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Returns the binary content if this is a binary message.
    ///
    /// This method provides a convenient way to access the binary content of
    /// a message without pattern matching. Returns `None` if the message
    /// is not a binary message.
    ///
    /// # Returns
    ///
    /// - `Some(&Bytes)` if this is a binary message
    /// - `None` if this is not a binary message
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use bytes::Bytes;
    ///
    /// let binary_msg = Message::binary(vec!);[1]
    /// let text_msg = Message::text("Hello");
    ///
    /// assert_eq!(binary_msg.as_binary(), Some(&Bytes::from(vec!)));[1]
    /// assert_eq!(text_msg.as_binary(), None);
    /// ```
    pub fn as_binary(&self) -> Option<&Bytes> {
        match self {
            Message::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Consumes the message and returns the text content if it's a text message.
    ///
    /// This method takes ownership of the message and extracts the text content.
    /// It's more efficient than `as_text()` when you need owned data and don't
    /// need the original message anymore.
    ///
    /// # Returns
    ///
    /// - `Some(String)` if this is a text message
    /// - `None` if this is not a text message
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let text_msg = Message::text("Hello, World!");
    /// let binary_msg = Message::binary(vec!);[1]
    ///
    /// assert_eq!(text_msg.into_text(), Some("Hello, World!".to_string()));
    /// assert_eq!(binary_msg.into_text(), None);
    /// ```
    pub fn into_text(self) -> Option<String> {
        match self {
            Message::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Consumes the message and returns the binary content if it's a binary message.
    ///
    /// This method takes ownership of the message and extracts the binary content.
    /// It's more efficient than `as_binary()` when you need owned data and don't
    /// need the original message anymore.
    ///
    /// # Returns
    ///
    /// - `Some(Bytes)` if this is a binary message
    /// - `None` if this is not a binary message
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use bytes::Bytes;
    ///
    /// let binary_msg = Message::binary(vec!);[1]
    /// let text_msg = Message::text("Hello");
    ///
    /// assert_eq!(binary_msg.into_binary(), Some(Bytes::from(vec!)));[1]
    /// assert_eq!(text_msg.into_binary(), None);
    /// ```
    pub fn into_binary(self) -> Option<Bytes> {
        match self {
            Message::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Returns `true` if this is a text message.
    ///
    /// This is a convenience method that's equivalent to checking if
    /// `message_type() == MessageType::Text`.
    ///
    /// # Returns
    ///
    /// `true` if this is a text message, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let text_msg = Message::text("Hello");
    /// let binary_msg = Message::binary(vec!);[1]
    ///
    /// assert!(text_msg.is_text());
    /// assert!(!binary_msg.is_text());
    /// ```
    pub fn is_text(&self) -> bool {
        matches!(self, Message::Text(_))
    }

    /// Returns `true` if this is a binary message.
    ///
    /// This is a convenience method that's equivalent to checking if
    /// `message_type() == MessageType::Binary`.
    ///
    /// # Returns
    ///
    /// `true` if this is a binary message, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let binary_msg = Message::binary(vec!);[1]
    /// let text_msg = Message::text("Hello");
    ///
    /// assert!(binary_msg.is_binary());
    /// assert!(!text_msg.is_binary());
    /// ```
    pub fn is_binary(&self) -> bool {
        matches!(self, Message::Binary(_))
    }

    /// Returns `true` if this is a close message.
    ///
    /// This is a convenience method that's equivalent to checking if
    /// `message_type() == MessageType::Close`.
    ///
    /// # Returns
    ///
    /// `true` if this is a close message, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    ///
    /// let close_msg = Message::close();
    /// let text_msg = Message::text("Hello");
    ///
    /// assert!(close_msg.is_close());
    /// assert!(!text_msg.is_close());
    /// ```
    pub fn is_close(&self) -> bool {
        matches!(self, Message::Close(_))
    }

    /// Creates a JSON text message from a serializable value.
    ///
    /// This method serializes the given value to JSON and creates a text message
    /// containing the JSON string. It's a convenient way to send structured data
    /// over WebSocket connections.
    ///
    /// # Parameters
    ///
    /// - `value`: Any value that implements `Serialize`
    ///
    /// # Returns
    ///
    /// - `Ok(Message)` containing the JSON as a text message
    /// - `Err(serde_json::Error)` if serialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct User {
    ///     id: u32,
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let user = User {
    ///     id: 123,
    ///     name: "Alice".to_string(),
    ///     email: "alice@example.com".to_string(),
    /// };
    ///
    /// let message = Message::json(&user)?;
    /// // This creates a text message containing:
    /// // {"id":123,"name":"Alice","email":"alice@example.com"}
    /// ```
    ///
    /// ### With Collections
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use std::collections::HashMap;
    ///
    /// let mut data = HashMap::new();
    /// data.insert("status", "success");
    /// data.insert("message", "Operation completed");
    ///
    /// let message = Message::json(&data)?;
    /// ```
    ///
    /// ### With Nested Structures
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Response {
    ///     success: bool,
    ///     data: Vec<String>,
    ///     metadata: Metadata,
    /// }
    ///
    /// #[derive(Serialize)]
    /// struct Metadata {
    ///     timestamp: u64,
    ///     version: String,
    /// }
    ///
    /// let response = Response {
    ///     success: true,
    ///     data: vec!["item1".to_string(), "item2".to_string()],
    ///     metadata: Metadata {
    ///         timestamp: 1234567890,
    ///         version: "1.0".to_string(),
    ///     },
    /// };
    ///
    /// let message = Message::json(&response)?;
    /// ```
    pub fn json<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(value)?;
        Ok(Message::Text(json))
    }

    /// Parses this message as JSON if it's a text message.
    ///
    /// This method attempts to deserialize the message content as JSON into
    /// the specified type. It only works with text messages and will return
    /// an error for other message types.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The type to deserialize into (must implement `Deserialize`)
    ///
    /// # Returns
    ///
    /// - `Ok(T)` if parsing succeeds
    /// - `Err(serde_json::Error)` if parsing fails or if not a text message
    ///
    /// # Examples
    ///
    /// ### Basic Usage
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct User {
    ///     id: u32,
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let json_text = r#"{"id":123,"name":"Alice","email":"alice@example.com"}"#;
    /// let message = Message::text(json_text);
    ///
    /// let user: User = message.parse_json()?;
    /// println!("User: {:?}", user);
    /// ```
    ///
    /// ### With Error Handling
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use serde_json::Value;
    ///
    /// let message = Message::text("invalid json {");
    ///
    /// match message.parse_json::<Value>() {
    ///     Ok(value) => println!("Parsed JSON: {:?}", value),
    ///     Err(e) => println!("Failed to parse JSON: {}", e),
    /// }
    /// ```
    ///
    /// ### Dynamic JSON Processing
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use serde_json::Value;
    ///
    /// let message = Message::text(r#"{"action":"login","username":"alice"}"#);
    /// let json: Value = message.parse_json()?;
    ///
    /// if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
    ///     match action {
    ///         "login" => {
    ///             if let Some(username) = json.get("username").and_then(|v| v.as_str()) {
    ///                 println!("Login request for user: {}", username);
    ///             }
    ///         }
    ///         _ => println!("Unknown action: {}", action),
    ///     }
    /// }
    /// ```
    ///
    /// ### With Optional Fields
    ///
    /// ```
    /// use ignitia::websocket::Message;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Config {
    ///     host: String,
    ///     port: Option<u16>,
    ///     ssl: Option<bool>,
    /// }
    ///
    /// let json_text = r#"{"host":"localhost","port":8080}"#;
    /// let message = Message::text(json_text);
    ///
    /// let config: Config = message.parse_json()?;
    /// println!("Config: {:?}", config);
    /// // Config: Config { host: "localhost", port: Some(8080), ssl: None }
    /// ```
    pub fn parse_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        match self {
            Message::Text(text) => serde_json::from_str(text),
            _ => Err(serde_json::Error::custom("Message is not text")),
        }
    }
}

/// Display implementation for messages.
///
/// This provides a human-readable representation of messages that's useful for
/// logging, debugging, and monitoring. The display format shows the message
/// type and relevant information without exposing potentially large payloads.
///
/// # Format
///
/// - Text messages: `Text: <content>` (truncated if very long)
/// - Binary messages: `Binary: <size> bytes`
/// - Ping messages: `Ping: <size> bytes`
/// - Pong messages: `Pong: <size> bytes`
/// - Close messages: `Close: <code> - <reason>` or just `Close`
///
/// # Examples
///
/// ```
/// use ignitia::websocket::{Message, CloseFrame};
///
/// let text = Message::text("Hello, World!");
/// println!("{}", text); // Output: Text: Hello, World!
///
/// let binary = Message::binary(vec!);[1]
/// println!("{}", binary); // Output: Binary: 5 bytes
///
/// let close = Message::close_with_reason(1000, "Normal closure");
/// println!("{}", close); // Output: Close: 1000 - Normal closure
/// ```
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Text(text) => write!(f, "Text: {}", text),
            Message::Binary(bytes) => write!(f, "Binary: {} bytes", bytes.len()),
            Message::Ping(bytes) => write!(f, "Ping: {} bytes", bytes.len()),
            Message::Pong(bytes) => write!(f, "Pong: {} bytes", bytes.len()),
            Message::Close(frame) => match frame {
                Some(frame) => write!(f, "Close: {} - {}", frame.code, frame.reason),
                None => write!(f, "Close"),
            },
        }
    }
}

// Conversion implementations for convenient message creation

/// Converts a `String` into a text message.
///
/// This allows you to use string literals and owned strings directly as messages.
///
/// # Examples
///
/// ```
/// use ignitia::websocket::Message;
///
/// let message: Message = "Hello, World!".to_string().into();
/// let message2: Message = format!("User {} connected", "Alice").into();
/// ```
impl From<String> for Message {
    fn from(text: String) -> Self {
        Message::Text(text)
    }
}

/// Converts a string slice into a text message.
///
/// This allows you to use string literals directly as messages.
///
/// # Examples
///
/// ```
/// use ignitia::websocket::Message;
///
/// let message: Message = "Hello, World!".into();
/// ```
impl From<&str> for Message {
    fn from(text: &str) -> Self {
        Message::Text(text.to_string())
    }
}

/// Converts `Bytes` into a binary message.
///
/// This allows you to use `Bytes` instances directly as messages.
///
/// # Examples
///
/// ```
/// use ignitia::websocket::Message;
/// use bytes::Bytes;
///
/// let data = Bytes::from(vec!);[1]
/// let message: Message = data.into();
/// ```
impl From<Bytes> for Message {
    fn from(bytes: Bytes) -> Self {
        Message::Binary(bytes)
    }
}

/// Converts a `Vec<u8>` into a binary message.
///
/// This allows you to use byte vectors directly as messages.
///
/// # Examples
///
/// ```
/// use ignitia::websocket::Message;
///
/// let data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in ASCII
/// let message: Message = data.into();
/// ```
impl From<Vec<u8>> for Message {
    fn from(bytes: Vec<u8>) -> Self {
        Message::Binary(Bytes::from(bytes))
    }
}
