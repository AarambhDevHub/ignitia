use super::connection::WebSocketConnection;
use super::handler::WebSocketHandler;
use crate::{Request, Response, Result};
use http::StatusCode;
use hyper_util::rt::TokioIo;
use sha1::{Digest, Sha1};
use std::sync::Arc;
use tokio_tungstenite::WebSocketStream;

const WEBSOCKET_MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub fn is_websocket_request(req: &Request) -> bool {
    let connection = req.header("connection").unwrap_or("").to_lowercase();
    let upgrade = req.header("upgrade").unwrap_or("").to_lowercase();
    let websocket_version = req.header("sec-websocket-version").unwrap_or("");

    connection.contains("upgrade")
        && upgrade.contains("websocket")
        && websocket_version == "13"
        && req.header("sec-websocket-key").is_some()
}

pub fn upgrade_connection(req: Request) -> Result<Response> {
    let websocket_key = req
        .header("sec-websocket-key")
        .ok_or_else(|| crate::Error::BadRequest("Missing Sec-WebSocket-Key header".into()))?;

    let accept_key = generate_accept_key(websocket_key);

    let mut response = Response::new(StatusCode::SWITCHING_PROTOCOLS);
    response
        .headers
        .insert("upgrade", "websocket".parse().unwrap());
    response
        .headers
        .insert("connection", "Upgrade".parse().unwrap());
    response
        .headers
        .insert("sec-websocket-accept", accept_key.parse().unwrap());

    if let Some(protocols) = req.header("sec-websocket-protocol") {
        if let Some(protocol) = protocols.split(',').next() {
            response
                .headers
                .insert("sec-websocket-protocol", protocol.trim().parse().unwrap());
        }
    }

    Ok(response)
}

pub async fn handle_websocket_upgrade(
    upgraded: hyper::upgrade::Upgraded,
    handler: Arc<dyn WebSocketHandler>,
) -> Result<()> {
    let io = TokioIo::new(upgraded);
    let ws_stream = WebSocketStream::from_raw_socket(
        io,
        tokio_tungstenite::tungstenite::protocol::Role::Server,
        None,
    )
    .await;

    let connection = WebSocketConnection::new(ws_stream);

    // Call the user's WebSocket handler
    if let Err(e) = handler.handle(connection).await {
        tracing::error!("❌ WebSocket handler error: {}", e);
    }

    Ok(())
}

fn generate_accept_key(websocket_key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(websocket_key.as_bytes());
    hasher.update(WEBSOCKET_MAGIC_STRING.as_bytes());
    let hash = hasher.finalize();
    base64::encode(hash)
}
