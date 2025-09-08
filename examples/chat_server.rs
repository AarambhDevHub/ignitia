use ignitia::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

#[cfg(feature = "websocket")]
use ignitia::websocket::{Message, WebSocketConnection};

// Chat message structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    message: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "join")]
    Join { username: String },
    #[serde(rename = "message")]
    Message { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "user_joined")]
    UserJoined { username: String, count: usize },
    #[serde(rename = "user_left")]
    UserLeft { username: String, count: usize },
    #[serde(rename = "message")]
    Message(ChatMessage),
    #[serde(rename = "error")]
    Error { message: String },
}

// Shared state for the chat server
#[derive(Clone)]
struct ChatState {
    #[cfg(feature = "websocket")]
    broadcaster: broadcast::Sender<ServerMessage>,
    users: Arc<RwLock<HashMap<String, String>>>, // username -> connection_id
}

impl ChatState {
    fn new() -> Self {
        #[cfg(feature = "websocket")]
        let (broadcaster, _) = broadcast::channel(1000);

        Self {
            #[cfg(feature = "websocket")]
            broadcaster,
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[cfg(feature = "websocket")]
    async fn add_user(&self, username: String) -> usize {
        let mut users = self.users.write().await;
        users.insert(username.clone(), "active".to_string());
        let count = users.len();
        drop(users);

        let _ = self
            .broadcaster
            .send(ServerMessage::UserJoined { username, count });
        count
    }

    #[cfg(feature = "websocket")]
    async fn remove_user(&self, username: &str) -> usize {
        let mut users = self.users.write().await;
        users.remove(username);
        let count = users.len();
        drop(users);

        let _ = self.broadcaster.send(ServerMessage::UserLeft {
            username: username.to_string(),
            count,
        });
        count
    }

    #[cfg(feature = "websocket")]
    async fn broadcast_message(&self, message: ChatMessage) {
        let _ = self.broadcaster.send(ServerMessage::Message(message));
    }

    async fn get_user_count(&self) -> usize {
        self.users.read().await.len()
    }
}

// Serve the chat UI
async fn serve_chat_ui() -> Result<Response> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🔥 Ignitia Chat</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Arial', sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .chat-container {
            background: white;
            border-radius: 20px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
            width: 90%;
            max-width: 600px;
            height: 80vh;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }

        .chat-header {
            background: linear-gradient(45deg, #ff6b6b, #ee5a24);
            color: white;
            padding: 20px;
            text-align: center;
            font-size: 1.3em;
            font-weight: bold;
        }

        .user-count {
            font-size: 0.9em;
            opacity: 0.9;
            margin-top: 5px;
        }

        .messages {
            flex: 1;
            overflow-y: auto;
            padding: 20px;
            background: #f8f9fa;
        }

        .message {
            margin-bottom: 15px;
            padding: 12px 16px;
            border-radius: 15px;
            max-width: 80%;
            word-wrap: break-word;
            animation: slideIn 0.3s ease;
        }

        @keyframes slideIn {
            from { opacity: 0; transform: translateY(20px); }
            to { opacity: 1; transform: translateY(0); }
        }

        .message.own {
            background: #007bff;
            color: white;
            margin-left: auto;
            text-align: right;
        }

        .message.other {
            background: white;
            border: 2px solid #e9ecef;
        }

        .message.system {
            background: #28a745;
            color: white;
            margin: 5px auto;
            text-align: center;
            max-width: 60%;
            font-style: italic;
            font-size: 0.9em;
        }

        .username {
            font-weight: bold;
            margin-bottom: 4px;
            color: #666;
        }

        .message.own .username {
            color: #cce7ff;
        }

        .timestamp {
            font-size: 0.8em;
            opacity: 0.7;
            margin-top: 4px;
        }

        .input-area {
            padding: 20px;
            background: white;
            border-top: 1px solid #dee2e6;
            display: flex;
            gap: 10px;
        }

        #messageInput {
            flex: 1;
            padding: 12px;
            border: 2px solid #dee2e6;
            border-radius: 20px;
            outline: none;
            font-size: 16px;
        }

        #messageInput:focus {
            border-color: #007bff;
        }

        #sendButton {
            padding: 12px 20px;
            background: #007bff;
            color: white;
            border: none;
            border-radius: 20px;
            cursor: pointer;
            font-weight: bold;
        }

        #sendButton:hover {
            background: #0056b3;
        }

        #sendButton:disabled {
            background: #6c757d;
            cursor: not-allowed;
        }

        .login-overlay {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0,0,0,0.8);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 1000;
        }

        .login-form {
            background: white;
            padding: 30px;
            border-radius: 15px;
            text-align: center;
            min-width: 300px;
        }

        .login-form h2 {
            margin-bottom: 20px;
            color: #333;
        }

        .login-form input {
            width: 100%;
            padding: 12px;
            margin-bottom: 15px;
            border: 2px solid #dee2e6;
            border-radius: 8px;
            font-size: 16px;
        }

        .login-form button {
            width: 100%;
            padding: 12px;
            background: #007bff;
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: bold;
            cursor: pointer;
        }
    </style>
</head>
<body>
    <div class="chat-container">
        <div class="chat-header">
            🔥 Ignitia Chat
            <div class="user-count" id="userCount">Connecting...</div>
        </div>

        <div class="messages" id="messages"></div>

        <div class="input-area">
            <input type="text" id="messageInput" placeholder="Type a message..." disabled>
            <button id="sendButton" disabled>Send</button>
        </div>

        <div class="login-overlay" id="loginOverlay">
            <div class="login-form">
                <h2>Join Chat</h2>
                <input type="text" id="usernameInput" placeholder="Enter your username..." maxlength="20">
                <button id="joinButton">Join</button>
            </div>
        </div>
    </div>

    <script>
        class ChatClient {
            constructor() {
                this.ws = null;
                this.username = null;
                this.messages = document.getElementById('messages');
                this.messageInput = document.getElementById('messageInput');
                this.sendButton = document.getElementById('sendButton');
                this.userCount = document.getElementById('userCount');
                this.loginOverlay = document.getElementById('loginOverlay');
                this.usernameInput = document.getElementById('usernameInput');
                this.joinButton = document.getElementById('joinButton');

                this.setupEvents();
            }

            setupEvents() {
                this.joinButton.addEventListener('click', () => this.joinChat());
                this.usernameInput.addEventListener('keypress', (e) => {
                    if (e.key === 'Enter') this.joinChat();
                });

                this.sendButton.addEventListener('click', () => this.sendMessage());
                this.messageInput.addEventListener('keypress', (e) => {
                    if (e.key === 'Enter') this.sendMessage();
                });
            }

            joinChat() {
                const username = this.usernameInput.value.trim();
                if (!username) {
                    alert('Please enter a username!');
                    return;
                }

                this.username = username;
                this.connect();
            }

            connect() {
                const wsUrl = `ws://${window.location.host}/ws`;
                this.ws = new WebSocket(wsUrl);

                this.ws.onopen = () => {
                    this.loginOverlay.style.display = 'none';
                    this.messageInput.disabled = false;
                    this.sendButton.disabled = false;
                    this.messageInput.focus();

                    this.send({ type: 'join', username: this.username });
                };

                this.ws.onclose = () => {
                    this.userCount.textContent = 'Disconnected';
                    this.messageInput.disabled = true;
                    this.sendButton.disabled = true;
                };

                this.ws.onmessage = (event) => {
                    const message = JSON.parse(event.data);
                    this.handleMessage(message);
                };

                this.ws.onerror = (error) => {
                    console.error('WebSocket error:', error);
                    this.addSystemMessage('Connection error');
                };
            }

            send(data) {
                if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                    this.ws.send(JSON.stringify(data));
                }
            }

            sendMessage() {
                const message = this.messageInput.value.trim();
                if (!message) return;

                this.send({ type: 'message', message });
                this.messageInput.value = '';
            }

            handleMessage(msg) {
                switch (msg.type) {
                    case 'user_joined':
                        if (msg.username !== this.username) {
                            this.addSystemMessage(`${msg.username} joined`);
                        }
                        this.userCount.textContent = `${msg.count} users online`;
                        break;

                    case 'user_left':
                        this.addSystemMessage(`${msg.username} left`);
                        this.userCount.textContent = `${msg.count} users online`;
                        break;

                    case 'message':
                        this.addMessage(msg.username, msg.message, msg.timestamp);
                        break;

                    case 'error':
                        this.addSystemMessage(`Error: ${msg.message}`);
                        break;
                }
            }

            addMessage(username, message, timestamp) {
                const div = document.createElement('div');
                div.className = `message ${username === this.username ? 'own' : 'other'}`;

                const time = new Date(timestamp).toLocaleTimeString();

                div.innerHTML = `
                    <div class="username">${username}</div>
                    <div>${this.escapeHtml(message)}</div>
                    <div class="timestamp">${time}</div>
                `;

                this.messages.appendChild(div);
                this.messages.scrollTop = this.messages.scrollHeight;
            }

            addSystemMessage(message) {
                const div = document.createElement('div');
                div.className = 'message system';
                div.textContent = message;

                this.messages.appendChild(div);
                this.messages.scrollTop = this.messages.scrollHeight;
            }

            escapeHtml(text) {
                const div = document.createElement('div');
                div.textContent = text;
                return div.innerHTML;
            }
        }

        new ChatClient();
    </script>
</body>
</html>"#;

    Ok(Response::html(html))
}

// WebSocket chat handler with optimized performance
#[cfg(feature = "websocket")]
async fn handle_chat_websocket(ws: WebSocketConnection, state: ChatState) -> Result<()> {
    let mut username: Option<String> = None;
    let mut broadcast_rx = state.broadcaster.subscribe();

    info!("🔌 New WebSocket connection established");

    // Handle broadcast messages in background with timeout
    let ws_clone = ws.clone();
    let mut broadcast_task = tokio::spawn(async move {
        let mut message_count = 0;
        while let Ok(server_msg) = broadcast_rx.recv().await {
            message_count += 1;

            // Batch messages for better performance
            if let Ok(json) = serde_json::to_string(&server_msg) {
                if ws_clone.send_text(json).await.is_err() {
                    break;
                }
            }

            // Add small delay to prevent overwhelming the connection
            if message_count % 10 == 0 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    });

    // Main message handling loop with timeout
    let mut message_count = 0;
    let timeout = std::time::Duration::from_secs(30);

    loop {
        tokio::select! {
            // Handle broadcast task completion
            _ = &mut broadcast_task => {
                warn!("Broadcast task ended");
                break;
            }

            // Handle incoming WebSocket messages with timeout
            msg_result = ws.recv_timeout(timeout) => {
                match msg_result {
                    Some(Message::Text(text)) => {
                        message_count += 1;

                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Join { username: new_username }) => {
                                info!("👤 User joining: {}", new_username);
                                username = Some(new_username.clone());
                                state.add_user(new_username).await;
                            }
                            Ok(ClientMessage::Message { message }) => {
                                if let Some(ref user) = username {
                                    let chat_msg = ChatMessage {
                                        username: user.clone(),
                                        message,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    info!("💬 Message from {}: {}", user, chat_msg.message);
                                    state.broadcast_message(chat_msg).await;
                                }
                            }
                            Err(e) => {
                                warn!("Invalid message format: {}", e);
                                let error_msg = ServerMessage::Error {
                                    message: "Invalid message format".to_string(),
                                };
                                if let Ok(json) = serde_json::to_string(&error_msg) {
                                    let _ = ws.send_text(json).await;
                                }
                            }
                        }
                    }
                    Some(Message::Close(_)) => {
                        info!("🔌 WebSocket connection closed");
                        break;
                    }
                    Some(Message::Ping(data)) => {
                        // Auto-respond to pings (handled by optimized connection)
                        let _ = ws.pong(data).await;
                    }
                    None => {
                        info!("🔌 WebSocket connection ended (timeout or no messages)");
                        break;
                    }
                    _ => {} // Handle other message types if needed
                }
            }
        }
    }

    // Cleanup on disconnect
    if let Some(user) = username {
        info!("👋 User disconnected: {}", user);
        state.remove_user(&user).await;
    }

    info!(
        "🔌 WebSocket handler completed (processed {} messages)",
        message_count
    );
    Ok(())
}

// Non-WebSocket fallback
#[cfg(not(feature = "websocket"))]
async fn websocket_not_supported() -> Result<Response> {
    Ok(Response::text(
        "WebSocket support is not enabled. Compile with --features websocket",
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let state = ChatState::new();
    let mut router = Router::new()
        .get("/", serve_chat_ui)
        .middleware(middleware::LoggerMiddleware)
        .middleware(middleware::CorsMiddleware::new());

    // Add WebSocket support if feature is enabled
    #[cfg(feature = "websocket")]
    {
        let state_clone = state.clone();
        router = router.websocket_fn("/ws", move |ws| {
            let state = state_clone.clone();
            async move { handle_chat_websocket(ws, state).await }
        });
        info!("✅ WebSocket support enabled");
    }

    #[cfg(not(feature = "websocket"))]
    {
        router = router.get("/ws", websocket_not_supported);
        info!("⚠️  WebSocket support disabled");
    }

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🔥 Ignitia Chat Server running on http://{}", addr);
    println!("💡 Open multiple browser tabs to test real-time chat!");

    server.ignitia().await.unwrap();
    Ok(())
}
