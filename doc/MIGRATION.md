# Migration Guide

This guide helps you migrate to Ignitia from other Rust web frameworks and navigate version upgrades within Ignitia itself.

## Table of Contents

- [Migrating from Other Frameworks](#migrating-from-other-frameworks)
  - [From Actix-web](#from-actix-web)
  - [From Axum](#from-axum)
  - [From Rocket](#from-rocket)
  - [From Warp](#from-warp)
- [Ignitia Version Migrations](#ignitia-version-migrations)
  - [v0.1.x to v0.2.x](#v01x-to-v02x)
- [Common Migration Patterns](#common-migration-patterns)
- [Breaking Changes](#breaking-changes)
- [Migration Checklist](#migration-checklist)
- [Troubleshooting](#troubleshooting)

## Migrating from Other Frameworks

### From Actix-web

Ignitia shares many concepts with Actix-web, making migration relatively straightforward.

#### Basic Server Setup

**Before (Actix-web):**
```rust
use actix_web::{web, App, HttpServer, Result};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/users/{id}", web::get().to(get_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn hello() -> Result<String> {
    Ok("Hello World!".to_string())
}

async fn get_user(path: web::Path<u32>) -> Result<String> {
    Ok(format!("User ID: {}", path.into_inner()))
}
```

**After (Ignitia):**
```rust
use ignitia::{Router, Server, Response, Path};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", hello)
        .get("/users/:id", get_user);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn hello() -> ignitia::Result<Response> {
    Ok(Response::text("Hello World!"))
}

async fn get_user(Path(id): Path<u32>) -> ignitia::Result<Response> {
    Ok(Response::text(format!("User ID: {}", id)))
}
```

#### Middleware Migration

**Before (Actix-web):**
```rust
use actix_web::{middleware::Logger, App};

App::new()
    .wrap(Logger::default())
    .wrap(actix_cors::Cors::default())
```

**After (Ignitia):**
```rust
use ignitia::{Router, LoggerMiddleware, CorsMiddleware};

Router::new()
    .middleware(LoggerMiddleware)
    .middleware(CorsMiddleware::new().allow_any_origin())
```

#### JSON Handling

**Before (Actix-web):**
```rust
use actix_web::{web, Result};
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

async fn create_user(user: web::Json<CreateUser>) -> Result<web::Json<User>> {
    let new_user = User {
        id: 1,
        name: user.name.clone(),
        email: user.email.clone(),
    };
    Ok(web::Json(new_user))
}
```

**After (Ignitia):**
```rust
use ignitia::{Json, Response, Result};
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

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    let new_user = User {
        id: 1,
        name: user.name,
        email: user.email,
    };
    Response::json(new_user)
}
```

### From Axum

#### Basic Router Setup

**Before (Axum):**
```rust
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**After (Ignitia):**
```rust
use ignitia::{Router, Server, Response, Path, Json};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", root)
        .get("/users/:id", get_user)
        .post("/users", create_user);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}
```

#### State Management

**Before (Axum):**
```rust
use axum::{extract::State, Extension};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Database,
}

let shared_state = Arc::new(AppState {
    db: Database::new(),
});

let app = Router::new()
    .route("/users", get(list_users))
    .with_state(shared_state);

async fn list_users(State(state): State<Arc<AppState>>) -> Json<Vec<User>> {
    // Use state.db
}
```

**After (Ignitia):**
```rust
use ignitia::{Router, State, Json};

#[derive(Clone)]
struct AppState {
    db: Database,
}

let app_state = AppState {
    db: Database::new(),
};

let router = Router::new()
    .state(app_state)
    .get("/users", list_users);

async fn list_users(State(state): State<AppState>) -> ignitia::Result<Response> {
    // Use state.db
    Response::json(users)
}
```

### From Rocket

#### Route Definitions

**Before (Rocket):**
```rust
#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/users/<id>")]
fn get_user(id: u32) -> String {
    format!("User ID: {}", id)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, get_user])
}
```

**After (Ignitia):**
```rust
use ignitia::{Router, Server, Response, Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", index)
        .get("/users/:id", get_user);

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn index() -> ignitia::Result<Response> {
    Ok(Response::text("Hello, world!"))
}

async fn get_user(Path(id): Path<u32>) -> ignitia::Result<Response> {
    Ok(Response::text(format!("User ID: {}", id)))
}
```

#### JSON Guards

**Before (Rocket):**
```rust
use rocket::serde::{Deserialize, Serialize, json::Json};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct User {
    name: String,
    email: String,
}

#[post("/users", data = "<user>")]
fn create_user(user: Json<User>) -> Json<User> {
    Json(user.into_inner())
}
```

**After (Ignitia):**
```rust
use ignitia::{Json, Response, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    email: String,
}

async fn create_user(Json(user): Json<User>) -> Result<Response> {
    Response::json(user)
}
```

### From Warp

#### Filter-based to Router-based

**Before (Warp):**
```rust
use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let routes = warp::get()
        .and(hello);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

**After (Ignitia):**
```rust
use ignitia::{Router, Server, Response, Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/hello/:name", hello);

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn hello(Path(name): Path<String>) -> ignitia::Result<Response> {
    Ok(Response::text(format!("Hello, {}!", name)))
}
```

## Ignitia Version Migrations

### v0.1.x to v0.2.x

#### Major Changes

1. **Improved HTTP/2 Support**: Enhanced configuration options
2. **WebSocket API Refinements**: Simplified handler creation
3. **Middleware System Updates**: More flexible middleware composition
4. **Performance Optimizations**: Better connection handling

#### Handler Function Signatures

**v0.1.x:**
```rust
use ignitia::{Request, Response, Result};

async fn handler(req: Request) -> Result<Response> {
    Ok(Response::text("Hello"))
}
```

**v0.2.x:**
```rust
use ignitia::{Response, Result};

async fn handler() -> Result<Response> {
    Ok(Response::text("Hello"))
}

// Or with extractors
async fn handler_with_json(Json(data): Json<MyData>) -> Result<Response> {
    Ok(Response::json(data)?)
}
```

#### WebSocket Handler Updates

**v0.1.x:**
```rust
use ignitia::websocket::{WebSocketHandler, WebSocketConnection, Message};

struct MyHandler;

impl WebSocketHandler for MyHandler {
    async fn handle_connection(&self, ws: WebSocketConnection) -> Result<()> {
        // Manual message loop
        while let Some(msg) = ws.recv().await {
            ws.send(Message::text("Echo")).await?;
        }
        Ok(())
    }
}
```

**v0.2.x:**
```rust
use ignitia::websocket::{websocket_handler, WebSocketConnection, Message};

let handler = websocket_handler(|ws: WebSocketConnection| async move {
    while let Some(msg) = ws.recv().await {
        ws.send(Message::text("Echo")).await?;
    }
    Ok(())
});

// Or use the message handler for individual messages
let message_handler = websocket_message_handler(|ws: WebSocketConnection, msg: Message| async move {
    ws.send(Message::text("Echo")).await
});
```

#### Server Configuration

**v0.1.x:**
```rust
use ignitia::{Server, Router};

let server = Server::new(router, addr)
    .enable_http2(true)
    .run().await?;
```

**v0.2.x:**
```rust
use ignitia::{Server, Router, ServerConfig, Http2Config};

let config = ServerConfig {
    http2: Http2Config {
        enabled: true,
        max_concurrent_streams: Some(1000),
        initial_connection_window_size: Some(1024 * 1024),
        ..Default::default()
    },
    ..Default::default()
};

let server = Server::new(router, addr)
    .with_config(config)
    .ignitia().await?;
```

## Common Migration Patterns

### Error Handling

Most frameworks use custom error types. Ignitia provides comprehensive error handling:

```rust
use ignitia::{Error, Result, Response};

// Convert from other error types
async fn handler() -> Result<Response> {
    let data = some_operation()
        .map_err(|e| Error::internal(format!("Operation failed: {}", e)))?;

    Ok(Response::json(data)?)
}

// Custom error types
use ignitia::define_error;

define_error! {
    MyError {
        NotFound(http::StatusCode::NOT_FOUND, "not_found"),
        Validation(http::StatusCode::BAD_REQUEST, "validation_error"),
    }
}

async fn handler() -> Result<Response> {
    Err(MyError::NotFound("User not found".into()).into())
}
```

### Middleware Conversion

Convert middleware from other frameworks:

```rust
use ignitia::{Middleware, Request, Response, Result};

struct CustomMiddleware {
    config: String,
}

#[async_trait::async_trait]
impl Middleware for CustomMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Process request
        Ok(())
    }

    async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        // Process response
        Ok(())
    }
}
```

### State Management

Migrate application state:

```rust
use ignitia::{Router, State};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    cache: Arc<RwLock<Cache>>,
}

let app_state = AppState {
    db: Arc::new(Database::new()),
    cache: Arc::new(RwLock::new(Cache::new())),
};

let router = Router::new()
    .state(app_state)
    .get("/data", get_data);

async fn get_data(State(state): State<AppState>) -> Result<Response> {
    let data = state.db.fetch_data().await?;
    Response::json(data)
}
```

## Breaking Changes

### v0.2.0 Breaking Changes

1. **Handler Signatures**: Direct request parameter removed in favor of extractors
2. **WebSocket API**: Simplified handler creation functions
3. **Middleware Trait**: Updated method signatures for better performance
4. **Error Types**: Consolidated error handling system

### Migration Steps

1. **Update Dependencies**:
   ```toml
   [dependencies]
   ignitia = "0.2.0"
   ```

2. **Update Handler Functions**:
   - Remove `Request` parameter
   - Use extractors for request data
   - Update return types if needed

3. **Update WebSocket Handlers**:
   - Use new handler creation functions
   - Update message handling patterns

4. **Update Middleware**:
   - Implement new middleware trait methods
   - Update method signatures

## Migration Checklist

### Pre-Migration

- [ ] Review current framework usage patterns
- [ ] Identify custom middleware and handlers
- [ ] Document current API endpoints
- [ ] Set up test environment
- [ ] Backup current codebase

### During Migration

- [ ] Update `Cargo.toml` dependencies
- [ ] Migrate basic server setup
- [ ] Convert route definitions
- [ ] Update handler signatures
- [ ] Migrate middleware
- [ ] Convert state management
- [ ] Update error handling
- [ ] Migrate WebSocket handlers (if applicable)
- [ ] Update configuration

### Post-Migration

- [ ] Run comprehensive tests
- [ ] Performance benchmarking
- [ ] Update documentation
- [ ] Deploy to staging environment
- [ ] Monitor for issues
- [ ] Update CI/CD pipelines

## Troubleshooting

### Common Issues

#### Handler Compilation Errors

**Issue**: Handler functions not compiling with new signatures.

**Solution**: Use extractors instead of direct `Request` parameter:

```rust
// Instead of
async fn handler(req: Request) -> Result<Response> {
    let body = req.body;
    // ...
}

// Use
async fn handler(Body(body): Body) -> Result<Response> {
    // ...
}
```

#### Middleware Not Working

**Issue**: Custom middleware not being called.

**Solution**: Ensure proper trait implementation:

```rust
use async_trait::async_trait;

#[async_trait]
impl Middleware for MyMiddleware {
    // Implement required methods
}
```

#### WebSocket Connection Issues

**Issue**: WebSocket handlers not receiving messages.

**Solution**: Use the correct handler pattern:

```rust
let handler = websocket_handler(|ws| async move {
    while let Some(msg) = ws.recv().await {
        // Handle message
    }
    Ok(())
});
```

#### State Not Available

**Issue**: State extractor failing in handlers.

**Solution**: Ensure state is properly registered:

```rust
let router = Router::new()
    .state(my_state)  // Register state
    .get("/endpoint", handler);
```

### Performance Considerations

1. **Connection Pooling**: Migrate database connections to use Ignitia's state management
2. **Middleware Order**: Optimize middleware order for better performance
3. **HTTP/2 Configuration**: Tune HTTP/2 settings for your use case
4. **Memory Usage**: Review memory allocations in handlers

### Getting Help

- Check the [documentation](./README.md)
- Review [examples](./EXAMPLES.md)
- Submit issues on GitHub
- Join community discussions

***

This migration guide should help you transition smoothly to Ignitia. For specific use cases not covered here, please refer to the framework documentation or reach out to the community for assistance.
