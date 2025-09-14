# Quick Start Guide for Ignitia Web Framework

Welcome to Ignitia! 🔥 This guide will help you get up and running with Ignitia, a blazing fast, lightweight web framework for Rust.

## Installation

Add Ignitia to your `Cargo.toml`:

```toml
[dependencies]
ignitia = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

For additional features, add them to your dependency:

```toml
[dependencies]
ignitia = { version = "0.2.0", features = ["websocket", "tls"] }
```

Available features:
- `websocket` - WebSocket support
- `tls` - HTTPS/TLS support
- `self-signed` - Self-signed certificate generation (development only)

## Hello World

Create your first Ignitia server in `src/main.rs`:

```rust
use ignitia::{Router, Server, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", || async {
            Ok(Response::text("Hello, Ignitia! 🔥"))
        });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());

    println!("🔥 Server starting on http://127.0.0.1:3000");
    server.ignitia().await?;

    Ok(())
}
```

Run your server:

```bash
cargo run
```

Visit `http://127.0.0.1:3000` to see your response!

## Basic Routing

```rust
use ignitia::{Router, Server, Response, Json, Path, Query};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        // GET route
        .get("/", || async {
            Ok(Response::html("<h1>Welcome to Ignitia!</h1>"))
        })

        // Route with path parameters
        .get("/users/:id", |Path(id): Path<u32>| async move {
            Ok(Response::json(User {
                id,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            })?)
        })

        // POST route with JSON body
        .post("/users", |Json(user): Json<CreateUser>| async move {
            let new_user = User {
                id: 1,
                name: user.name,
                email: user.email,
            };
            Ok(Response::json(&new_user)?)
        })

        // Query parameters
        .get("/search", |Query(params): Query<std::collections::HashMap<String, String>>| async move {
            let query = params.get("q").unwrap_or(&"".to_string()).clone();
            Ok(Response::text(format!("Searching for: {}", query)))
        });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;

    Ok(())
}
```

## Request Extractors

Ignitia provides powerful extractors to handle different parts of HTTP requests:

```rust
use ignitia::{Router, Response, Path, Query, Json, Headers, Body, Method, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct UserQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        // Path parameters
        .get("/users/:id", |Path(id): Path<u32>| async move {
            Ok(Response::text(format!("User ID: {}", id)))
        })

        // Query parameters with deserialization
        .get("/users", |Query(query): Query<UserQuery>| async move {
            let page = query.page.unwrap_or(1);
            let limit = query.limit.unwrap_or(10);
            Ok(Response::text(format!("Page: {}, Limit: {}", page, limit)))
        })

        // JSON body
        .post("/data", |Json(data): Json<serde_json::Value>| async move {
            Ok(Response::json(&data)?)
        })

        // Headers
        .get("/headers", |Headers(headers): Headers| async move {
            Ok(Response::json(&headers)?)
        })

        // Raw body
        .post("/raw", |Body(body): Body| async move {
            Ok(Response::text(format!("Body size: {} bytes", body.len())))
        })

        // Multiple extractors
        .post("/complex/:id", |
            Path(id): Path<u32>,
            Query(params): Query<HashMap<String, String>>,
            Json(body): Json<serde_json::Value>,
            Method(method): Method
        | async move {
            Ok(Response::json(serde_json::json!({
                "id": id,
                "method": method.to_string(),
                "params": params,
                "body": body
            }))?)
        });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;

    Ok(())
}
```

## Middleware

Add middleware for logging, CORS, authentication, and more:

```rust
use ignitia::{Router, Server, Response};
use ignitia::middleware::{LoggerMiddleware, CorsMiddleware};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cors = ignitia::middleware::Cors::new()
        .allow_any_origin()
        .allowed_methods(&[
            ignitia::Method::GET,
            ignitia::Method::POST,
            ignitia::Method::PUT,
            ignitia::Method::DELETE,
        ])
        .build()?;

    let router = Router::new()
        .middleware(LoggerMiddleware)
        .middleware(cors)
        .get("/", || async {
            Ok(Response::text("Hello with middleware!"))
        })
        .get("/api/data", || async {
            Ok(Response::json(serde_json::json!({
                "message": "API response with CORS headers"
            }))?)
        });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;

    Ok(())
}
```

## Error Handling

```rust
use ignitia::{Router, Server, Response, Error, Result, Json};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

async fn login_handler(Json(data): Json<LoginData>) -> Result<Response> {
    if data.username.is_empty() {
        return Err(Error::bad_request("Username is required"));
    }

    if data.password != "secret123" {
        return Err(Error::unauthorized());
    }

    Ok(Response::json(serde_json::json!({
        "message": "Login successful",
        "token": "abc123"
    }))?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .post("/login", login_handler)
        .get("/protected", || async {
            // This will return a 404 for demo
            Err(Error::not_found("/protected"))
        });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;

    Ok(())
}
```

## State Management

Share application state across handlers:

```rust
use ignitia::{Router, Server, Response, State};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

struct AppState {
    counter: AtomicU32,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState {
        counter: AtomicU32::new(0),
        name: "Ignitia App".to_string(),
    });

    let router = Router::new()
        .state(app_state)
        .get("/", |State(state): State<Arc<AppState>>| async move {
            let count = state.counter.fetch_add(1, Ordering::SeqCst);
            Ok(Response::json(serde_json::json!({
                "app_name": state.name,
                "visit_count": count + 1
            }))?)
        });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;

    Ok(())
}
```

## WebSocket Support

Enable real-time communication with WebSockets:

```toml
[dependencies]
ignitia = { version = "0.2.0", features = ["websocket"] }
```

```rust
use ignitia::{Router, Server, Response};
use ignitia::websocket::{WebSocketConnection, Message};

async fn websocket_handler(mut websocket: WebSocketConnection) -> ignitia::Result<()> {
    // Send welcome message
    websocket.send_text("Welcome to Ignitia WebSocket!".to_string()).await?;

    // Echo messages back
    while let Some(message) = websocket.recv().await {
        match message {
            Message::Text(text) => {
                websocket.send_text(format!("Echo: {}", text)).await?;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", || async {
            Ok(Response::html(r#"
                <script>
                    const ws = new WebSocket('ws://localhost:3000/ws');
                    ws.onmessage = (event) => console.log('Received:', event.data);
                    ws.onopen = () => ws.send('Hello from browser!');
                </script>
                <h1>WebSocket Example</h1>
                <p>Check browser console for messages.</p>
            "#))
        })
        .websocket_fn("/ws", websocket_handler);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;

    Ok(())
}
```

## Testing Your Routes

```bash
# Test basic route
curl http://127.0.0.1:3000/

# Test JSON API
curl -X POST http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "email": "alice@example.com"}'

# Test path parameters
curl http://127.0.0.1:3000/users/123

# Test query parameters
curl "http://127.0.0.1:3000/search?q=rust"
```

## Next Steps

Now that you have the basics, explore these guides for more advanced features:

- **[Routing Guide](ROUTING_GUIDE.md)** - Advanced routing patterns and nested routes
- **[Middleware Guide](MIDDLEWARE_GUIDE.md)** - Custom middleware and built-in options
- **[Extractors](EXTRACTORS.md)** - All available request extractors
- **[Error Handling](ERROR_HANDLING.md)** - Custom errors and error handling strategies
- **[WebSockets](WEB_SOCKETS.md)** - Real-time communication features
- **[File Uploads](FILE_UPLOADS.md)** - Handling multipart form data
- **[Server Config](SERVER_CONFIG.md)** - HTTPS, HTTP/2, and server optimization
- **[Security](SECURITY.md)** - Security best practices and middleware

## Performance Tips

- Use `Arc<State>` for shared state instead of cloning large objects
- Enable HTTP/2 for better performance: `ServerConfig::default().with_http2(true)`
- Use connection pooling for databases
- Consider middleware order for optimal performance

## Community & Support

- GitHub: [https://github.com/AarambhDevHub/ignitia](https://github.com/AarambhDevHub/ignitia)
- Documentation: [https://docs.rs/ignitia](https://docs.rs/ignitia)
- Examples: Check out the `examples/` directory in the repository

Happy coding with Ignitia! 🔥🚀
