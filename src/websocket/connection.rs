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
    message_rx: Arc<Mutex<mpsc::Receiver<Message>>>,
}

impl WebSocketConnection {
    pub fn new(ws: WsStream) -> Self {
        let (sender, receiver) = ws.split();
        // Use bounded channel to prevent memory exhaustion
        let (message_tx, message_rx) = mpsc::channel(1024);

        let sender_arc = Arc::new(Mutex::new(sender));

        // Spawn optimized message processing task
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

    async fn message_processor(
        mut receiver: SplitStream<WsStream>,
        message_tx: mpsc::Sender<Message>,
        sender: Arc<Mutex<SplitSink<WsStream, TungsteniteMessage>>>,
    ) {
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(tungstenite_msg) => {
                    match tungstenite_msg {
                        TungsteniteMessage::Close(_) => {
                            tracing::debug!("WebSocket connection closed by client");
                            break;
                        }
                        TungsteniteMessage::Ping(data) => {
                            // Auto-respond to pings immediately
                            if let Err(e) = Self::handle_ping(&sender, data).await {
                                tracing::warn!("Failed to send pong: {}", e);
                            }
                            continue; // Skip forwarding ping messages
                        }
                        _ => {
                            if let Some(converted_msg) = convert_message(tungstenite_msg) {
                                if message_tx.send(converted_msg).await.is_err() {
                                    break; // Receiver dropped
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
    }

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

    /// Receive with timeout to prevent blocking
    pub async fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Message> {
        let mut receiver = self.message_rx.lock().await;
        tokio::time::timeout(timeout, receiver.recv())
            .await
            .ok()
            .flatten()
    }

    pub async fn send_text(&self, text: String) -> Result<()> {
        self.send(Message::Text(text)).await
    }

    pub async fn send_bytes(&self, data: bytes::Bytes) -> Result<()> {
        self.send(Message::Binary(data)).await
    }

    pub async fn send_json<T: serde::Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_vec(data)
            .map_err(|e| Error::Internal(format!("JSON serialization error: {}", e)))?;
        self.send_bytes(bytes::Bytes::from(json)).await
    }

    pub async fn close(&self) -> Result<()> {
        self.send(Message::Close(None)).await
    }

    pub async fn close_with_reason(&self, code: u16, reason: String) -> Result<()> {
        self.send(Message::Close(Some(CloseFrame { code, reason })))
            .await
    }

    pub async fn ping(&self, data: impl Into<bytes::Bytes>) -> Result<()> {
        self.send(Message::Ping(data.into())).await
    }

    pub async fn pong(&self, data: impl Into<bytes::Bytes>) -> Result<()> {
        self.send(Message::Pong(data.into())).await
    }

    /// Batch send multiple messages efficiently
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

fn convert_message(msg: TungsteniteMessage) -> Option<Message> {
    match msg {
        TungsteniteMessage::Text(text) => Some(Message::Text(text)),
        TungsteniteMessage::Binary(data) => Some(Message::Binary(bytes::Bytes::from(data))),
        TungsteniteMessage::Pong(data) => Some(Message::Pong(bytes::Bytes::from(data))),
        TungsteniteMessage::Close(frame) => Some(Message::Close(frame.map(|f| CloseFrame {
            code: f.code.into(),
            reason: f.reason.to_string(),
        }))),
        TungsteniteMessage::Ping(_) => None,
        TungsteniteMessage::Frame(_) => None,
    }
}
