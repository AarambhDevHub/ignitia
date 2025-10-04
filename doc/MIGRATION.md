# Migration Guide

This guide helps you migrate to Ignitia from other Rust web frameworks and navigate version upgrades within Ignitia itself.

## Table of Contents

- [Migrating from Other Frameworks](#migrating-from-other-frameworks)
  - [From Actix-web](#from-actix-web)
  - [From Axum](#from-axum)
  - [From Rocket](#from-rocket)
  - [From Warp](#from-warp)
- [Ignitia Version Migrations](#ignitia-version-migrations)
  - [v0.2.3 to v0.2.4](#v023-to-v024)
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

**After (Ignitia v0.2.4+):**
```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", hello)
        .get("/users/{id}", get_user);

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn hello() -> &'static str {
    "Hello World!"
}

async fn get_user(Path(id): Path<u32>) -> String {
    format!("User ID: {}", id)
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

**After (Ignitia v0.2.4+):**
```rust
use ignitia::prelude::*;

Router::new()
    .middleware(LoggerMiddleware::new())
    .middleware(CorsMiddleware::new().build()?)
```

#### JSON Handling

**Before (Actix-web):**
```rust
async fn create_user(user: web::Json<CreateUser>) -> Result<web::Json<User>> {
    let new_user = User {
        id: 1,
        name: user.name.clone(),
        email: user.email.clone(),
    };
    Ok(web::Json(new_user))
}
```

**After (Ignitia v0.2.4+):**
```rust
async fn create_user(Json(user): Json<CreateUser>) -> impl IntoResponse {
    let new_user = User {
        id: 1,
        name: user.name,
        email: user.email,
    };
    Response::json(new_user)  // Infallible in v0.2.4+
}
```

### From Axum

Ignitia v0.2.4+ adopts middleware patterns inspired by Axum, making migration even smoother.

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
        .route("/users/{id}", get(get_user))
        .route("/users", post(create_user));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**After (Ignitia v0.2.4+):**
```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", root)
        .get("/users/{id}", get_user)
        .post("/users", create_user);

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}
```

#### Middleware (Axum-compatible in v0.2.4)

**Axum:**
```rust
use axum::{
    middleware::{self, Next},
    response::Response,
    http::Request,
};

async fn my_middleware(req: Request<Body>, next: Next<Body>) -> Response {
    println!("Request: {}", req.uri());
    next.run(req).await
}

let app = Router::new()
    .route("/", get(handler))
    .layer(middleware::from_fn(my_middleware));
```

**Ignitia v0.2.4+ (Same Pattern!):**
```rust
use ignitia::prelude::*;
use ignitia::middleware::from_fn;

let my_middleware = from_fn(|req, next| async move {
    println!("Request: {}", req.uri.path());
    next.run(req).await
});

let router = Router::new()
    .get("/", handler)
    .middleware(my_middleware);
```

#### State Management

**Axum:**
```rust
async fn list_users(State(state): State<Arc<AppState>>) -> Json<Vec<User>> {
    // Use state
}

let app = Router::new()
    .route("/users", get(list_users))
    .with_state(shared_state);
```

**Ignitia v0.2.4+:**
```rust
async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    // Use state
    Response::json(users)
}

let router = Router::new()
    .state(app_state)
    .get("/users", list_users);
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

**After (Ignitia v0.2.4+):**
```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", index)
        .get("/users/{id}", get_user);

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn index() -> &'static str {
    "Hello, world!"
}

async fn get_user(Path(id): Path<u32>) -> String {
    format!("User ID: {}", id)
}
```

#### JSON Guards

**Before (Rocket):**
```rust
#[post("/users", data = "<user>")]
fn create_user(user: Json<User>) -> Json<User> {
    Json(user.into_inner())
}
```

**After (Ignitia v0.2.4+):**
```rust
async fn create_user(Json(user): Json<User>) -> impl IntoResponse {
    Response::json(user)  // Infallible
}
```

### From Warp

#### Filter-based to Router-based

**Before (Warp):**
```rust
use warp::Filter;

let hello = warp::path!("hello" / String)
    .map(|name| format!("Hello, {}!", name));

let routes = warp::get().and(hello);

warp::serve(routes)
    .run((, 3030))
    .await;
```

**After (Ignitia v0.2.4+):**
```rust
use ignitia::prelude::*;

let router = Router::new()
    .get("/hello/{name}", hello);

Server::new(router, "127.0.0.1:8080".parse()?).ignitia().await
```

## Ignitia Version Migrations

### v0.2.3 to v0.2.4

**This is a major update with breaking changes.** Ignitia v0.2.4 introduces a new middleware system inspired by Axum and several ergonomic improvements.

#### Breaking Changes Summary

1. **Middleware API**: Changed from `before/after` to `handle(req, next)`
2. **Handler Trait**: Renamed to `UniversalHandler`
3. **Response API**: `Response::json()` now returns `Response` directly (not `Result`)
4. **Removed Middleware**: `AuthMiddleware` and `ErrorHandlerMiddleware` removed
5. **New Feature**: `IntoResponse` trait for automatic error conversion

#### 1. Middleware API Migration

**Old (v0.2.3):**
```rust
use async_trait::async_trait;

#[async_trait]
impl Middleware for MyMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Modify request
        req.headers.insert("X-Custom", "value".parse().unwrap());
        Ok(())
    }

    async fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        // Modify response
        res.headers.insert("X-Processed", "true".parse().unwrap());
        Ok(())
    }
}
```

**New (v0.2.4+, Axum-inspired):**
```rust
impl Middleware for MyMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        // Modify request
        req.headers.insert("X-Custom", "value".parse().unwrap());

        // Call next middleware/handler
        let mut response = next.run(req).await;

        // Modify response
        response.headers.insert("X-Processed", "true".parse().unwrap());

        response
    }
}
```

**Using from_fn (Recommended for simple cases):**
```rust
use ignitia::middleware::from_fn;

let my_middleware = from_fn(|mut req, next| async move {
    req.headers.insert("X-Custom", "value".parse().unwrap());
    let mut response = next.run(req).await;
    response.headers.insert("X-Processed", "true".parse().unwrap());
    response
});

router.middleware(my_middleware)
```

#### 2. Response API Migration

**Old (v0.2.3):**
```rust
async fn handler() -> Result<Response> {
    let data = MyData { field: "value" };
    Ok(Response::json(data)?)  // Double Result
}
```

**New (v0.2.4+):**
```rust
async fn handler() -> Result<Response> {
    let data = MyData { field: "value" };
    Ok(Response::json(data))  // Single Result, infallible
}

// Or even simpler with IntoResponse:
async fn handler() -> impl IntoResponse {
    let data = MyData { field: "value" };
    Response::json(data)  // No Result wrapper needed!
}
```

#### 3. Error Handling Migration

**Old (v0.2.3):**
```rust
// Required ErrorHandlerMiddleware
let router = Router::new()
    .middleware(ErrorHandlerMiddleware::new())
    .get("/users/{id}", get_user);

async fn get_user(Path(id): Path<u64>) -> Result<Response> {
    let user = database::find_user(id)
        .await
        .map_err(|_| Error::not_found("User not found"))?;

    Ok(Response::json(user)?)
}
```

**New (v0.2.4+):**
```rust
// No error middleware needed - automatic via IntoResponse
let router = Router::new()
    .get("/users/{id}", get_user);

async fn get_user(Path(id): Path<u64>) -> Result<Response> {
    let user = database::find_user(id)
        .await
        .ok_or_else(|| Error::not_found("User not found"))?;

    Ok(Response::json(user))  // Errors auto-convert to responses
}
```

#### 4. Auth Middleware Migration

**Old (v0.2.3):**
```rust
let router = Router::new()
    .middleware(AuthMiddleware::new(secret_key))
    .get("/admin", admin_handler);
```

**New (v0.2.4+):**
```rust
use ignitia::middleware::from_fn;

let auth_middleware = from_fn(|req, next| async move {
    if let Some(token) = req.headers.get("Authorization") {
        if verify_token(token) {
            return next.run(req).await;
        }
    }
    Response::new().with_status(StatusCode::UNAUTHORIZED)
});

let router = Router::new()
    .middleware(auth_middleware)
    .get("/admin", admin_handler);
```

#### 5. Handler Signature Updates

**Old (v0.2.3):**
```rust
// Implementing Handler trait
impl Handler for MyHandler {
    async fn handle(&self, req: Request) -> Result<Response> {
        Ok(Response::json(data)?)
    }
}
```

**New (v0.2.4+):**
```rust
// UniversalHandler automatically implemented for functions
async fn my_handler() -> impl IntoResponse {
    Response::json(data)
}

// Or with extractors
async fn my_handler(
    Path(id): Path<u64>,
    Json(body): Json<CreateData>
) -> Result<impl IntoResponse> {
    Ok(Response::json(data))
}
```

#### 6. Complete Migration Example

**Before (v0.2.3):**
```rust
use ignitia::prelude::*;

#[async_trait]
struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        println!("Request: {}", req.uri);
        Ok(())
    }

    async fn after(&self, _req: &Request, res: &mut Response) -> Result<()> {
        println!("Response: {}", res.status);
        Ok(())
    }
}

async fn get_user(Path(id): Path<u64>) -> Result<Response> {
    let user = fetch_user(id).await?;
    Ok(Response::json(user)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .middleware(LoggingMiddleware)
        .middleware(ErrorHandlerMiddleware::new())
        .middleware(AuthMiddleware::new(secret))
        .get("/users/{id}", get_user);

    Server::new(router, "127.0.0.1:8080".parse()?).ignitia().await
}
```

**After (v0.2.4+):**
```rust
use ignitia::prelude::*;
use ignitia::middleware::from_fn;

async fn get_user(Path(id): Path<u64>) -> Result<Response> {
    let user = fetch_user(id).await?;
    Ok(Response::json(user))  // Infallible now!
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logging middleware using from_fn
    let logging = from_fn(|req, next| async move {
        println!("Request: {}", req.uri.path());
        let response = next.run(req).await;
        println!("Response: {}", response.status);
        response
    });

    // Auth middleware using from_fn
    let auth = from_fn(|req, next| async move {
        if let Some(token) = req.headers.get("Authorization") {
            if verify_token(token) {
                return next.run(req).await;
            }
        }
        Response::new().with_status(StatusCode::UNAUTHORIZED)
    });

    let router = Router::new()
        .middleware(logging)
        .middleware(auth)  // No ErrorHandlerMiddleware needed!
        .get("/users/{id}", get_user);

    Server::new(router, "127.0.0.1:8080".parse()?).ignitia().await
}
```

#### Migration Checklist for v0.2.4

- [ ] Update `Cargo.toml` to version `0.2.4`
- [ ] Replace `#[async_trait]` middleware with `handle(req, next)` pattern
- [ ] Remove `?` from all `Response::json()` calls
- [ ] Remove `ErrorHandlerMiddleware` usage
- [ ] Replace `AuthMiddleware` with `from_fn` implementation
- [ ] Update custom middleware to use `handle` method
- [ ] Change handler returns to use `impl IntoResponse` where beneficial
- [ ] Test all endpoints thoroughly
- [ ] Update tests to match new API

### v0.1.x to v0.2.x

#### Major Changes

1. **Improved HTTP/2 Support**: Enhanced configuration options
2. **WebSocket API Refinements**: Simplified handler creation
3. **Middleware System Updates**: More flexible middleware composition
4. **Performance Optimizations**: Better connection handling

#### Handler Function Signatures

**v0.1.x:**
```rust
async fn handler(req: Request) -> Result<Response> {
    Ok(Response::text("Hello"))
}
```

**v0.2.x:**
```rust
async fn handler() -> Result<Response> {
    Ok(Response::text("Hello"))
}

// Or with extractors
async fn handler(Json(data): Json<MyData>) -> Result<Response> {
    Ok(Response::json(data))
}
```

#### WebSocket Handler Updates

**v0.1.x:**
```rust
struct MyHandler;

impl WebSocketHandler for MyHandler {
    async fn handle_connection(&self, ws: WebSocketConnection) -> Result<()> {
        while let Some(msg) = ws.recv().await {
            ws.send(Message::text("Echo")).await?;
        }
        Ok(())
    }
}
```

**v0.2.x:**
```rust
let handler = websocket_handler(|ws| async move {
    while let Some(msg) = ws.recv().await {
        ws.send(Message::text("Echo")).await?;
    }
    Ok(())
});
```

#### Server Configuration

**v0.1.x:**
```rust
let server = Server::new(router, addr)
    .enable_http2(true)
    .run().await?;
```

**v0.2.x:**
```rust
let config = ServerConfig {
    http2: Http2Config {
        enabled: true,
        max_concurrent_streams: Some(1000),
        ..Default::default()
    },
    ..Default::default()
};

let server = Server::new(router, addr)
    .with_config(config)
    .ignitia().await?;
```

## Common Migration Patterns

### Error Handling (v0.2.4+)

**Using IntoResponse:**
```rust
async fn handler() -> Result<impl IntoResponse, Error> {
    let data = some_operation()
        .await
        .map_err(|e| Error::internal(format!("Operation failed: {}", e)))?;

    Ok(Response::json(data))
}

// Errors automatically convert to HTTP responses
async fn handler2() -> Result<Response> {
    Err(Error::not_found("Resource not found"))  // Auto-converts to 404
}
```

### Middleware Conversion (v0.2.4+)

**Simple Middleware:**
```rust
use ignitia::middleware::from_fn;

let simple = from_fn(|req, next| async move {
    println!("Before handler");
    let response = next.run(req).await;
    println!("After handler");
    response
});
```

**Complex Middleware:**
```rust
struct ComplexMiddleware {
    config: Arc<Config>,
}

impl Middleware for ComplexMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Pre-processing
        if !self.validate_request(&req) {
            return Response::new()
                .with_status(StatusCode::BAD_REQUEST);
        }

        // Execute handler
        let mut response = next.run(req).await;

        // Post-processing
        response.headers.insert("X-Custom", "value".parse().unwrap());

        response
    }
}
```

### State Management

```rust
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
    Ok(Response::json(data))
}
```

## Breaking Changes

### v0.2.4 Breaking Changes

1. **Middleware Trait**: Changed from `before/after` to `handle(req, next)`
2. **Response API**: `Response::json()` is now infallible
3. **Handler Trait**: Renamed to `UniversalHandler`
4. **Removed Middleware**: `AuthMiddleware` and `ErrorHandlerMiddleware`
5. **Error Handling**: Automatic via `IntoResponse` trait

### v0.2.3 Breaking Changes

1. **Handler Signatures**: Direct request parameter removed
2. **WebSocket API**: Simplified handler creation
3. **Middleware Trait**: Updated method signatures
4. **Error Types**: Consolidated error system

## Migration Checklist

### Pre-Migration

- [ ] Review current framework usage patterns
- [ ] Identify custom middleware and handlers
- [ ] Document current API endpoints
- [ ] Set up test environment
- [ ] Backup current codebase
- [ ] Read CHANGELOG for your target version

### During Migration (v0.2.4)

- [ ] Update `Cargo.toml` to `ignitia = "0.2.4"`
- [ ] Replace all `#[async_trait]` middleware implementations
- [ ] Update middleware from `before/after` to `handle(req, next)`
- [ ] Remove `?` from `Response::json()` calls
- [ ] Remove `ErrorHandlerMiddleware` usage
- [ ] Replace `AuthMiddleware` with `from_fn`
- [ ] Update handler return types to use `impl IntoResponse`
- [ ] Convert custom middleware to new pattern
- [ ] Update tests for new API

### Post-Migration

- [ ] Run comprehensive tests
- [ ] Performance benchmarking
- [ ] Update documentation
- [ ] Deploy to staging environment
- [ ] Monitor for issues
- [ ] Update CI/CD pipelines

## Troubleshooting

### Common Issues (v0.2.4)

#### "Method `before` not found" Error

**Issue**: Old middleware implementation.

**Solution**: Update to new `handle` method:
```rust
impl Middleware for MyMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Your logic here
        next.run(req).await
    }
}
```

#### "Expected Result, found Response" Error

**Issue**: `Response::json()` no longer returns `Result`.

**Solution**: Remove the `?` operator:
```rust
// Old
Ok(Response::json(data)?)

// New
Ok(Response::json(data))
```

#### "ErrorHandlerMiddleware not found" Error

**Issue**: Middleware removed in v0.2.4.

**Solution**: Remove it - errors now auto-convert via `IntoResponse`:
```rust
// Old
router.middleware(ErrorHandlerMiddleware::new())

// New - just remove it!
router  // Errors handle automatically
```

#### "AuthMiddleware not found" Error

**Issue**: Middleware removed in v0.2.4.

**Solution**: Use `from_fn` to create custom auth:
```rust
let auth = from_fn(|req, next| async move {
    if let Some(token) = req.headers.get("Authorization") {
        if verify_token(token) {
            return next.run(req).await;
        }
    }
    Response::new().with_status(StatusCode::UNAUTHORIZED)
});

router.middleware(auth)
```

#### Middleware Not Working

**Issue**: Middleware not being called.

**Solution**: Ensure proper `handle` implementation:
```rust
impl Middleware for MyMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Must call next.run() for handler to execute
        next.run(req).await
    }
}
```

#### State Not Available

**Issue**: State extractor failing in handlers.

**Solution**: Ensure state is properly registered:
```rust
let router = Router::new()
    .state(my_state)  // Must register before routes
    .get("/endpoint", handler);
```

### Performance Considerations

1. **Connection Pooling**: Use Ignitia's state management for database connections
2. **Middleware Order**: Optimize for early rejection (auth → validation → handler)
3. **HTTP/2 Configuration**: Tune settings for your use case
4. **from_fn Performance**: Zero-cost abstraction, use freely

### Getting Help

- Check the [documentation](./README.md)
- Review [CHANGELOG.md](./CHANGELOG.md) for version details
- Review [examples](./EXAMPLES.md)
- Check [MIDDLEWARE_GUIDE.md](./MIDDLEWARE_GUIDE.md)
- Submit issues on GitHub
- Join community discussions

***

This migration guide covers all major version transitions in Ignitia. For specific use cases not covered here, please refer to the framework documentation or reach out to the community for assistance.
