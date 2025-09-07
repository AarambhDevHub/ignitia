use super::connection::WebSocketConnection;
use super::handler::WebSocketHandler;
use crate::{Request, Response, Result};
use http::StatusCode;
use hyper_util::rt::TokioIo;
use sha1::{Digest, Sha1};
use std::sync::Arc;

const WEBSOCKET_MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

// Use once_cell instead of lazy_static since it's already in dependencies
use once_cell::sync::Lazy;

static UPGRADE_HEADER: Lazy<http::HeaderValue> = Lazy::new(|| "websocket".parse().unwrap());
static CONNECTION_HEADER: Lazy<http::HeaderValue> = Lazy::new(|| "Upgrade".parse().unwrap());

pub fn is_websocket_request(req: &Request) -> bool {
    // Fast path checks with early returns
    let connection = match req.header("connection") {
        Some(c) => c.to_lowercase(),
        None => return false,
    };

    let upgrade = match req.header("upgrade") {
        Some(u) => u.to_lowercase(),
        None => return false,
    };

    let websocket_version = req.header("sec-websocket-version").unwrap_or("");
    let has_key = req.header("sec-websocket-key").is_some();

    connection.contains("upgrade")
        && upgrade.contains("websocket")
        && websocket_version == "13"
        && has_key
}

pub fn upgrade_connection(req: Request) -> Result<Response> {
    let websocket_key = req
        .header("sec-websocket-key")
        .ok_or_else(|| crate::Error::BadRequest("Missing Sec-WebSocket-Key header".into()))?;

    let accept_key = generate_accept_key(websocket_key);

    let mut response = Response::new(StatusCode::SWITCHING_PROTOCOLS);

    // Use pre-computed headers
    response.headers.insert("upgrade", UPGRADE_HEADER.clone());
    response
        .headers
        .insert("connection", CONNECTION_HEADER.clone());
    response
        .headers
        .insert("sec-websocket-accept", accept_key.parse().unwrap());

    // Handle protocol negotiation efficiently
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

pub async fn handle_websocket_upgrade(
    upgraded: hyper::upgrade::Upgraded,
    handler: Arc<dyn WebSocketHandler>,
) -> Result<()> {
    let io = TokioIo::new(upgraded);

    // Use efficient WebSocket setup
    let ws_stream = tokio_tungstenite::WebSocketStream::from_raw_socket(
        io,
        tokio_tungstenite::tungstenite::protocol::Role::Server,
        None,
    )
    .await;

    let connection = WebSocketConnection::new(ws_stream);

    // Execute handler with timeout to prevent hanging
    // match tokio::time::timeout(
    //     std::time::Duration::from_secs(30),
    //     handler.handle_connection(connection),
    // )
    // .await
    // {
    //     Ok(result) => result,
    //     Err(_) => {
    //         tracing::warn!("WebSocket handler timed out");
    //         Ok(())
    //     }
    // }
    //
    handler.handle_connection(connection).await
}

// Optimized accept key generation
fn generate_accept_key(websocket_key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(websocket_key.as_bytes());
    hasher.update(WEBSOCKET_MAGIC_STRING.as_bytes());
    base64::encode(hasher.finalize())
}
