use super::connection::WebSocketConnection;
use super::message::Message;
use crate::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[async_trait::async_trait]
pub trait WebSocketHandler: Send + Sync {
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()>;

    async fn on_message(&self, _websocket: &WebSocketConnection, _message: Message) -> Result<()> {
        Ok(())
    }

    async fn on_connect(&self, _websocket: &WebSocketConnection) -> Result<()> {
        Ok(())
    }

    async fn on_disconnect(
        &self,
        _websocket: &WebSocketConnection,
        _reason: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }
}

pub type WebSocketHandlerFn =
    Arc<dyn Fn(WebSocketConnection) -> BoxFuture<'static, Result<()>> + Send + Sync>;

#[async_trait::async_trait]
impl WebSocketHandler for WebSocketHandlerFn {
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
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

pub struct BatchMessageHandler<F> {
    handler: Arc<F>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl<F> BatchMessageHandler<F> {
    pub fn new(handler: F, batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            handler: Arc::new(handler),
            batch_size,
            batch_timeout,
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut> WebSocketHandler for BatchMessageHandler<F>
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
        let mut message_batch = Vec::with_capacity(self.batch_size);

        loop {
            match websocket.recv().await {
                Some(Message::Close(_)) => {
                    break;
                }
                Some(message) => {
                    message_batch.push(message);

                    if message_batch.len() >= self.batch_size {
                        if let Err(e) =
                            (self.handler)(websocket.clone(), std::mem::take(&mut message_batch))
                                .await
                        {
                            tracing::debug!("WebSocket batch handler error: {}", e);
                            break;
                        }
                        message_batch = Vec::with_capacity(self.batch_size);
                    }
                }
                None => {
                    if !message_batch.is_empty() {
                        if let Err(e) =
                            (self.handler)(websocket.clone(), std::mem::take(&mut message_batch))
                                .await
                        {
                            tracing::debug!("WebSocket batch handler error: {}", e);
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }
}

pub struct OptimizedMessageHandler<F> {
    handler: Arc<F>,
}

impl<F> OptimizedMessageHandler<F> {
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut> WebSocketHandler for OptimizedMessageHandler<F>
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()> {
        while let Some(message) = websocket.recv().await {
            match message {
                Message::Close(_) => break,
                _ => {
                    if let Err(e) = (self.handler)(websocket.clone(), message).await {
                        tracing::debug!("WebSocket message handler error: {}", e);
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn websocket_message_handler<F, Fut>(f: F) -> impl WebSocketHandler
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    OptimizedMessageHandler::new(f)
}

pub fn websocket_batch_handler<F, Fut>(
    f: F,
    batch_size: usize,
    timeout_ms: u64,
) -> impl WebSocketHandler
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    BatchMessageHandler::new(f, batch_size, Duration::from_millis(timeout_ms))
}
