use super::connection::WebSocketConnection;
use super::message::Message;
use crate::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[async_trait::async_trait]
pub trait WebSocketHandler: Send + Sync {
    async fn handle(&self, websocket: WebSocketConnection) -> Result<()>;

    /// Called when a message is received (optional override)
    async fn on_message(&self, _websocket: &WebSocketConnection, _message: Message) -> Result<()> {
        Ok(())
    }

    /// Called when connection is established (optional override)
    async fn on_connect(&self, _websocket: &WebSocketConnection) -> Result<()> {
        Ok(())
    }

    /// Called when connection is closed (optional override)
    async fn on_disconnect(&self, _websocket: &WebSocketConnection) -> Result<()> {
        Ok(())
    }
}

pub type WebSocketHandlerFn =
    Arc<dyn Fn(WebSocketConnection) -> BoxFuture<'static, Result<()>> + Send + Sync>;

#[async_trait::async_trait]
impl WebSocketHandler for WebSocketHandlerFn {
    async fn handle(&self, websocket: WebSocketConnection) -> Result<()> {
        (self)(websocket).await
    }
}

pub fn websocket_handler<F, Fut>(f: F) -> WebSocketHandlerFn
where
    F: Fn(WebSocketConnection) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    Arc::new(move |ws| Box::pin(f(ws)))
}

/// Convenient message-based handler
pub fn websocket_message_handler<F, Fut>(f: F) -> impl WebSocketHandler
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    MessageHandler { handler: Arc::new(f) }
}

struct MessageHandler<F> {
    handler: Arc<F>,
}

#[async_trait::async_trait]
impl<F, Fut> WebSocketHandler for MessageHandler<F>
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    async fn handle(&self, websocket: WebSocketConnection) -> Result<()> {
        loop {
            match websocket.recv().await {
                Some(Message::Close(_)) => {
                    tracing::info!("🔌 WebSocket connection closed");
                    break;
                }
                Some(Message::Ping(data)) => {
                    // Auto-respond to pings
                    let _ = websocket.pong(data).await;
                }
                Some(message) => {
                    if let Err(e) = (self.handler)(websocket.clone(), message).await {
                        tracing::error!("❌ WebSocket message handler error: {}", e);
                        break;
                    }
                }
                None => {
                    tracing::info!("🔌 WebSocket connection closed (no more messages)");
                    break;
                }
            }
        }
        Ok(())
    }
}
