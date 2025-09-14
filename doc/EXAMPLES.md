# Examples Guide

Welcome to the Ignitia Examples Guide! This comprehensive collection demonstrates the power and flexibility of the Ignitia web framework through practical, real-world examples.

## Table of Contents

1. [Quick Start Examples](#quick-start-examples)
2. [HTTP Methods & Routing](#http-methods--routing)
3. [Request Extractors](#request-extractors)
4. [Middleware Examples](#middleware-examples)
5. [WebSocket Applications](#websocket-applications)
6. [File Upload Handling](#file-upload-handling)
7. [Error Handling Patterns](#error-handling-patterns)
8. [State Management](#state-management)
9. [Authentication & Authorization](#authentication--authorization)
10. [Real-World Applications](#real-world-applications)
11. [Performance Optimization](#performance-optimization)
12. [Testing Examples](#testing-examples)

***

## Quick Start Examples

### Hello World Server

The simplest possible Ignitia application:

```rust
use ignitia::{Router, Server, Response, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Hello from Ignitia! 🔥")) });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Server blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}
```

### JSON API Server

A simple JSON API with multiple endpoints:

```rust
use ignitia::{Router, Server, Response, Result, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    status: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/api/health", health_check)
        .post("/api/users", create_user)
        .get("/api/users/:id", get_user);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await.unwrap();
    Ok(())
}

async fn health_check() -> Result<Response> {
    Response::json(&ApiResponse {
        message: "Service is healthy".to_string(),
        status: "ok".to_string(),
    })
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    println!("Creating user: {} ({})", user.name, user.email);
    Response::json(&ApiResponse {
        message: format!("User {} created successfully", user.name),
        status: "created".to_string(),
    })
}

async fn get_user(ignitia::Path(id): ignitia::Path<u32>) -> Result<Response> {
    Response::json(&serde_json::json!({
        "id": id,
        "name": "John Doe",
        "email": "john@example.com"
    }))
}
```

***

## HTTP Methods & Routing

### Complete CRUD API

```rust
use ignitia::{Router, Server, Response, Result, Json, Path};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    completed: bool,
}

type TaskStore = Arc<Mutex<HashMap<u32, Task>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let store: TaskStore = Arc::new(Mutex::new(HashMap::new()));

    let router = Router::new()
        .state(store)
        // CRUD operations
        .get("/tasks", list_tasks)
        .post("/tasks", create_task)
        .get("/tasks/:id", get_task)
        .put("/tasks/:id", update_task)
        .delete("/tasks/:id", delete_task)
        // Additional routes
        .patch("/tasks/:id/complete", complete_task)
        .get("/tasks/completed", list_completed_tasks);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Task API blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}

async fn list_tasks(ignitia::State(store): ignitia::State<TaskStore>) -> Result<Response> {
    let tasks = store.lock().unwrap();
    let task_list: Vec<&Task> = tasks.values().collect();
    Response::json(&task_list)
}

async fn create_task(
    Json(mut task): Json<Task>,
    ignitia::State(store): ignitia::State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().unwrap();
    let id = tasks.len() as u32 + 1;
    task.id = id;
    tasks.insert(id, task.clone());
    Response::json(&task)
}

async fn get_task(
    Path(id): Path<u32>,
    ignitia::State(store): ignitia::State<TaskStore>
) -> Result<Response> {
    let tasks = store.lock().unwrap();
    match tasks.get(&id) {
        Some(task) => Response::json(task),
        None => Err(ignitia::Error::NotFound(format!("Task {} not found", id)))
    }
}

async fn update_task(
    Path(id): Path<u32>,
    Json(updated_task): Json<Task>,
    ignitia::State(store): ignitia::State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().unwrap();
    match tasks.get_mut(&id) {
        Some(task) => {
            task.title = updated_task.title;
            task.completed = updated_task.completed;
            Response::json(task)
        }
        None => Err(ignitia::Error::NotFound(format!("Task {} not found", id)))
    }
}

async fn delete_task(
    Path(id): Path<u32>,
    ignitia::State(store): ignitia::State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().unwrap();
    match tasks.remove(&id) {
        Some(_) => Ok(Response::new(http::StatusCode::NO_CONTENT)),
        None => Err(ignitia::Error::NotFound(format!("Task {} not found", id)))
    }
}

async fn complete_task(
    Path(id): Path<u32>,
    ignitia::State(store): ignitia::State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().unwrap();
    match tasks.get_mut(&id) {
        Some(task) => {
            task.completed = true;
            Response::json(task)
        }
        None => Err(ignitia::Error::NotFound(format!("Task {} not found", id)))
    }
}

async fn list_completed_tasks(ignitia::State(store): ignitia::State<TaskStore>) -> Result<Response> {
    let tasks = store.lock().unwrap();
    let completed: Vec<&Task> = tasks.values().filter(|t| t.completed).collect();
    Response::json(&completed)
}
```

### Dynamic Route Patterns

```rust
use ignitia::{Router, Server, Response, Result, Path, Query};
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    size: Option<u32>,
}

#[derive(Deserialize)]
struct UserParams {
    id: u32,
    action: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        // Simple path parameters
        .get("/users/:id", get_user_by_id)
        .get("/posts/:slug", get_post_by_slug)

        // Multiple path parameters
        .get("/users/:id/:action", user_action)
        .get("/categories/:category/posts/:id", get_category_post)

        // Wildcard routes
        .get("/static/*path", serve_static)
        .get("/docs/*path", serve_docs)

        // Query parameters
        .get("/search", search_with_query)
        .get("/api/posts", list_posts_paginated);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await.unwrap();
    Ok(())
}

async fn get_user_by_id(Path(id): Path<u32>) -> Result<Response> {
    Response::json(&serde_json::json!({
        "user_id": id,
        "message": format!("Retrieved user {}", id)
    }))
}

async fn get_post_by_slug(Path(slug): Path<String>) -> Result<Response> {
    Response::json(&serde_json::json!({
        "slug": slug,
        "title": format!("Post: {}", slug.replace("-", " "))
    }))
}

async fn user_action(Path(params): Path<UserParams>) -> Result<Response> {
    Response::json(&serde_json::json!({
        "user_id": params.id,
        "action": params.action,
        "message": format!("Executing {} for user {}", params.action, params.id)
    }))
}

async fn get_category_post(Path((category, id)): Path<(String, u32)>) -> Result<Response> {
    Response::json(&serde_json::json!({
        "category": category,
        "post_id": id,
        "url": format!("/categories/{}/posts/{}", category, id)
    }))
}

async fn serve_static(Path(path): Path<String>) -> Result<Response> {
    // In a real application, you'd serve actual files
    Response::text(&format!("Serving static file: {}", path))
}

async fn serve_docs(Path(path): Path<String>) -> Result<Response> {
    Response::html(&format!("<h1>Documentation: {}</h1>", path))
}

async fn search_with_query(Query(params): Query<std::collections::HashMap<String, String>>) -> Result<Response> {
    let query = params.get("q").unwrap_or(&"".to_string());
    let category = params.get("category").unwrap_or(&"all".to_string());

    Response::json(&serde_json::json!({
        "query": query,
        "category": category,
        "results": []
    }))
}

async fn list_posts_paginated(Query(pagination): Query<Pagination>) -> Result<Response> {
    let page = pagination.page.unwrap_or(1);
    let size = pagination.size.unwrap_or(10);

    Response::json(&serde_json::json!({
        "page": page,
        "size": size,
        "total": 100,
        "posts": []
    }))
}
```

***

## Request Extractors

### Form Data Handling

```rust
use ignitia::{Router, Server, Response, Result, Form, Json, Path, Query, Headers, Body, Cookies};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct LoginForm {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct ContactForm {
    name: String,
    email: String,
    subject: String,
    message: String,
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    q: String,
    category: Option<String>,
    sort: Option<String>,
    page: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .post("/login", handle_login)
        .post("/contact", handle_contact)
        .get("/search", handle_search)
        .post("/api/data", handle_json_data)
        .get("/debug", debug_request)
        .post("/raw", handle_raw_body);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await.unwrap();
    Ok(())
}

// Form data extractor
async fn handle_login(Form(login): Form<LoginForm>) -> Result<Response> {
    println!("Login attempt: {:?}", login);

    // Simulate authentication
    if login.username == "admin" && login.password == "secret" {
        let mut response = Response::json(&serde_json::json!({
            "status": "success",
            "message": "Login successful"
        }))?;

        // Set session cookie
        response = response.add_cookie(
            ignitia::Cookie::new("session_id", "abc123")
                .http_only()
                .secure()
                .path("/")
        );

        Ok(response)
    } else {
        Err(ignitia::Error::Unauthorized)
    }
}

async fn handle_contact(Form(contact): Form<ContactForm>) -> Result<Response> {
    println!("Contact form received: {:?}", contact);

    // Simulate sending email
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Response::json(&serde_json::json!({
        "status": "success",
        "message": "Message sent successfully",
        "id": uuid::Uuid::new_v4()
    }))
}

// Query parameters extractor
async fn handle_search(Query(search): Query<SearchQuery>) -> Result<Response> {
    println!("Search query: {:?}", search);

    // Simulate database search
    let results = vec![
        format!("Result 1 for '{}'", search.q),
        format!("Result 2 for '{}'", search.q),
    ];

    Response::json(&serde_json::json!({
        "query": search.q,
        "category": search.category,
        "sort": search.sort,
        "page": search.page.unwrap_or(1),
        "results": results
    }))
}

// JSON data extractor
async fn handle_json_data(Json(data): Json<serde_json::Value>) -> Result<Response> {
    println!("Received JSON: {}", data);

    Response::json(&serde_json::json!({
        "received": data,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Multiple extractors in one handler
async fn debug_request(
    headers: Headers,
    cookies: Cookies,
    Path(segments): Path<Vec<String>>
) -> Result<Response> {
    let debug_info = serde_json::json!({
        "headers": headers.iter().collect::<std::collections::HashMap<_, _>>(),
        "cookies": cookies.all(),
        "path_segments": segments,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Response::json(&debug_info)
}

// Raw body extractor
async fn handle_raw_body(body: Body) -> Result<Response> {
    let body_size = body.len();
    let body_preview = if body_size > 100 {
        format!("{}... ({} bytes total)",
               String::from_utf8_lossy(&body[..100]),
               body_size)
    } else {
        String::from_utf8_lossy(&body).to_string()
    };

    Response::json(&serde_json::json!({
        "body_size": body_size,
        "body_preview": body_preview,
        "content_type": "raw"
    }))
}
```

***

## Middleware Examples

### Complete Middleware Stack

```rust
use ignitia::{
    Router, Server, Response, Result,
    middleware::{LoggerMiddleware, CorsMiddleware, SecurityMiddleware, RateLimitingMiddleware},
    Cors
};

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        // Add middleware in order
        .middleware(LoggerMiddleware)
        .middleware(
            Cors::new()
                .allowed_origins(&["http://localhost:3000", "https://myapp.com"])
                .allowed_methods(&[
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::PUT,
                    http::Method::DELETE,
                ])
                .allowed_headers(&["Content-Type", "Authorization"])
                .allow_credentials()
                .build()?
        )
        .middleware(SecurityMiddleware::for_web())
        .middleware(RateLimitingMiddleware::per_minute(100))

        // Routes
        .get("/", home_page)
        .get("/api/data", get_data)
        .post("/api/upload", upload_data)
        .get("/admin", admin_panel);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await.unwrap();
    Ok(())
}

async fn home_page() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Ignitia App</title>
        </head>
        <body>
            <h1>Welcome to Ignitia! 🔥</h1>
            <script>
                fetch('/api/data')
                    .then(r => r.json())
                    .then(data => console.log(data));
            </script>
        </body>
        </html>
    "#))
}

async fn get_data() -> Result<Response> {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Response::json(&serde_json::json!({
        "message": "Data retrieved successfully",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn upload_data(ignitia::Json(data): ignitia::Json<serde_json::Value>) -> Result<Response> {
    println!("Uploading data: {}", data);
    Response::json(&serde_json::json!({
        "status": "uploaded",
        "size": data.to_string().len()
    }))
}

async fn admin_panel() -> Result<Response> {
    Ok(Response::html("<h1>Admin Panel - Protected Route</h1>"))
}
```

### Custom Authentication Middleware

```rust
use ignitia::{Router, Server, Response, Result, AuthMiddleware};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        // Public routes
        .get("/", public_home)
        .post("/login", login)
        .get("/register", register_form)

        // Protected routes with auth middleware
        .middleware(
            AuthMiddleware::new("secret-jwt-token")
                .protect_paths(vec!["/api", "/admin", "/profile"])
        )
        .get("/profile", user_profile)
        .get("/admin", admin_dashboard)
        .get("/api/users", list_users)
        .post("/api/posts", create_post);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await.unwrap();
    Ok(())
}

async fn public_home() -> Result<Response> {
    Ok(Response::html(r#"
        <h1>Public Home</h1>
        <a href="/login">Login</a> | <a href="/register">Register</a>
    "#))
}

async fn login() -> Result<Response> {
    // Simulate login logic
    Response::json(&serde_json::json!({
        "token": "secret-jwt-token",
        "expires_in": 3600
    }))
}

async fn register_form() -> Result<Response> {
    Ok(Response::html(r#"
        <form method="POST" action="/register">
            <input name="username" placeholder="Username" required>
            <input name="password" type="password" placeholder="Password" required>
            <button>Register</button>
        </form>
    "#))
}

async fn user_profile() -> Result<Response> {
    Response::json(&serde_json::json!({
        "user": {
            "id": 1,
            "username": "john_doe",
            "email": "john@example.com"
        }
    }))
}

async fn admin_dashboard() -> Result<Response> {
    Ok(Response::html("<h1>Admin Dashboard - Authenticated Access</h1>"))
}

async fn list_users() -> Result<Response> {
    Response::json(&vec![
        serde_json::json!({"id": 1, "name": "John"}),
        serde_json::json!({"id": 2, "name": "Jane"}),
    ])
}

async fn create_post(ignitia::Json(post): ignitia::Json<serde_json::Value>) -> Result<Response> {
    Response::json(&serde_json::json!({
        "id": 123,
        "title": post["title"],
        "status": "created"
    }))
}
```

***

## WebSocket Applications

### Real-time Chat Application

```rust
use ignitia::{Router, Server, Response, Result, websocket::{WebSocketConnection, Message}};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use std::collections::HashMap;

type ClientId = String;
type ChatRoom = Arc<RwLock<HashMap<ClientId, broadcast::Sender<String>>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let chat_room: ChatRoom = Arc::new(RwLock::new(HashMap::new()));

    let router = Router::new()
        .state(chat_room)
        .get("/", chat_page)
        .websocket("/ws", handle_websocket);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Chat server blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}

async fn chat_page() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Ignitia Chat 🔥</title>
            <style>
                #messages { height: 400px; overflow-y: scroll; border: 1px solid #ccc; padding: 10px; }
                #messageInput { width: 300px; }
            </style>
        </head>
        <body>
            <h1>Ignitia Chat Room 🔥</h1>
            <div id="messages"></div>
            <input id="messageInput" placeholder="Type a message..." />
            <button onclick="sendMessage()">Send</button>

            <script>
                const ws = new WebSocket('ws://localhost:3000/ws');
                const messages = document.getElementById('messages');
                const messageInput = document.getElementById('messageInput');

                ws.onmessage = function(event) {
                    const div = document.createElement('div');
                    div.textContent = event.data;
                    messages.appendChild(div);
                    messages.scrollTop = messages.scrollHeight;
                };

                function sendMessage() {
                    const message = messageInput.value;
                    if (message) {
                        ws.send(JSON.stringify({
                            type: 'chat_message',
                            content: message,
                            timestamp: new Date().toISOString()
                        }));
                        messageInput.value = '';
                    }
                }

                messageInput.addEventListener('keypress', function(e) {
                    if (e.key === 'Enter') {
                        sendMessage();
                    }
                });

                ws.onopen = function() {
                    console.log('Connected to chat server');
                };

                ws.onclose = function() {
                    console.log('Disconnected from chat server');
                };
            </script>
        </body>
        </html>
    "#))
}

async fn handle_websocket(
    websocket: WebSocketConnection,
    ignitia::State(chat_room): ignitia::State<ChatRoom>
) -> Result<()> {
    let client_id = uuid::Uuid::new_v4().to_string();
    let (tx, mut rx) = broadcast::channel::<String>(100);

    // Add client to chat room
    {
        let mut room = chat_room.write().await;
        room.insert(client_id.clone(), tx);
    }

    println!("Client {} connected", client_id);

    // Send welcome message
    websocket.send_json(&serde_json::json!({
        "type": "system",
        "message": "Welcome to Ignitia Chat! 🔥"
    })).await?;

    // Handle incoming messages
    let websocket_clone = websocket.clone();
    let chat_room_clone = Arc::clone(&chat_room);
    let client_id_clone = client_id.clone();

    tokio::spawn(async move {
        while let Some(message) = websocket_clone.recv().await {
            match message {
                Message::Text(text) => {
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        if msg["type"] == "chat_message" {
                            let chat_msg = format!(
                                "[{}] {}: {}",
                                msg["timestamp"].as_str().unwrap_or(""),
                                client_id_clone,
                                msg["content"].as_str().unwrap_or("")
                            );

                            // Broadcast to all clients
                            let room = chat_room_clone.read().await;
                            for (id, sender) in room.iter() {
                                if *id != client_id_clone {  // Don't send back to sender
                                    let _ = sender.send(chat_msg.clone());
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    println!("Client {} disconnected", client_id_clone);
                    break;
                }
                _ => {}
            }
        }

        // Remove client from chat room
        let mut room = chat_room_clone.write().await;
        room.remove(&client_id_clone);
    });

    // Handle broadcast messages
    tokio::spawn(async move {
        while let Ok(broadcast_msg) = rx.recv().await {
            if websocket.send_text(broadcast_msg).await.is_err() {
                break;
            }
        }
    });

    Ok(())
}
```

### Live Data Dashboard

```rust
use ignitia::{Router, Server, Response, Result, websocket::{WebSocketConnection, Message}};
use tokio::time::{interval, Duration};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", dashboard_page)
        .websocket("/ws/data", handle_data_stream);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Dashboard blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}

async fn dashboard_page() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Live Dashboard 🔥</title>
            <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
            <style>
                .metric { display: inline-block; margin: 20px; padding: 20px; border: 1px solid #ccc; }
                .value { font-size: 2em; font-weight: bold; color: #ff6b35; }
            </style>
        </head>
        <body>
            <h1>Live System Dashboard 🔥</h1>

            <div class="metric">
                <div>CPU Usage</div>
                <div class="value" id="cpu">--</div>
            </div>

            <div class="metric">
                <div>Memory Usage</div>
                <div class="value" id="memory">--</div>
            </div>

            <div class="metric">
                <div>Active Users</div>
                <div class="value" id="users">--</div>
            </div>

            <canvas id="chart" width="800" height="400"></canvas>

            <script>
                const ws = new WebSocket('ws://localhost:3000/ws/data');
                const ctx = document.getElementById('chart').getContext('2d');
                const chart = new Chart(ctx, {
                    type: 'line',
                    data: {
                        labels: [],
                        datasets: [{
                            label: 'CPU Usage %',
                            data: [],
                            borderColor: '#ff6b35',
                            tension: 0.1
                        }]
                    },
                    options: {
                        responsive: true,
                        scales: {
                            y: { beginAtZero: true, max: 100 }
                        }
                    }
                });

                ws.onmessage = function(event) {
                    const data = JSON.parse(event.data);

                    // Update metrics
                    document.getElementById('cpu').textContent = data.cpu + '%';
                    document.getElementById('memory').textContent = data.memory + '%';
                    document.getElementById('users').textContent = data.active_users;

                    // Update chart
                    const time = new Date().toLocaleTimeString();
                    chart.data.labels.push(time);
                    chart.data.datasets[0].data.push(data.cpu);

                    // Keep only last 20 points
                    if (chart.data.labels.length > 20) {
                        chart.data.labels.shift();
                        chart.data.datasets[0].data.shift();
                    }

                    chart.update('none');
                };
            </script>
        </body>
        </html>
    "#))
}

async fn handle_data_stream(websocket: WebSocketConnection) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(1));

    loop {
        ticker.tick().await;

        // Generate mock metrics
        let cpu_usage = rand::random::<f32>() * 100.0;
        let memory_usage = 60.0 + rand::random::<f32>() * 30.0;
        let active_users = 50 + (rand::random::<u32>() % 100);

        let metrics = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "cpu": (cpu_usage as u32),
            "memory": (memory_usage as u32),
            "active_users": active_users,
            "requests_per_sec": rand::random::<u32>() % 1000,
            "error_rate": rand::random::<f32>() * 5.0
        });

        if websocket.send_json(&metrics).await.is_err() {
            println!("Client disconnected from data stream");
            break;
        }
    }

    Ok(())
}
```

***

## File Upload Handling

### Complete File Upload System

```rust
use ignitia::{Router, Server, Response, Result, Multipart, MultipartConfig};
use std::path::PathBuf;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Create uploads directory
    fs::create_dir_all("uploads").await.unwrap();

    let router = Router::new()
        .get("/", upload_page)
        .post("/upload", handle_file_upload)
        .post("/upload/multiple", handle_multiple_uploads)
        .get("/files/:filename", serve_file)
        .get("/files", list_files);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 File server blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}

async fn upload_page() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>File Upload 🔥</title>
            <style>
                .upload-area {
                    border: 2px dashed #ccc;
                    padding: 40px;
                    text-align: center;
                    margin: 20px 0;
                }
                .upload-area:hover { border-color: #ff6b35; }
            </style>
        </head>
        <body>
            <h1>Ignitia File Upload System 🔥</h1>

            <h2>Single File Upload</h2>
            <form action="/upload" method="post" enctype="multipart/form-data">
                <div class="upload-area">
                    <input type="file" name="file" required>
                    <input type="text" name="description" placeholder="File description">
                    <br><br>
                    <button type="submit">Upload File</button>
                </div>
            </form>

            <h2>Multiple Files Upload</h2>
            <form action="/upload/multiple" method="post" enctype="multipart/form-data">
                <div class="upload-area">
                    <input type="file" name="files" multiple required>
                    <br><br>
                    <button type="submit">Upload Files</button>
                </div>
            </form>

            <h2>Uploaded Files</h2>
            <div id="fileList">Loading...</div>

            <script>
                // Load file list
                fetch('/files')
                    .then(r => r.json())
                    .then(files => {
                        const fileList = document.getElementById('fileList');
                        fileList.innerHTML = files.map(file =>
                            `<div><a href="/files/${file.name}">${file.name}</a> (${file.size} bytes)</div>`
                        ).join('');
                    });
            </script>
        </body>
        </html>
    "#))
}

async fn handle_file_upload(mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().to_string();

        if field_name == "file" && field.is_file() {
            let filename = field.file_name()
                .unwrap_or("unknown")
                .to_string();

            // Generate safe filename
            let safe_filename = sanitize_filename(&filename);
            let file_path = PathBuf::from("uploads").join(&safe_filename);

            // Save file
            let file_field = field.save_to_file(&file_path).await?;

            uploaded_files.push(serde_json::json!({
                "original_name": filename,
                "saved_name": safe_filename,
                "size": file_field.size,
                "path": file_path.to_string_lossy()
            }));
        } else if field_name == "description" {
            let description = field.text().await?;
            println!("File description: {}", description);
        }
    }

    Response::json(&serde_json::json!({
        "status": "success",
        "uploaded_files": uploaded_files,
        "count": uploaded_files.len()
    }))
}

async fn handle_multiple_uploads(mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.name() == "files" && field.is_file() {
            let filename = field.file_name()
                .unwrap_or("unknown")
                .to_string();

            let safe_filename = sanitize_filename(&filename);
            let file_path = PathBuf::from("uploads").join(&safe_filename);

            let file_field = field.save_to_file(&file_path).await?;

            uploaded_files.push(serde_json::json!({
                "original_name": filename,
                "saved_name": safe_filename,
                "size": file_field.size,
                "content_type": file_field.content_type
            }));
        }
    }

    Response::json(&serde_json::json!({
        "status": "success",
        "uploaded_files": uploaded_files,
        "total_count": uploaded_files.len()
    }))
}

async fn serve_file(ignitia::Path(filename): ignitia::Path<String>) -> Result<Response> {
    let file_path = PathBuf::from("uploads").join(&filename);

    // Security check - ensure file is in uploads directory
    if !file_path.starts_with("uploads") {
        return Err(ignitia::Error::Forbidden);
    }

    match fs::read(&file_path).await {
        Ok(contents) => {
            let content_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            let mut response = Response::new(http::StatusCode::OK)
                .with_body(contents);

            response.headers.insert(
                "Content-Type",
                content_type.parse().unwrap()
            );

            // Add download headers for certain file types
            if !content_type.starts_with("image/") && !content_type.starts_with("text/") {
                response.headers.insert(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", filename).parse().unwrap()
                );
            }

            Ok(response)
        }
        Err(_) => Err(ignitia::Error::NotFound(filename))
    }
}

async fn list_files() -> Result<Response> {
    let mut files = Vec::new();
    let mut dir = fs::read_dir("uploads").await.unwrap();

    while let Some(entry) = dir.next_entry().await.unwrap() {
        let metadata = entry.metadata().await.unwrap();
        if metadata.is_file() {
            files.push(serde_json::json!({
                "name": entry.file_name().to_string_lossy(),
                "size": metadata.len(),
                "modified": metadata.modified().unwrap()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }));
        }
    }

    Response::json(&files)
}

fn sanitize_filename(filename: &str) -> String {
    // Remove path separators and dangerous characters
    filename
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect::<String>()
        .trim()
        .to_string()
}
```

***

## Error Handling Patterns

### Custom Error Types and Handlers

```rust
use ignitia::{Router, Server, Response, Result, define_error};
use serde::{Deserialize, Serialize};

// Define custom application errors
define_error! {
    AppError {
        DatabaseUnavailable(http::StatusCode::SERVICE_UNAVAILABLE, "database_unavailable", "DB_001"),
        UserNotFound(http::StatusCode::NOT_FOUND, "user_not_found", "USER_001"),
        InvalidInput(http::StatusCode::BAD_REQUEST, "invalid_input", "INPUT_001"),
        RateLimited(http::StatusCode::TOO_MANY_REQUESTS, "rate_limited", "RATE_001")
    }
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    age: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", home)
        .post("/users", create_user)
        .get("/users/:id", get_user)
        .get("/error-demo/:type", demonstrate_errors)
        .get("/panic", trigger_panic)

        // Custom error handler
        .middleware(ignitia::middleware::ErrorHandlerMiddleware::new()
            .with_details(true)
            .with_json_format(ignitia::middleware::ErrorFormat::Detailed));

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await.unwrap();
    Ok(())
}

async fn home() -> Result<Response> {
    Ok(Response::html(r#"
        <h1>Error Handling Demo 🔥</h1>
        <ul>
            <li><a href="/users/123">Valid User</a></li>
            <li><a href="/users/999">Non-existent User</a></li>
            <li><a href="/error-demo/db">Database Error</a></li>
            <li><a href="/error-demo/rate">Rate Limit Error</a></li>
            <li><a href="/panic">Panic Recovery</a></li>
        </ul>
    "#))
}

async fn create_user(ignitia::Json(req): ignitia::Json<CreateUserRequest>) -> Result<Response> {
    // Validate input
    if req.username.is_empty() {
        return Err(AppError::InvalidInput("Username cannot be empty".into()).into());
    }

    if req.age < 13 {
        return Err(AppError::InvalidInput("Age must be at least 13".into()).into());
    }

    if !req.email.contains('@') {
        return Err(AppError::InvalidInput("Invalid email format".into()).into());
    }

    // Simulate database check
    if req.username == "admin" {
        return Err(ignitia::Error::Validation("Username 'admin' is reserved".into()));
    }

    // Success response
    Response::json(&serde_json::json!({
        "id": 123,
        "username": req.username,
        "email": req.email,
        "status": "created"
    }))
}

async fn get_user(ignitia::Path(user_id): ignitia::Path<u32>) -> Result<Response> {
    // Simulate database lookup
    match user_id {
        1..=100 => {
            Response::json(&serde_json::json!({
                "id": user_id,
                "username": format!("user_{}", user_id),
                "email": format!("user_{}@example.com", user_id)
            }))
        }
        999 => Err(AppError::UserNotFound(format!("User {} not found", user_id)).into()),
        _ => Err(ignitia::Error::NotFound(format!("Invalid user ID: {}", user_id)))
    }
}

async fn demonstrate_errors(ignitia::Path(error_type): ignitia::Path<String>) -> Result<Response> {
    match error_type.as_str() {
        "db" => Err(AppError::DatabaseUnavailable("Primary database is down".into()).into()),
        "rate" => Err(AppError::RateLimited("Too many requests from this client".into()).into()),
        "validation" => Err(ignitia::Error::Validation("Invalid data format".into())),
        "auth" => Err(ignitia::Error::Unauthorized),
        "forbidden" => Err(ignitia::Error::Forbidden),
        _ => Err(ignitia::Error::BadRequest("Unknown error type".into()))
    }
}

async fn trigger_panic() -> Result<Response> {
    // This will be caught by the framework's panic handler
    panic!("This is a test panic - the framework should handle this gracefully!");
}

// Error response transformation example
fn transform_database_error(db_error: &str) -> ignitia::Error {
    if db_error.contains("connection refused") {
        AppError::DatabaseUnavailable("Database connection failed".into()).into()
    } else if db_error.contains("timeout") {
        ignitia::Error::Internal("Database operation timed out".into())
    } else {
        ignitia::Error::Internal(format!("Database error: {}", db_error))
    }
}

// Result helper functions
async fn safe_database_operation() -> Result<serde_json::Value> {
    // Simulate database operation that might fail
    let success = rand::random::<bool>();

    if success {
        Ok(serde_json::json!({"status": "success", "data": "Database operation completed"}))
    } else {
        Err(transform_database_error("connection refused"))
    }
}

// Custom error handler function
async fn custom_404_handler() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Page Not Found</title></head>
        <body>
            <h1>🔥 Oops! Page not found</h1>
            <p>The page you're looking for doesn't exist.</p>
            <a href="/">Go Home</a>
        </body>
        </html>
    "#).with_status(http::StatusCode::NOT_FOUND))
}
```

***

## State Management

### Application State with Database Connection

```rust
use ignitia::{Router, Server, Response, Result, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    username: String,
    email: String,
    created_at: String,
}

#[derive(Debug, Clone)]
struct AppState {
    users: Arc<RwLock<HashMap<u32, User>>>,
    config: AppConfig,
    metrics: Arc<RwLock<AppMetrics>>,
}

#[derive(Debug, Clone)]
struct AppConfig {
    database_url: String,
    jwt_secret: String,
    max_users: u32,
}

#[derive(Debug, Clone, Default)]
struct AppMetrics {
    requests_count: u64,
    active_users: u32,
    uptime: std::time::SystemTime,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize application state
    let state = AppState {
        users: Arc::new(RwLock::new(HashMap::new())),
        config: AppConfig {
            database_url: "postgresql://localhost/myapp".to_string(),
            jwt_secret: "super-secret-key".to_string(),
            max_users: 1000,
        },
        metrics: Arc::new(RwLock::new(AppMetrics {
            uptime: std::time::SystemTime::now(),
            ..Default::default()
        })),
    };

    let router = Router::new()
        .state(state)
        .get("/", home)
        .post("/users", create_user)
        .get("/users", list_users)
        .get("/users/:id", get_user)
        .put("/users/:id", update_user)
        .delete("/users/:id", delete_user)
        .get("/metrics", get_metrics)
        .get("/config", get_config);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Server with state management blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}

async fn home() -> Result<Response> {
    Ok(Response::html(r#"
        <h1>State Management Demo 🔥</h1>
        <ul>
            <li><a href="/users">List Users</a></li>
            <li><a href="/metrics">App Metrics</a></li>
            <li><a href="/config">App Config</a></li>
        </ul>
    "#))
}

async fn create_user(
    ignitia::Json(mut user): ignitia::Json<User>,
    State(state): State<AppState>
) -> Result<Response> {
    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.requests_count += 1;
    }

    // Check user limit
    let users_count = state.users.read().await.len();
    if users_count >= state.config.max_users as usize {
        return Err(ignitia::Error::BadRequest("User limit reached".into()));
    }

    // Generate new user ID
    let mut users = state.users.write().await;
    let new_id = users.keys().max().unwrap_or(&0) + 1;
    user.id = new_id;
    user.created_at = chrono::Utc::now().to_rfc3339();

    users.insert(new_id, user.clone());

    // Update active users count
    {
        let mut metrics = state.metrics.write().await;
        metrics.active_users = users.len() as u32;
    }

    Response::json(&user)
}

async fn list_users(State(state): State<AppState>) -> Result<Response> {
    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.requests_count += 1;
    }

    let users = state.users.read().await;
    let user_list: Vec<&User> = users.values().collect();
    Response::json(&user_list)
}

async fn get_user(
    ignitia::Path(user_id): ignitia::Path<u32>,
    State(state): State<AppState>
) -> Result<Response> {
    let users = state.users.read().await;
    match users.get(&user_id) {
        Some(user) => Response::json(user),
        None => Err(ignitia::Error::NotFound(format!("User {} not found", user_id)))
    }
}

async fn update_user(
    ignitia::Path(user_id): ignitia::Path<u32>,
    ignitia::Json(updated_user): ignitia::Json<User>,
    State(state): State<AppState>
) -> Result<Response> {
    let mut users = state.users.write().await;
    match users.get_mut(&user_id) {
        Some(user) => {
            user.username = updated_user.username;
            user.email = updated_user.email;
            Response::json(user)
        }
        None => Err(ignitia::Error::NotFound(format!("User {} not found", user_id)))
    }
}

async fn delete_user(
    ignitia::Path(user_id): ignitia::Path<u32>,
    State(state): State<AppState>
) -> Result<Response> {
    let mut users = state.users.write().await;
    match users.remove(&user_id) {
        Some(_) => {
            // Update metrics
            tokio::spawn(async move {
                let mut metrics = state.metrics.write().await;
                metrics.active_users = users.len() as u32;
            });

            Ok(Response::new(http::StatusCode::NO_CONTENT))
        }
        None => Err(ignitia::Error::NotFound(format!("User {} not found", user_id)))
    }
}

async fn get_metrics(State(state): State<AppState>) -> Result<Response> {
    let metrics = state.metrics.read().await;
    let uptime = std::time::SystemTime::now()
        .duration_since(metrics.uptime)
        .unwrap()
        .as_secs();

    Response::json(&serde_json::json!({
        "requests_count": metrics.requests_count,
        "active_users": metrics.active_users,
        "uptime_seconds": uptime,
        "memory_usage": get_memory_usage(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_config(State(state): State<AppState>) -> Result<Response> {
    // Don't expose sensitive information like JWT secret
    Response::json(&serde_json::json!({
        "max_users": state.config.max_users,
        "database_configured": !state.config.database_url.is_empty(),
        "environment": "development"
    }))
}

fn get_memory_usage() -> serde_json::Value {
    // In a real application, you'd use a proper memory profiling library
    serde_json::json!({
        "estimated_mb": 42.5,
        "note": "Memory usage is estimated"
    })
}
```

***

## Real-World Applications

### Blog API with Full CRUD

```rust
use ignitia::{Router, Server, Response, Result, Json, Path, Query, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlogPost {
    id: u32,
    title: String,
    content: String,
    author: String,
    tags: Vec<String>,
    published: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CreatePostRequest {
    title: String,
    content: String,
    author: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpdatePostRequest {
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
    published: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct BlogQuery {
    author: Option<String>,
    tag: Option<String>,
    published: Option<bool>,
    limit: Option<u32>,
    offset: Option<u32>,
}

type BlogStore = Arc<RwLock<HashMap<u32, BlogPost>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let store: BlogStore = Arc::new(RwLock::new(HashMap::new()));

    // Add sample data
    {
        let mut posts = store.write().await;
        posts.insert(1, BlogPost {
            id: 1,
            title: "Welcome to Ignitia Blog".to_string(),
            content: "This is the first post on our Ignitia-powered blog!".to_string(),
            author: "admin".to_string(),
            tags: vec!["welcome".to_string(), "ignitia".to_string()],
            published: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    let router = Router::new()
        .state(store)
        .get("/", blog_home)

        // Blog post CRUD
        .get("/posts", list_posts)
        .post("/posts", create_post)
        .get("/posts/:id", get_post)
        .put("/posts/:id", update_post)
        .delete("/posts/:id", delete_post)

        // Blog operations
        .post("/posts/:id/publish", publish_post)
        .post("/posts/:id/unpublish", unpublish_post)
        .get("/posts/search", search_posts)
        .get("/authors/:author/posts", get_posts_by_author)
        .get("/tags/:tag/posts", get_posts_by_tag);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Blog API blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}

async fn blog_home() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Ignitia Blog API 🔥</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
                .endpoint { background: #f5f5f5; padding: 10px; margin: 10px 0; border-radius: 5px; }
                .method { font-weight: bold; color: #ff6b35; }
            </style>
        </head>
        <body>
            <h1>Ignitia Blog API 🔥</h1>

            <h2>Available Endpoints</h2>

            <div class="endpoint">
                <span class="method">GET</span> /posts - List all posts
            </div>

            <div class="endpoint">
                <span class="method">POST</span> /posts - Create new post
            </div>

            <div class="endpoint">
                <span class="method">GET</span> /posts/:id - Get specific post
            </div>

            <div class="endpoint">
                <span class="method">PUT</span> /posts/:id - Update post
            </div>

            <div class="endpoint">
                <span class="method">DELETE</span> /posts/:id - Delete post
            </div>

            <div class="endpoint">
                <span class="method">POST</span> /posts/:id/publish - Publish post
            </div>

            <div class="endpoint">
                <span class="method">GET</span> /posts/search?q=term - Search posts
            </div>

            <div class="endpoint">
                <span class="method">GET</span> /authors/:author/posts - Posts by author
            </div>

            <div class="endpoint">
                <span class="method">GET</span> /tags/:tag/posts - Posts by tag
            </div>

            <h2>Try it out</h2>
            <a href="/posts">View all posts</a>
        </body>
        </html>
    "#))
}

async fn list_posts(
    Query(query): Query<BlogQuery>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let posts = store.read().await;
    let mut filtered_posts: Vec<&BlogPost> = posts.values().collect();

    // Apply filters
    if let Some(author) = &query.author {
        filtered_posts.retain(|post| post.author == *author);
    }

    if let Some(tag) = &query.tag {
        filtered_posts.retain(|post| post.tags.contains(tag));
    }

    if let Some(published) = query.published {
        filtered_posts.retain(|post| post.published == published);
    }

    // Sort by creation date (newest first)
    filtered_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Apply pagination
    let offset = query.offset.unwrap_or(0) as usize;
    let limit = query.limit.unwrap_or(10) as usize;

    let paginated_posts: Vec<&BlogPost> = filtered_posts
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Response::json(&serde_json::json!({
        "posts": paginated_posts,
        "pagination": {
            "offset": offset,
            "limit": limit,
            "total": posts.len()
        }
    }))
}

async fn create_post(
    Json(req): Json<CreatePostRequest>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let mut posts = store.write().await;
    let new_id = posts.keys().max().unwrap_or(&0) + 1;

    let new_post = BlogPost {
        id: new_id,
        title: req.title,
        content: req.content,
        author: req.author,
        tags: req.tags,
        published: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    posts.insert(new_id, new_post.clone());

    Response::json(&new_post).map(|r| r.with_status(http::StatusCode::CREATED))
}

async fn get_post(
    Path(post_id): Path<u32>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let posts = store.read().await;
    match posts.get(&post_id) {
        Some(post) => Response::json(post),
        None => Err(ignitia::Error::NotFound(format!("Post {} not found", post_id)))
    }
}

async fn update_post(
    Path(post_id): Path<u32>,
    Json(req): Json<UpdatePostRequest>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let mut posts = store.write().await;
    match posts.get_mut(&post_id) {
        Some(post) => {
            if let Some(title) = req.title {
                post.title = title;
            }
            if let Some(content) = req.content {
                post.content = content;
            }
            if let Some(tags) = req.tags {
                post.tags = tags;
            }
            if let Some(published) = req.published {
                post.published = published;
            }
            post.updated_at = chrono::Utc::now().to_rfc3339();

            Response::json(post)
        }
        None => Err(ignitia::Error::NotFound(format!("Post {} not found", post_id)))
    }
}

async fn delete_post(
    Path(post_id): Path<u32>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let mut posts = store.write().await;
    match posts.remove(&post_id) {
        Some(_) => Ok(Response::new(http::StatusCode::NO_CONTENT)),
        None => Err(ignitia::Error::NotFound(format!("Post {} not found", post_id)))
    }
}

async fn publish_post(
    Path(post_id): Path<u32>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let mut posts = store.write().await;
    match posts.get_mut(&post_id) {
        Some(post) => {
            post.published = true;
            post.updated_at = chrono::Utc::now().to_rfc3339();
            Response::json(post)
        }
        None => Err(ignitia::Error::NotFound(format!("Post {} not found", post_id)))
    }
}

async fn unpublish_post(
    Path(post_id): Path<u32>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let mut posts = store.write().await;
    match posts.get_mut(&post_id) {
        Some(post) => {
            post.published = false;
            post.updated_at = chrono::Utc::now().to_rfc3339();
            Response::json(post)
        }
        None => Err(ignitia::Error::NotFound(format!("Post {} not found", post_id)))
    }
}

async fn search_posts(
    Query(params): Query<HashMap<String, String>>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let query = params.get("q").unwrap_or(&"".to_string()).to_lowercase();
    let posts = store.read().await;

    let matching_posts: Vec<&BlogPost> = posts
        .values()
        .filter(|post| {
            post.title.to_lowercase().contains(&query) ||
            post.content.to_lowercase().contains(&query) ||
            post.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
        })
        .collect();

    Response::json(&serde_json::json!({
        "query": query,
        "results": matching_posts,
        "count": matching_posts.len()
    }))
}

async fn get_posts_by_author(
    Path(author): Path<String>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let posts = store.read().await;
    let author_posts: Vec<&BlogPost> = posts
        .values()
        .filter(|post| post.author == author)
        .collect();

    Response::json(&serde_json::json!({
        "author": author,
        "posts": author_posts,
        "count": author_posts.len()
    }))
}

async fn get_posts_by_tag(
    Path(tag): Path<String>,
    State(store): State<BlogStore>
) -> Result<Response> {
    let posts = store.read().await;
    let tagged_posts: Vec<&BlogPost> = posts
        .values()
        .filter(|post| post.tags.contains(&tag))
        .collect();

    Response::json(&serde_json::json!({
        "tag": tag,
        "posts": tagged_posts,
        "count": tagged_posts.len()
    }))
}
```

***

## Performance Optimization

### High-Performance API Server

```rust
use ignitia::{
    Router, Server, Response, Result, Json, State,
    middleware::{CompressionMiddleware, RateLimitingMiddleware}
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
struct PerformanceMetrics {
    request_count: u64,
    average_response_time: f64,
    cache_hits: u64,
    cache_misses: u64,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    data: String,
    timestamp: Instant,
    ttl: std::time::Duration,
}

impl CachedResponse {
    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }
}

#[derive(Debug, Clone)]
struct AppState {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        cache: Arc::new(RwLock::new(HashMap::new())),
        metrics: Arc::new(RwLock::new(PerformanceMetrics {
            request_count: 0,
            average_response_time: 0.0,
            cache_hits: 0,
            cache_misses: 0,
        })),
    };

    let router = Router::new()
        .state(state)

        // Performance middleware
        .middleware(CompressionMiddleware::high_compression())
        .middleware(RateLimitingMiddleware::per_second(1000)) // High rate limit for performance

        // Routes
        .get("/", performance_home)
        .get("/api/data", cached_data_endpoint)
        .get("/api/heavy", heavy_computation)
        .get("/api/bulk/:count", bulk_data_generation)
        .get("/metrics", get_performance_metrics)
        .post("/cache/clear", clear_cache);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 High-performance server blazing on http://127.0.0.1:3000");
    server.ignitia().await.unwrap();
    Ok(())
}

async fn performance_home() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Performance Demo 🔥</title>
            <script>
                async function runBenchmark() {
                    const results = document.getElementById('results');
                    const iterations = 100;
                    const start = performance.now();

                    const promises = [];
                    for (let i = 0; i < iterations; i++) {
                        promises.push(fetch('/api/data'));
                    }

                    await Promise.all(promises);
                    const end = performance.now();

                    const totalTime = end - start;
                    const avgTime = totalTime / iterations;

                    results.innerHTML = `
                        <h3>Benchmark Results</h3>
                        <p>Total time: ${totalTime.toFixed(2)}ms</p>
                        <p>Average time per request: ${avgTime.toFixed(2)}ms</p>
                        <p>Requests per second: ${(1000 / avgTime * iterations).toFixed(0)}</p>
                    `;
                }
            </script>
        </head>
        <body>
            <h1>Performance Optimization Demo 🔥</h1>
            <button onclick="runBenchmark()">Run Benchmark</button>
            <div id="results"></div>

            <h2>Test Endpoints</h2>
            <ul>
                <li><a href="/api/data">Cached Data (Fast)</a></li>
                <li><a href="/api/heavy">Heavy Computation (Slow)</a></li>
                <li><a href="/api/bulk/1000">Bulk Data Generation</a></li>
                <li><a href="/metrics">Performance Metrics</a></li>
            </ul>
        </body>
        </html>
    "#))
}

async fn cached_data_endpoint(State(state): State<AppState>) -> Result<Response> {
    let start_time = Instant::now();
    let cache_key = "data_endpoint".to_string();

    // Try cache first
    {
        let cache = state.cache.read().await;
        if let Some(cached) = cache.get(&cache_key) {
            if !cached.is_expired() {
                // Update metrics for cache hit
                {
                    let mut metrics = state.metrics.write().await;
                    metrics.cache_hits += 1;
                    metrics.request_count += 1;
                }

                return Ok(Response::json(&serde_json::json!({
                    "data": cached.data,
                    "cached": true,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))?);
            }
        }
    }

    // Cache miss - generate data
    let expensive_data = generate_expensive_data().await;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.insert(cache_key, CachedResponse {
            data: expensive_data.clone(),
            timestamp: Instant::now(),
            ttl: std::time::Duration::from_secs(60), // 1 minute TTL
        });
    }

    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.cache_misses += 1;
        metrics.request_count += 1;

        let response_time = start_time.elapsed().as_secs_f64();
        metrics.average_response_time =
            (metrics.average_response_time * (metrics.request_count - 1) as f64 + response_time)
            / metrics.request_count as f64;
    }

    Response::json(&serde_json::json!({
        "data": expensive_data,
        "cached": false,
        "generated_at": chrono::Utc::now().to_rfc3339()
    }))
}

async fn heavy_computation() -> Result<Response> {
    let start = Instant::now();

    // Simulate heavy computation
    let result = tokio::task::spawn_blocking(|| {
        let mut sum = 0u64;
        for i in 0..1_000_000 {
            sum += fibonacci(i % 30);
        }
        sum
    }).await.unwrap();

    let duration = start.elapsed();

    Response::json(&serde_json::json!({
        "result": result,
        "computation_time_ms": duration.as_millis(),
        "note": "This endpoint performs heavy computation without caching"
    }))
}

async fn bulk_data_generation(
    ignitia::Path(count): ignitia::Path<u32>,
    State(state): State<AppState>
) -> Result<Response> {
    let start_time = Instant::now();

    // Limit count to prevent abuse
    let safe_count = count.min(10000);

    // Generate bulk data efficiently
    let data: Vec<serde_json::Value> = (0..safe_count)
        .map(|i| serde_json::json!({
            "id": i,
            "name": format!("Item {}", i),
            "value": rand::random::<f64>() * 100.0,
            "active": i % 2 == 0
        }))
        .collect();

    let generation_time = start_time.elapsed();

    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.request_count += 1;
    }

    Response::json(&serde_json::json!({
        "data": data,
        "count": safe_count,
        "generation_time_ms": generation_time.as_millis(),
        "items_per_second": (safe_count as f64 / generation_time.as_secs_f64()) as u32
    }))
}

async fn get_performance_metrics(State(state): State<AppState>) -> Result<Response> {
    let metrics = state.metrics.read().await;
    let cache = state.cache.read().await;

    Response::json(&serde_json::json!({
        "metrics": *metrics,
        "cache_size": cache.len(),
        "uptime": "N/A", // Could track server start time
        "memory_usage": get_memory_info()
    }))
}

async fn clear_cache(State(state): State<AppState>) -> Result<Response> {
    let mut cache = state.cache.write().await;
    let cleared_count = cache.len();
    cache.clear();

    Response::json(&serde_json::json!({
        "message": "Cache cleared successfully",
        "cleared_items": cleared_count
    }))
}

async fn generate_expensive_data() -> String {
    // Simulate expensive data generation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    format!("Expensive data generated at {}", chrono::Utc::now().to_rfc3339())
}

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

fn get_memory_info() -> serde_json::Value {
    // In a real app, you'd use a proper memory profiling crate
    serde_json::json!({
        "estimated_mb": 45.2,
        "note": "Use proper profiling tools in production"
    })
}
```

***

## Testing Examples

### Comprehensive Test Suite

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ignitia::{Router, Server, Response, Result};
    use serde_json::json;

    // Integration tests
    #[tokio::test]
    async fn test_basic_routes() {
        let router = Router::new()
            .get("/", || async { Ok(Response::text("Hello")) })
            .get("/json", || async {
                Response::json(&json!({"message": "test"}))
            });

        // Test GET /
        let request = Request::new(
            http::Method::GET,
            "/".parse().unwrap(),
            http::Version::HTTP_11,
            http::HeaderMap::new(),
            bytes::Bytes::new(),
        );

        let response = router.handle(request).await.unwrap();
        assert_eq!(response.status, http::StatusCode::OK);
        assert_eq!(response.body, bytes::Bytes::from("Hello"));
    }

    #[tokio::test]
    async fn test_json_endpoint() {
        let router = Router::new()
            .get("/api/test", || async {
                Response::json(&json!({
                    "status": "success",
                    "data": {"id": 1, "name": "Test"}
                }))
            });

        let request = Request::new(
            http::Method::GET,
            "/api/test".parse().unwrap(),
            http::Version::HTTP_11,
            http::HeaderMap::new(),
            bytes::Bytes::new(),
        );

        let response = router.handle(request).await.unwrap();
        assert_eq!(response.status, http::StatusCode::OK);

        let body_str = String::from_utf8(response.body.to_vec()).unwrap();
        let json_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json_response["status"], "success");
        assert_eq!(json_response["data"]["name"], "Test");
    }

    #[tokio::test]
    async fn test_path_parameters() {
        let router = Router::new()
            .get("/users/:id", |ignitia::Path(id): ignitia::Path<u32>| async move {
                Response::json(&json!({"user_id": id}))
            });

        let mut request = Request::new(
            http::Method::GET,
            "/users/123".parse().unwrap(),
            http::Version::HTTP_11,
            http::HeaderMap::new(),
            bytes::Bytes::new(),
        );

        // Simulate route matching (normally done by the router)
        request.params.insert("id".to_string(), "123".to_string());

        let response = router.handle(request).await.unwrap();
        let body_str = String::from_utf8(response.body.to_vec()).unwrap();
        let json_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json_response["user_id"], 123);
    }

    #[tokio::test]
    async fn test_post_with_json() {
        let router = Router::new()
            .post("/users", |ignitia::Json(user): ignitia::Json<serde_json::Value>| async move {
                Response::json(&json!({
                    "created": user,
                    "id": 456
                }))
            });

        let user_data = json!({"name": "John", "email": "john@test.com"});
        let body = serde_json::to_vec(&user_data).unwrap();

        let mut headers = http::HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());

        let request = Request::new(
            http::Method::POST,
            "/users".parse().unwrap(),
            http::Version::HTTP_11,
            headers,
            bytes::Bytes::from(body),
        );

        let response = router.handle(request).await.unwrap();
        assert_eq!(response.status, http::StatusCode::OK);

        let body_str = String::from_utf8(response.body.to_vec()).unwrap();
        let json_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json_response["created"]["name"], "John");
        assert_eq!(json_response["id"], 456);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let router = Router::new()
            .get("/error", || async {
                Err(ignitia::Error::BadRequest("Test error".into()))
            })
            .get("/not-found", || async {
                Err(ignitia::Error::NotFound("Resource not found".into()))
            });

        // Test 400 error
        let request = Request::new(
            http::Method::GET,
            "/error".parse().unwrap(),
            http::Version::HTTP_11,
            http::HeaderMap::new(),
            bytes::Bytes::new(),
        );

        let response = router.handle(request).await.unwrap();
        assert_eq!(response.status, http::StatusCode::BAD_REQUEST);

        // Test 404 error
        let request = Request::new(
            http::Method::GET,
            "/not-found".parse().unwrap(),
            http::Version::HTTP_11,
            http::HeaderMap::new(),
            bytes::Bytes::new(),
        );

        let response = router.handle(request).await.unwrap();
        assert_eq!(response.status, http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_middleware() {
        use ignitia::middleware::LoggerMiddleware;

        let router = Router::new()
            .middleware(LoggerMiddleware)
            .get("/", || async { Ok(Response::text("Hello with middleware")) });

        let request = Request::new(
            http::Method::GET,
            "/".parse().unwrap(),
            http::Version::HTTP_11,
            http::HeaderMap::new(),
            bytes::Bytes::new(),
        );

        let response = router.handle(request).await.unwrap();
        assert_eq!(response.status, http::StatusCode::OK);
        assert!(response.body.len() > 0);
    }

    // Benchmark tests
    #[tokio::test]
    async fn benchmark_simple_route() {
        let router = Router::new()
            .get("/bench", || async { Ok(Response::text("Benchmark")) });

        let start = std::time::Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let request = Request::new(
                http::Method::GET,
                "/bench".parse().unwrap(),
                http::Version::HTTP_11,
                http::HeaderMap::new(),
                bytes::Bytes::new(),
            );

            let _response = router.handle(request).await.unwrap();
        }

        let duration = start.elapsed();
        let rps = (iterations as f64 / duration.as_secs_f64()) as u32;

        println!("Processed {} requests in {:?}", iterations, duration);
        println!("Requests per second: {}", rps);

        // Assert that we can handle at least 10,000 RPS
        assert!(rps > 10_000, "Performance regression: {} RPS", rps);
    }

    // Helper function for creating test requests
    fn create_test_request(
        method: http::Method,
        path: &str,
        headers: Option<http::HeaderMap>,
        body: Option<&str>,
    ) -> Request {
        Request::new(
            method,
            path.parse().unwrap(),
            http::Version::HTTP_11,
            headers.unwrap_or_default(),
            body.map(|b| bytes::Bytes::from(b.to_string()))
                .unwrap_or_default(),
        )
    }

    // Test state management
    #[tokio::test]
    async fn test_state_management() {
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let counter = Arc::new(Mutex::new(0u32));

        let router = Router::new()
            .state(counter)
            .get("/increment", |ignitia::State(counter): ignitia::State<Arc<Mutex<u32>>>| async move {
                let mut count = counter.lock().await;
                *count += 1;
                Response::json(&json!({"count": *count}))
            });

        // Test multiple increments
        for expected_count in 1..=5 {
            let request = create_test_request(http::Method::GET, "/increment", None, None);
            let response = router.handle(request).await.unwrap();

            let body_str = String::from_utf8(response.body.to_vec()).unwrap();
            let json_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();

            assert_eq!(json_response["count"], expected_count);
        }
    }
}

// Load testing utility
#[cfg(test)]
mod load_tests {
    use super::*;
    use tokio::time::{Duration, Instant};

    #[tokio::test]
    #[ignore] // Use `cargo test -- --ignored` to run load tests
    async fn load_test_concurrent_requests() {
        let router = Router::new()
            .get("/load", || async {
                // Simulate some work
                tokio::time::sleep(Duration::from_millis(10)).await;
                Response::json(&json!({"timestamp": chrono::Utc::now().to_rfc3339()}))
            });

        let concurrent_requests = 100;
        let requests_per_client = 10;
        let start = Instant::now();

        let mut handles = Vec::new();

        for _ in 0..concurrent_requests {
            let router_clone = router.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..requests_per_client {
                    let request = Request::new(
                        http::Method::GET,
                        "/load".parse().unwrap(),
                        http::Version::HTTP_11,
                        http::HeaderMap::new(),
                        bytes::Bytes::new(),
                    );

                    let response = router_clone.handle(request).await.unwrap();
                    assert_eq!(response.status, http::StatusCode::OK);
                }
            }));
        }

        // Wait for all requests to complete
        for handle in handles {
            handle.await.unwrap();
        }

        let duration = start.elapsed();
        let total_requests = concurrent_requests * requests_per_client;
        let rps = (total_requests as f64 / duration.as_secs_f64()) as u32;

        println!("Load test results:");
        println!("- Total requests: {}", total_requests);
        println!("- Duration: {:?}", duration);
        println!("- Requests per second: {}", rps);
        println!("- Average response time: {:?}", duration / total_requests);

        // Assert performance requirements
        assert!(rps > 1000, "Load test failed: {} RPS", rps);
    }
}
```

***

## Conclusion

These examples demonstrate the full power and flexibility of the Ignitia web framework. From simple "Hello World" applications to complex real-world systems with authentication, file uploads, WebSockets, and performance optimizations, Ignitia provides all the tools you need to build blazing fast web applications in Rust.
