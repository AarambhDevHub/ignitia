#[cfg(feature = "websocket")]
pub mod connection;
#[cfg(feature = "websocket")]
pub mod handler;
#[cfg(feature = "websocket")]
pub mod message;
#[cfg(feature = "websocket")]
pub mod upgrade;

#[cfg(feature = "websocket")]
pub use connection::WebSocketConnection;
#[cfg(feature = "websocket")]
pub use handler::{
    websocket_batch_handler, websocket_handler, websocket_message_handler, BatchMessageHandler,
    OptimizedMessageHandler, WebSocketHandler,
};
#[cfg(feature = "websocket")]
pub use message::{CloseFrame, Message, MessageType};
#[cfg(feature = "websocket")]
pub use upgrade::{handle_websocket_upgrade, is_websocket_request, upgrade_connection};

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

#[cfg(not(feature = "websocket"))]
pub fn websocket_handler<F>(_f: F) -> impl WebSocketHandler {
    panic!("WebSocket support is not enabled. Add 'websocket' feature to your Cargo.toml");
}

#[cfg(not(feature = "websocket"))]
pub fn is_websocket_request(_req: &crate::Request) -> bool {
    false
}

#[cfg(not(feature = "websocket"))]
pub fn upgrade_connection(_req: crate::Request) -> crate::Result<crate::Response> {
    Err(crate::Error::Internal("WebSocket not supported".into()))
}
