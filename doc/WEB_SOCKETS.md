# WebSocket Guide

This guide covers WebSocket support in Ignitia, enabling real-time, bidirectional communication between clients and servers.

## Table of Contents

- [Overview](#overview)
- [Basic Usage](#basic-usage)
- [WebSocket Handlers](#websocket-handlers)
- [Message Types](#message-types)
- [Connection Management](#connection-management)
- [Advanced Features](#advanced-features)
- [Error Handling](#error-handling)
- [Performance Considerations](#performance-considerations)
- [Examples](#examples)

## Overview

Ignitia provides comprehensive WebSocket support built on top of `tokio-tungstenite`. WebSockets enable persistent, full-duplex communication channels between clients and servers, perfect for:

- **Real-time chat applications**
- **Live updates and notifications**
- **Multiplayer games**
- **Financial dashboards**
- **IoT device communication**
- **Collaborative tools**

### Key Features

- ✅ **Automatic protocol negotiation** and upgrade handling
- ✅ **Multiple handler types** for different use cases
- ✅ **Batch message processing** for high throughput
- ✅ **Automatic ping/pong handling** for connection health
- ✅ **JSON message support** with serialization/deserialization
- ✅ **Connection pooling** and management
- ✅ **Graceful error handling** and reconnection support

## Basic Usage

Enable WebSocket support by adding the `websocket` feature to your `Cargo.toml`:

```toml
[dependencies]
ignitia = { version = "0.2.1", features = ["websocket"] }
```

### Simple Echo Server

```rust
use ignitia::{Router, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .websocket("/echo", |mut ws| async move {
            while let Some(message) = ws.recv().await {
                match message {
                    ignitia::Message::Text(text) => {
                        ws.send_text(format!("Echo: {}", text)).await?;
                    }
                    ignitia::Message::Binary(data) => {
                        ws.send_bytes(data).await?;
                    }
                    ignitia::Message::Close(_) => break,
                    _ => {} // Handle other message types as needed
                }
            }
            Ok(())
        });

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let server = Server::new(app, addr);

    println!("WebSocket server running on ws://127.0.0.1:3000/echo");
    server.ignitia().await
}
```

### Using `.websocket_fn()` Method

```rust
use ignitia::{websocket::WebSocketConnection, Router, Result};

async fn handle_websocket(mut ws: WebSocketConnection) -> Result<()> {
    // Send welcome message
    ws.send_text("Welcome to the chat!").await?;

    while let Some(message) = ws.recv().await {
        if let ignitia::Message::Text(text) = message {
            // Broadcast to all connections (implement your logic)
            ws.send_text(format!("You said: {}", text)).await?;
        }
    }
    Ok(())
}

let app = Router::new()
    .websocket_fn("/chat", handle_websocket);
```

## WebSocket Handlers

Ignitia provides several handler types for different use cases:

### 1. Basic Handler

For simple message-by-message processing:

```rust
use ignitia::websocket::{websocket_handler, WebSocketConnection};

let handler = websocket_handler(|mut ws: WebSocketConnection| async move {
    while let Some(message) = ws.recv().await {
        // Process message
        match message {
            ignitia::Message::Text(text) => {
                println!("Received: {}", text);
                ws.send_text("Acknowledged").await?;
            }
            ignitia::Message::Close(_) => break,
            _ => {}
        }
    }
    Ok(())
});

let app = Router::new().websocket("/simple", handler);
```

### 2. Message Handler

Optimized for processing individual messages:

```rust
use ignitia::websocket::{websocket_message_handler, WebSocketConnection, Message};

let handler = websocket_message_handler(|ws: WebSocketConnection, message: Message| async move {
    match message {
        Message::Text(text) => {
            // Process text message
            let response = process_text_message(&text).await?;
            ws.send_text(response).await?;
        }
        Message::Binary(data) => {
            // Process binary message
            let processed = process_binary_data(&data).await?;
            ws.send_bytes(processed).await?;
        }
        _ => {}
    }
    Ok(())
});
```

### 3. Batch Handler

For high-throughput scenarios where you want to process messages in batches:

```rust
use ignitia::websocket::{websocket_batch_handler, WebSocketConnection, Message};

let handler = websocket_batch_handler(
    |ws: WebSocketConnection, messages: Vec<Message>| async move {
        // Process batch of messages
        let responses = process_message_batch(messages).await?;

        // Send batch response
        ws.send_batch(responses).await?;
        Ok(())
    },
    50,    // Batch size
    100,   // Timeout in milliseconds
);
```

## Message Types

Ignitia supports all standard WebSocket message types:

```rust
use ignitia::websocket::{Message, CloseFrame};

// Text messages
let text_msg = Message::text("Hello, World!");
let text_msg = Message::Text("Hello".to_string());

// Binary messages
let binary_msg = Message::binary(vec![1, 2, 3, 4]);
let binary_msg = Message::Binary(bytes::Bytes::from("data"));

// Control messages
let ping = Message::ping(b"ping data");
let pong = Message::pong(b"pong data");
let close = Message::close();
let close_with_reason = Message::close_with_reason(1000, "Normal closure");

// JSON messages (convenience method)
#[derive(Serialize)]
struct ChatMessage {
    user: String,
    text: String,
    timestamp: u64,
}

let json_msg = Message::json(&ChatMessage {
    user: "Alice".to_string(),
    text: "Hello everyone!".to_string(),
    timestamp: 1634567890,
})?;
```

### Working with JSON Messages

```rust
use serde::{Deserialize, Serialize};
use ignitia::websocket::{WebSocketConnection, Message};

#[derive(Serialize, Deserialize)]
struct GameMove {
    player_id: u32,
    x: i32,
    y: i32,
    piece: String,
}

async fn handle_game_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    while let Some(message) = ws.recv().await {
        if let Message::Text(text) = message {
            // Parse incoming JSON
            match serde_json::from_str::<GameMove>(&text) {
                Ok(game_move) => {
                    // Process game move
                    let response = process_move(game_move).await;

                    // Send JSON response
                    ws.send_json(&response).await?;
                }
                Err(e) => {
                    ws.send_text(format!("Invalid JSON: {}", e)).await?;
                }
            }
        }
    }
    Ok(())
}
```

## Connection Management

### Connection Information

```rust
async fn handle_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    // The connection is already established at this point
    println!("New WebSocket connection established");

    // Send welcome message
    ws.send_text("Connected successfully!").await?;

    // Handle messages...
    Ok(())
}
```

### Ping/Pong Handling

Ignitia automatically handles ping/pong frames for connection health:

```rust
async fn handle_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    while let Some(message) = ws.recv().await {
        match message {
            Message::Text(text) => {
                ws.send_text(format!("Echo: {}", text)).await?;
            }
            Message::Ping(data) => {
                // Automatically handled, but you can add custom logic
                println!("Received ping with {} bytes", data.len());
            }
            Message::Pong(data) => {
                println!("Received pong with {} bytes", data.len());
            }
            Message::Close(_) => {
                println!("Connection closed by client");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
```

### Manual Ping/Pong

```rust
use tokio::time::{interval, Duration};

async fn websocket_with_heartbeat(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    let mut heartbeat = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                // Send ping every 30 seconds
                ws.ping("heartbeat").await?;
            }

            message = ws.recv() => {
                if let Some(msg) = message {
                    match msg {
                        Message::Text(text) => {
                            ws.send_text(format!("Got: {}", text)).await?;
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                } else {
                    break; // Connection closed
                }
            }
        }
    }

    Ok(())
}
```

### Graceful Shutdown

```rust
async fn handle_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    // Set up graceful shutdown
    let shutdown_signal = tokio::signal::ctrl_c();

    loop {
        tokio::select! {
            message = ws.recv() => {
                if let Some(msg) = message {
                    // Handle message...
                } else {
                    break;
                }
            }

            _ = shutdown_signal => {
                println!("Shutting down gracefully...");
                ws.close_with_reason(1001, "Server shutdown").await?;
                break;
            }
        }
    }

    Ok(())
}
```

## Advanced Features

### Connection Pooling and Broadcasting

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

type ConnectionPool = Arc<Mutex<HashMap<String, WebSocketConnection>>>;

struct ChatServer {
    connections: ConnectionPool,
}

impl ChatServer {
    fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_connection(&self, ws: WebSocketConnection) -> ignitia::Result<()> {
        let id = Uuid::new_v4().to_string();

        // Add connection to pool
        {
            let mut connections = self.connections.lock().await;
            connections.insert(id.clone(), ws.clone());
        }

        // Handle messages
        while let Some(message) = ws.recv().await {
            if let Message::Text(text) = message {
                self.broadcast_message(&text, &id).await?;
            }
        }

        // Remove connection from pool
        {
            let mut connections = self.connections.lock().await;
            connections.remove(&id);
        }

        Ok(())
    }

    async fn broadcast_message(&self, message: &str, sender_id: &str) -> ignitia::Result<()> {
        let connections = self.connections.lock().await;
        let broadcast_msg = format!("[{}]: {}", sender_id, message);

        for (id, connection) in connections.iter() {
            if id != sender_id {
                let _ = connection.send_text(broadcast_msg.clone()).await;
            }
        }

        Ok(())
    }
}

// Usage in router
let chat_server = Arc::new(ChatServer::new());

let app = Router::new()
    .websocket("/chat", {
        let server = Arc::clone(&chat_server);
        move |ws| {
            let server = Arc::clone(&server);
            async move {
                server.handle_connection(ws).await
            }
        }
    });
```

### Rate Limiting WebSocket Messages

```rust
use tokio::time::{Duration, Instant};
use std::collections::VecDeque;

struct RateLimiter {
    requests: VecDeque<Instant>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: VecDeque::new(),
            max_requests,
            window,
        }
    }

    fn is_allowed(&mut self) -> bool {
        let now = Instant::now();

        // Remove old requests outside the window
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        if self.requests.len() < self.max_requests {
            self.requests.push_back(now);
            true
        } else {
            false
        }
    }
}

async fn rate_limited_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    let mut rate_limiter = RateLimiter::new(10, Duration::from_secs(60)); // 10 req/min

    while let Some(message) = ws.recv().await {
        if !rate_limiter.is_allowed() {
            ws.send_text("Rate limit exceeded. Please slow down.").await?;
            continue;
        }

        // Process message...
        match message {
            Message::Text(text) => {
                ws.send_text(format!("Processed: {}", text)).await?;
            }
            _ => {}
        }
    }

    Ok(())
}
```

## Error Handling

### Connection Errors

```rust
use ignitia::websocket::{WebSocketConnection, Message};

async fn robust_websocket_handler(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    loop {
        match ws.recv_timeout(Duration::from_secs(30)).await {
            Some(message) => {
                match message {
                    Message::Text(text) => {
                        if let Err(e) = process_message(&text).await {
                            eprintln!("Error processing message: {}", e);
                            ws.send_text("Error processing your message").await?;
                        }
                    }
                    Message::Close(frame) => {
                        if let Some(frame) = frame {
                            println!("Connection closed: {} - {}", frame.code, frame.reason);
                        }
                        break;
                    }
                    _ => {}
                }
            }
            None => {
                // Timeout - connection might be dead
                println!("No message received within timeout, checking connection...");
                if let Err(_) = ws.ping("keepalive").await {
                    println!("Connection lost");
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn process_message(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Your message processing logic
    Ok(())
}
```

### Error Recovery

```rust
async fn resilient_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    let mut error_count = 0;
    const MAX_ERRORS: usize = 5;

    while let Some(message) = ws.recv().await {
        match handle_message(&mut ws, message).await {
            Ok(_) => {
                error_count = 0; // Reset on successful processing
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Error handling message ({}): {}", error_count, e);

                if error_count >= MAX_ERRORS {
                    ws.close_with_reason(1011, "Too many errors").await?;
                    break;
                }

                // Send error response
                ws.send_text(format!("Error: {}", e)).await?;
            }
        }
    }

    Ok(())
}

async fn handle_message(
    ws: &mut WebSocketConnection,
    message: Message
) -> Result<(), Box<dyn std::error::Error>> {
    match message {
        Message::Text(text) => {
            // Process text message
            let result = some_processing_function(&text).await?;
            ws.send_text(result).await?;
        }
        Message::Binary(data) => {
            // Process binary data
            let processed = process_binary(&data).await?;
            ws.send_bytes(processed).await?;
        }
        _ => {}
    }
    Ok(())
}
```

## Performance Considerations

### Batch Processing for High Throughput

```rust
use tokio::time::{timeout, Duration};

async fn high_throughput_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    let mut message_buffer = Vec::new();
    const BATCH_SIZE: usize = 100;
    const BATCH_TIMEOUT: Duration = Duration::from_millis(50);

    loop {
        match timeout(BATCH_TIMEOUT, ws.recv()).await {
            Ok(Some(message)) => {
                message_buffer.push(message);

                if message_buffer.len() >= BATCH_SIZE {
                    process_message_batch(&mut ws, &mut message_buffer).await?;
                }
            }
            Ok(None) => break, // Connection closed
            Err(_) => {
                // Timeout - process accumulated messages
                if !message_buffer.is_empty() {
                    process_message_batch(&mut ws, &mut message_buffer).await?;
                }
            }
        }
    }

    Ok(())
}

async fn process_message_batch(
    ws: &mut WebSocketConnection,
    messages: &mut Vec<Message>
) -> ignitia::Result<()> {
    // Process all messages in batch
    let responses = messages
        .drain(..)
        .map(|msg| process_single_message(msg))
        .collect::<Vec<_>>();

    // Send responses in batch
    let response_messages: Vec<Message> = responses
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    if !response_messages.is_empty() {
        ws.send_batch(response_messages).await?;
    }

    Ok(())
}

fn process_single_message(message: Message) -> Result<Message, Box<dyn std::error::Error>> {
    match message {
        Message::Text(text) => {
            let processed = text.to_uppercase();
            Ok(Message::Text(processed))
        }
        _ => Err("Unsupported message type".into())
    }
}
```

### Memory Optimization

```rust
async fn memory_efficient_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    // Pre-allocate buffers to avoid frequent allocations
    let mut response_buffer = String::with_capacity(1024);

    while let Some(message) = ws.recv().await {
        match message {
            Message::Text(text) => {
                response_buffer.clear();

                // Process message without creating new strings
                response_buffer.push_str("Echo: ");
                response_buffer.push_str(&text);

                ws.send_text(response_buffer.clone()).await?;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}
```

## Examples

### Real-Time Chat Application

```rust
use ignitia::{Router, Server, websocket::WebSocketConnection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    user: String,
    message: String,
    timestamp: u64,
}

struct ChatRoom {
    connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    broadcast: broadcast::Sender<ChatMessage>,
}

impl ChatRoom {
    fn new() -> Self {
        let (broadcast, _) = broadcast::channel(100);
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            broadcast,
        }
    }

    async fn handle_user(&self, ws: WebSocketConnection, user_id: String) -> ignitia::Result<()> {
        // Add user to room
        {
            let mut connections = self.connections.lock().await;
            connections.insert(user_id.clone(), ws.clone());
        }

        // Subscribe to broadcasts
        let mut receiver = self.broadcast.subscribe();

        // Spawn broadcast listener
        let ws_clone = ws.clone();
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                let _ = ws_clone.send_json(&message).await;
            }
        });

        // Handle incoming messages
        while let Some(message) = ws.recv().await {
            if let Message::Text(text) = message {
                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    // Broadcast to all users
                    let _ = self.broadcast.send(chat_msg);
                }
            }
        }

        // Remove user from room
        {
            let mut connections = self.connections.lock().await;
            connections.remove(&user_id);
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chat_room = Arc::new(ChatRoom::new());

    let app = Router::new()
        .websocket("/chat/:user_id", {
            let room = Arc::clone(&chat_room);
            move |ws: WebSocketConnection, path: ignitia::Path<String>| {
                let room = Arc::clone(&room);
                async move {
                    room.handle_user(ws, path.into_inner()).await
                }
            }
        });

    let server = Server::new(app, "127.0.0.1:3000".parse()?);
    server.ignitia().await
}
```

### WebSocket with Authentication

```rust
use ignitia::{Router, middleware::AuthMiddleware, websocket::WebSocketConnection};

async fn authenticated_websocket(
    ws: WebSocketConnection,
    auth: ignitia::Extension<AuthInfo>
) -> ignitia::Result<()> {
    let user = auth.user_id;
    println!("Authenticated user {} connected", user);

    // Send personalized welcome
    ws.send_json(&serde_json::json!({
        "type": "welcome",
        "user_id": user,
        "message": "Welcome to the authenticated channel!"
    })).await?;

    while let Some(message) = ws.recv().await {
        // Handle authenticated user messages
        match message {
            Message::Text(text) => {
                // Process message with user context
                let response = process_authenticated_message(&user, &text).await?;
                ws.send_text(response).await?;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}

#[derive(Clone)]
struct AuthInfo {
    user_id: String,
    permissions: Vec<String>,
}

let app = Router::new()
    .middleware(AuthMiddleware::new("secret-token"))
    .websocket("/secure-chat", authenticated_websocket);
```

### Game Server Example

```rust
use ignitia::websocket::{WebSocketConnection, Message};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum GameMessage {
    Join { player_name: String },
    Move { x: i32, y: i32 },
    Leave,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum GameResponse {
    Joined { player_id: u32, game_state: String },
    MoveResult { success: bool, new_state: String },
    Error { message: String },
}

async fn game_websocket(mut ws: WebSocketConnection) -> ignitia::Result<()> {
    let mut player_id: Option<u32> = None;

    while let Some(message) = ws.recv().await {
        if let Message::Text(text) = message {
            match serde_json::from_str::<GameMessage>(&text) {
                Ok(game_msg) => match game_msg {
                    GameMessage::Join { player_name } => {
                        let id = create_player(&player_name).await;
                        player_id = Some(id);

                        ws.send_json(&GameResponse::Joined {
                            player_id: id,
                            game_state: get_game_state().await,
                        }).await?;
                    }

                    GameMessage::Move { x, y } => {
                        if let Some(id) = player_id {
                            let success = make_move(id, x, y).await;
                            ws.send_json(&GameResponse::MoveResult {
                                success,
                                new_state: get_game_state().await,
                            }).await?;
                        }
                    }

                    GameMessage::Leave => {
                        if let Some(id) = player_id {
                            remove_player(id).await;
                        }
                        break;
                    }
                }

                Err(e) => {
                    ws.send_json(&GameResponse::Error {
                        message: format!("Invalid message: {}", e),
                    }).await?;
                }
            }
        }
    }

    // Cleanup on disconnect
    if let Some(id) = player_id {
        remove_player(id).await;
    }

    Ok(())
}

// Mock game functions
async fn create_player(name: &str) -> u32 { 1 }
async fn get_game_state() -> String { "game_state".to_string() }
async fn make_move(player_id: u32, x: i32, y: i32) -> bool { true }
async fn remove_player(player_id: u32) {}
```

***

## Best Practices

1. **Always handle connection closure gracefully**
2. **Implement proper error recovery mechanisms**
3. **Use appropriate handler types for your use case**
4. **Consider rate limiting for public endpoints**
5. **Validate all incoming messages**
6. **Use structured logging for debugging**
7. **Implement heartbeat/ping mechanisms for long-lived connections**
8. **Consider message queuing for high-throughput scenarios**
9. **Use JSON for structured data exchange**
10. **Implement proper authentication and authorization**

For more examples and advanced patterns, check the [examples directory](../examples/) in the Ignitia repository.
