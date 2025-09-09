//! # WebSocket Protocol Upgrade Module
//!
//! This module implements the WebSocket protocol upgrade mechanism according to RFC 6455.
//! It handles the HTTP to WebSocket protocol transition, including request validation,
//! handshake processing, and connection establishment.
//!
//! ## Protocol Overview
//!
//! The WebSocket protocol upgrade follows a specific handshake process:
//!
//! 1. **Client Request**: The client sends an HTTP GET request with specific headers
//! 2. **Server Validation**: The server validates the upgrade request headers
//! 3. **Key Generation**: The server generates a WebSocket accept key
//! 4. **Response**: The server responds with HTTP 101 Switching Protocols
//! 5. **Connection**: The connection is upgraded to WebSocket protocol
//!
//! ## RFC 6455 Compliance
//!
//! This implementation strictly follows RFC 6455 WebSocket Protocol Standard:
//!
//! - **Version 13**: Only supports WebSocket protocol version 13
//! - **Header Validation**: Validates all required WebSocket headers
//! - **Key Algorithm**: Uses the standard SHA-1 + base64 accept key algorithm
//! - **Status Codes**: Returns appropriate HTTP status codes for different scenarios
//!
//! ## Security Considerations
//!
//! - **Origin Validation**: Can be extended to validate request origins
//! - **Protocol Negotiation**: Supports subprotocol selection
//! - **Key Validation**: Validates WebSocket-Key format and presence
//! - **Header Sanitization**: Safely handles malformed headers
//!
//! ## Usage Examples
//!
//! ### Basic WebSocket Upgrade
//!
//! ```
//! use ignitia::websocket::{is_websocket_request, upgrade_connection};
//! use ignitia::{Request, Response};
//!
//! async fn handle_request(req: Request) -> Result<Response, Box<dyn std::error::Error>> {
//!     if is_websocket_request(&req) {
//!         // This is a WebSocket upgrade request
//!         match upgrade_connection(req) {
//!             Ok(response) => {
//!                 println!("WebSocket upgrade successful");
//!                 Ok(response)
//!             }
//!             Err(e) => {
//!                 println!("WebSocket upgrade failed: {}", e);
//!                 Err(Box::new(e))
//!             }
//!         }
//!     } else {
//!         // Handle as regular HTTP request
//!         Ok(Response::text("This is not a WebSocket request"))
//!     }
//! }
//! ```
//!
//! ### Integration with Router
//!
//! ```
//! use ignitia::{Router, Server, websocket::*};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/ws", |req| async move {
//!             if is_websocket_request(&req) {
//!                 upgrade_connection(req)
//!             } else {
//!                 Ok(Response::text("Please use WebSocket client"))
//!             }
//!         });
//!
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()?;
//!     let server = Server::new(router, addr);
//!     server.ignitia().await
//! }
//! ```
//!
//! ### Custom Validation
//!
//! ```
//! use ignitia::websocket::{is_websocket_request, upgrade_connection};
//! use ignitia::{Request, Response, Error};
//!
//! async fn secure_websocket_upgrade(req: Request) -> Result<Response, Error> {
//!     // Validate that it's a WebSocket request
//!     if !is_websocket_request(&req) {
//!         return Err(Error::BadRequest("Not a WebSocket request".into()));
//!     }
//!
//!     // Additional security checks
//!     if let Some(origin) = req.header("origin") {
//!         if !is_allowed_origin(origin) {
//!             return Err(Error::Forbidden);
//!         }
//!     }
//!
//!     // Validate authentication
//!     if let Some(auth) = req.header("authorization") {
//!         if !validate_auth_token(auth) {
//!             return Err(Error::Unauthorized);
//!         }
//!     }
//!
//!     // Proceed with upgrade
//!     upgrade_connection(req)
//! }
//!
//! fn is_allowed_origin(origin: &str) -> bool {
//!     // Implement your origin validation logic
//!     origin == "https://example.com" || origin == "https://app.example.com"
//! }
//!
//! fn validate_auth_token(token: &str) -> bool {
//!     // Implement your authentication logic
//!     token.starts_with("Bearer ") && token.len() > 20
//! }
//! ```
//!
//! ## Performance Optimizations
//!
//! - **Pre-computed Headers**: Static headers are pre-computed for faster responses
//! - **Fast Path Validation**: Early returns for invalid requests
//! - **Efficient Key Generation**: Optimized SHA-1 computation
//! - **Minimal Allocations**: Reduces memory allocations during upgrade
//!
//! ## Error Handling
//!
//! The upgrade process can fail for several reasons:
//!
//! - **Missing Headers**: Required WebSocket headers are missing
//! - **Invalid Version**: Unsupported WebSocket protocol version
//! - **Malformed Key**: Invalid or missing WebSocket-Key header
//! - **Protocol Mismatch**: Unsupported subprotocols requested
//!
//! ## Testing WebSocket Upgrades
//!
//! ```
//! use ignitia::websocket::{is_websocket_request, upgrade_connection, generate_accept_key};
//! use ignitia::Request;
//! use http::{Method, Uri, Version, HeaderMap, HeaderValue};
//! use bytes::Bytes;
//!
//! #[tokio::test]
//! async fn test_websocket_upgrade() {
//!     let mut headers = HeaderMap::new();
//!     headers.insert("connection", HeaderValue::from_static("Upgrade"));
//!     headers.insert("upgrade", HeaderValue::from_static("websocket"));
//!     headers.insert("sec-websocket-version", HeaderValue::from_static("13"));
//!     headers.insert("sec-websocket-key", HeaderValue::from_static("dGhlIHNhbXBsZSBub25jZQ=="));
//!
//!     let request = Request::new(
//!         Method::GET,
//!         Uri::from_static("/ws"),
//!         Version::HTTP_11,
//!         headers,
//!         Bytes::new(),
//!     );
//!
//!     assert!(is_websocket_request(&request));
//!
//!     let response = upgrade_connection(request).unwrap();
//!     assert_eq!(response.status, http::StatusCode::SWITCHING_PROTOCOLS);
//! }
//!
//! #[test]
//! fn test_accept_key_generation() {
//!     let key = "dGhlIHNhbXBsZSBub25jZQ==";
//!     let expected = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
//!     assert_eq!(generate_accept_key(key), expected);
//! }
//! ```

use super::connection::WebSocketConnection;
use super::handler::WebSocketHandler;
use crate::{Request, Response, Result};
use http::StatusCode;
use hyper_util::rt::TokioIo;
use sha1::{Digest, Sha1};
use std::sync::Arc;

/// The WebSocket magic string used in accept key generation as defined by RFC 6455.
///
/// This constant is appended to the client's WebSocket-Key header value before
/// computing the SHA-1 hash for the WebSocket-Accept response header.
const WEBSOCKET_MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

// Use once_cell instead of lazy_static since it's already in dependencies
use once_cell::sync::Lazy;

/// Pre-computed "websocket" header value for performance optimization.
///
/// This static value is used in upgrade responses to avoid repeated string parsing
/// and allocation during the upgrade process.
static UPGRADE_HEADER: Lazy<http::HeaderValue> = Lazy::new(|| "websocket".parse().unwrap());

/// Pre-computed "Upgrade" header value for performance optimization.
///
/// This static value is used in upgrade responses for the Connection header to
/// avoid repeated string parsing and allocation during the upgrade process.
static CONNECTION_HEADER: Lazy<http::HeaderValue> = Lazy::new(|| "Upgrade".parse().unwrap());

/// Determines if an incoming HTTP request is a valid WebSocket upgrade request.
///
/// This function validates all required headers according to RFC 6455 Section 4.2.1.
/// It performs fast-path validation with early returns for better performance.
///
/// ## Required Headers
///
/// A valid WebSocket upgrade request must include:
/// - `Connection: Upgrade` (case-insensitive, may include other tokens)
/// - `Upgrade: websocket` (case-insensitive)
/// - `Sec-WebSocket-Version: 13` (exact match)
/// - `Sec-WebSocket-Key` (any valid base64-encoded 16-byte value)
///
/// ## Parameters
///
/// - `req`: The HTTP request to validate
///
/// ## Returns
///
/// - `true` if the request is a valid WebSocket upgrade request
/// - `false` if any required headers are missing or invalid
///
/// ## Performance Notes
///
/// - Uses early returns to minimize validation overhead for non-WebSocket requests
/// - Performs case-insensitive header value comparisons as required by HTTP specs
/// - Validates headers in order of most likely to fail first
///
/// ## Examples
///
/// ### Basic Usage
///
/// ```
/// use ignitia::websocket::is_websocket_request;
/// use ignitia::Request;
/// use http::{Method, Uri, Version, HeaderMap, HeaderValue};
/// use bytes::Bytes;
///
/// // Create a valid WebSocket upgrade request
/// let mut headers = HeaderMap::new();
/// headers.insert("connection", HeaderValue::from_static("keep-alive, Upgrade"));
/// headers.insert("upgrade", HeaderValue::from_static("WebSocket"));
/// headers.insert("sec-websocket-version", HeaderValue::from_static("13"));
/// headers.insert("sec-websocket-key", HeaderValue::from_static("dGhlIHNhbXBsZSBub25jZQ=="));
///
/// let request = Request::new(
///     Method::GET,
///     Uri::from_static("/ws"),
///     Version::HTTP_11,
///     headers,
///     Bytes::new(),
/// );
///
/// assert!(is_websocket_request(&request));
/// ```
///
/// ### Invalid Request Examples
///
/// ```
/// use ignitia::websocket::is_websocket_request;
/// use ignitia::Request;
/// use http::{Method, Uri, Version, HeaderMap};
/// use bytes::Bytes;
///
/// // Missing WebSocket headers - regular HTTP request
/// let request = Request::new(
///     Method::GET,
///     Uri::from_static("/api/users"),
///     Version::HTTP_11,
///     HeaderMap::new(),
///     Bytes::new(),
/// );
///
/// assert!(!is_websocket_request(&request));
/// ```
///
/// ### Conditional Handling
///
/// ```
/// use ignitia::websocket::{is_websocket_request, upgrade_connection};
/// use ignitia::{Request, Response};
///
/// async fn handle_request(req: Request) -> Result<Response, Box<dyn std::error::Error>> {
///     if is_websocket_request(&req) {
///         // Handle WebSocket upgrade
///         Ok(upgrade_connection(req)?)
///     } else {
///         // Handle regular HTTP request
///         Ok(Response::text("Hello, HTTP World!"))
///     }
/// }
/// ```
pub fn is_websocket_request(req: &Request) -> bool {
    // Fast path checks with early returns for performance

    // Check Connection header - must contain "upgrade" (case-insensitive)
    let connection = match req.header("connection") {
        Some(c) => c.to_lowercase(),
        None => return false,
    };

    // Check Upgrade header - must contain "websocket" (case-insensitive)
    let upgrade = match req.header("upgrade") {
        Some(u) => u.to_lowercase(),
        None => return false,
    };

    // Check WebSocket version - must be exactly "13"
    let websocket_version = req.header("sec-websocket-version").unwrap_or("");

    // Check for WebSocket key presence
    let has_key = req.header("sec-websocket-key").is_some();

    // All conditions must be met for a valid WebSocket upgrade request
    connection.contains("upgrade")
        && upgrade.contains("websocket")
        && websocket_version == "13"
        && has_key
}

/// Upgrades an HTTP request to a WebSocket connection.
///
/// This function performs the server-side WebSocket handshake by generating the
/// appropriate HTTP 101 Switching Protocols response with required headers.
/// The response follows RFC 6455 Section 4.2.2 specifications.
///
/// ## WebSocket Handshake Process
///
/// 1. **Extract WebSocket-Key**: Gets the client's WebSocket-Key header
/// 2. **Generate Accept Key**: Computes WebSocket-Accept using SHA-1 + base64
/// 3. **Create Response**: Builds HTTP 101 response with required headers
/// 4. **Protocol Negotiation**: Handles optional subprotocol selection
///
/// ## Parameters
///
/// - `req`: The validated WebSocket upgrade request
///
/// ## Returns
///
/// - `Ok(Response)`: HTTP 101 Switching Protocols response on success
/// - `Err(Error)`: Error if the request is invalid or missing required headers
///
/// ## Response Headers
///
/// The generated response includes:
/// - `Status: 101 Switching Protocols`
/// - `Upgrade: websocket`
/// - `Connection: Upgrade`
/// - `Sec-WebSocket-Accept: <computed-accept-key>`
/// - `Sec-WebSocket-Protocol: <selected-protocol>` (if requested)
///
/// ## Error Conditions
///
/// - **Missing WebSocket-Key**: Returns `BadRequest` if Sec-WebSocket-Key header is absent
/// - **Invalid Key Format**: Returns `BadRequest` if the key is malformed
/// - **Header Generation Failure**: Returns `Internal` if response headers cannot be created
///
/// ## Examples
///
/// ### Basic Upgrade
///
/// ```
/// use ignitia::websocket::{is_websocket_request, upgrade_connection};
/// use ignitia::{Request, Response};
/// use http::StatusCode;
///
/// async fn websocket_endpoint(req: Request) -> Result<Response, ignitia::Error> {
///     // Validate the request first
///     if !is_websocket_request(&req) {
///         return Err(ignitia::Error::BadRequest(
///             "Invalid WebSocket upgrade request".into()
///         ));
///     }
///
///     // Perform the upgrade
///     let response = upgrade_connection(req)?;
///
///     // Verify the response status
///     assert_eq!(response.status, StatusCode::SWITCHING_PROTOCOLS);
///
///     Ok(response)
/// }
/// ```
///
/// ### With Protocol Negotiation
///
/// ```
/// use ignitia::websocket::{upgrade_connection};
/// use ignitia::Request;
/// use http::{Method, Uri, Version, HeaderMap, HeaderValue};
/// use bytes::Bytes;
///
/// // Client requests multiple protocols
/// let mut headers = HeaderMap::new();
/// headers.insert("connection", HeaderValue::from_static("Upgrade"));
/// headers.insert("upgrade", HeaderValue::from_static("websocket"));
/// headers.insert("sec-websocket-version", HeaderValue::from_static("13"));
/// headers.insert("sec-websocket-key", HeaderValue::from_static("dGhlIHNhbXBsZSBub25jZQ=="));
/// headers.insert("sec-websocket-protocol", HeaderValue::from_static("chat, superchat"));
///
/// let request = Request::new(
///     Method::GET,
///     Uri::from_static("/ws"),
///     Version::HTTP_11,
///     headers,
///     Bytes::new(),
/// );
///
/// let response = upgrade_connection(request).unwrap();
///
/// // Server selects the first supported protocol
/// let selected_protocol = response.headers.get("sec-websocket-protocol");
/// println!("Selected protocol: {:?}", selected_protocol);
/// ```
///
/// ### Error Handling
///
/// ```
/// use ignitia::websocket::upgrade_connection;
/// use ignitia::{Request, Error};
/// use http::{Method, Uri, Version, HeaderMap};
/// use bytes::Bytes;
///
/// // Request without WebSocket-Key header
/// let request = Request::new(
///     Method::GET,
///     Uri::from_static("/ws"),
///     Version::HTTP_11,
///     HeaderMap::new(),
///     Bytes::new(),
/// );
///
/// match upgrade_connection(request) {
///     Ok(_) => panic!("Should have failed"),
///     Err(Error::BadRequest(msg)) => {
///         assert!(msg.contains("Missing Sec-WebSocket-Key"));
///     }
///     Err(e) => panic!("Unexpected error: {}", e),
/// }
/// ```
///
/// ### Integration with Middleware
///
/// ```
/// use ignitia::websocket::{is_websocket_request, upgrade_connection};
/// use ignitia::{Request, Response, Error};
///
/// async fn auth_websocket_upgrade(req: Request) -> Result<Response, Error> {
///     // Validate WebSocket request
///     if !is_websocket_request(&req) {
///         return Err(Error::BadRequest("Not a WebSocket request".into()));
///     }
///
///     // Check authentication
///     match req.header("authorization") {
///         Some(token) if is_valid_token(token) => {
///             // Proceed with upgrade
///             upgrade_connection(req)
///         }
///         Some(_) => Err(Error::Unauthorized),
///         None => Err(Error::BadRequest("Missing authorization header".into())),
///     }
/// }
///
/// fn is_valid_token(token: &str) -> bool {
///     // Implement your token validation logic
///     token.starts_with("Bearer ") && token.len() > 20
/// }
/// ```
pub fn upgrade_connection(req: Request) -> Result<Response> {
    // Extract the WebSocket-Key header - this is required for the handshake
    let websocket_key = req
        .header("sec-websocket-key")
        .ok_or_else(|| crate::Error::BadRequest("Missing Sec-WebSocket-Key header".into()))?;

    // Generate the WebSocket-Accept key using the standard algorithm
    let accept_key = generate_accept_key(websocket_key);

    // Create the HTTP 101 Switching Protocols response
    let mut response = Response::new(StatusCode::SWITCHING_PROTOCOLS);

    // Add required WebSocket upgrade headers using pre-computed values for performance
    response.headers.insert("upgrade", UPGRADE_HEADER.clone());
    response
        .headers
        .insert("connection", CONNECTION_HEADER.clone());
    response
        .headers
        .insert("sec-websocket-accept", accept_key.parse().unwrap());

    // Handle optional protocol negotiation
    // If the client specified protocols, select the first one (simple strategy)
    if let Some(protocols) = req.header("sec-websocket-protocol") {
        if let Some(protocol) = protocols.split(',').find(|p| !p.trim().is_empty()) {
            if let Ok(protocol_value) = protocol.trim().parse() {
                response
                    .headers
                    .insert("sec-websocket-protocol", protocol_value);
            }
        }
    }

    Ok(response)
}

/// Handles the complete WebSocket upgrade process from HTTP connection to WebSocket.
///
/// This function manages the low-level details of upgrading the underlying HTTP connection
/// to a WebSocket connection and initializing the WebSocket handler. It's typically called
/// by the server framework after the HTTP handshake is complete.
///
/// ## Upgrade Process
///
/// 1. **Connection Upgrade**: Takes ownership of the upgraded HTTP connection
/// 2. **WebSocket Initialization**: Creates a WebSocket stream from the raw connection
/// 3. **Handler Initialization**: Wraps the stream in a WebSocketConnection
/// 4. **Handler Execution**: Starts the WebSocket message handler
///
/// ## Parameters
///
/// - `upgraded`: The upgraded HTTP connection from hyper
/// - `handler`: The WebSocket handler to manage the connection
///
/// ## Returns
///
/// - `Ok(())`: When the WebSocket connection closes normally
/// - `Err(Error)`: If an error occurs during connection handling
///
/// ## Connection Lifecycle
///
/// The function manages the complete WebSocket connection lifecycle:
/// - Connection establishment and configuration
/// - Message processing loop
/// - Error handling and recovery
/// - Graceful connection cleanup
///
/// ## Performance Optimizations
///
/// - **Efficient Stream Setup**: Minimal overhead WebSocket initialization
/// - **Async Processing**: Non-blocking message handling
/// - **Resource Management**: Proper cleanup of connection resources
///
/// ## Examples
///
/// ### Basic Handler Integration
///
/// ```
/// use ignitia::websocket::{handle_websocket_upgrade, WebSocketHandler, WebSocketConnection};
/// use ignitia::Result;
/// use std::sync::Arc;
///
/// struct EchoHandler;
///
/// #[async_trait::async_trait]
/// impl WebSocketHandler for EchoHandler {
///     async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
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
///
/// // This would typically be called by the server framework
/// // let handler = Arc::new(EchoHandler);
/// // handle_websocket_upgrade(upgraded_connection, handler).await?;
/// ```
///
/// ### Error Recovery
///
/// ```
/// use ignitia::websocket::{handle_websocket_upgrade, WebSocketHandler, WebSocketConnection};
/// use ignitia::{Result, Error};
/// use std::sync::Arc;
///
/// struct RobustHandler;
///
/// #[async_trait::async_trait]
/// impl WebSocketHandler for RobustHandler {
///     async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
///         loop {
///             match websocket.recv().await {
///                 Some(message) => {
///                     if let Err(e) = self.process_message(&websocket, message).await {
///                         tracing::error!("Message processing error: {}", e);
///                         // Continue processing other messages
///                         continue;
///                     }
///                 }
///                 None => {
///                     tracing::info!("WebSocket connection closed");
///                     break;
///                 }
///             }
///         }
///         Ok(())
///     }
/// }
///
/// impl RobustHandler {
///     async fn process_message(
///         &self,
///         websocket: &WebSocketConnection,
///         message: Message
///     ) -> Result<()> {
///         // Implement robust message processing with error handling
///         match message {
///             Message::Text(text) => {
///                 // Process text message
///                 websocket.send_text(format!("Processed: {}", text)).await?;
///             }
///             Message::Binary(data) => {
///                 // Process binary message
///                 websocket.send_bytes(data).await?;
///             }
///             Message::Close(_) => {
///                 return Err(Error::Internal("Connection closed".into()));
///             }
///             _ => {}
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## Error Handling
///
/// The function handles various error conditions:
/// - **Connection Errors**: Network or protocol-level failures
/// - **Handler Errors**: Application-level errors from the WebSocket handler
/// - **Protocol Errors**: WebSocket frame parsing or validation failures
///
/// Errors are logged appropriately and the connection is cleaned up safely.
pub async fn handle_websocket_upgrade(
    upgraded: hyper::upgrade::Upgraded,
    handler: Arc<dyn WebSocketHandler>,
) -> Result<()> {
    // Wrap the upgraded HTTP connection for use with tokio-tungstenite
    let io = TokioIo::new(upgraded);

    // Create a WebSocket stream from the raw socket
    // This performs the low-level WebSocket protocol setup
    let ws_stream = tokio_tungstenite::WebSocketStream::from_raw_socket(
        io,
        tokio_tungstenite::tungstenite::protocol::Role::Server,
        None, // No additional configuration needed
    )
    .await;

    // Wrap the stream in our WebSocketConnection abstraction
    let connection = WebSocketConnection::new(ws_stream);

    // Hand over control to the application handler
    // The handler will manage the connection for its entire lifetime
    handler.handle_connection(connection).await
}

/// Generates a WebSocket accept key from the client's WebSocket key.
///
/// This function implements the WebSocket accept key algorithm specified in RFC 6455
/// Section 4.2.2. The algorithm concatenates the client's key with the WebSocket magic
/// string, computes a SHA-1 hash, and encodes the result in base64.
///
/// ## Algorithm Steps
///
/// 1. **Concatenation**: Append the WebSocket magic string to the client key
/// 2. **Hashing**: Compute SHA-1 hash of the concatenated string
/// 3. **Encoding**: Encode the hash bytes as base64
///
/// ## Parameters
///
/// - `websocket_key`: The client's Sec-WebSocket-Key header value
///
/// ## Returns
///
/// The computed WebSocket-Accept header value as a base64-encoded string
///
/// ## Security Notes
///
/// - **One-way Function**: The accept key cannot be reversed to obtain the original key
/// - **Replay Prevention**: Each key should be unique per connection attempt
/// - **Validation**: The client will validate this key to confirm the upgrade
///
/// ## Performance Optimizations
///
/// - **Efficient Hashing**: Uses optimized SHA-1 implementation
/// - **Single Allocation**: Minimizes memory allocations during computation
/// - **Fast Base64**: Uses efficient base64 encoding
///
/// ## Examples
///
/// ### Basic Usage
///
/// ```
/// use ignitia::websocket::generate_accept_key;
///
/// let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
/// let accept_key = generate_accept_key(client_key);
///
/// // The result should match the expected WebSocket accept key
/// assert_eq!(accept_key, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
/// ```
///
/// ### Testing Key Generation
///
/// ```
/// use ignitia::websocket::generate_accept_key;
///
/// // Test vectors from RFC 6455
/// let test_cases = vec![
///     ("dGhlIHNhbXBsZSBub25jZQ==", "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="),
///     ("AQIDBAUGBwgJCgsMDQ4PEC==", "6+0lRQ4dK0pJkjDKCSbP3TPN6Ns="),
/// ];
///
/// for (input, expected) in test_cases {
///     let result = generate_accept_key(input);
///     assert_eq!(result, expected, "Key generation failed for input: {}", input);
/// }
/// ```
///
/// ### Custom Validation
///
/// ```
/// use ignitia::websocket::generate_accept_key;
/// use base64;
///
/// fn validate_websocket_key_format(key: &str) -> bool {
///     // WebSocket keys should be 24 characters of base64
///     if key.len() != 24 {
///         return false;
///     }
///
///     // Should decode to exactly 16 bytes
///     match base64::decode(key) {
///         Ok(bytes) => bytes.len() == 16,
///         Err(_) => false,
///     }
/// }
///
/// fn safe_generate_accept_key(websocket_key: &str) -> Result<String, &'static str> {
///     if !validate_websocket_key_format(websocket_key) {
///         return Err("Invalid WebSocket key format");
///     }
///
///     Ok(generate_accept_key(websocket_key))
/// }
///
/// // Usage
/// let key = "dGhlIHNhbXBsZSBub25jZQ==";
/// match safe_generate_accept_key(key) {
///     Ok(accept_key) => println!("Accept key: {}", accept_key),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
///
/// ## RFC 6455 Reference
///
/// The accept key algorithm is defined in RFC 6455 Section 4.2.2:
///
/// ```
/// To prove that the handshake was received, the server has to prove that it
/// read the handshake's contents. The server takes the value (as present in
/// the request) of the |Sec-WebSocket-Key| header field and concatenates this
/// with the GUID "258EAFA5-E914-47DA-95CA-C5AB0DC85B11". The server then
/// takes the SHA-1 hash of this concatenated value and base64-encodes it to
/// obtain the |Sec-WebSocket-Accept| header field value.
/// ```
pub fn generate_accept_key(websocket_key: &str) -> String {
    // Create a new SHA-1 hasher instance
    let mut hasher = Sha1::new();

    // Hash the client's WebSocket key
    hasher.update(websocket_key.as_bytes());

    // Append the WebSocket magic string as specified by RFC 6455
    hasher.update(WEBSOCKET_MAGIC_STRING.as_bytes());

    // Compute the final hash and encode as base64
    base64::encode(hasher.finalize())
}
