//! WebSocket Protocol Upgrade Implementation
//!
//! This module handles the WebSocket protocol upgrade handshake as defined in
//! [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) for HTTP/1.1.
//!
//! # WebSocket Protocol Overview
//!
//! The WebSocket protocol enables bidirectional, full-duplex communication between
//! a client and server over a single TCP connection. It starts as an HTTP/1.1
//! upgrade request and then switches to the WebSocket protocol.
//!
//! # Handshake Process
//!
//! 1. **Client sends upgrade request**:
//!    ```
//!    GET /ws HTTP/1.1
//!    Host: localhost:8080
//!    Upgrade: websocket
//!    Connection: Upgrade
//!    Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
//!    Sec-WebSocket-Version: 13
//!    ```
//!
//! 2. **Server responds with 101 Switching Protocols**:
//!    ```
//!    HTTP/1.1 101 Switching Protocols
//!    Upgrade: websocket
//!    Connection: Upgrade
//!    Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
//!    ```
//!
//! 3. **Connection upgraded to WebSocket protocol**
//!
//! # Security
//!
//! The module implements the required security checks:
//! - Validates `Sec-WebSocket-Key` header presence
//! - Generates proper `Sec-WebSocket-Accept` value using SHA-1 and Base64
//! - Verifies all required headers are present
//! - Supports TLS encryption (wss://)
//!
//! # Secure WebSocket (wss://)
//!
//! When using TLS/HTTPS:
//! - Initial HTTP/1.1 handshake occurs over TLS
//! - WebSocket data frames are encrypted via TLS
//! - All security properties of TLS apply to WebSocket traffic
//!
//! # Examples
//!
//! ## Basic Upgrade Check
//!
//! ```
//! use ignitia::websocket::is_websocket_request;
//! use ignitia::{Request, Method};
//! use http::{HeaderMap, HeaderValue, Version};
//!
//! let mut headers = HeaderMap::new();
//! headers.insert("upgrade", HeaderValue::from_static("websocket"));
//! headers.insert("connection", HeaderValue::from_static("Upgrade"));
//! headers.insert("sec-websocket-key", HeaderValue::from_static("dGhlIHNhbXBsZSBub25jZQ=="));
//!
//! let req = Request::new(
//!     Method::GET,
//!     "/ws".parse().unwrap(),
//!     Version::HTTP_11,
//!     headers,
//!     bytes::Bytes::new()
//! );
//!
//! assert!(is_websocket_request(&req));
//! ```
//!
//! ## Generate Upgrade Response
//!
//! ```
//! use ignitia::websocket::upgrade_connection;
//! use ignitia::Request;
//!
//! let req = create_websocket_request(); // Your request
//! let response = upgrade_connection(&req).unwrap();
//!
//! assert_eq!(response.status, 101); // Switching Protocols
//! assert!(response.headers.contains_key("sec-websocket-accept"));
//! ```

use super::connection::WebSocketConnection;
use super::handler::WebSocketHandler;
use crate::{Error, Request, Response, Result};
use base64::Engine;
use http::{header, HeaderValue, StatusCode, Version};
use hyper_util::rt::TokioIo;
use sha1::{Digest, Sha1};
use std::sync::Arc;

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Check if a request is a valid WebSocket upgrade request (HTTP/1.1).
///
/// This function validates that the request contains all required headers
/// for a WebSocket upgrade as specified in RFC 6455.
///
/// # Required Headers
///
/// - `Upgrade: websocket` (case-insensitive)
/// - `Connection: Upgrade` (must contain "upgrade", case-insensitive)
/// - `Sec-WebSocket-Key: <base64-encoded-value>`
///
/// # Parameters
///
/// * `req` - The HTTP request to validate
///
/// # Returns
///
/// `true` if the request is a valid WebSocket upgrade request, `false` otherwise
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
///
/// fn handle_request(req: Request) {
///     if is_websocket_request(&req) {
///         // Handle WebSocket upgrade
///     } else {
///         // Handle regular HTTP request
///     }
/// }
/// ```
///
/// # Protocol Support
///
/// Currently supports **HTTP/1.1 only**. HTTP/2 WebSocket support (RFC 8441)
/// is not yet implemented due to limitations in the Rust ecosystem.
///
/// # Reference
///
/// - [RFC 6455 - The WebSocket Protocol](https://datatracker.ietf.org/doc/html/rfc6455)
/// - [RFC 6455 Section 4.1 - Client Requirements](https://datatracker.ietf.org/doc/html/rfc6455#section-4.1)
pub fn is_websocket_request(req: &Request) -> bool {
    req.version == Version::HTTP_11
        && req
            .headers
            .get(header::UPGRADE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("websocket"))
            .unwrap_or(false)
        && req
            .headers
            .get(header::CONNECTION)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase().contains("upgrade"))
            .unwrap_or(false)
        && req.headers.contains_key(header::SEC_WEBSOCKET_KEY)
}

/// Upgrade an HTTP/1.1 connection to WebSocket protocol.
///
/// This function generates the appropriate HTTP response to complete the
/// WebSocket handshake, transitioning from HTTP to WebSocket protocol.
///
/// # Parameters
///
/// * `req` - The HTTP request initiating the WebSocket upgrade
///
/// # Returns
///
/// - `Ok(Response)` - A 101 Switching Protocols response with proper headers
/// - `Err(Error)` - If the request is invalid or missing required headers
///
/// # Response Headers
///
/// The generated response includes:
/// - `Status: 101 Switching Protocols`
/// - `Upgrade: websocket`
/// - `Connection: Upgrade`
/// - `Sec-WebSocket-Accept: <computed-value>`
/// - `Sec-WebSocket-Protocol: <selected-protocol>` (if negotiated)
///
/// # Protocol Negotiation
///
/// If the client sends `Sec-WebSocket-Protocol` header with a list of
/// subprotocols, the server selects the first one and includes it in
/// the response.
///
/// # Errors
///
/// Returns an error if:
/// - The request is not a valid WebSocket upgrade request
/// - `Sec-WebSocket-Key` header is missing
/// - Required headers are malformed
///
/// # Examples
///
/// ## Basic Upgrade
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn handle_upgrade(req: Request) -> Result<Response> {
///     if is_websocket_request(&req) {
///         let response = upgrade_connection(&req)?;
///         // Spawn WebSocket handler after sending response
///         Ok(response)
///     } else {
///         Ok(Response::bad_request("Not a WebSocket request"))
///     }
/// }
/// ```
///
/// ## With Protocol Negotiation
///
/// ```
/// // Client sends: Sec-WebSocket-Protocol: chat, superchat
/// // Server responds with: Sec-WebSocket-Protocol: chat
/// let response = upgrade_connection(&req)?;
/// ```
///
/// # Security Considerations
///
/// - Always validate the origin header in production
/// - Use `wss://` (WebSocket Secure) in production environments
/// - Implement authentication/authorization before upgrading
/// - Validate all headers to prevent injection attacks
///
/// # Reference
///
/// - [RFC 6455 Section 4.2.2 - Sending the Server's Opening Handshake](https://datatracker.ietf.org/doc/html/rfc6455#section-4.2.2)
pub fn upgrade_connection(req: &Request) -> Result<Response> {
    // Validate request
    if !is_websocket_request(req) {
        return Err(Error::bad_request("Not a valid WebSocket upgrade request"));
    }

    // Extract Sec-WebSocket-Key header
    let key = req
        .headers
        .get(header::SEC_WEBSOCKET_KEY)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::bad_request("Missing Sec-WebSocket-Key header"))?;

    // Generate Sec-WebSocket-Accept value
    let accept_key = generate_accept_key(key);

    // Build 101 Switching Protocols response
    let mut response = Response::new(StatusCode::SWITCHING_PROTOCOLS);

    // Add required headers
    response
        .headers
        .insert(header::UPGRADE, HeaderValue::from_static("websocket"));
    response
        .headers
        .insert(header::CONNECTION, HeaderValue::from_static("Upgrade"));
    response.headers.insert(
        header::SEC_WEBSOCKET_ACCEPT,
        HeaderValue::from_str(&accept_key).unwrap(),
    );

    // Handle subprotocol negotiation (optional)
    if let Some(protocols) = req.headers.get(header::SEC_WEBSOCKET_PROTOCOL) {
        if let Some(protocol) = protocols.to_str().ok().and_then(|p| p.split(',').next()) {
            response.headers.insert(
                header::SEC_WEBSOCKET_PROTOCOL,
                HeaderValue::from_str(protocol.trim())
                    .unwrap_or_else(|_| HeaderValue::from_static("")),
            );
        }
    }

    tracing::debug!("✅ WebSocket upgrade successful - returning 101 Switching Protocols");
    Ok(response)
}

/// Handle the WebSocket connection after protocol upgrade.
///
/// This function is called after the HTTP upgrade response has been sent.
/// It wraps the upgraded connection in a WebSocket stream and passes it
/// to the user-defined handler.
///
/// # Parameters
///
/// * `req` - The original HTTP request (for context like headers, path, etc.)
/// * `upgraded` - The upgraded connection from Hyper
/// * `handler` - The WebSocket handler to process messages
///
/// # Returns
///
/// A `Response` indicating the result of the WebSocket session. This response
/// is typically used for logging/metrics and is not sent to the client.
///
/// # Flow
///
/// 1. Wrap upgraded connection with `TokioIo` adapter
/// 2. Create WebSocket stream using `tokio-tungstenite`
/// 3. Wrap in `WebSocketConnection` for convenient API
/// 4. Pass to user handler for message processing
/// 5. Return handler's response when connection closes
///
/// # Example Usage (Internal)
///
/// ```
/// // After sending 101 response
/// tokio::spawn(async move {
///     match hyper::upgrade::on(req).await {
///         Ok(upgraded) => {
///             let response = handle_websocket_upgrade(
///                 framework_req,
///                 upgraded,
///                 handler
///             ).await;
///
///             if !response.status.is_success() {
///                 tracing::error!("WebSocket error: {}", response.status);
///             }
///         }
///         Err(e) => {
///             tracing::error!("Upgrade failed: {}", e);
///         }
///     }
/// });
/// ```
///
/// # Performance
///
/// The WebSocket stream is configured for optimal performance:
/// - Zero-copy message passing where possible
/// - Efficient framing and masking
/// - Automatic fragmentation handling
/// - Configurable buffer sizes
///
/// # Error Handling
///
/// Errors during the WebSocket session are returned as `Response` objects
/// with appropriate status codes. The handler can return error responses
/// which will close the connection gracefully.
///
/// # Reference
///
/// - [RFC 6455 Section 5 - Data Framing](https://datatracker.ietf.org/doc/html/rfc6455#section-5)
/// - [tokio-tungstenite Documentation](https://docs.rs/tokio-tungstenite)
pub async fn handle_websocket_upgrade(
    req: Request,
    upgraded: hyper::upgrade::Upgraded,
    handler: Arc<dyn WebSocketHandler>,
) -> Response {
    let io = TokioIo::new(upgraded);

    let ws_stream = tokio_tungstenite::WebSocketStream::from_raw_socket(
        io,
        tokio_tungstenite::tungstenite::protocol::Role::Server,
        None,
    )
    .await;

    let connection = WebSocketConnection::new(ws_stream);

    // Pass both request and connection to handler
    handler.handle(req, connection).await
}

/// Generate the `Sec-WebSocket-Accept` value from the client's key.
///
/// This function implements the algorithm specified in RFC 6455:
/// 1. Concatenate the client's `Sec-WebSocket-Key` with the WebSocket GUID
/// 2. Compute SHA-1 hash of the result
/// 3. Base64-encode the hash
///
/// # Parameters
///
/// * `key` - The client's `Sec-WebSocket-Key` header value
///
/// # Returns
///
/// A Base64-encoded string to be sent as `Sec-WebSocket-Accept`
///
/// # Algorithm
///
/// ```
/// Sec-WebSocket-Accept = base64(sha1(Sec-WebSocket-Key + GUID))
/// ```
///
/// where GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
///
/// # Example
///
/// ```
/// let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
/// let accept_key = generate_accept_key(client_key);
/// // accept_key = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
/// ```
///
/// # Security Note
///
/// This function uses SHA-1, which is cryptographically weak but acceptable
/// for this specific use case as defined by RFC 6455. The hash is used only
/// for handshake validation, not for cryptographic security.
///
/// # Reference
///
/// [RFC 6455 Section 4.2.2](https://datatracker.ietf.org/doc/html/rfc6455#section-4.2.2)
pub fn generate_accept_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WEBSOCKET_GUID.as_bytes());
    let hash = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(hash)
}
