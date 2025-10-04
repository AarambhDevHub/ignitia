# WebSocket Guide

This guide covers WebSocket support in Ignitia, enabling real-time, bidirectional communication between clients and servers.

## Table of Contents

- [Overview](#overview)
- [Protocol Support](#protocol-support)
- [Basic Usage](#basic-usage)
- [WebSocket Handlers](#websocket-handlers)
- [Extractor Support](#extractor-support)
- [Message Types](#message-types)
- [Connection Management](#connection-management)
- [Advanced Features](#advanced-features)
- [Error Handling](#error-handling)
- [Performance Considerations](#performance-considerations)
- [Security](#security)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Overview

Ignitia provides comprehensive WebSocket support built on top of `tokio-tungstenite`. WebSockets enable persistent, full-duplex communication channels between clients and servers, perfect for:

- **Real-time chat applications**
- **Live updates and notifications**
- **Multiplayer games**
- **Financial dashboards**
- **IoT device communication**
- **Collaborative tools**

### Key Features

- ✅ **Automatic protocol negotiation** and upgrade handling (HTTP/1.1)
- ✅ **Universal extractor support** (State, Path, Query, Headers, etc.)
- ✅ **Multiple handler types** for different use cases
- ✅ **Batch message processing** for high throughput
- ✅ **Automatic ping/pong handling** for connection health
- ✅ **JSON message support** with serialization/deserialization
- ✅ **Secure WebSocket (wss://)** over TLS/HTTPS
- ✅ **Graceful error handling** and reconnection support
- ✅ **Type-safe parameter extraction** from requests

## Protocol Support

### HTTP/1.1 WebSocket (Fully Supported) ✅

Ignitia fully supports WebSocket over HTTP/1.1 as defined in [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455):

- Standard WebSocket upgrade handshake
- Works over both HTTP (`ws://`) and HTTPS (`wss://`)
- 101 Switching Protocols response
- Full TLS encryption support

```
// Works with both:
// ws://localhost:3000/ws   (non-secure)
// wss://localhost:3000/ws  (secure over TLS)
```

### HTTP/2 WebSocket (Not Yet Supported) ⏳

HTTP/2 WebSocket support (RFC 8441) is **not currently implemented** due to limitations in the Rust ecosystem:

- Requires Hyper/h2 crate enhancements
- Limited browser support
- Most production systems use HTTP/1.1 for WebSockets

**Current behavior**: WebSocket connections automatically use HTTP/1.1 even when the server supports HTTP/2 for regular requests. Browsers handle this negotiation automatically.

### Secure WebSocket (wss://)

When using HTTPS with TLS:

- Initial HTTP/1.1 handshake occurs over TLS
- All WebSocket frames are encrypted
- Full security properties of TLS apply
- Certificate validation required in production

```rust
let server = Server::new(router, addr)
    .with_self_signed_cert("localhost")?; // For development
    // Use proper certificates in production
```

## Basic Usage

Enable WebSocket support by adding the `websocket` feature to your `Cargo.toml`:

```toml
[dependencies]
ignitia = { version = "0.2.4", features = ["websocket"] }
```

### Simple Echo Server

```rust
use ignitia::prelude::*;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .websocket("/echo", |mut ws: WebSocketConnection| async move {
            while let Some(message) = ws.recv().await {
                match message {
                    Message::Text(text) => {
                        ws.send_text(format!("Echo: {}", text)).await.ok();
                    }
                    Message::Binary(data) => {
                        ws.send_binary(data).await.ok();
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            Response::ok()
        });

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let server = Server::new(router, addr);

    println!("WebSocket server running on ws://127.0.0.1:3000/echo");
    server.ignitia().await
}
```

### Using Named Handler Functions

```rust
use ignitia::prelude::*;

async fn handle_websocket(mut ws: WebSocketConnection) -> Response {
    // Send welcome message
    ws.send_text("Welcome to the chat!").await.ok();

    while let Some(message) = ws.recv().await {
        if let Message::Text(text) = message {
            ws.send_text(format!("You said: {}", text)).await.ok();
        }
    }

    Response::ok()
}

let router = Router::new()
    .websocket("/chat", handle_websocket);
```

## WebSocket Handlers

### Return Type: IntoResponse

All WebSocket handlers must return a type that implements `IntoResponse`:

```rust
// ✅ Valid return types:
Response::ok()
"Connection closed"
Json(json!({"status": "completed"}))
(StatusCode::OK, "Success")
Result<Response>
Result<String>

// ❌ Invalid:
Ok(())  // Empty tuple doesn't implement IntoResponse
```

### Handler Styles

#### 1. Closure Handler (Most Common)

```rust
router.websocket("/ws", |mut ws: WebSocketConnection| async move {
    while let Some(msg) = ws.recv().await {
        // Handle messages
    }
    Response::ok()
})
```

#### 2. Function Pointer Handler

```rust
async fn my_handler(mut ws: WebSocketConnection) -> Response {
    // Implementation
    Response::ok()
}

router.websocket("/ws", my_handler)
```

#### 3. Message Handler (Per-Message Processing)

```rust
use ignitia::websocket::websocket_message_handler;

let handler = websocket_message_handler(|ws: WebSocketConnection, msg: Message| async move {
    match msg {
        Message::Text(text) => {
            ws.send_text(format!("Got: {}", text)).await.ok();
            Response::ok()
        }
        _ => Response::ok()
    }
});

router.websocket("/ws", handler)
```

#### 4. Batch Handler (High Throughput)

```rust
use ignitia::websocket::websocket_batch_handler;

let handler = websocket_batch_handler(
    |ws: WebSocketConnection, messages: Vec<Message>| async move {
        // Process up to 50 messages at once
        for msg in messages {
            // Handle batch...
        }
        Response::ok()
    },
    50,    // Batch size
    100    // Timeout in milliseconds
);

router.websocket("/ws/batch", handler)
```

## Extractor Support

WebSocket handlers support the same extractors as HTTP handlers, allowing you to access request data before the WebSocket upgrade.

### Available Extractors

All standard extractors work with WebSocket handlers:

- `State<T>` - Shared application state
- `Path<T>` - URL path parameters
- `Query<T>` - Query string parameters
- `Headers` - Request headers
- `Extension<T>` - Request extensions
- `Cookies` - Cookie jar
- `Json<T>` - JSON body (for POST upgrade requests)

### Basic WebSocket with State

```rust
use ignitia::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    connections: Arc<RwLock<Vec<String>>>,
}

router
    .state(AppState {
        connections: Arc::new(RwLock::new(Vec::new())),
    })
    .websocket("/ws", |
        State(state): State<AppState>,
        mut ws: WebSocketConnection
    | async move {
        // Access shared state
        state.connections.write().await.push("new_user".to_string());

        ws.send_text("Connected!").await.ok();

        while let Some(msg) = ws.recv().await {
            // Handle messages...
        }

        "Connection closed successfully"
    })
```

### WebSocket with Path Parameters

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct RoomParams {
    room_id: String,
}

router.websocket("/ws/room/{room_id}", |
    Path(params): Path<RoomParams>,
    mut ws: WebSocketConnection
| async move {
    ws.send_text(format!("Welcome to room: {}", params.room_id)).await.ok();

    while let Some(msg) = ws.recv().await {
        // Handle messages in this specific room...
    }

    Json(json!({ "room": params.room_id, "status": "closed" }))
})
```

### WebSocket with Query Parameters

```rust
#[derive(Deserialize)]
struct WsQuery {
    token: String,
    user_id: Option<String>,
}

router.websocket("/ws", |
    Query(query): Query<WsQuery>,
    mut ws: WebSocketConnection
| async move {
    // Validate token
    if !validate_token(&query.token).await {
        return Response::unauthorized("Invalid token");
    }

    ws.send_text("Authenticated!").await.ok();

    while let Some(msg) = ws.recv().await {
        // Handle authenticated messages...
    }

    Response::ok()
})
```

### WebSocket with Multiple Extractors

```rust
router.websocket("/ws/{user_id}", |
    Path(path): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    Headers(headers): Headers,
    mut ws: WebSocketConnection
| async move {
    let user_id = &path["user_id"];
    let token = query.get("token");

    // Check authentication header
    if let Some(auth) = headers.get("authorization") {
        if !is_valid_auth(auth) {
            return Response::unauthorized("Invalid auth");
        }
    }

    ws.send_text(format!("Welcome, {}", user_id)).await.ok();

    while let Some(msg) = ws.recv().await {
        // Handle messages with full context...
    }

    (StatusCode::OK, "Session ended")
})
```

### WebSocket with Extensions

```rust
#[derive(Clone)]
struct Database {
    // Your database connection
}

router
    .extension(Arc::new(Database::new()))
    .websocket("/ws", |
        Extension(db): Extension<Arc<Database>>,
        mut ws: WebSocketConnection
    | async move {
        // Access database in WebSocket handler
        let user_data = db.fetch_user_data().await;

        ws.send_json(&user_data).await.ok();

        while let Some(msg) = ws.recv().await {
            // Use database for message processing...
        }

        Response::ok()
    })
```

### Combining Up to 7 Extractors

```rust
router.websocket("/ws/{room}/{user}", |
    Path(path): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    Extension(db): Extension<Arc<Database>>,
    Headers(headers): Headers,
    Cookies(cookies): Cookies,
    // RequestId(id): RequestId,  // 7th extractor if needed
    mut ws: WebSocketConnection
| async move {
    // Access all extracted data before handling WebSocket
    let room = &path["room"];
    let user = &path["user"];
    let session = cookies.get("session_id");

    // Full context available for WebSocket handling
    ws.send_text(format!("Welcome to {} room, {}!", room, user)).await.ok();

    while let Some(msg) = ws.recv().await {
        // Process with full context...
    }

    Response::ok()
})
```

## Message Types

Ignitia supports all standard WebSocket message types:

```rust
use ignitia::websocket::{Message, CloseFrame};

// Text messages
ws.send_text("Hello, World!").await?;
ws.send(Message::Text("Hello".to_string())).await?;

// Binary messages
ws.send_binary(vec!).await?;[1]
ws.send(Message::Binary(bytes::Bytes::from(vec![1]))).await?;

// Control messages
ws.send_ping(b"ping").await?;
ws.send_pong(b"pong").await?;
ws.close(None).await?;
ws.close(Some(CloseFrame {
    code: 1000,
    reason: "Normal closure".into()
})).await?;
```

### JSON Messages

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    user: String,
    text: String,
    timestamp: u64,
}

// Send JSON
let msg = ChatMessage {
    user: "Alice".to_string(),
    text: "Hello!".to_string(),
    timestamp: 1234567890,
};
ws.send_json(&msg).await?;

// Receive and parse JSON
if let Message::Text(text) = msg {
    let parsed: ChatMessage = serde_json::from_str(&text)?;
    println!("{}: {}", parsed.user, parsed.text);
}
```

## Connection Management

### Connection Lifecycle

```rust
async fn handle_websocket(mut ws: WebSocketConnection) -> Response {
    // 1. Connection established (after HTTP upgrade)
    tracing::info!("WebSocket connected");

    // 2. Send welcome message
    ws.send_text("Welcome!").await.ok();

    // 3. Message loop
    while let Some(msg) = ws.recv().await {
        match msg {
            Message::Text(text) => {
                // Process message
            }
            Message::Close(_) => {
                // 4. Connection closing
                tracing::info!("Client closed connection");
                break;
            }
            _ => {}
        }
    }

    // 5. Cleanup and return response
    tracing::info!("WebSocket handler completed");
    Response::ok()
}
```

### Heartbeat / Keep-Alive

```rust
use tokio::time::{interval, Duration};

async fn websocket_with_heartbeat(mut ws: WebSocketConnection) -> Response {
    let mut heartbeat = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                // Send ping every 30 seconds
                if ws.send_ping(b"heartbeat").await.is_err() {
                    break;
                }
            }

            message = ws.recv() => {
                if let Some(msg) = message {
                    match msg {
                        Message::Text(text) => {
                            ws.send_text(format!("Got: {}", text)).await.ok();
                        }
                        Message::Pong(_) => {
                            tracing::debug!("Heartbeat acknowledged");
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

    Response::ok()
}
```

### Graceful Shutdown

```rust
async fn handle_websocket(mut ws: WebSocketConnection) -> Response {
    let shutdown = tokio::signal::ctrl_c();

    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            message = ws.recv() => {
                if let Some(msg) = message {
                    // Handle message...
                } else {
                    break;
                }
            }

            _ = &mut shutdown => {
                tracing::info!("Shutting down gracefully...");
                ws.close(Some(CloseFrame {
                    code: 1001,
                    reason: "Server shutdown".into()
                })).await.ok();
                break;
            }
        }
    }

    Response::ok()
}
```

## Advanced Features

### Connection Pool and Broadcasting

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

#[derive(Clone)]
struct ChatRoom {
    connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    broadcast_tx: broadcast::Sender<String>,
}

impl ChatRoom {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx: tx,
        }
    }
}

async fn handle_chat(
    State(room): State<ChatRoom>,
    mut ws: WebSocketConnection
) -> Response {
    let id = Uuid::new_v4().to_string();

    // Add connection to pool
    {
        let mut connections = room.connections.lock().await;
        connections.insert(id.clone(), ws.clone());
    }

    // Subscribe to broadcasts
    let mut rx = room.broadcast_tx.subscribe();

    // Spawn broadcast listener
    let ws_clone = ws.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = ws_clone.send_text(msg).await;
        }
    });

    // Handle incoming messages
    while let Some(msg) = ws.recv().await {
        if let Message::Text(text) = msg {
            let broadcast_msg = format!("[{}]: {}", id, text);
            let _ = room.broadcast_tx.send(broadcast_msg);
        }
    }

    // Remove connection
    {
        let mut connections = room.connections.lock().await;
        connections.remove(&id);
    }

    Response::ok()
}
```

### Rate Limiting

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

        // Remove old requests
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

async fn rate_limited_websocket(mut ws: WebSocketConnection) -> Response {
    let mut limiter = RateLimiter::new(10, Duration::from_secs(60)); // 10/min

    while let Some(msg) = ws.recv().await {
        if !limiter.is_allowed() {
            ws.send_text("Rate limit exceeded").await.ok();
            continue;
        }

        // Process message...
    }

    Response::ok()
}
```

## Error Handling

### Robust Error Handling

```rust
async fn robust_handler(mut ws: WebSocketConnection) -> Response {
    let mut error_count = 0;
    const MAX_ERRORS: usize = 5;

    while let Some(msg) = ws.recv().await {
        match process_message(&mut ws, msg).await {
            Ok(_) => {
                error_count = 0; // Reset on success
            }
            Err(e) => {
                error_count += 1;
                tracing::error!("Error ({}): {}", error_count, e);

                if error_count >= MAX_ERRORS {
                    ws.close(Some(CloseFrame {
                        code: 1011,
                        reason: "Too many errors".into()
                    })).await.ok();
                    return Response::internal_error();
                }

                ws.send_text(format!("Error: {}", e)).await.ok();
            }
        }
    }

    Response::ok()
}

async fn process_message(
    ws: &mut WebSocketConnection,
    msg: Message
) -> Result<(), Box<dyn std::error::Error>> {
    match msg {
        Message::Text(text) => {
            let result = some_processing(&text).await?;
            ws.send_text(result).await?;
        }
        _ => {}
    }
    Ok(())
}
```

## Performance Considerations

### Batch Processing

```rust
use tokio::time::{timeout, Duration};

async fn high_throughput_handler(mut ws: WebSocketConnection) -> Response {
    let mut buffer = Vec::new();
    const BATCH_SIZE: usize = 100;
    const TIMEOUT: Duration = Duration::from_millis(50);

    loop {
        match timeout(TIMEOUT, ws.recv()).await {
            Ok(Some(msg)) => {
                buffer.push(msg);

                if buffer.len() >= BATCH_SIZE {
                    process_batch(&mut ws, &mut buffer).await;
                }
            }
            Ok(None) => break,
            Err(_) => {
                if !buffer.is_empty() {
                    process_batch(&mut ws, &mut buffer).await;
                }
            }
        }
    }

    Response::ok()
}

async fn process_batch(ws: &mut WebSocketConnection, msgs: &mut Vec<Message>) {
    for msg in msgs.drain(..) {
        // Process message...
    }
}
```

## Security

### TLS/SSL Configuration

```rust
// Development (self-signed certificate)
let server = Server::new(router, addr)
    .with_self_signed_cert("localhost")?;

// Production (proper certificates)
let server = Server::new(router, addr)
    .with_server_config(ServerConfig {
        tls: Some(TlsConfig {
            cert: std::fs::read("cert.pem")?,
            key: std::fs::read("key.pem")?,
            ..Default::default()
        }),
        ..Default::default()
    });
```

### Authentication Example

```rust
async fn authenticated_websocket(
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    mut ws: WebSocketConnection
) -> Response {
    // Validate token from query
    let token = match query.get("token") {
        Some(t) => t,
        None => return Response::unauthorized("Token required"),
    };

    if !state.validate_token(token).await {
        return Response::unauthorized("Invalid token");
    }

    // Token valid - proceed with WebSocket
    ws.send_text("Authenticated successfully!").await.ok();

    while let Some(msg) = ws.recv().await {
        // Handle authenticated messages...
    }

    Response::ok()
}
```

## Examples

### Complete Chat Application

```rust
use ignitia::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    user: String,
    message: String,
}

#[derive(Clone)]
struct ChatState {
    users: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    broadcast: broadcast::Sender<ChatMessage>,
}

async fn handle_chat(
    State(state): State<ChatState>,
    Path(username): Path<String>,
    mut ws: WebSocketConnection
) -> Response {
    // Add user
    {
        let mut users = state.users.lock().await;
        users.insert(username.clone(), ws.clone());
    }

    // Subscribe to broadcasts
    let mut rx = state.broadcast.subscribe();
    let ws_clone = ws.clone();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            ws_clone.send_json(&msg).await.ok();
        }
    });

    // Handle messages
    while let Some(msg) = ws.recv().await {
        if let Message::Text(text) = msg {
            let chat_msg = ChatMessage {
                user: username.clone(),
                message: text,
            };
            state.broadcast.send(chat_msg).ok();
        }
    }

    // Remove user
    {
        let mut users = state.users.lock().await;
        users.remove(&username);
    }

    Response::ok()
}

#[tokio::main]
async fn main() -> Result<()> {
    let (broadcast_tx, _) = broadcast::channel(100);

    let state = ChatState {
        users: Arc::new(Mutex::new(HashMap::new())),
        broadcast: broadcast_tx,
    };

    let router = Router::new()
        .state(state)
        .websocket("/chat/{username}", handle_chat);

    let server = Server::new(router, "127.0.0.1:3000".parse()?);
    server.ignitia().await
}
```

## Best Practices

1. ✅ **Always return IntoResponse types** - Use `Response::ok()`, `String`, `Json<T>`, etc.
2. ✅ **Handle connection closure gracefully** - Check for `Message::Close`
3. ✅ **Use extractors for request data** - Leverage State, Path, Query, etc.
4. ✅ **Implement heartbeat mechanisms** - Keep long-lived connections alive
5. ✅ **Validate all incoming messages** - Don't trust client input
6. ✅ **Use structured logging** - Log connection lifecycle events
7. ✅ **Implement rate limiting** - Prevent abuse on public endpoints
8. ✅ **Use TLS in production** - Always use `wss://` for security
9. ✅ **Handle errors robustly** - Implement retry logic and error recovery
10. ✅ **Consider message batching** - For high-throughput scenarios

## Additional Resources

- [RFC 6455 - The WebSocket Protocol](https://datatracker.ietf.org/doc/html/rfc6455)
- [tokio-tungstenite Documentation](https://docs.rs/tokio-tungstenite)
- [Ignitia Examples](../examples/)
- [WebSocket Security Best Practices](https://owasp.org/www-community/vulnerabilities/WebSocket_security)

---

**Note**: This documentation reflects Ignitia v0.2+ with full extractor support and improved WebSocket handling. For older versions, refer to the legacy documentation.
