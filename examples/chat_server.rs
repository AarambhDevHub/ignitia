use ignitia::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

#[cfg(feature = "websocket")]
use ignitia::websocket::{Message, WebSocketConnection};

// Configuration for WebSocket timeouts and limits
#[derive(Debug, Clone)]
struct ChatConfig {
    pub ping_interval: Duration,      // How often to send keepalive pings
    pub pong_timeout: Duration,       // How long to wait for pong response
    pub idle_timeout: Duration,       // How long before disconnecting idle users
    pub max_message_size: usize,      // Maximum message length
    pub max_messages_per_minute: u32, // Rate limiting
    pub max_username_length: usize,   // Maximum username length
    pub broadcast_batch_size: usize,  // Batch size for broadcast messages
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(30), // Send ping every 30 seconds
            pong_timeout: Duration::from_secs(10),  // Wait 10 seconds for pong
            idle_timeout: Duration::from_secs(600), // 10-minute idle timeout
            max_message_size: 1000,                 // 1KB per message
            max_messages_per_minute: 60,            // 1 message per second max
            max_username_length: 30,                // 30 character username limit
            broadcast_batch_size: 10,               // Batch 10 messages at a time
        }
    }
}

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
    #[serde(rename = "ping")]
    Ping,
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
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "user_list")]
    UserList { users: Vec<String> },
}

// Rate limiting structure
#[derive(Debug)]
struct RateLimit {
    messages: Vec<Instant>,
    max_per_minute: u32,
}

impl RateLimit {
    fn new(max_per_minute: u32) -> Self {
        Self {
            messages: Vec::new(),
            max_per_minute,
        }
    }

    fn check_and_update(&mut self) -> bool {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        // Remove messages older than 1 minute
        self.messages.retain(|&time| time > one_minute_ago);

        if self.messages.len() >= self.max_per_minute as usize {
            false // Rate limit exceeded
        } else {
            self.messages.push(now);
            true
        }
    }
}

// Enhanced user tracking
#[derive(Debug)]
struct UserInfo {
    username: String,
    joined_at: Instant,
    last_activity: Instant,
    message_count: u32,
    rate_limit: RateLimit,
}

impl UserInfo {
    fn new(username: String, max_messages_per_minute: u32) -> Self {
        let now = Instant::now();
        Self {
            username,
            joined_at: now,
            last_activity: now,
            message_count: 0,
            rate_limit: RateLimit::new(max_messages_per_minute),
        }
    }

    fn update_activity(&mut self) {
        self.last_activity = Instant::now();
        self.message_count += 1;
    }
}

// Shared state for the chat server
#[derive(Clone)]
struct ChatState {
    #[cfg(feature = "websocket")]
    broadcaster: broadcast::Sender<ServerMessage>,
    users: Arc<RwLock<HashMap<String, UserInfo>>>,
    config: ChatConfig,
}

impl ChatState {
    fn new() -> Self {
        #[cfg(feature = "websocket")]
        let (broadcaster, _) = broadcast::channel(1000);

        Self {
            #[cfg(feature = "websocket")]
            broadcaster,
            users: Arc::new(RwLock::new(HashMap::new())),
            config: ChatConfig::default(),
        }
    }

    #[cfg(feature = "websocket")]
    async fn add_user(&self, username: String) -> Result<usize> {
        // Validate username
        if username.trim().is_empty() {
            return Err(Error::BadRequest("Username cannot be empty".to_string()));
        }

        if username.len() > self.config.max_username_length {
            return Err(Error::BadRequest(format!(
                "Username too long (max {} characters)",
                self.config.max_username_length
            )));
        }

        // Check for invalid characters
        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(Error::BadRequest(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
            ));
        }

        let mut users = self.users.write().await;

        // Check if username is already taken
        if users.contains_key(&username) {
            return Err(Error::BadRequest("Username already taken".to_string()));
        }

        let user_info = UserInfo::new(username.clone(), self.config.max_messages_per_minute);
        users.insert(username.clone(), user_info);
        let count = users.len();
        drop(users);

        let _ = self
            .broadcaster
            .send(ServerMessage::UserJoined { username, count });

        Ok(count)
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
    async fn broadcast_message(&self, message: ChatMessage) -> Result<()> {
        // Update user activity
        {
            let mut users = self.users.write().await;
            if let Some(user_info) = users.get_mut(&message.username) {
                user_info.update_activity();
            }
        }

        let _ = self.broadcaster.send(ServerMessage::Message(message));
        Ok(())
    }

    #[cfg(feature = "websocket")]
    async fn check_rate_limit(&self, username: &str) -> bool {
        let mut users = self.users.write().await;
        if let Some(user_info) = users.get_mut(username) {
            user_info.rate_limit.check_and_update()
        } else {
            false
        }
    }

    async fn get_user_count(&self) -> usize {
        self.users.read().await.len()
    }

    #[cfg(feature = "websocket")]
    async fn get_user_list(&self) -> Vec<String> {
        let users = self.users.read().await;
        users.keys().cloned().collect()
    }
}

// Input validation and sanitization
fn sanitize_message(message: &str, max_length: usize) -> Result<String> {
    let trimmed = message.trim();

    if trimmed.is_empty() {
        return Err(Error::BadRequest("Message cannot be empty".to_string()));
    }

    if trimmed.len() > max_length {
        return Err(Error::BadRequest(format!(
            "Message too long (max {} characters)",
            max_length
        )));
    }

    // Basic HTML escape for security
    let sanitized = trimmed
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");

    Ok(sanitized)
}

// Serve the chat UI with enhanced features
async fn serve_chat_ui() -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🔥 Ignitia Chat</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            height: 100vh; display: flex; align-items: center; justify-content: center;
        }
        .chat-container {
            background: white; border-radius: 15px; box-shadow: 0 20px 40px rgba(0,0,0,0.1);
            width: 90%; max-width: 800px; height: 90vh; display: flex; flex-direction: column;
            overflow: hidden;
        }
        .chat-header {
            background: linear-gradient(45deg, #ff6b6b, #feca57);
            color: white; padding: 20px; text-align: center; font-size: 24px; font-weight: bold;
        }
        .user-count { font-size: 14px; opacity: 0.9; margin-top: 5px; }
        .chat-messages {
            flex: 1; padding: 20px; overflow-y: auto; background: #f8f9fa;
        }
        .message {
            margin-bottom: 15px; padding: 12px 15px; border-radius: 18px; max-width: 70%;
            word-wrap: break-word;
        }
        .message.own {
            background: linear-gradient(45deg, #667eea, #764ba2); color: white;
            margin-left: auto; text-align: right;
        }
        .message.other { background: white; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }
        .message-info { font-size: 12px; opacity: 0.7; margin-bottom: 5px; }
        .message-text { font-size: 14px; line-height: 1.4; }
        .system-message {
            text-align: center; color: #666; font-style: italic; margin: 10px 0;
            background: #e9ecef; padding: 8px 15px; border-radius: 15px; font-size: 13px;
        }
        .chat-input-container {
            padding: 20px; background: white; border-top: 1px solid #eee;
            display: flex; gap: 10px;
        }
        .chat-input {
            flex: 1; padding: 12px 15px; border: 2px solid #eee; border-radius: 25px;
            font-size: 14px; outline: none;
        }
        .chat-input:focus { border-color: #667eea; }
        .send-button {
            padding: 12px 25px; background: linear-gradient(45deg, #667eea, #764ba2);
            color: white; border: none; border-radius: 25px; cursor: pointer;
            font-weight: bold; transition: transform 0.2s;
        }
        .send-button:hover { transform: scale(1.05); }
        .send-button:disabled { opacity: 0.5; cursor: not-allowed; transform: none; }
        .status {
            text-align: center; padding: 10px; font-size: 13px; color: #666;
            background: #fff3cd; border-bottom: 1px solid #ffeaa7;
        }
        .status.connected { background: #d4edda; color: #155724; }
        .status.error { background: #f8d7da; color: #721c24; }
        .login-container {
            position: absolute; top: 0; left: 0; right: 0; bottom: 0;
            background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center;
        }
        .login-form {
            background: white; padding: 40px; border-radius: 15px; text-align: center;
            min-width: 300px; box-shadow: 0 20px 40px rgba(0,0,0,0.2);
        }
        .login-input {
            width: 100%; padding: 15px; margin: 10px 0; border: 2px solid #eee;
            border-radius: 10px; font-size: 16px; outline: none;
        }
        .login-input:focus { border-color: #667eea; }
        .login-button {
            width: 100%; padding: 15px; background: linear-gradient(45deg, #667eea, #764ba2);
            color: white; border: none; border-radius: 10px; font-size: 16px;
            font-weight: bold; cursor: pointer; margin-top: 10px;
        }
        .error-message { color: #e74c3c; margin-top: 10px; font-size: 14px; }
    </style>
</head>
<body>
    <div class="chat-container">
        <div class="chat-header">
            🔥 Ignitia Chat
            <div class="user-count" id="userCount">Connecting...</div>
        </div>
        <div class="status" id="status">Connecting to chat server...</div>
        <div class="chat-messages" id="messages"></div>
        <div class="chat-input-container">
            <input type="text" class="chat-input" id="messageInput" placeholder="Type your message..." maxlength="1000" disabled>
            <button class="send-button" id="sendButton" disabled>Send</button>
        </div>
    </div>

    <div class="login-container" id="loginContainer">
        <div class="login-form">
            <h2>Join Chat</h2>
            <input type="text" class="login-input" id="usernameInput" placeholder="Enter your username" maxlength="30">
            <button class="login-button" id="joinButton">Join</button>
            <div class="error-message" id="loginError"></div>
        </div>
    </div>

    <script>
        class ChatClient {
            constructor() {
                this.ws = null;
                this.username = null;
                this.connected = false;
                this.messageCount = 0;
                this.lastActivity = Date.now();

                this.initElements();
                this.bindEvents();
                this.connect();
            }

            initElements() {
                this.elements = {
                    loginContainer: document.getElementById('loginContainer'),
                    usernameInput: document.getElementById('usernameInput'),
                    joinButton: document.getElementById('joinButton'),
                    loginError: document.getElementById('loginError'),
                    status: document.getElementById('status'),
                    userCount: document.getElementById('userCount'),
                    messages: document.getElementById('messages'),
                    messageInput: document.getElementById('messageInput'),
                    sendButton: document.getElementById('sendButton')
                };
            }

            bindEvents() {
                this.elements.joinButton.addEventListener('click', () => this.joinChat());
                this.elements.usernameInput.addEventListener('keypress', (e) => {
                    if (e.key === 'Enter') this.joinChat();
                });
                this.elements.sendButton.addEventListener('click', () => this.sendMessage());
                this.elements.messageInput.addEventListener('keypress', (e) => {
                    if (e.key === 'Enter') this.sendMessage();
                });
                this.elements.messageInput.addEventListener('input', () => {
                    this.lastActivity = Date.now();
                });
            }

            connect() {
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                const wsUrl = `${protocol}//${window.location.host}/ws`;

                this.ws = new WebSocket(wsUrl);

                this.ws.onopen = () => this.handleConnection();
                this.ws.onmessage = (event) => this.handleMessage(event);
                this.ws.onclose = () => this.handleDisconnection();
                this.ws.onerror = (error) => this.handleError(error);
            }

            handleConnection() {
                this.connected = true;
                this.updateStatus('Connected to server', 'connected');
            }

            handleMessage(event) {
                try {
                    const message = JSON.parse(event.data);
                    this.processMessage(message);
                } catch (error) {
                    console.error('Failed to parse message:', error);
                }
            }

            processMessage(message) {
                switch (message.type) {
                    case 'user_joined':
                        this.updateUserCount(message.count);
                        this.addSystemMessage(`${message.username} joined the chat`);
                        break;
                    case 'user_left':
                        this.updateUserCount(message.count);
                        this.addSystemMessage(`${message.username} left the chat`);
                        break;
                    case 'message':
                        this.addChatMessage(message);
                        break;
                    case 'error':
                        this.showError(message.message);
                        break;
                    case 'pong':
                        // Handle pong response
                        break;
                }
            }

            handleDisconnection() {
                this.connected = false;
                this.updateStatus('Disconnected from server. Reconnecting...', 'error');

                // Disable inputs
                this.elements.messageInput.disabled = true;
                this.elements.sendButton.disabled = true;

                // Attempt to reconnect after 3 seconds
                setTimeout(() => this.connect(), 3000);
            }

            handleError(error) {
                console.error('WebSocket error:', error);
                this.updateStatus('Connection error', 'error');
            }

            joinChat() {
                const username = this.elements.usernameInput.value.trim();

                if (!username) {
                    this.showLoginError('Please enter a username');
                    return;
                }

                if (username.length > 30) {
                    this.showLoginError('Username too long (max 30 characters)');
                    return;
                }

                if (!/^[a-zA-Z0-9_-]+$/.test(username)) {
                    this.showLoginError('Username can only contain letters, numbers, underscores, and hyphens');
                    return;
                }

                if (this.connected && this.ws.readyState === WebSocket.OPEN) {
                    this.ws.send(JSON.stringify({
                        type: 'join',
                        username: username
                    }));

                    this.username = username;
                    this.elements.loginContainer.style.display = 'none';
                    this.elements.messageInput.disabled = false;
                    this.elements.sendButton.disabled = false;
                    this.elements.messageInput.focus();
                    this.clearLoginError();
                } else {
                    this.showLoginError('Not connected to server');
                }
            }

            sendMessage() {
                const message = this.elements.messageInput.value.trim();

                if (!message || !this.connected || this.ws.readyState !== WebSocket.OPEN) {
                    return;
                }

                if (message.length > 1000) {
                    this.showError('Message too long (max 1000 characters)');
                    return;
                }

                this.ws.send(JSON.stringify({
                    type: 'message',
                    message: message
                }));

                this.elements.messageInput.value = '';
                this.messageCount++;
                this.lastActivity = Date.now();
            }

            addChatMessage(chatMessage) {
                const messageElement = document.createElement('div');
                messageElement.className = `message ${chatMessage.username === this.username ? 'own' : 'other'}`;

                const time = new Date(chatMessage.timestamp).toLocaleTimeString();
                messageElement.innerHTML = `
                    <div class="message-info">${chatMessage.username} • ${time}</div>
                    <div class="message-text">${chatMessage.message}</div>
                `;

                this.elements.messages.appendChild(messageElement);
                this.scrollToBottom();
            }

            addSystemMessage(text) {
                const messageElement = document.createElement('div');
                messageElement.className = 'system-message';
                messageElement.textContent = text;

                this.elements.messages.appendChild(messageElement);
                this.scrollToBottom();
            }

            scrollToBottom() {
                this.elements.messages.scrollTop = this.elements.messages.scrollHeight;
            }

            updateStatus(text, className = '') {
                this.elements.status.textContent = text;
                this.elements.status.className = `status ${className}`;
            }

            updateUserCount(count) {
                this.elements.userCount.textContent = `${count} user${count !== 1 ? 's' : ''} online`;
            }

            showError(message) {
                this.updateStatus(`Error: ${message}`, 'error');
                setTimeout(() => {
                    if (this.connected) {
                        this.updateStatus('Connected', 'connected');
                    }
                }, 5000);
            }

            showLoginError(message) {
                this.elements.loginError.textContent = message;
            }

            clearLoginError() {
                this.elements.loginError.textContent = '';
            }
        }

        // Initialize chat when page loads
        document.addEventListener('DOMContentLoaded', () => {
            new ChatClient();
        });
    </script>
</body>
</html>
"#;
    Ok(Response::html(html))
}

// Enhanced WebSocket handler with proper timeout and heartbeat management
#[cfg(feature = "websocket")]
async fn handle_chat_websocket(ws: WebSocketConnection, state: ChatState) -> Result<()> {
    let mut username: Option<String> = None;
    let mut broadcast_rx = state.broadcaster.subscribe();
    let mut last_activity = Instant::now();
    let mut last_ping = Instant::now();
    let mut pending_pong = false;
    let mut pong_deadline: Option<Instant> = None;
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;

    info!("🔌 New WebSocket connection established");

    // Handle broadcast messages in background
    let ws_clone = ws.clone();
    let config = state.config.clone();
    let mut broadcast_task = tokio::spawn(async move {
        // let mut message_batch = Vec::new();
        let mut batch_count = 0;

        while let Ok(server_msg) = broadcast_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&server_msg) {
                match ws_clone.send_text(json).await {
                    Ok(_) => {
                        retry_count = 0; // Reset retry count on success
                        batch_count += 1;

                        // Add small delay every batch to prevent overwhelming
                        if batch_count % config.broadcast_batch_size == 0 {
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to send broadcast message: {}", e);
                        retry_count += 1;
                        if retry_count > MAX_RETRIES {
                            debug!("Max retries exceeded, closing broadcast task");
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    });

    // Main message handling loop with comprehensive timeout management
    let mut ping_interval = tokio::time::interval(state.config.ping_interval);
    let mut message_count = 0;

    loop {
        tokio::select! {
            // Handle broadcast task completion
            _ = &mut broadcast_task => {
                warn!("Broadcast task ended");
                break;
            }

            // Send periodic pings for keepalive
            _ = ping_interval.tick() => {
                if pending_pong {
                    if let Some(deadline) = pong_deadline {
                        if Instant::now() > deadline {
                            warn!("Pong timeout - closing connection");
                            break;
                        }
                    }
                } else {
                    if let Err(e) = ws.ping(b"keepalive".as_ref()).await {
                        warn!("Failed to send ping: {}", e);
                        break;
                    }
                    pending_pong = true;
                    pong_deadline = Some(Instant::now() + state.config.pong_timeout);
                    last_ping = Instant::now();
                    debug!("Sent keepalive ping");
                }
            }

            // Handle incoming WebSocket messages with timeout
            msg_result = ws.recv_timeout(Duration::from_secs(60)) => {
                match msg_result {
                    Some(Message::Text(text)) => {
                        last_activity = Instant::now();
                        message_count += 1;

                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Join { username: new_username }) => {
                                info!("👤 User attempting to join: {}", new_username);

                                match state.add_user(new_username.clone()).await {
                                    Ok(_count) => {
                                        username = Some(new_username.clone());
                                        info!("✅ User joined successfully: {}", new_username);

                                        // Send user list
                                        let user_list = state.get_user_list().await;
                                        let user_list_msg = ServerMessage::UserList { users: user_list };
                                        if let Ok(json) = serde_json::to_string(&user_list_msg) {
                                            let _ = ws.send_text(json).await;
                                        }
                                    }
                                    Err(error_msg) => {
                                        warn!("❌ User join failed: {}", error_msg);
                                        let error_response = ServerMessage::Error {
                                            message: error_msg.to_string(),
                                        };
                                        if let Ok(json) = serde_json::to_string(&error_response) {
                                            let _ = ws.send_text(json).await;
                                        }
                                    }
                                }
                            }

                            Ok(ClientMessage::Message { message }) => {
                                if let Some(ref user) = username {
                                    // Check rate limiting
                                    if !state.check_rate_limit(user).await {
                                        let error_msg = ServerMessage::Error {
                                            message: "Rate limit exceeded. Please slow down.".to_string(),
                                        };
                                        if let Ok(json) = serde_json::to_string(&error_msg) {
                                            let _ = ws.send_text(json).await;
                                        }
                                        continue;
                                    }

                                    // Sanitize and validate message
                                    match sanitize_message(&message, state.config.max_message_size) {
                                        Ok(sanitized_message) => {
                                            let chat_msg = ChatMessage {
                                                username: user.clone(),
                                                message: sanitized_message,
                                                timestamp: chrono::Utc::now().to_rfc3339(),
                                            };

                                            info!("💬 Message from {}: {}", user, chat_msg.message);

                                            if let Err(e) = state.broadcast_message(chat_msg).await {
                                                warn!("Failed to broadcast message: {}", e);
                                            }
                                        }
                                        Err(validation_error) => {
                                            let error_msg = ServerMessage::Error {
                                                message: validation_error.to_string(),
                                            };
                                            if let Ok(json) = serde_json::to_string(&error_msg) {
                                                let _ = ws.send_text(json).await;
                                            }
                                        }
                                    }
                                } else {
                                    let error_msg = ServerMessage::Error {
                                        message: "You must join the chat first".to_string(),
                                    };
                                    if let Ok(json) = serde_json::to_string(&error_msg) {
                                        let _ = ws.send_text(json).await;
                                    }
                                }
                            }

                            Ok(ClientMessage::Ping) => {
                                // Respond to client ping with pong
                                let pong_msg = ServerMessage::Pong;
                                if let Ok(json) = serde_json::to_string(&pong_msg) {
                                    let _ = ws.send_text(json).await;
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
                        info!("🔌 WebSocket connection closed by client");
                        break;
                    }

                    Some(Message::Pong(_)) => {
                        pending_pong = false;
                        pong_deadline = None;
                        debug!("Received pong response");
                    }

                    Some(Message::Ping(data)) => {
                        // Respond to ping with pong (handled automatically by framework)
                        let _ = ws.pong(data).await;
                        last_activity = Instant::now();
                    }

                    None => {
                        // Check if connection has been idle too long
                        if last_activity.elapsed() > state.config.idle_timeout {
                            info!("🔌 WebSocket connection ended due to inactivity");
                            break;
                        }
                        // Continue loop for timeout case
                    }

                    _ => {
                        // Handle other message types if needed
                        last_activity = Instant::now();
                    }
                }
            }
        }
    }

    // Cleanup on disconnect
    if let Some(user) = username {
        info!("👋 User disconnected: {}", user);
        state.remove_user(&user).await;
    }

    broadcast_task.abort();
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
        info!("✅ WebSocket support enabled with enhanced timeout management");
    }

    #[cfg(not(feature = "websocket"))]
    {
        router = router.get("/ws", websocket_not_supported);
        info!("⚠️ WebSocket support disabled");
    }

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🔥 Ignitia Chat Server running on http://{}", addr);
    println!("💡 Open multiple browser tabs to test real-time chat!");
    println!("⚙️  Configuration:");
    println!("   • Ping interval: 30s");
    println!("   • Pong timeout: 10s");
    println!("   • Idle timeout: 10 minutes");
    println!("   • Max message size: 1000 chars");
    println!("   • Rate limit: 60 messages/minute");

    server.ignitia().await.unwrap();
    Ok(())
}
