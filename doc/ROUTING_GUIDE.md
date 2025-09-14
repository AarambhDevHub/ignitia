# Routing Guide

A comprehensive guide to routing in Ignitia - covering everything from basic route definition to advanced patterns and performance optimization.

## Table of Contents

- [Basic Concepts](#basic-concepts)
- [Route Definition](#route-definition)
- [HTTP Methods](#http-methods)
- [Path Parameters](#path-parameters)
- [Query Parameters](#query-parameters)
- [Route Groups and Nesting](#route-groups-and-nesting)
- [Route Middleware](#route-middleware)
- [State Management](#state-management)
- [Advanced Routing Patterns](#advanced-routing-patterns)
- [WebSocket Routing](#websocket-routing)
- [Performance Considerations](#performance-considerations)
- [Best Practices](#best-practices)

## Basic Concepts

Ignitia's routing system is built around the `Router` struct, which uses a compile-time optimized matching system for maximum performance. Routes are matched in order of specificity, with more specific routes taking precedence.

### Route Compilation

Ignitia compiles routes at startup for optimal runtime performance:

```rust
use ignitia::{Router, Response};

let router = Router::new()
    .get("/", || async { Ok(Response::text("Home")) })
    .get("/about", || async { Ok(Response::text("About")) });
```

### Route Matching Priority

Routes are automatically sorted by specificity:
1. **Exact matches** (e.g., `/users/profile`)
2. **Parameterized routes** (e.g., `/users/:id`)
3. **Wildcard routes** (e.g., `/files/*path`)

## Route Definition

### Basic Route Registration

```rust
use ignitia::{Router, Response, Result};

async fn home_handler() -> Result<Response> {
    Ok(Response::text("Welcome to Ignitia! 🔥"))
}

async fn about_handler() -> Result<Response> {
    Ok(Response::html("<h1>About Us</h1>"))
}

let router = Router::new()
    .get("/", home_handler)
    .get("/about", about_handler);
```

### Inline Route Handlers

```rust
let router = Router::new()
    .get("/", || async { Ok(Response::text("Home")) })
    .post("/submit", || async {
        Ok(Response::json(serde_json::json!({
            "status": "received"
        }))?)
    });
```

## HTTP Methods

Ignitia supports all standard HTTP methods with dedicated builder methods:

### GET Routes

```rust
let router = Router::new()
    .get("/users", list_users)
    .get("/users/:id", get_user)
    .get("/search", search_users);
```

### POST Routes

```rust
use ignitia::{Json, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
}

let router = Router::new()
    .post("/users", |Json(user): Json<CreateUser>| async move {
        // Create user logic here
        let new_user = UserResponse {
            id: 1,
            name: user.name,
            email: user.email,
        };
        Ok(Response::json(new_user)?)
    });
```

### PUT and PATCH Routes

```rust
let router = Router::new()
    .put("/users/:id", |Path(id): Path<u32>, Json(user): Json<UpdateUser>| async move {
        // Full update logic
        Ok(Response::json(updated_user)?)
    })
    .patch("/users/:id", |Path(id): Path<u32>, Json(patch): Json<UserPatch>| async move {
        // Partial update logic
        Ok(Response::json(patched_user)?)
    });
```

### DELETE Routes

```rust
let router = Router::new()
    .delete("/users/:id", |Path(id): Path<u32>| async move {
        // Delete user logic
        Ok(Response::new(StatusCode::NO_CONTENT))
    });
```

### HEAD and OPTIONS Routes

```rust
let router = Router::new()
    .head("/users/:id", |Path(id): Path<u32>| async move {
        // Return headers only
        Ok(Response::new(StatusCode::OK))
    })
    .options("/users", || async {
        Ok(Response::new(StatusCode::OK)
            .header("Allow", "GET, POST, PUT, DELETE"))
    });
```

## Path Parameters

### Single Parameters

```rust
use ignitia::Path;

// Extract single parameter
let router = Router::new()
    .get("/users/:id", |Path(id): Path<u32>| async move {
        Ok(Response::text(format!("User ID: {}", id)))
    });
```

### Multiple Parameters

```rust
// Extract multiple parameters as tuple
let router = Router::new()
    .get("/users/:user_id/posts/:post_id",
        |Path((user_id, post_id)): Path<(u32, u32)>| async move {
            Ok(Response::text(format!("User {} Post {}", user_id, post_id)))
        });
```

### Named Parameter Extraction

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPost {
    user_id: u32,
    post_id: u32,
}

let router = Router::new()
    .get("/users/:user_id/posts/:post_id",
        |Path(params): Path<UserPost>| async move {
            Ok(Response::json(params)?)
        });
```

### Optional Parameters with Defaults

```rust
#[derive(Deserialize)]
struct PaginationParams {
    page: Option<u32>,
    limit: Option<u32>,
}

let router = Router::new()
    .get("/users/:id/posts", |Path(id): Path<u32>, Query(params): Query<PaginationParams>| async move {
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(10);

        Ok(Response::json(serde_json::json!({
            "user_id": id,
            "page": page,
            "limit": limit
        }))?)
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
}

let router = Router::new()
    .get("/search", |Query(params): Query<SearchParams>| async move {
        Ok(Response::json(serde_json::json!({
            "query": params.q,
            "category": params.category,
            "sort": params.sort.unwrap_or("relevance".to_string())
        }))?)
    });
```

### Complex Query Parameters

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
        // Filter products based on parameters
        Ok(Response::json(filtered_products)?)
    });
```

## Route Groups and Nesting

### Basic Route Grouping

```rust
// Create API v1 routes
let api_v1 = Router::new()
    .get("/users", list_users_v1)
    .post("/users", create_user_v1)
    .get("/users/:id", get_user_v1);

// Create API v2 routes
let api_v2 = Router::new()
    .get("/users", list_users_v2)
    .post("/users", create_user_v2)
    .get("/users/:id", get_user_v2);

// Main router with nested routes
let router = Router::new()
    .get("/", home)
    .nest("/api/v1", api_v1)
    .nest("/api/v2", api_v2);
```

### Nested Route Groups

```rust
// Admin routes
let admin_routes = Router::new()
    .get("/dashboard", admin_dashboard)
    .get("/users", admin_users)
    .post("/users/:id/ban", ban_user);

// User management routes
let user_routes = Router::new()
    .get("/", list_users)
    .post("/", create_user)
    .get("/:id", get_user)
    .put("/:id", update_user)
    .delete("/:id", delete_user);

// Main application router
let app = Router::new()
    .get("/", home)
    .nest("/admin", admin_routes)
    .nest("/users", user_routes);
```

### Shared State in Nested Routes

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_pool: Arc<DatabasePool>,
    cache: Arc<Cache>,
}

let api_routes = Router::new()
    .get("/users", get_users)
    .post("/users", create_user)
    .state(app_state.clone());

let admin_routes = Router::new()
    .get("/stats", get_stats)
    .state(app_state.clone());

let app = Router::new()
    .nest("/api", api_routes)
    .nest("/admin", admin_routes);
```

## Route Middleware

### Per-Route Middleware

```rust
use ignitia::{LayeredHandler, AuthMiddleware, LoggerMiddleware};

let protected_handler = LayeredHandler::new(secret_handler)
    .layer(AuthMiddleware::new("secret-token"))
    .layer(LoggerMiddleware);

let router = Router::new()
    .get("/public", public_handler)
    .route_with_layered("/secret", Method::GET, protected_handler);
```

### Route-Specific Middleware Chains

```rust
use ignitia::middleware::{RateLimitingMiddleware, SecurityMiddleware};

// API routes with rate limiting
let api_handler = LayeredHandler::new(api_endpoint)
    .layer(RateLimitingMiddleware::per_minute(100))
    .layer(LoggerMiddleware);

// Admin routes with authentication and security
let admin_handler = LayeredHandler::new(admin_endpoint)
    .layer(AuthMiddleware::new("admin-token"))
    .layer(SecurityMiddleware::high_security())
    .layer(RateLimitingMiddleware::per_minute(10));

let router = Router::new()
    .route_with_layered("/api/data", Method::GET, api_handler)
    .route_with_layered("/admin/dashboard", Method::GET, admin_handler);
```

## State Management

### Application State

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    counter: Arc<RwLock<u64>>,
    config: Arc<AppConfig>,
}

async fn increment_counter(State(state): State<AppState>) -> Result<Response> {
    let mut counter = state.counter.write().await;
    *counter += 1;
    Ok(Response::json(serde_json::json!({
        "counter": *counter
    }))?)
}

let app_state = AppState {
    counter: Arc::new(RwLock::new(0)),
    config: Arc::new(app_config),
};

let router = Router::new()
    .state(app_state)
    .get("/counter", increment_counter);
```

### Database Connection Pools

```rust
use sqlx::PgPool;

#[derive(Clone)]
struct DatabaseState {
    pool: PgPool,
}

async fn get_users(State(db): State<DatabaseState>) -> Result<Response> {
    let users = sqlx::query!("SELECT * FROM users")
        .fetch_all(&db.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    Ok(Response::json(users)?)
}

let db_state = DatabaseState {
    pool: PgPool::connect(&database_url).await?,
};

let router = Router::new()
    .state(db_state)
    .get("/users", get_users);
```

## Advanced Routing Patterns

### Wildcard Routes

```rust
// Catch-all file serving
let router = Router::new()
    .get("/files/*path", |Path(path): Path<String>| async move {
        // Serve file from path
        let file_path = format!("./static/{}", path);
        serve_file(&file_path).await
    });
```

### Route Guards

```rust
async fn admin_only_guard(req: &Request) -> bool {
    req.header("x-admin-token").is_some()
}

// Custom route matching with guards
let router = Router::new()
    .get("/admin/*path", |req: Request| async move {
        if !admin_only_guard(&req).await {
            return Ok(Response::new(StatusCode::FORBIDDEN));
        }
        // Handle admin route
        Ok(Response::text("Admin area"))
    });
```

### Dynamic Route Registration

```rust
let mut router = Router::new();

// Register routes dynamically
for endpoint in api_endpoints {
    router = router.get(&endpoint.path, endpoint.handler);
}
```

### Route Versioning

```rust
// Version-aware routing
let router = Router::new()
    .get("/api/v1/users", users_v1)
    .get("/api/v2/users", users_v2)
    .get("/api/users", |headers: Headers| async move {
        let version = headers.get("API-Version").unwrap_or("v1");
        match version {
            "v2" => users_v2().await,
            _ => users_v1().await,
        }
    });
```

## WebSocket Routing

When the `websocket` feature is enabled, Ignitia supports WebSocket routing:

### Basic WebSocket Routes

```rust
use ignitia::websocket::{WebSocketConnection, Message};

let router = Router::new()
    .websocket("/ws", |ws: WebSocketConnection| async move {
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

### WebSocket with State

```rust
async fn chat_handler(
    ws: WebSocketConnection,
    State(chat_state): State<ChatState>
) -> Result<()> {
    // Handle chat WebSocket connection
    while let Some(msg) = ws.recv().await {
        if let Message::Text(text) = msg {
            // Broadcast to all connected clients
            chat_state.broadcast(text).await?;
        }
    }
    Ok(())
}

let router = Router::new()
    .state(chat_state)
    .websocket("/chat", chat_handler);
```

## Performance Considerations

### Route Compilation

Ignitia compiles routes at startup for optimal performance:

```rust
// Routes are automatically sorted by specificity
let router = Router::new()
    .get("/users/:id/posts/:post_id", specific_handler)  // More specific
    .get("/users/:id/posts", less_specific_handler)      // Less specific
    .get("/users/:id", general_handler);                 // Most general
```

### Route Caching

```rust
// Pre-compile regex patterns for path matching
let router = Router::new()
    .get("/users/:id", handler); // Regex compiled once at startup
```

### Memory Efficiency

```rust
// Use Arc for shared state to avoid cloning
#[derive(Clone)]
struct SharedState {
    data: Arc<RwLock<HashMap<String, String>>>,
}

let router = Router::new()
    .state_arc(Arc::new(shared_data)) // Use Arc directly
    .get("/data", get_data_handler);
```

## Best Practices

### 1. Route Organization

```rust
// Organize routes by feature/module
mod user_routes {
    use super::*;

    pub fn routes() -> Router {
        Router::new()
            .get("/", list_users)
            .post("/", create_user)
            .get("/:id", get_user)
            .put("/:id", update_user)
            .delete("/:id", delete_user)
    }
}

mod post_routes {
    use super::*;

    pub fn routes() -> Router {
        Router::new()
            .get("/", list_posts)
            .post("/", create_post)
            .get("/:id", get_post)
    }
}

// Main router assembly
let app = Router::new()
    .nest("/users", user_routes::routes())
    .nest("/posts", post_routes::routes());
```

### 2. Error Handling

```rust
async fn safe_user_handler(Path(id): Path<u32>) -> Result<Response> {
    let user = get_user_by_id(id).await
        .map_err(|e| Error::Database(e.to_string()))?;

    match user {
        Some(user) => Ok(Response::json(user)?),
        None => Err(Error::NotFound(format!("User {} not found", id))),
    }
}
```

### 3. Input Validation

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUserRequest {
    #[serde(deserialize_with = "validate_email")]
    email: String,
    #[serde(deserialize_with = "validate_password")]
    password: String,
}

async fn create_user_handler(Json(req): Json<CreateUserRequest>) -> Result<Response> {
    // Request is already validated by serde
    let user = create_user(req).await?;
    Ok(Response::json(user)?)
}
```

### 4. Route Documentation

```rust
/// GET /users/:id
///
/// Retrieves a user by their unique identifier.
///
/// # Parameters
/// - `id`: The unique user ID (positive integer)
///
/// # Returns
/// - `200 OK`: User found and returned
/// - `404 Not Found`: User does not exist
/// - `400 Bad Request`: Invalid ID format
async fn get_user(Path(id): Path<u32>) -> Result<Response> {
    // Implementation
}
```

### 5. Testing Routes

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ignitia::test::TestRequest;

    #[tokio::test]
    async fn test_get_user() {
        let router = create_test_router();

        let response = TestRequest::get("/users/1")
            .send(&router)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

***

This routing guide covers the essential patterns and advanced features of Ignitia's routing system. For more specific examples and use cases, refer to the [Examples](EXAMPLES.md) documentation.
