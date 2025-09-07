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

type WsStream = WebSocketStream<TokioIo<hyper::upgrade::Upgraded>>;

#[derive(Clone)]
pub struct WebSocketConnection {
    sender: Arc<Mutex<SplitSink<WsStream, TungsteniteMessage>>>,
    // Channel for receiving messages
    message_rx: Arc<Mutex<mpsc::UnboundedReceiver<Message>>>,
    // Keep the sender for the message channel
    _message_tx: Arc<mpsc::UnboundedSender<Message>>,
}

impl WebSocketConnection {
    pub fn new(ws: WsStream) -> Self {
        let (sender, mut receiver) = ws.split();
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let message_tx = Arc::new(message_tx);
        let message_tx_clone = Arc::clone(&message_tx);

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(msg_result) = receiver.next().await {
                match msg_result {
                    Ok(tungstenite_msg) => {
                        match tungstenite_msg {
                            TungsteniteMessage::Close(_) => {
                                tracing::info!("🔌 WebSocket connection closed by client");
                                break;
                            }
                            TungsteniteMessage::Ping(data) => {
                                // Auto-respond to pings - this should be handled at the sender level
                                if let Some(converted_msg) = convert_message(TungsteniteMessage::Ping(data)) {
                                    let _ = message_tx_clone.send(converted_msg);
                                }
                            }
                            _ => {
                                if let Some(converted_msg) = convert_message(tungstenite_msg) {
                                    if message_tx_clone.send(converted_msg).is_err() {
                                        break; // Receiver dropped
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("❌ WebSocket message error: {}", e);
                        break;
                    }
                }
            }
        });

        Self {
            sender: Arc::new(Mutex::new(sender)),
            message_rx: Arc::new(Mutex::new(message_rx)),
            _message_tx: message_tx,
        }
    }

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

    /// Receive the next message from the WebSocket
    pub async fn recv(&self) -> Option<Message> {
        let mut receiver = self.message_rx.lock().await;
        receiver.recv().await
    }

    /// Auto-handle ping with pong response
    async fn handle_ping(&self, data: bytes::Bytes) -> Result<()> {
        self.send(Message::Pong(data)).await
    }

    pub async fn send_text(&self, text: impl Into<String>) -> Result<()> {
        self.send(Message::Text(text.into())).await
    }

    pub async fn send_binary(&self, data: impl Into<bytes::Bytes>) -> Result<()> {
        self.send(Message::Binary(data.into())).await
    }

    pub async fn send_json<T: serde::Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_string(data)
            .map_err(|e| Error::Internal(format!("JSON serialization error: {}", e)))?;
        self.send_text(json).await
    }

    pub async fn ping(&self, data: impl Into<bytes::Bytes>) -> Result<()> {
        self.send(Message::Ping(data.into())).await
    }

    pub async fn pong(&self, data: impl Into<bytes::Bytes>) -> Result<()> {
        self.send(Message::Pong(data.into())).await
    }

    pub async fn close(&self) -> Result<()> {
        self.send(Message::Close(None)).await
    }

    pub async fn close_with_reason(&self, code: u16, reason: impl Into<String>) -> Result<()> {
        self.send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })))
        .await
    }
}

// Helper function to convert from tungstenite message to our message
pub fn convert_message(msg: TungsteniteMessage) -> Option<Message> {
    match msg {
        TungsteniteMessage::Text(text) => Some(Message::Text(text)),
        TungsteniteMessage::Binary(data) => Some(Message::Binary(bytes::Bytes::from(data))),
        TungsteniteMessage::Ping(data) => Some(Message::Ping(bytes::Bytes::from(data))),
        TungsteniteMessage::Pong(data) => Some(Message::Pong(bytes::Bytes::from(data))),
        TungsteniteMessage::Close(frame) => Some(Message::Close(frame.map(|f| CloseFrame {
            code: f.code.into(),
            reason: f.reason.to_string(),
        }))),
        TungsteniteMessage::Frame(_) => None,
    }
}
