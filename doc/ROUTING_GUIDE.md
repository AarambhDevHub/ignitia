# Ignitia Routing Guide 🔥

A complete guide to routing in Ignitia - covering everything from basic route definition to advanced patterns and performance optimization.

## Table of Contents

- [Basic Concepts](#basic-concepts)
- [Route Definition](#route-definition)
- [HTTP Methods](#http-methods)
- [Path Parameters](#path-parameters)
- [Query Parameters](#query-parameters)
- [Route Groups and Nesting](#route-groups-and-nesting)
- [Router Merging](#router-merging)
- [Route Middleware](#route-middleware)
- [State Management](#state-management)
- [Advanced Routing Patterns](#advanced-routing-patterns)
- [WebSocket Routing](#websocket-routing)
- [Performance Considerations](#performance-considerations)
- [Best Practices](#best-practices)

## Basic Concepts

Ignitia's routing system is built around the `Router` struct, which uses a high-performance **radix tree** for route matching. Routes are matched based on specificity, with more specific routes taking precedence over general ones.

### Route Compilation

Ignitia compiles routes at startup for optimal runtime performance:

```rust
use ignitia::{Router, Response};

let router = Router::new()
    .get("/", || async { "Home" })
    .get("/about", || async { "About" });
```

### Route Matching Priority

Routes are automatically sorted by specificity:

1. **Exact static matches** (e.g., `/users/profile`)
2. **Parameterized routes** (e.g., `/users/{id}`)
3. **Wildcard routes** (e.g., `/files/{*path}`)

## Radix Tree Routing

Ignitia uses a **Radix Tree** (compressed trie) data structure for ultra-fast route matching. This is the only routing mode starting from version 0.2.4.

### How It Works

```rust
use ignitia::Router;

let router = Router::new()
    .get("/users/{id}", get_user)
    .get("/users/{id}/posts", get_user_posts)
    .get("/api/v1/health", health_check);
```

**Advantages:**
- **Ultra-fast matching**: O(log n) lookup time
- **Memory efficient**: Shared path prefixes reduce memory usage
- **Zero regex compilation**: No regex overhead during startup
- **Better cache locality**: Tree structure improves CPU cache efficiency
- **Handles complex patterns**: Efficiently manages overlapping routes

**Performance Characteristics:**
- **Lookup Time**: O(log n) where n is the number of routes
- **Memory Usage**: ~50% less than regex-based routing
- **Startup Time**: Instant (no regex compilation)
- **Throughput**: 51,000+ requests/second (benchmarked)

## Route Definition

### Basic Route Registration

```rust
use ignitia::{Router, Response};

async fn home_handler() -> &'static str {
    "Welcome to Ignitia! 🔥"
}

async fn about_handler() -> &'static str {
    "<h1>About Us</h1>"
}

let router = Router::new()
    .get("/", home_handler)
    .get("/about", about_handler);
```

### Inline Route Handlers

```rust
let router = Router::new()
    .get("/", || async { "Home" })
    .post("/submit", || async {
        Json(serde_json::json!({
            "status": "received"
        }))
    });
```

### Route with Request Access

```rust
use ignitia::Request;

let router = Router::new()
    .get("/custom", |req: Request| async move {
        let user_agent = req.headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Unknown");
        format!("User-Agent: {}", user_agent)
    });
```

## HTTP Methods

Ignitia supports all standard HTTP methods with dedicated builder methods:

### GET Routes

```rust
let router = Router::new()
    .get("/users", list_users)
    .get("/users/{id}", get_user)
    .get("/search", search_users)
    .get("/health", || async {
        Json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    });
```

### POST Routes

```rust
use ignitia::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    role: Option<String>,
}

#[derive(Serialize)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
    role: String,
    created_at: String,
}

let router = Router::new()
    .post("/users", |Json(user): Json<CreateUser>| async move {
        // Validate input
        if user.name.is_empty() || user.email.is_empty() {
            return Err(ignitia::Error::BadRequest(
                "Name and email are required".to_string()
            ));
        }

        // Create user logic here
        let new_user = UserResponse {
            id: 1,
            name: user.name,
            email: user.email,
            role: user.role.unwrap_or("user".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(Json(new_user))
    });
```

### PUT and PATCH Routes

```rust
#[derive(Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
    role: Option<String>,
}

#[derive(Deserialize)]
struct UserPatch {
    name: Option<String>,
    email: Option<String>,
}

let router = Router::new()
    .put("/users/{id}", |Path(id): Path<u32>, Json(user): Json<UpdateUser>| async move {
        // Full update logic
        Json(serde_json::json!({
            "id": id,
            "updated": true,
            "type": "full_update"
        }))
    })
    .patch("/users/{id}", |Path(id): Path<u32>, Json(patch): Json<UserPatch>| async move {
        // Partial update logic
        Json(serde_json::json!({
            "id": id,
            "updated": true,
            "type": "partial_update"
        }))
    });
```

### DELETE Routes

```rust
use http::StatusCode;

let router = Router::new()
    .delete("/users/{id}", |Path(id): Path<u32>| async move {
        // Delete logic here
        StatusCode::NO_CONTENT
    })
    .delete("/users/{id}/sessions", |Path(id): Path<u32>| async move {
        Json(serde_json::json!({
            "message": "All sessions cleared",
            "user_id": id
        }))
    });
```

### ANY Method

Register a handler for all HTTP methods:

```rust
let router = Router::new()
    .any("/health", || async { "OK" });
```

## Path Parameters

### Single Parameters

```rust
use ignitia::Path;

// Extract single parameter
let router = Router::new()
    .get("/users/{id}", |Path(id): Path<u32>| async move {
        if id == 0 {
            return Err(ignitia::Error::BadRequest("Invalid user ID".into()));
        }
        format!("User ID: {}", id)
    })
    .get("/posts/{slug}", |Path(slug): Path<String>| async move {
        Json(serde_json::json!({
            "slug": slug
        }))
    });
```

### Multiple Parameters

```rust
// Extract multiple parameters as tuple
let router = Router::new()
    .get("/users/{user_id}/posts/{post_id}",
        |Path((user_id, post_id)): Path<(u32, u32)>| async move {
            Json(serde_json::json!({
                "user_id": user_id,
                "post_id": post_id
            }))
        });
```

### Named Parameter Extraction

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPostParams {
    user_id: u32,
    post_id: u32,
}

let router = Router::new()
    .get("/users/{user_id}/posts/{post_id}",
        |Path(params): Path<UserPostParams>| async move {
            Json(params)
        });
```

### Wildcard Parameters

```rust
// Catch-all routes with wildcard parameters
let router = Router::new()
    .get("/files/{*path}", |Path(path): Path<String>| async move {
        // Secure file serving with path validation
        let safe_path = path.replace("..", "");
        Json(serde_json::json!({
            "path": safe_path
        }))
    });
```

## Query Parameters

### Basic Query Extraction

```rust
use ignitia::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    category: Option<String>,
    sort: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
}

let router = Router::new()
    .get("/search", |Query(params): Query<SearchParams>| async move {
        if params.q.is_empty() {
            return Err(ignitia::Error::BadRequest("Query required".into()));
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(10).min(100);

        Json(serde_json::json!({
            "query": params.q,
            "page": page,
            "per_page": per_page
        }))
    });
```

### Advanced Query Parameters

```rust
#[derive(Deserialize)]
struct FilterParams {
    tags: Vec<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
    in_stock: Option<bool>,
}

let router = Router::new()
    .get("/products", |Query(filters): Query<FilterParams>| async move {
        // Validate price range
        if let (Some(min), Some(max)) = (filters.min_price, filters.max_price) {
            if min > max {
                return Err(ignitia::Error::BadRequest(
                    "min_price cannot exceed max_price".into()
                ));
            }
        }

        Json(serde_json::json!({
            "filters": filters
        }))
    });
```

## Route Groups and Nesting

### Basic Route Grouping

```rust
// Create API v1 routes
let api_v1 = Router::new()
    .get("/users", list_users_v1)
    .post("/users", create_user_v1)
    .get("/users/{id}", get_user_v1);

// Create API v2 routes
let api_v2 = Router::new()
    .get("/users", list_users_v2)
    .post("/users", create_user_v2)
    .get("/users/{id}", get_user_v2);

// Main router with nested routes
let router = Router::new()
    .get("/", home_page)
    .nest("/api/v1", api_v1)
    .nest("/api/v2", api_v2);
```

### Complex Nested Structure

```rust
// Blog routes
let blog_routes = Router::new()
    .get("/", list_posts)
    .get("/{slug}", get_post_by_slug);

// User routes
let user_routes = Router::new()
    .get("/", list_users)
    .post("/", create_user)
    .get("/{id}", get_user);

// Main application
let app = Router::new()
    .nest("/blog", blog_routes)
    .nest("/users", user_routes);
```

## Router Merging

Combine multiple routers into one:

```rust
let user_router = Router::new()
    .get("/users", list_users)
    .post("/users", create_user);

let post_router = Router::new()
    .get("/posts", list_posts)
    .post("/posts", create_post);

let main_router = Router::new()
    .get("/", home_page)
    .merge(user_router)
    .merge(post_router);
```

### Merging with State and Middleware

```rust
#[derive(Clone)]
struct DatabaseState {
    pool: PgPool,
}

let db_router = Router::new()
    .state(DatabaseState { pool: db_pool })
    .middleware(DatabaseMiddleware::new())
    .get("/db/users", get_users_from_db);

let app_router = Router::new()
    .merge(db_router);
```

## Route Middleware

### Per-Route Middleware

```rust
use ignitia::LayeredHandler;

let protected_handler = LayeredHandler::new(secret_handler)
    .layer(AuthMiddleware::new())
    .layer(LoggerMiddleware::new());

let router = Router::new()
    .route_with_layered("/secret", Method::GET, protected_handler);
```

### Global Middleware

```rust
let router = Router::new()
    .middleware(LoggerMiddleware::new())
    .middleware(CorsMiddleware::new())
    .get("/users", list_users)
    .get("/posts", list_posts);
```

## State Management

### Application State

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

async fn get_user(
    Path(id): Path<u32>,
    State(state): State<AppState>
) -> Result<Json<User>> {
    // Use state
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id as i32)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(user))
}

let app_state = AppState {
    db: db_pool,
    cache: Arc::new(RwLock::new(HashMap::new())),
};

let router = Router::new()
    .state(app_state)
    .get("/users/{id}", get_user);
```

## Advanced Routing Patterns

### Custom 404 Handler

```rust
let router = Router::new()
    .get("/users", list_users)
    .not_found(|| async {
        (StatusCode::NOT_FOUND, "Page not found")
    });
```

### API Versioning

```rust
// URL-based versioning
let router = Router::new()
    .nest("/api/v1", build_v1_routes())
    .nest("/api/v2", build_v2_routes());

// Header-based versioning
let router = Router::new()
    .get("/api/users", |headers: HeaderMap| async move {
        let version = headers.get("API-Version")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("v1");

        match version {
            "v1" => users_v1_handler().await,
            "v2" => users_v2_handler().await,
            _ => Err(ignitia::Error::BadRequest("Unsupported version".into())),
        }
    });
```

## WebSocket Routing

With the `websocket` feature enabled:

```rust
#[cfg(feature = "websocket")]
use ignitia::websocket::{WebSocketConnection, Message};

#[cfg(feature = "websocket")]
let router = Router::new()
    .websocket("/ws", |mut ws: WebSocketConnection| async move {
        while let Some(msg) = ws.recv().await {
            match msg {
                Message::Text(text) => {
                    ws.send_text(format!("Echo: {}", text)).await?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    });
```

## Performance Considerations

### Router Optimization

Ignitia's radix tree router provides:
- **O(log n) lookup time**: Fast route matching even with thousands of routes
- **Minimal allocations**: Zero-allocation route matching
- **Cache-friendly**: Tree structure improves CPU cache efficiency

### Benchmarks (wrk -c100 -d30s)

```
Throughput:    51,574 req/sec
Latency:       1.90ms avg, 10.60ms max
Transfer:      7.97 MB/sec
```

### Memory Efficiency

```rust
// Use Arc for shared data
#[derive(Clone)]
struct SharedData {
    config: Arc<AppConfig>,
    templates: Arc<HashMap<String, String>>,
}

let router = Router::new()
    .state(SharedData {
        config: Arc::new(config),
        templates: Arc::new(templates),
    });
```

## Best Practices

### 1. Organize Routes by Feature

```rust
mod user_routes {
    pub fn routes() -> Router {
        Router::new()
            .get("/", list_users)
            .post("/", create_user)
            .get("/{id}", get_user)
    }
}

fn build_app() -> Router {
    Router::new()
        .nest("/users", user_routes::routes())
}
```

### 2. Validate Input

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 2, max = 100))]
    name: String,

    #[validate(email)]
    email: String,
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Json<User>> {
    user.validate()?;
    // Create user logic
}
```

### 3. Use Proper Status Codes

```rust
let router = Router::new()
    .post("/users", || async {
        (StatusCode::CREATED, Json(new_user))
    })
    .delete("/users/{id}", || async {
        StatusCode::NO_CONTENT
    });
```

### 4. Implement Health Checks

```rust
let router = Router::new()
    .get("/health", || async {
        Json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now()
        }))
    });
```

### 5. Production Configuration

```rust
fn create_production_router() -> Router {
    Router::new()
        .middleware(RequestIdMiddleware::new())
        .middleware(LoggerMiddleware::new())
        .middleware(SecurityMiddleware::new())
        .middleware(RateLimitingMiddleware::new())
        .get("/health", health_check)
        .nest("/api/v1", build_api_routes())
}
```

---

**🔥 Ready to ignite your web development with high-performance routing? Start building with Ignitia today!**

For more information, visit the [API Documentation](https://docs.rs/ignitia) and [Examples](https://github.com/AarambhDevHub/ignitia/tree/main/examples).
