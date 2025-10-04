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
12. [TLS/HTTPS Examples](#tlshttps-examples)
13. [Testing Examples](#testing-examples)

---

## Quick Start Examples

### Hello World Server

The simplest possible Ignitia application:

```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Hello from Ignitia! 🔥")) });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Server blazing on http://127.0.0.1:3000");
    server.ignitia().await?;
    Ok(())
}
```

### JSON API Server

A simple JSON API with multiple endpoints:

```rust
use ignitia::prelude::*;
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
        .get("/api/users/{id}", get_user);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}

async fn health_check() -> Result<Response> {
    Ok(Response::json(&ApiResponse {
        message: "Service is healthy".to_string(),
        status: "ok".to_string(),
    })?)
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    println!("Creating user: {} ({})", user.name, user.email);
    Ok(Response::json(&ApiResponse {
        message: format!("User {} created successfully", user.name),
        status: "created".to_string(),
    })?)
}

async fn get_user(Path(id): Path<u32>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "id": id,
        "name": "John Doe",
        "email": "john@example.com"
    }))?)
}
```

---

## HTTP Methods & Routing

### Complete CRUD API

```rust
use ignitia::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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
        .get("/tasks/{id}", get_task)
        .put("/tasks/{id}", update_task)
        .delete("/tasks/{id}", delete_task)
        // Additional routes
        .patch("/tasks/{id}/complete", complete_task)
        .get("/tasks/completed", list_completed_tasks);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Task API blazing on http://127.0.0.1:3000");
    server.ignitia().await?;
    Ok(())
}

async fn list_tasks(State(store): State<TaskStore>) -> Result<Response> {
    let tasks = store.lock().await;
    let task_list: Vec<&Task> = tasks.values().collect();
    Ok(Response::json(&task_list)?)
}

async fn create_task(
    Json(mut task): Json<Task>,
    State(store): State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().await;
    let id = tasks.len() as u32 + 1;
    task.id = id;
    tasks.insert(id, task.clone());
    Ok(Response::json(&task)?)
}

async fn get_task(
    Path(id): Path<u32>,
    State(store): State<TaskStore>
) -> Result<Response> {
    let tasks = store.lock().await;
    match tasks.get(&id) {
        Some(task) => Ok(Response::json(task)?),
        None => Err(Error::not_found(&format!("Task {} not found", id)))
    }
}

async fn update_task(
    Path(id): Path<u32>,
    Json(updated_task): Json<Task>,
    State(store): State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().await;
    match tasks.get_mut(&id) {
        Some(task) => {
            task.title = updated_task.title;
            task.completed = updated_task.completed;
            Ok(Response::json(task)?)
        }
        None => Err(Error::not_found(&format!("Task {} not found", id)))
    }
}

async fn delete_task(
    Path(id): Path<u32>,
    State(store): State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().await;
    match tasks.remove(&id) {
        Some(_) => Ok(Response::new(StatusCode::NO_CONTENT)),
        None => Err(Error::not_found(&format!("Task {} not found", id)))
    }
}

async fn complete_task(
    Path(id): Path<u32>,
    State(store): State<TaskStore>
) -> Result<Response> {
    let mut tasks = store.lock().await;
    match tasks.get_mut(&id) {
        Some(task) => {
            task.completed = true;
            Ok(Response::json(task)?)
        }
        None => Err(Error::not_found(&format!("Task {} not found", id)))
    }
}

async fn list_completed_tasks(State(store): State<TaskStore>) -> Result<Response> {
    let tasks = store.lock().await;
    let completed: Vec<&Task> = tasks.values().filter(|t| t.completed).collect();
    Ok(Response::json(&completed)?)
}
```

### Dynamic Route Patterns

```rust
use ignitia::prelude::*;
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
        .get("/users/{id}", get_user_by_id)
        .get("/posts/{slug}", get_post_by_slug)

        // Multiple path parameters
        .get("/users/{id}/{action}", user_action)
        .get("/categories/{category}/posts/{id}", get_category_post)

        // Wildcard routes
        .get("/static/{*path}", serve_static)
        .get("/docs/{*path}", serve_docs)

        // Query parameters
        .get("/search", search_with_query)
        .get("/api/posts", list_posts_paginated);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}

async fn get_user_by_id(Path(id): Path<u32>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "user_id": id,
        "message": format!("Retrieved user {}", id)
    }))?)
}

async fn get_post_by_slug(Path(slug): Path<String>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "slug": slug,
        "title": format!("Post: {}", slug.replace("-", " "))
    }))?)
}

async fn user_action(Path(params): Path<UserParams>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "user_id": params.id,
        "action": params.action,
        "message": format!("Executing {} for user {}", params.action, params.id)
    }))?)
}

async fn get_category_post(Path((category, id)): Path<(String, u32)>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "category": category,
        "post_id": id,
        "url": format!("/categories/{}/posts/{}", category, id)
    }))?)
}

async fn serve_static(Path(path): Path<String>) -> Result<Response> {
    // In a real application, you'd serve actual files
    Ok(Response::text(&format!("Serving static file: {}", path)))
}

async fn serve_docs(Path(path): Path<String>) -> Result<Response> {
    Ok(Response::html(&format!("<h1>Documentation: {}</h1>", path)))
}

async fn search_with_query(Query(params): Query<std::collections::HashMap<String, String>>) -> Result<Response> {
    let query = params.get("q").unwrap_or(&"".to_string());
    let category = params.get("category").unwrap_or(&"all".to_string());

    Ok(Response::json(&serde_json::json!({
        "query": query,
        "category": category,
        "results": []
    }))?)
}

async fn list_posts_paginated(Query(pagination): Query<Pagination>) -> Result<Response> {
    let page = pagination.page.unwrap_or(1);
    let size = pagination.size.unwrap_or(10);

    Ok(Response::json(&serde_json::json!({
        "page": page,
        "size": size,
        "total": 100,
        "posts": []
    }))?)
}
```

---

## Request Extractors

### All Available Extractors

```rust
use ignitia::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .post("/login", handle_login)
        .post("/api/data", handle_json)
        .get("/debug", debug_request)
        .post("/raw", handle_raw_body);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}

// Form extractor
async fn handle_login(Form(login): Form<LoginForm>) -> Result<Response> {
    if login.username == "admin" && login.password == "secret" {
        Ok(Response::json(&serde_json::json!({
            "status": "success",
            "message": "Login successful"
        }))?)
    } else {
        Err(Error::unauthorized("Invalid credentials"))
    }
}

// JSON extractor
async fn handle_json(Json(data): Json<serde_json::Value>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "received": data,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))?)
}

// Multiple extractors
async fn debug_request(
    Headers(headers): Headers,
    Cookies(cookies): Cookies,
    Uri(uri): Uri
) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "headers": headers,
        "cookies": cookies.all(),
        "uri": uri.to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))?)
}

// Raw body extractor
async fn handle_raw_body(Body(body): Body) -> Result<Response> {
    let body_size = body.len();
    Ok(Response::json(&serde_json::json!({
        "body_size": body_size,
        "content_type": "raw"
    }))?)
}
```

---

## Middleware Examples

### Complete Middleware Stack

```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        // Add middleware in order
        .middleware(LoggerMiddleware::new())
        .middleware(
            CorsMiddleware::builder()
                .allowed_origins(vec!["http://localhost:3000".to_string()])
                .allowed_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allowed_headers(vec!["Content-Type".to_string(), "Authorization".to_string()])
                .allow_credentials(true)
                .build()
        )
        .middleware(SecurityMiddleware::strict())
        .middleware(RateLimitingMiddleware::per_minute(100))

        // Routes
        .get("/", home_page)
        .get("/api/data", get_data)
        .post("/api/upload", upload_data);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}

async fn home_page() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Ignitia App</title></head>
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
    Ok(Response::json(&serde_json::json!({
        "message": "Data retrieved successfully",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))?)
}

async fn upload_data(Json(data): Json<serde_json::Value>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "status": "uploaded",
        "size": data.to_string().len()
    }))?)
}
```

### Custom Middleware

```rust
use ignitia::prelude::*;
use std::time::Instant;

#[derive(Clone)]
struct TimingMiddleware;

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Result<Response> {
        let start = Instant::now();
        let path = req.uri.path().to_string();

        let response = next.run(req).await?;

        let duration = start.elapsed();
        println!("⏱️  {} took {:?}", path, duration);

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .middleware(TimingMiddleware)
        .get("/", || async { Ok(Response::text("Hello!")) });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

---

## TLS/HTTPS Examples

### Production HTTPS Server

```rust
#[cfg(feature = "tls")]
use ignitia::prelude::*;

#[cfg(feature = "tls")]
#[tokio::main]
async fn main() -> Result<()> {
    // Configure TLS
    let tls_config = TlsConfig::new()
        .with_cert_and_key("cert.pem", "key.pem")?
        .with_tls_versions(vec![TlsVersion::TLS_1_2, TlsVersion::TLS_1_3])?;

    let router = Router::new()
        .get("/", || async { Ok(Response::text("Secure connection! 🔒")) });

    let config = ServerConfig::builder()
        .tls(tls_config)
        .build();

    let server = Server::with_config(router, "0.0.0.0:443".parse().unwrap(), config);
    println!("🔥 Secure server running on https://0.0.0.0:443");
    server.ignitia().await?;
    Ok(())
}

#[cfg(not(feature = "tls"))]
fn main() {
    println!("Enable 'tls' feature to run this example");
}
```

### Self-Signed Certificate for Development

```rust
#[cfg(feature = "self-signed")]
use ignitia::prelude::*;

#[cfg(feature = "self-signed")]
#[tokio::main]
async fn main() -> Result<()> {
    // Generate self-signed certificate
    let tls_config = TlsConfig::self_signed()?;

    let router = Router::new()
        .get("/", || async { Ok(Response::text("Development HTTPS 🔧")) });

    let config = ServerConfig::builder()
        .tls(tls_config)
        .build();

    let server = Server::with_config(router, "127.0.0.1:8443".parse().unwrap(), config);
    println!("🔥 Development server running on https://127.0.0.1:8443");
    println!("⚠️  Warning: Using self-signed certificate");
    server.ignitia().await?;
    Ok(())
}

#[cfg(not(feature = "self-signed"))]
fn main() {
    println!("Enable 'self-signed' feature to run this example");
}
```

---

## File Upload Handling

### Complete File Upload System

```rust
use ignitia::prelude::*;
use std::path::PathBuf;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Create uploads directory
    fs::create_dir_all("uploads").await?;

    let router = Router::new()
        .get("/", upload_page)
        .post("/upload", handle_file_upload)
        .post("/upload/multiple", handle_multiple_uploads)
        .get("/files/{filename}", serve_file)
        .get("/files", list_files);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 File server blazing on http://127.0.0.1:3000");
    server.ignitia().await?;
    Ok(())
}

async fn upload_page() -> Result<Response> {
    Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>File Upload 🔥</title></head>
        <body>
            <h1>Ignitia File Upload System 🔥</h1>
            <form action="/upload" method="post" enctype="multipart/form-data">
                <input type="file" name="file" required>
                <input type="text" name="description" placeholder="Description">
                <button type="submit">Upload</button>
            </form>
        </body>
        </html>
    "#))
}

async fn handle_file_upload(mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            let filename = field.file_name()
                .unwrap_or("unknown")
                .to_string();

            let safe_filename = sanitize_filename(&filename);
            let file_path = PathBuf::from("uploads").join(&safe_filename);

            let file_field = field.save_to_file(&file_path).await?;

            uploaded_files.push(serde_json::json!({
                "original_name": filename,
                "saved_name": safe_filename,
                "size": file_field.size
            }));
        }
    }

    Ok(Response::json(&serde_json::json!({
        "status": "success",
        "uploaded_files": uploaded_files
    }))?)
}

async fn handle_multiple_uploads(mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            let filename = field.file_name()
                .unwrap_or("unknown")
                .to_string();

            let safe_filename = sanitize_filename(&filename);
            let file_path = PathBuf::from("uploads").join(&safe_filename);

            let file_field = field.save_to_file(&file_path).await?;

            uploaded_files.push(serde_json::json!({
                "filename": safe_filename,
                "size": file_field.size
            }));
        }
    }

    Ok(Response::json(&serde_json::json!({
        "status": "success",
        "total_count": uploaded_files.len(),
        "files": uploaded_files
    }))?)
}

async fn serve_file(Path(filename): Path<String>) -> Result<Response> {
    let file_path = PathBuf::from("uploads").join(&filename);

    if !file_path.starts_with("uploads") {
        return Err(Error::forbidden("Invalid file path"));
    }

    match fs::read(&file_path).await {
        Ok(contents) => {
            let content_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            let mut response = Response::new(StatusCode::OK)
                .with_body(contents);

            response.headers.insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type)?
            );

            Ok(response)
        }
        Err(_) => Err(Error::not_found(&filename))
    }
}

async fn list_files() -> Result<Response> {
    let mut files = Vec::new();
    let mut dir = fs::read_dir("uploads").await?;

    while let Some(entry) = dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            files.push(serde_json::json!({
                "name": entry.file_name().to_string_lossy(),
                "size": metadata.len()
            }));
        }
    }

    Ok(Response::json(&files)?)
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect::<String>()
        .trim()
        .to_string()
}
```

---

## Error Handling Patterns

### Custom Error Types

```rust
use ignitia::prelude::*;

// Define custom application errors
define_error! {
    AppError {
        DatabaseUnavailable(StatusCode::SERVICE_UNAVAILABLE, "database_unavailable", "DB_001"),
        UserNotFound(StatusCode::NOT_FOUND, "user_not_found", "USER_001"),
        InvalidInput(StatusCode::BAD_REQUEST, "invalid_input", "INPUT_001")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/users/{id}", get_user)
        .post("/users", create_user);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}

async fn get_user(Path(id): Path<u32>) -> Result<Response> {
    if id == 0 {
        return Err(AppError::InvalidInput("User ID cannot be zero".into()).into());
    }

    // Simulate database lookup
    if id > 100 {
        return Err(AppError::UserNotFound(format!("User {} not found", id)).into());
    }

    Ok(Response::json(&serde_json::json!({
        "id": id,
        "name": "John Doe"
    }))?)
}

async fn create_user(Json(data): Json<serde_json::Value>) -> Result<Response> {
    if !data.get("name").and_then(|v| v.as_str()).is_some() {
        return Err(AppError::InvalidInput("Name is required".into()).into());
    }

    Ok(Response::json(&serde_json::json!({
        "status": "created",
        "user": data
    }))?)
}
```

### Structured Error Responses

```rust
use ignitia::prelude::*;

async fn validation_example() -> Result<Response> {
    let validation_errors = vec![
        "Email is required".to_string(),
        "Password must be at least 8 characters".to_string(),
    ];

    Response::validation_error(validation_errors)
}

async fn error_with_metadata() -> Result<Response> {
    let error = Error::bad_request("Invalid request payload");

    let error_response = error.to_response(true);
    Ok(Response::json(&error_response)?)
}
```

---

## Performance Optimization

### Optimized Server Configuration

```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Optimized! ⚡")) });

    let config = ServerConfig::builder()
        .http2(
            Http2Config::builder()
                .initial_stream_window_size(1024 * 1024)
                .initial_connection_window_size(1024 * 1024)
                .max_concurrent_streams(200)
                .enable_connect_protocol(true)
                .build()
        )
        .pool(
            PoolConfig::builder()
                .workers(num_cpus::get())
                .max_blocking_threads(512)
                .build()
        )
        .performance(
            PerformanceConfig::builder()
                .tcp_nodelay(true)
                .tcp_fastopen(true)
                .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
                .reuse_port(true)
                .build()
        )
        .build();

    let server = Server::with_config(router, "0.0.0.0:3000".parse().unwrap(), config);
    println!("🔥 Optimized server running on http://0.0.0.0:3000");
    server.ignitia().await?;
    Ok(())
}
```

---

## Testing Examples

### Unit Testing Handlers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_user() {
        let user = CreateUser {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        let response = create_user(Json(user)).await.unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        let store = Arc::new(Mutex::new(HashMap::new()));

        // Create task
        let task = Task {
            id: 0,
            title: "Test Task".to_string(),
            completed: false,
        };

        let response = create_task(Json(task.clone()), State(store.clone())).await.unwrap();
        assert_eq!(response.status, StatusCode::OK);

        // List tasks
        let response = list_tasks(State(store.clone())).await.unwrap();
        assert_eq!(response.status, StatusCode::OK);
    }
}
```

---

## Complete Examples

For more comprehensive examples, check out:

1. **REST API**: Full-featured REST API with authentication
2. **WebSocket Chat**: Real-time chat application
3. **File Server**: Complete file upload/download system
4. **Blog Platform**: Multi-user blog with comments
5. **Microservices**: Service mesh with multiple Ignitia services

Visit the [examples directory](https://github.com/AarambhDevHub/ignitia/tree/main/examples) in the repository for complete, runnable examples.

---

🔥 Happy coding with Ignitia!
