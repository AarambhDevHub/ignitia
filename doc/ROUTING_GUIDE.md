# Ignitia Routing Guide 🔥

A complete guide to routing in Ignitia - covering everything from basic route definition to advanced patterns, router modes, and performance optimization.

## Table of Contents

- [Basic Concepts](#basic-concepts)
- [Router Modes](#router-modes)
- [Route Definition](#route-definition)
- [HTTP Methods](#http-methods)
- [Path Parameters](#path-parameters)
- [Query Parameters](#query-parameters)
- [Route Groups and Nesting](#route-groups-and-nesting)
- [Router Merging - NEW!](#router-merging---new)
- [Route Middleware](#route-middleware)
- [State Management](#state-management)
- [Advanced Routing Patterns](#advanced-routing-patterns)
- [WebSocket Routing](#websocket-routing)
- [Performance Considerations](#performance-considerations)
- [Best Practices](#best-practices)
- [Migration Guide](#migration-guide)

## Basic Concepts

Ignitia's routing system is built around the `Router` struct, which uses either a high-performance radix tree or traditional regex-based matching system. Routes are matched based on specificity, with more specific routes taking precedence over general ones.

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

1. **Exact static matches** (e.g., `/users/profile`)
2. **Parameterized routes** (e.g., `/users/:id`)
3. **Wildcard routes** (e.g., `/files/*path`)

## Router Modes

Ignitia provides two distinct routing modes, each optimized for different use cases:

### RouterMode::Radix (Default - Recommended)

The **Radix Tree** mode uses a compressed trie (radix tree) data structure for ultra-fast route matching. This is the **default and recommended mode** for production applications.

```rust
use ignitia::{Router, RouterMode};

// Explicit radix mode (default behavior)
let router = Router::new()
    .with_mode(RouterMode::Radix)
    .get("/users/:id", get_user)
    .get("/users/:id/posts", get_user_posts)
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
- **Memory Usage**: ~50% less than regex mode for typical applications
- **Startup Time**: Instant (no regex compilation)
- **Throughput**: Up to 3x faster than regex mode

**Use Cases:**
- Production applications with many routes (>50)
- High-performance APIs requiring maximum throughput
- Applications with complex nested route structures
- Microservices with predictable route patterns

### RouterMode::Base (Legacy)

The **Base** mode uses traditional regex-based route matching. This mode is maintained for compatibility but is **not recommended** for new applications.

```rust
use ignitia::{Router, RouterMode};

// Legacy regex-based mode
let router = Router::new()
    .with_mode(RouterMode::Base)
    .get("/users/:id", get_user)
    .get("/posts/*path", serve_posts);
```

**Disadvantages:**
- **Slower matching**: O(n) time complexity
- **Higher memory usage**: Each route stores a compiled regex
- **Startup overhead**: Regex compilation during initialization
- **Complex regex patterns**: More prone to performance issues

**When to Use:**
- Legacy applications requiring regex-specific features
- Debugging or testing scenarios
- Temporary compatibility during migration

### Choosing the Right Mode

| Feature | Radix Mode | Base Mode |
|---------|------------|-----------|
| **Performance** | ⭐⭐⭐⭐⭐ Ultra-fast | ⭐⭐⭐ Good |
| **Memory Usage** | ⭐⭐⭐⭐⭐ Efficient | ⭐⭐⭐ Higher |
| **Startup Time** | ⭐⭐⭐⭐⭐ Instant | ⭐⭐ Slower |
| **Route Complexity** | ⭐⭐⭐⭐⭐ Excellent | ⭐⭐⭐⭐ Good |
| **Maintenance** | ⭐⭐⭐⭐⭐ Active | ⭐⭐ Legacy |

### Router Mode Configuration

```rust
use ignitia::{Router, RouterMode};

// Default (recommended)
let router = Router::new(); // Uses RouterMode::Radix

// Explicit radix mode
let router = Router::new()
    .with_mode(RouterMode::Radix);

// Legacy regex mode (not recommended)
let router = Router::new()
    .with_mode(RouterMode::Base);
```

### Performance Comparison

```rust
use ignitia::{Router, RouterMode};
use std::time::Instant;

async fn benchmark_router_modes() {
    let routes = vec![
        "/api/v1/users",
        "/api/v1/users/:id",
        "/api/v1/users/:id/posts",
        "/api/v1/users/:id/posts/:post_id",
        "/api/v2/users",
        "/api/v2/users/:id",
        // ... many more routes
    ];

    // Radix mode
    let start = Instant::now();
    let radix_router = build_router(RouterMode::Radix, &routes);
    let radix_build_time = start.elapsed();

    // Base mode
    let start = Instant::now();
    let base_router = build_router(RouterMode::Base, &routes);
    let base_build_time = start.elapsed();

    println!("Build time - Radix: {:?}, Base: {:?}",
             radix_build_time, base_build_time);

    // Benchmark route matching...
}
```

### Migration Between Modes

Switching between router modes is seamless:

```rust
// From Base to Radix (recommended upgrade)
let router = Router::new()
    .with_mode(RouterMode::Base) // Remove this line
    .with_mode(RouterMode::Radix) // Add this line
    .get("/api/users", list_users);

// No other changes needed - all route definitions remain the same
```

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

### Route with Custom Handler

```rust
use ignitia::handler::raw_handler;

let router = Router::new()
    .get("/custom", raw_handler(|req| async move {
        let user_agent = req.header("user-agent").unwrap_or("Unknown");
        Ok(Response::text(format!("User-Agent: {}", user_agent)))
    }));
```

## HTTP Methods

Ignitia supports all standard HTTP methods with dedicated builder methods:

### GET Routes

```rust
let router = Router::new()
    .get("/users", list_users)
    .get("/users/:id", get_user)
    .get("/search", search_users)
    .get("/health", || async {
        Ok(Response::json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().timestamp()
        }))?)
    });
```

### POST Routes

```rust
use ignitia::{Json, Response};
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
            return Err(ignitia::Error::BadRequest("Name and email are required".to_string()));
        }

        // Create user logic here
        let new_user = UserResponse {
            id: 1,
            name: user.name,
            email: user.email,
            role: user.role.unwrap_or("user".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(Response::json(new_user)?)
    })
    .post("/upload", |body: ignitia::Body| async move {
        // Handle file upload
        let file_size = body.len();
        Ok(Response::json(serde_json::json!({
            "uploaded": true,
            "size": file_size
        }))?)
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
    .put("/users/:id", |Path(id): Path<u32>, Json(user): Json<UpdateUser>| async move {
        // Full update logic - replace entire resource
        Ok(Response::json(serde_json::json!({
            "id": id,
            "updated": true,
            "type": "full_update"
        }))?)
    })
    .patch("/users/:id", |Path(id): Path<u32>, Json(patch): Json<UserPatch>| async move {
        // Partial update logic - update only provided fields
        Ok(Response::json(serde_json::json!({
            "id": id,
            "updated": true,
            "type": "partial_update",
            "fields": serde_json::json!(patch)
        }))?)
    });
```

### DELETE Routes

```rust
let router = Router::new()
    .delete("/users/:id", |Path(id): Path<u32>| async move {
        // Soft delete or hard delete logic

        // Return 204 No Content for successful deletion
        Ok(Response::new(ignitia::StatusCode::NO_CONTENT))
    })
    .delete("/users/:id/sessions", |Path(id): Path<u32>| async move {
        // Logout user by clearing sessions
        Ok(Response::json(serde_json::json!({
            "message": "All sessions cleared",
            "user_id": id
        }))?)
    });
```

### HEAD and OPTIONS Routes

```rust
let router = Router::new()
    .head("/users/:id", |Path(id): Path<u32>| async move {
        // Return headers only (no body)
        let mut response = Response::new(ignitia::StatusCode::OK);
        response.headers.insert("Content-Type", "application/json");
        response.headers.insert("X-User-Exists", "true");
        Ok(response)
    })
    .options("/users", || async {
        let mut response = Response::new(ignitia::StatusCode::OK);
        response.headers.insert("Allow", "GET, POST, PUT, PATCH, DELETE, OPTIONS");
        response.headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE");
        response.headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization");
        Ok(response)
    })
    // CORS preflight for specific routes
    .options("/api/*path", |Path(path): Path<String>| async move {
        let mut response = Response::new(ignitia::StatusCode::OK);
        response.headers.insert("Access-Control-Allow-Origin", "*");
        response.headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE");
        response.headers.insert("Access-Control-Max-Age", "86400"); // 24 hours
        Ok(response)
    });
```

### Custom HTTP Methods

```rust
use ignitia::Method;

let router = Router::new()
    .route("/custom", Method::from_bytes(b"CUSTOM").unwrap(), |_| async {
        Ok(Response::text("Custom method handler"))
    });
```

## Path Parameters

### Single Parameters

```rust
use ignitia::Path;

// Extract single parameter
let router = Router::new()
    .get("/users/:id", |Path(id): Path<u32>| async move {
        if id == 0 {
            return Err(ignitia::Error::BadRequest("Invalid user ID".to_string()));
        }
        Ok(Response::text(format!("User ID: {}", id)))
    })
    .get("/posts/:slug", |Path(slug): Path<String>| async move {
        // URL decode the slug
        let decoded_slug = urlencoding::decode(&slug)
            .map_err(|_| ignitia::Error::BadRequest("Invalid slug encoding".to_string()))?;

        Ok(Response::json(serde_json::json!({
            "slug": decoded_slug.as_ref(),
            "original": slug
        }))?)
    });
```

### Multiple Parameters

```rust
// Extract multiple parameters as tuple
let router = Router::new()
    .get("/users/:user_id/posts/:post_id",
        |Path((user_id, post_id)): Path<(u32, u32)>| async move {
            // Validate parameters
            if user_id == 0 || post_id == 0 {
                return Err(ignitia::Error::BadRequest("Invalid ID parameters".to_string()));
            }

            Ok(Response::json(serde_json::json!({
                "user_id": user_id,
                "post_id": post_id,
                "relationship": "user_post"
            }))?)
        })

    .get("/api/:version/:resource/:id",
        |Path((version, resource, id)): Path<(String, String, u32)>| async move {
            // API versioning with parameters
            match version.as_str() {
                "v1" | "v2" => {},
                _ => return Err(ignitia::Error::BadRequest("Unsupported API version".to_string()))
            }

            Ok(Response::json(serde_json::json!({
                "api_version": version,
                "resource": resource,
                "id": id
            }))?)
        });
```

### Named Parameter Extraction

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct UserPostParams {
    user_id: u32,
    post_id: u32,
}

#[derive(Deserialize, Debug)]
struct ApiParams {
    version: String,
    resource: String,
    id: u32,
}

let router = Router::new()
    .get("/users/:user_id/posts/:post_id",
        |Path(params): Path<UserPostParams>| async move {
            Ok(Response::json(serde_json::json!({
                "params": params,
                "extracted_via": "named_struct"
            }))?)
        })

    .get("/api/:version/:resource/:id",
        |Path(params): Path<ApiParams>| async move {
            // Validate API version
            if !["v1", "v2", "v3"].contains(&params.version.as_str()) {
                return Err(ignitia::Error::BadRequest(
                    format!("Unsupported API version: {}", params.version)
                ));
            }

            Ok(Response::json(params)?)
        });
```

### Optional and Default Parameters

```rust
#[derive(Deserialize)]
struct PaginationParams {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
    order: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(10),
            sort: Some("created_at".to_string()),
            order: Some("desc".to_string()),
        }
    }
}

let router = Router::new()
    .get("/users/:id/posts", |
        Path(id): Path<u32>,
        Query(params): Query<PaginationParams>
    | async move {
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(10).min(100); // Cap at 100
        let sort = params.sort.unwrap_or_else(|| "created_at".to_string());
        let order = params.order.unwrap_or_else(|| "desc".to_string());

        // Validate sort field
        let valid_sorts = ["created_at", "updated_at", "title", "views"];
        if !valid_sorts.contains(&sort.as_str()) {
            return Err(ignitia::Error::BadRequest("Invalid sort field".to_string()));
        }

        Ok(Response::json(serde_json::json!({
            "user_id": id,
            "pagination": {
                "page": page,
                "limit": limit,
                "sort": sort,
                "order": order
            },
            "total_pages": 10, // Would come from database
            "total_items": 95
        }))?)
    });
```

### Wildcard Parameters

```rust
// Catch-all routes with wildcard parameters
let router = Router::new()
    .get("/files/*path", |Path(path): Path<String>| async move {
        // Secure file serving with path validation
        let safe_path = path.replace("..", ""); // Basic security
        let file_path = format!("./static/{}", safe_path);

        // Check if file exists and is within allowed directory
        if !std::path::Path::new(&file_path).exists() {
            return Err(ignitia::Error::NotFound(format!("File not found: {}", path)));
        }

        Ok(Response::json(serde_json::json!({
            "requested_path": path,
            "resolved_path": file_path,
            "file_exists": true
        }))?)
    })

    .get("/proxy/*upstream", |Path(upstream): Path<String>| async move {
        // Proxy requests to upstream services
        let upstream_url = format!("http://internal-service/{}", upstream);

        Ok(Response::json(serde_json::json!({
            "proxy_target": upstream_url,
            "original_path": upstream
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
    page: Option<u32>,
    per_page: Option<u32>,
}

let router = Router::new()
    .get("/search", |Query(params): Query<SearchParams>| async move {
        // Validate required parameters
        if params.q.is_empty() {
            return Err(ignitia::Error::BadRequest("Search query 'q' is required".to_string()));
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(10).min(100); // Limit page size

        Ok(Response::json(serde_json::json!({
            "search": {
                "query": params.q,
                "category": params.category,
                "sort": params.sort.unwrap_or_else(|| "relevance".to_string()),
                "pagination": {
                    "page": page,
                    "per_page": per_page,
                    "offset": (page - 1) * per_page
                }
            },
            "results": [], // Would contain actual search results
            "total": 0,
            "took_ms": 42
        }))?)
    });
```

### Advanced Query Parameters

```rust
#[derive(Deserialize)]
struct FilterParams {
    // Array parameters (e.g., ?tags=rust&tags=web&tags=api)
    tags: Vec<String>,

    // Range parameters
    min_price: Option<f64>,
    max_price: Option<f64>,

    // Date range parameters
    date_from: Option<String>, // ISO 8601 date string
    date_to: Option<String>,

    // Boolean parameters
    in_stock: Option<bool>,
    featured: Option<bool>,

    // Enum-like parameters
    status: Option<String>, // "active", "inactive", "pending"
    sort_order: Option<String>, // "asc", "desc"
}

let router = Router::new()
    .get("/products", |Query(filters): Query<FilterParams>| async move {
        // Validate price range
        if let (Some(min), Some(max)) = (filters.min_price, filters.max_price) {
            if min > max {
                return Err(ignitia::Error::BadRequest("min_price cannot be greater than max_price".to_string()));
            }
        }

        // Validate status values
        if let Some(ref status) = filters.status {
            let valid_statuses = ["active", "inactive", "pending"];
            if !valid_statuses.contains(&status.as_str()) {
                return Err(ignitia::Error::BadRequest(
                    format!("Invalid status. Valid values: {}", valid_statuses.join(", "))
                ));
            }
        }

        // Parse date strings
        let parsed_date_from = if let Some(ref date_str) = filters.date_from {
            Some(chrono::DateTime::parse_from_rfc3339(date_str)
                .map_err(|_| ignitia::Error::BadRequest("Invalid date_from format. Use ISO 8601".to_string()))?)
        } else {
            None
        };

        Ok(Response::json(serde_json::json!({
            "filters_applied": {
                "tags": filters.tags,
                "price_range": {
                    "min": filters.min_price,
                    "max": filters.max_price
                },
                "date_range": {
                    "from": parsed_date_from.map(|d| d.to_rfc3339()),
                    "to": filters.date_to
                },
                "boolean_filters": {
                    "in_stock": filters.in_stock,
                    "featured": filters.featured
                },
                "status": filters.status,
                "sort_order": filters.sort_order.unwrap_or_else(|| "asc".to_string())
            },
            "products": [], // Filtered products would be here
            "total_found": 0
        }))?)
    });
```

### Query Parameter Validation

```rust
use serde::{Deserialize, Deserializer};

fn deserialize_positive_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<u32> = Option::deserialize(deserializer)?;
    match value {
        Some(v) if v > 0 => Ok(Some(v)),
        Some(_) => Err(serde::de::Error::custom("Value must be positive")),
        None => Ok(None),
    }
}

#[derive(Deserialize)]
struct ValidatedParams {
    #[serde(deserialize_with = "deserialize_positive_u32")]
    page: Option<u32>,

    #[serde(deserialize_with = "deserialize_positive_u32")]
    limit: Option<u32>,
}

let router = Router::new()
    .get("/validated", |Query(params): Query<ValidatedParams>| async move {
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(10);

        Ok(Response::json(serde_json::json!({
            "validated_params": {
                "page": page,
                "limit": limit
            }
        }))?)
    });
```

## Route Groups and Nesting

### Basic Route Grouping

```rust
// Create API v1 routes
let api_v1 = Router::new()
    .get("/users", list_users_v1)
    .post("/users", create_user_v1)
    .get("/users/:id", get_user_v1)
    .put("/users/:id", update_user_v1)
    .delete("/users/:id", delete_user_v1);

// Create API v2 routes with enhanced features
let api_v2 = Router::new()
    .get("/users", list_users_v2) // Includes additional fields
    .post("/users", create_user_v2) // Enhanced validation
    .get("/users/:id", get_user_v2) // More detailed response
    .patch("/users/:id", partial_update_user_v2); // PATCH support

// Admin routes
let admin_routes = Router::new()
    .get("/dashboard", admin_dashboard)
    .get("/analytics", admin_analytics)
    .post("/maintenance", toggle_maintenance_mode);

// Main router with nested routes
let router = Router::new()
    .get("/", home_page)
    .get("/health", health_check)
    .nest("/api/v1", api_v1)
    .nest("/api/v2", api_v2)
    .nest("/admin", admin_routes);
```

### Complex Nested Route Structure

```rust
// Blog routes
let blog_routes = Router::new()
    .get("/", list_posts)
    .get("/:slug", get_post_by_slug)
    .get("/category/:category", list_posts_by_category)
    .get("/tag/:tag", list_posts_by_tag);

// User profile routes
let profile_routes = Router::new()
    .get("/", get_profile)
    .put("/", update_profile)
    .post("/avatar", upload_avatar)
    .delete("/avatar", delete_avatar)
    .get("/settings", get_user_settings)
    .put("/settings", update_user_settings);

// User management routes
let user_routes = Router::new()
    .get("/", list_users)
    .post("/", create_user)
    .get("/:id", get_user)
    .put("/:id", update_user)
    .delete("/:id", delete_user)
    .nest("/:id/profile", profile_routes)
    .get("/:id/posts", get_user_posts);

// E-commerce routes
let product_routes = Router::new()
    .get("/", list_products)
    .get("/:id", get_product)
    .get("/category/:category", products_by_category)
    .get("/:id/reviews", get_product_reviews)
    .post("/:id/reviews", create_review);

let cart_routes = Router::new()
    .get("/", get_cart)
    .post("/items", add_to_cart)
    .put("/items/:item_id", update_cart_item)
    .delete("/items/:item_id", remove_from_cart)
    .post("/checkout", checkout);

// Main application router
let app = Router::new()
    .get("/", homepage)
    .nest("/blog", blog_routes)
    .nest("/users", user_routes)
    .nest("/products", product_routes)
    .nest("/cart", cart_routes);
```

### Shared State in Nested Routes

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    db_pool: Arc<DatabasePool>,
    redis_client: Arc<RedisClient>,
    config: Arc<AppConfig>,
    metrics: Arc<RwLock<Metrics>>,
}

#[derive(Clone)]
struct ApiState {
    rate_limiter: Arc<RateLimiter>,
    auth_service: Arc<AuthService>,
}

let api_routes = Router::new()
    .state(app_state.clone())
    .state(api_state.clone())
    .get("/users", get_users)
    .post("/users", create_user)
    .middleware(RateLimitingMiddleware::new())
    .middleware(AuthMiddleware::new());

let admin_routes = Router::new()
    .state(app_state.clone())
    .get("/stats", get_admin_stats)
    .post("/maintenance", toggle_maintenance)
    .middleware(AdminAuthMiddleware::new());

let public_routes = Router::new()
    .state(app_state.clone())
    .get("/health", health_check)
    .get("/version", version_info);

let app = Router::new()
    .nest("/api", api_routes)
    .nest("/admin", admin_routes)
    .nest("/public", public_routes);
```

### Conditional Route Nesting

```rust
fn build_router(config: &AppConfig) -> Router {
    let mut router = Router::new()
        .get("/", homepage)
        .nest("/api/v1", build_api_v1_routes());

    // Conditionally add API v2
    if config.api_v2_enabled {
        router = router.nest("/api/v2", build_api_v2_routes());
    }

    // Add admin routes only in development or with admin feature
    if config.environment == "development" || config.admin_enabled {
        router = router.nest("/admin", build_admin_routes());
    }

    // Add debug routes only in development
    if config.environment == "development" {
        router = router.nest("/debug", build_debug_routes());
    }

    router
}
```

## Router Merging - NEW!

The new `merge` functionality allows you to combine multiple routers into a single router, providing powerful composition capabilities.

### Basic Router Merging

```rust
use ignitia::Router;

// Create separate routers for different features
let user_router = Router::new()
    .get("/users", list_users)
    .post("/users", create_user)
    .get("/users/:id", get_user);

let post_router = Router::new()
    .get("/posts", list_posts)
    .post("/posts", create_post)
    .get("/posts/:id", get_post);

// Merge routers together
let main_router = Router::new()
    .get("/", home_page)
    .merge(user_router)
    .merge(post_router);
```

### Cross-Mode Router Merging

The merge functionality intelligently handles different router modes :

```rust
// Base mode router
let base_router = Router::new()
    .with_mode(RouterMode::Base)
    .get("/legacy", legacy_handler);

// Radix mode router (default)
let radix_router = Router::new()
    .get("/modern", modern_handler);

// Merge different modes - routes are automatically converted
let combined_router = radix_router.merge(base_router);
// Result: All routes in radix_router's mode (Radix)
```

### Merging with State and Middleware

```rust
#[derive(Clone)]
struct DatabaseState {
    pool: PgPool,
}

#[derive(Clone)]
struct CacheState {
    redis: Arc<Redis>,
}

// Router with database state
let db_router = Router::new()
    .state(DatabaseState { pool: db_pool })
    .middleware(DatabaseMiddleware::new())
    .get("/db/users", get_users_from_db);

// Router with cache state
let cache_router = Router::new()
    .state(CacheState { redis: redis_client })
    .middleware(CacheMiddleware::new())
    .get("/cache/stats", get_cache_stats);

// Merge routers - states and middleware are combined
let app_router = Router::new()
    .merge(db_router)
    .merge(cache_router);
// Result: Both states available, middleware chains combined
```

### Advanced Merging Scenarios

#### Conditional Router Merging

```rust
fn build_router(config: &AppConfig) -> Router {
    let mut router = Router::new()
        .get("/", homepage);

    // Always include core API
    router = router.merge(build_core_api());

    // Conditionally merge admin routes
    if config.admin_enabled {
        router = router.merge(build_admin_routes());
    }

    // Conditionally merge debug routes
    if config.environment == "development" {
        router = router.merge(build_debug_routes());
    }

    router
}
```

#### Plugin-Style Architecture

```rust
trait Plugin {
    fn router(&self) -> Router;
    fn name(&self) -> &str;
}

struct AuthPlugin;
impl Plugin for AuthPlugin {
    fn router(&self) -> Router {
        Router::new()
            .post("/login", login_handler)
            .post("/logout", logout_handler)
            .get("/profile", profile_handler)
    }

    fn name(&self) -> &str { "auth" }
}

struct PaymentPlugin;
impl Plugin for PaymentPlugin {
    fn router(&self) -> Router {
        Router::new()
            .post("/payment/charge", charge_handler)
            .get("/payment/status/:id", payment_status)
    }

    fn name(&self) -> &str { "payment" }
}

// Build app by merging plugin routers
fn build_app_with_plugins(plugins: Vec<Box<dyn Plugin>>) -> Router {
    let mut app = Router::new()
        .get("/", homepage);

    for plugin in plugins {
        println!("Loading plugin: {}", plugin.name());
        app = app.merge(plugin.router());
    }

    app
}

// Usage
let app = build_app_with_plugins(vec![
    Box::new(AuthPlugin),
    Box::new(PaymentPlugin),
]);
```

### Merge Behavior Rules

1. **Route Conflicts**: Current router routes take precedence over merged router routes
2. **Middleware Combination**: Middleware from merged routers is applied after current router middleware
3. **State Merging**: All states from both routers are available in the merged result
4. **Not Found Handlers**: Current router's not found handler takes precedence
5. **WebSocket Routes**: Only added if path doesn't already exist

### Performance Considerations for Merging

```rust
// Efficient: Merge once during startup
let app_router = base_router
    .merge(api_router)
    .merge(admin_router);

// Inefficient: Multiple merges in hot paths
// Don't do this in request handlers
async fn bad_example() {
    let router = Router::new().merge(other_router); // ❌ Avoid
}
```


## Route Middleware

### Per-Route Middleware

```rust
use ignitia::{LayeredHandler, AuthMiddleware, LoggerMiddleware};

// Create a protected handler with multiple middleware layers
let protected_handler = LayeredHandler::new(secret_handler)
    .layer(AuthMiddleware::bearer_token("secret-token"))
    .layer(LoggerMiddleware::detailed())
    .layer(RateLimitingMiddleware::per_minute(10));

let router = Router::new()
    .get("/public", public_handler)
    .route_with_layered("/secret", Method::GET, protected_handler);
```

### Advanced Route-Specific Middleware Chains

```rust
use ignitia::middleware::{
    RateLimitingMiddleware,
    SecurityMiddleware,
    CompressionMiddleware,
    CacheMiddleware
};

// Different middleware stacks for different route types
let api_handler = LayeredHandler::new(api_endpoint)
    .layer(RateLimitingMiddleware::per_minute(1000)) // High rate limit
    .layer(CompressionMiddleware::new()) // Response compression
    .layer(LoggerMiddleware::json_format());

let admin_handler = LayeredHandler::new(admin_endpoint)
    .layer(AuthMiddleware::admin_token("admin-secret"))
    .layer(SecurityMiddleware::strict())
    .layer(RateLimitingMiddleware::per_minute(100))
    .layer(LoggerMiddleware::with_request_body());

let public_api_handler = LayeredHandler::new(public_endpoint)
    .layer(CacheMiddleware::with_ttl(Duration::from_secs(300))) // 5 min cache
    .layer(CompressionMiddleware::gzip_only())
    .layer(RateLimitingMiddleware::per_minute(10000));

let router = Router::new()
    .route_with_layered("/api/data", Method::GET, api_handler)
    .route_with_layered("/admin/dashboard", Method::GET, admin_handler)
    .route_with_layered("/public/stats", Method::GET, public_api_handler);
```

### Middleware Composition Patterns

```rust
// Reusable middleware compositions
fn create_api_middleware() -> impl Middleware {
    LayeredHandler::new(())
        .layer(LoggerMiddleware::new())
        .layer(CompressionMiddleware::new())
        .layer(RateLimitingMiddleware::per_minute(1000))
}

fn create_admin_middleware() -> impl Middleware {
    LayeredHandler::new(())
        .layer(AuthMiddleware::admin_only())
        .layer(SecurityMiddleware::high_security())
        .layer(LoggerMiddleware::with_request_body())
}

// Apply to multiple routes
let router = Router::new()
    .middleware(create_api_middleware())
    .get("/api/users", list_users)
    .get("/api/posts", list_posts)

    .middleware(create_admin_middleware())
    .get("/admin/users", admin_list_users)
    .post("/admin/broadcast", admin_broadcast);
```

## State Management

### Application State with Database

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::PgPool;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    config: Arc<AppConfig>,
    metrics: Arc<RwLock<ApplicationMetrics>>,
}

#[derive(Debug)]
struct ApplicationMetrics {
    requests_count: u64,
    errors_count: u64,
    avg_response_time: Duration,
}

async fn get_user_with_cache(
    Path(id): Path<u32>,
    State(state): State<AppState>
) -> Result<Response> {
    let cache_key = format!("user:{}", id);

    // Check cache first
    {
        let cache = state.cache.read().await;
        if let Some(cached_user) = cache.get(&cache_key) {
            return Ok(Response::json(cached_user.clone())?);
        }
    }

    // Fetch from database
    let user = sqlx::query_as!(
        User,
        "SELECT id, name, email, created_at FROM users WHERE id = $1",
        id as i32
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ignitia::Error::Database(e.to_string()))?;

    match user {
        Some(user) => {
            let user_json = serde_json::to_value(&user)?;

            // Cache the result
            {
                let mut cache = state.cache.write().await;
                cache.insert(cache_key, user_json.clone());
            }

            Ok(Response::json(user_json)?)
        }
        None => Err(ignitia::Error::NotFound(format!("User {} not found", id))),
    }
}

// Initialize state
let db_pool = PgPool::connect(&database_url).await?;
let app_state = AppState {
    db: db_pool,
    cache: Arc::new(RwLock::new(HashMap::new())),
    config: Arc::new(load_config()),
    metrics: Arc::new(RwLock::new(ApplicationMetrics {
        requests_count: 0,
        errors_count: 0,
        avg_response_time: Duration::from_millis(0),
    })),
};

let router = Router::new()
    .state(app_state)
    .get("/users/:id", get_user_with_cache);
```

### Service Layer State Management

```rust
#[derive(Clone)]
struct ServiceContainer {
    user_service: Arc<UserService>,
    auth_service: Arc<AuthService>,
    notification_service: Arc<NotificationService>,
    storage_service: Arc<StorageService>,
}

impl ServiceContainer {
    async fn new(config: &AppConfig) -> Result<Self> {
        let db_pool = PgPool::connect(&config.database_url).await?;

        Ok(Self {
            user_service: Arc::new(UserService::new(db_pool.clone())),
            auth_service: Arc::new(AuthService::new(config.jwt_secret.clone())),
            notification_service: Arc::new(NotificationService::new(&config.redis_url).await?),
            storage_service: Arc::new(StorageService::new(&config.s3_config)),
        })
    }
}

async fn create_user_endpoint(
    Json(request): Json<CreateUserRequest>,
    State(services): State<ServiceContainer>
) -> Result<Response> {
    // Use services from the container
    let user = services.user_service.create(request).await?;
    services.notification_service.send_welcome_email(&user).await?;

    Ok(Response::json(user)?)
}

let services = ServiceContainer::new(&config).await?;
let router = Router::new()
    .state(services)
    .post("/users", create_user_endpoint);
```

### Multiple State Types

```rust
#[derive(Clone)]
struct DatabaseState {
    pool: PgPool,
}

#[derive(Clone)]
struct CacheState {
    redis: Arc<Redis>,
}

#[derive(Clone)]
struct ConfigState {
    app_config: Arc<AppConfig>,
}

async fn complex_handler(
    Path(id): Path<u32>,
    State(db): State<DatabaseState>,
    State(cache): State<CacheState>,
    State(config): State<ConfigState>
) -> Result<Response> {
    // Use multiple state objects
    let cache_ttl = config.app_config.cache_ttl;
    let cached_data = cache.redis.get(&format!("item:{}", id)).await?;

    if cached_data.is_none() {
        let data = fetch_from_db(&db.pool, id).await?;
        cache.redis.set(&format!("item:{}", id), &data, cache_ttl).await?;
        Ok(Response::json(data)?)
    } else {
        Ok(Response::json(cached_data.unwrap())?)
    }
}

let router = Router::new()
    .state(DatabaseState { pool: db_pool })
    .state(CacheState { redis: Arc::new(redis_client) })
    .state(ConfigState { app_config: Arc::new(config) })
    .get("/items/:id", complex_handler);
```

## Advanced Routing Patterns

### Wildcard Routes

```rust
// Secure file serving with proper validation
let router = Router::new()
    .get("/files/*path", |Path(path): Path<String>| async move {
        // Security: prevent directory traversal
        let safe_path = path.replace("..", "").replace("//", "/");

        // Validate file extension
        let allowed_extensions = [".jpg", ".png", ".gif", ".pdf", ".txt"];
        let has_valid_ext = allowed_extensions.iter()
            .any(|ext| safe_path.to_lowercase().ends_with(ext));

        if !has_valid_ext {
            return Err(ignitia::Error::Forbidden("File type not allowed".to_string()));
        }

        let file_path = format!("./static/{}", safe_path);

        // Check if file exists and is readable
        match tokio::fs::read(&file_path).await {
            Ok(contents) => {
                let content_type = match std::path::Path::new(&file_path)
                    .extension()
                    .and_then(|ext| ext.to_str()) {
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("png") => "image/png",
                    Some("gif") => "image/gif",
                    Some("pdf") => "application/pdf",
                    Some("txt") => "text/plain",
                    _ => "application/octet-stream",
                };

                let mut response = Response::new(ignitia::StatusCode::OK);
                response.headers.insert("Content-Type", content_type);
                response.headers.insert("Cache-Control", "public, max-age=3600");
                response.body = Arc::new(bytes::Bytes::from(contents));
                Ok(response)
            }
            Err(_) => Err(ignitia::Error::NotFound(format!("File not found: {}", path))),
        }
    });
```

### Route Guards with Custom Logic

```rust
async fn require_admin_role(req: &Request) -> Result<()> {
    let auth_header = req.header("authorization")
        .ok_or_else(|| ignitia::Error::Unauthorized)?;

    let token = auth_header.strip_prefix("Bearer ")
        .ok_or_else(|| ignitia::Error::Unauthorized)?;

    // Validate JWT and check role
    let claims = validate_jwt(token)?;
    if claims.role != "admin" {
        return Err(ignitia::Error::Forbidden("Admin role required".to_string()));
    }

    Ok(())
}

async fn admin_guard_handler(req: Request) -> Result<Response> {
    require_admin_role(&req).await?;

    // Admin-only logic here
    Ok(Response::json(serde_json::json!({
        "message": "Admin area accessed successfully",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))?)
}

let router = Router::new()
    .get("/admin/*path", admin_guard_handler);
```

### Dynamic Route Registration

```rust
struct ApiEndpoint {
    path: String,
    method: Method,
    handler: Box<dyn Fn() -> BoxFuture<'static, Result<Response>> + Send + Sync>,
}

fn register_crud_routes<T>(router: Router, resource: &str) -> Router
where
    T: CrudOperations + 'static,
{
    let list_path = format!("/{}", resource);
    let item_path = format!("/{}/:id", resource);

    router
        .get(&list_path, T::list)
        .post(&list_path, T::create)
        .get(&item_path, T::get)
        .put(&item_path, T::update)
        .delete(&item_path, T::delete)
}

let mut router = Router::new();

// Register CRUD routes for different resources
router = register_crud_routes::<User>(router, "users");
router = register_crud_routes::<Post>(router, "posts");
router = register_crud_routes::<Category>(router, "categories");
```

### API Versioning Strategies

```rust
// Header-based versioning
let router = Router::new()
    .get("/api/users", |headers: Headers| async move {
        let version = headers.get("API-Version").unwrap_or("v1");

        match version {
            "v1" => users_v1_handler().await,
            "v2" => users_v2_handler().await,
            "v3" => users_v3_handler().await,
            _ => Err(ignitia::Error::BadRequest(
                format!("Unsupported API version: {}. Supported: v1, v2, v3", version)
            )),
        }
    })

    // Accept header versioning
    .get("/api/data", |headers: Headers| async move {
        let accept = headers.get("accept").unwrap_or("");

        match accept {
            accept if accept.contains("application/vnd.api+json; version=1") => {
                data_v1_handler().await
            }
            accept if accept.contains("application/vnd.api+json; version=2") => {
                data_v2_handler().await
            }
            _ => data_latest_handler().await,
        }
    });

// URL-based versioning (recommended for REST APIs)
let router = Router::new()
    .nest("/api/v1", build_v1_routes())
    .nest("/api/v2", build_v2_routes())
    .nest("/api/v3", build_v3_routes())

    // Latest API without version (redirects to latest)
    .get("/api/users", || async {
        let mut response = Response::new(ignitia::StatusCode::MOVED_PERMANENTLY);
        response.headers.insert("Location", "/api/v3/users");
        Ok(response)
    });
```

### Route Aliasing and Redirects

```rust
let router = Router::new()
    // Main routes
    .get("/users/:id", get_user)
    .get("/posts/:id", get_post)

    // Legacy route aliases
    .get("/user/:id", |Path(id): Path<u32>| async move {
        // Redirect to new route
        let mut response = Response::new(ignitia::StatusCode::MOVED_PERMANENTLY);
        response.headers.insert("Location", &format!("/users/{}", id));
        Ok(response)
    })

    .get("/article/:id", |Path(id): Path<u32>| async move {
        // Redirect to new route
        let mut response = Response::new(ignitia::StatusCode::MOVED_PERMANENTLY);
        response.headers.insert("Location", &format!("/posts/{}", id));
        Ok(response)
    })

    // Shortcut URLs
    .get("/u/:id", |Path(id): Path<u32>| async move {
        // Temporary redirect to full URL
        let mut response = Response::new(ignitia::StatusCode::FOUND);
        response.headers.insert("Location", &format!("/users/{}", id));
        Ok(response)
    });
```

## WebSocket Routing

When the `websocket` feature is enabled, Ignitia provides powerful WebSocket routing capabilities:

### Basic WebSocket Routes

```rust
#[cfg(feature = "websocket")]
use ignitia::websocket::{WebSocketConnection, Message};

#[cfg(feature = "websocket")]
let router = Router::new()
    .websocket("/ws", |mut ws: WebSocketConnection| async move {
        println!("WebSocket connection established");

        while let Some(msg) = ws.recv().await {
            match msg {
                Message::Text(text) => {
                    println!("Received text: {}", text);
                    ws.send_text(format!("Echo: {}", text)).await?;
                }
                Message::Binary(data) => {
                    println!("Received {} bytes", data.len());
                    ws.send_bytes(data).await?;
                }
                Message::Ping(data) => {
                    ws.send_pong(data).await?;
                }
                Message::Close(frame) => {
                    println!("Connection closed: {:?}", frame);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    });
```

### WebSocket with Authentication and State

```rust
#[cfg(feature = "websocket")]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[cfg(feature = "websocket")]
#[derive(Clone)]
struct ChatState {
    clients: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    broadcast_tx: broadcast::Sender<String>,
}

#[cfg(feature = "websocket")]
async fn authenticated_chat_handler(
    ws: WebSocketConnection,
    headers: Headers,
    State(chat_state): State<ChatState>
) -> Result<()> {
    // Authenticate WebSocket connection
    let auth_token = headers.get("authorization")
        .ok_or_else(|| ignitia::Error::Unauthorized)?;

    let user = authenticate_user(auth_token).await?;
    let client_id = format!("user_{}", user.id);

    println!("User {} connected to chat", user.name);

    // Add client to active connections
    {
        let mut clients = chat_state.clients.lock().unwrap();
        clients.insert(client_id.clone(), ws.clone());
    }

    // Broadcast user joined message
    let join_message = format!("{} joined the chat", user.name);
    let _ = chat_state.broadcast_tx.send(join_message);

    // Listen for broadcasts
    let mut broadcast_rx = chat_state.broadcast_tx.subscribe();
    let ws_clone = ws.clone();
    tokio::spawn(async move {
        while let Ok(message) = broadcast_rx.recv().await {
            if let Err(_) = ws_clone.send_text(message).await {
                break; // Connection closed
            }
        }
    });

    // Handle incoming messages
    while let Some(message) = ws.recv().await {
        match message {
            Message::Text(text) => {
                // Parse message (could be JSON)
                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    let formatted_msg = format!("{}: {}", user.name, chat_msg.content);
                    let _ = chat_state.broadcast_tx.send(formatted_msg);
                }
            }
            Message::Close(_) => {
                println!("User {} disconnected", user.name);
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    {
        let mut clients = chat_state.clients.lock().unwrap();
        clients.remove(&client_id);
    }

    let leave_message = format!("{} left the chat", user.name);
    let _ = chat_state.broadcast_tx.send(leave_message);

    Ok(())
}

#[cfg(feature = "websocket")]
let router = Router::new()
    .state(chat_state)
    .websocket("/chat", authenticated_chat_handler);
```

### WebSocket Room System

```rust
#[cfg(feature = "websocket")]
#[derive(Clone)]
struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
}

#[cfg(feature = "websocket")]
struct Room {
    clients: HashMap<String, WebSocketConnection>,
    broadcast_tx: broadcast::Sender<RoomMessage>,
}

#[cfg(feature = "websocket")]
async fn room_websocket_handler(
    Path(room_id): Path<String>,
    ws: WebSocketConnection,
    headers: Headers,
    State(room_manager): State<RoomManager>
) -> Result<()> {
    let user = authenticate_websocket_user(&headers).await?;
    let client_id = uuid::Uuid::new_v4().to_string();

    // Join room
    let mut rooms = room_manager.rooms.write().await;
    let room = rooms.entry(room_id.clone()).or_insert_with(|| {
        let (tx, _) = broadcast::channel(1000);
        Room {
            clients: HashMap::new(),
            broadcast_tx: tx,
        }
    });

    room.clients.insert(client_id.clone(), ws.clone());

    // Subscribe to room broadcasts
    let mut room_rx = room.broadcast_tx.subscribe();
    let ws_clone = ws.clone();
    tokio::spawn(async move {
        while let Ok(room_msg) = room_rx.recv().await {
            if room_msg.sender_id != client_id {
                let _ = ws_clone.send_json(&room_msg).await;
            }
        }
    });

    drop(rooms); // Release write lock

    // Handle messages
    while let Some(message) = ws.recv().await {
        match message {
            Message::Text(text) => {
                if let Ok(user_msg) = serde_json::from_str::<UserMessage>(&text) {
                    let room_msg = RoomMessage {
                        sender_id: client_id.clone(),
                        sender_name: user.name.clone(),
                        content: user_msg.content,
                        timestamp: chrono::Utc::now(),
                        room_id: room_id.clone(),
                    };

                    // Broadcast to room
                    let rooms = room_manager.rooms.read().await;
                    if let Some(room) = rooms.get(&room_id) {
                        let _ = room.broadcast_tx.send(room_msg);
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup
    let mut rooms = room_manager.rooms.write().await;
    if let Some(room) = rooms.get_mut(&room_id) {
        room.clients.remove(&client_id);
        if room.clients.is_empty() {
            rooms.remove(&room_id);
        }
    }

    Ok(())
}

#[cfg(feature = "websocket")]
let router = Router::new()
    .state(room_manager)
    .websocket("/rooms/:room_id", room_websocket_handler);
```

## Performance Considerations

### Router Mode Performance Impact

The choice of router mode significantly affects application performance:

```rust
// Radix mode (recommended) - O(log n) lookup
let radix_router = Router::new()
    .with_mode(RouterMode::Radix); // Default

// Regex mode (legacy) - O(n) lookup
let regex_router = Router::new()
    .with_mode(RouterMode::Base);

// Performance comparison for 1000 routes:
// Radix: ~10-50 nanoseconds per lookup
// Regex: ~1-10 microseconds per lookup (100x slower)
```

### Route Organization for Performance

```rust
// Efficient route organization (specific to general)
let router = Router::new()
    // Most specific routes first (handled by router automatically in Radix mode)
    .get("/api/v1/users/profile", get_user_profile)
    .get("/api/v1/users/settings", get_user_settings)
    .get("/api/v1/users/:id/posts", get_user_posts)
    .get("/api/v1/users/:id", get_user)
    .get("/api/v1/users", list_users)

    // Wildcard routes last
    .get("/static/*path", serve_static_files);
```

### Route Caching and Optimization

```rust
// Enable route caching for better performance
let router = Router::new()
    .get("/expensive/:id", |Path(id): Path<u32>| async move {
        // Expensive computation cached at application level
        static CACHE: Lazy<Mutex<HashMap<u32, String>>> =
            Lazy::new(|| Mutex::new(HashMap::new()));

        let mut cache = CACHE.lock().unwrap();
        if let Some(result) = cache.get(&id) {
            return Ok(Response::text(result.clone()));
        }

        // Expensive computation
        let result = perform_expensive_computation(id).await?;
        cache.insert(id, result.clone());

        Ok(Response::text(result))
    });

// Route statistics and monitoring
let router = Router::new()
    .get("/stats/routes", || async {
        let stats = router.stats(); // Get router statistics
        Ok(Response::json(stats)?)
    });
```

### Memory-Efficient Route Patterns

```rust
// Use Arc for shared data to avoid cloning
#[derive(Clone)]
struct SharedData {
    config: Arc<AppConfig>,
    templates: Arc<HashMap<String, String>>,
}

// Efficient parameter extraction
let router = Router::new()
    .get("/efficient/:id", |Path(id): Path<u32>| async move {
        // Direct parameter usage without unnecessary allocations
        if id == 0 {
            return Ok(Response::new(ignitia::StatusCode::BAD_REQUEST));
        }

        Ok(Response::text(format!("ID: {}", id)))
    });
```

## Best Practices

### 1. Route Organization and Structure

```rust
// Organize routes by feature/domain
mod user_routes {
    use super::*;

    pub fn routes() -> Router {
        Router::new()
            .get("/", list_users)
            .post("/", create_user)
            .get("/:id", get_user)
            .put("/:id", update_user)
            .delete("/:id", delete_user)
            .get("/:id/profile", get_user_profile)
            .put("/:id/profile", update_user_profile)
            .middleware(UserAuthMiddleware::new())
    }

    // Handler implementations...
    async fn list_users(Query(params): Query<ListUsersQuery>) -> Result<Response> {
        // Implementation
    }
}

mod post_routes {
    use super::*;

    pub fn routes() -> Router {
        Router::new()
            .get("/", list_posts)
            .post("/", create_post)
            .get("/:id", get_post)
            .put("/:id", update_post)
            .delete("/:id", delete_post)
            .get("/:id/comments", get_post_comments)
            .post("/:id/comments", create_comment)
    }
}

// Main router assembly
fn build_app_router() -> Router {
    Router::new()
        .get("/", homepage)
        .get("/health", health_check)
        .nest("/users", user_routes::routes())
        .nest("/posts", post_routes::routes())
        .middleware(GlobalLoggingMiddleware::new())
        .middleware(CorsMiddleware::permissive()) // Configure appropriately
}
```

### 2. Comprehensive Error Handling

```rust
// Custom error types for different domains
define_error!(UserError {
    NotFound(404, "user_not_found"),
    InvalidEmail(400, "invalid_email"),
    EmailTaken(409, "email_already_taken"),
});

define_error!(PostError {
    NotFound(404, "post_not_found"),
    InvalidContent(400, "invalid_content"),
    Unauthorized(403, "post_access_denied"),
});

// Centralized error handling
async fn safe_user_handler(Path(id): Path<u32>) -> Result<Response> {
    if id == 0 {
        return Err(UserError::InvalidId.into());
    }

    let user = get_user_by_id(id).await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => UserError::NotFound,
            ErrorKind::Database => Error::Database(e.to_string()),
            _ => Error::Internal(e.to_string()),
        })?;

    Ok(Response::json(user)?)
}

// Global error middleware
let router = Router::new()
    .middleware(ErrorHandlerMiddleware::new()
        .with_custom_error_page(404, include_str!("../templates/404.html"))
        .with_error_logging(true)
        .with_detailed_errors(cfg!(debug_assertions)))
    .get("/users/:id", safe_user_handler);
```

### 3. Input Validation and Security

```rust
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 2, max = 100))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(length(min = 8), custom = "validate_password_strength")]
    password: String,

    #[validate(range(min = 13, max = 120))]
    age: Option<u8>,
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    if has_lowercase && has_uppercase && has_digit && has_special {
        Ok(())
    } else {
        Err(ValidationError::new("password_too_weak"))
    }
}

async fn create_user_handler(Json(req): Json<CreateUserRequest>) -> Result<Response> {
    // Validation happens automatically via serde + validator
    req.validate()
        .map_err(|e| Error::Validation(format!("Invalid input: {}", e)))?;

    // Additional business logic validation
    if is_email_blacklisted(&req.email).await? {
        return Err(Error::BadRequest("Email domain not allowed".to_string()));
    }

    let user = create_user(req).await?;
    Ok(Response::json(user)?)
}

// Rate limiting and security middleware
let router = Router::new()
    .middleware(SecurityMiddleware::new()
        .add_security_headers()
        .prevent_xss()
        .prevent_csrf())
    .middleware(RateLimitingMiddleware::new()
        .requests_per_minute(100)
        .per_ip()
        .with_whitelist(vec!["127.0.0.1", "::1"]))
    .post("/users", create_user_handler);
```

### 4. API Documentation and OpenAPI

```rust
// Document your routes for OpenAPI generation
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
///
/// # Example Response
/// ```json
/// {
///   "id": 123,
///   "name": "John Doe",
///   "email": "john@example.com",
///   "created_at": "2023-01-15T10:30:00Z"
/// }
///```
async fn get_user(Path(id): Path<u32>) -> Result<Response> {
    // Implementation with proper error handling
    if id == 0 {
        return Err(Error::BadRequest("User ID must be positive".to_string()));
    }

    let user = fetch_user_by_id(id).await
        .ok_or_else(|| Error::NotFound(format!("User {} not found", id)))?;

    Ok(Response::json(user)?)
}
```

### 5. Testing Routes

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ignitia::test::{TestRequest, TestRouter};
    use serde_json::json;

    #[tokio::test]
    async fn test_get_user_success() {
        let router = create_test_router().await;

        let response = TestRequest::get("/users/1")
            .header("authorization", "Bearer test-token")
            .send(&router)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let user: User = response.json().await.expect("Valid JSON response");
        assert_eq!(user.id, 1);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let router = create_test_router().await;

        let response = TestRequest::get("/users/999")
            .send(&router)
            .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_user_validation() {
        let router = create_test_router().await;

        let invalid_request = json!({
            "name": "", // Invalid: too short
            "email": "not-an-email", // Invalid: not an email
            "password": "123" // Invalid: too short
        });

        let response = TestRequest::post("/users")
            .json(&invalid_request)
            .send(&router)
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_middleware_authentication() {
        let router = create_test_router().await;

        // Request without authentication
        let response = TestRequest::get("/admin/dashboard")
            .send(&router)
            .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Request with valid authentication
        let response = TestRequest::get("/admin/dashboard")
            .header("authorization", "Bearer admin-token")
            .send(&router)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    async fn create_test_router() -> Router {
        let test_state = create_test_state().await;

        Router::new()
            .state(test_state)
            .get("/users/:id", get_user)
            .post("/users", create_user_handler)
            .middleware(AuthMiddleware::new("test-secret"))
            .get("/admin/dashboard", admin_dashboard)
    }
}
```

### 6. Production Deployment Considerations

```rust
// Production-ready router configuration
fn create_production_router(config: &AppConfig) -> Router {
    let mut router = Router::new()
        // Use Radix mode for best performance
        .with_mode(RouterMode::Radix)

        // Global middleware stack (order matters!)
        .middleware(RequestIdMiddleware::new()) // First: request tracking
        .middleware(LoggerMiddleware::json_format()) // Structured logging
        .middleware(SecurityMiddleware::production()) // Security headers
        .middleware(CorsMiddleware::from_config(&config.cors)) // CORS configuration
        .middleware(RateLimitingMiddleware::from_config(&config.rate_limit)) // Rate limiting
        .middleware(CompressionMiddleware::new()) // Response compression
        .middleware(ErrorHandlerMiddleware::production()) // Last: error handling

        // Health and monitoring endpoints
        .get("/health", health_check)
        .get("/metrics", prometheus_metrics)
        .get("/ready", readiness_check)

        // API routes
        .nest("/api/v1", build_api_v1_routes(config))
        .nest("/api/v2", build_api_v2_routes(config));

    // Add admin routes only if enabled
    if config.admin_enabled {
        router = router.nest("/admin", build_admin_routes(config));
    }

    // Add debug routes only in development
    if config.environment == "development" {
        router = router.nest("/debug", build_debug_routes());
    }

    router
}

// Environment-specific configurations
impl AppConfig {
    fn for_production() -> Self {
        Self {
            environment: "production".to_string(),
            log_level: "info".to_string(),
            cors: CorsConfig::strict(),
            rate_limit: RateLimitConfig::production(),
            admin_enabled: false,
            metrics_enabled: true,
        }
    }

    fn for_development() -> Self {
        Self {
            environment: "development".to_string(),
            log_level: "debug".to_string(),
            cors: CorsConfig::permissive(),
            rate_limit: RateLimitConfig::development(),
            admin_enabled: true,
            metrics_enabled: true,
        }
    }
}
```

## Migration Guide

### Migrating from Base to Radix Mode

The migration from Base (regex) to Radix mode is straightforward and backward-compatible:

#### Step 1: Update Router Configuration

```rust
// Before (Base mode)
let router = Router::new()
    .with_mode(RouterMode::Base) // Remove this line
    .get("/users/:id", get_user);

// After (Radix mode - default)
let router = Router::new()
    // .with_mode(RouterMode::Radix) // Optional - it's the default
    .get("/users/:id", get_user);
```

#### Step 2: No Route Definition Changes Required

All existing route patterns work identically:

```rust
// These patterns work in both modes
.get("/users/:id", handler)
.get("/posts/:slug", handler)
.get("/api/:version/users/:user_id/posts/:post_id", handler)
.get("/files/*path", handler)
```

#### Step 3: Performance Testing

```rust
// Test performance with both modes
#[cfg(test)]
mod migration_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_router_modes() {
        let routes = create_test_routes(1000); // 1000 test routes

        // Test Base mode
        let base_router = Router::new()
            .with_mode(RouterMode::Base);
        let base_router = add_routes(base_router, &routes);

        let start = Instant::now();
        for _ in 0..10000 {
            let _ = base_router.matches(Method::GET, "/api/v1/users/123");
        }
        let base_time = start.elapsed();

        // Test Radix mode
        let radix_router = Router::new()
            .with_mode(RouterMode::Radix);
        let radix_router = add_routes(radix_router, &routes);

        let start = Instant::now();
        for _ in 0..10000 {
            let _ = radix_router.matches(Method::GET, "/api/v1/users/123");
        }
        let radix_time = start.elapsed();

        println!("Base mode: {:?}, Radix mode: {:?}", base_time, radix_time);
        println!("Performance improvement: {:.2}x",
                 base_time.as_nanos() as f64 / radix_time.as_nanos() as f64);
    }
}
```

#### Step 4: Monitor in Production

```rust
// Add monitoring to track migration impact
let router = Router::new()
    .with_mode(RouterMode::Radix)
    .middleware(RouterStatsMiddleware::new()) // Track performance metrics
    .get("/metrics/router", || async {
        let stats = get_router_stats();
        Ok(Response::json(stats)?)
    });
```

### Breaking Changes (None Expected)

The migration from Base to Radix mode should be completely backward-compatible. However, if you encounter any issues:

1. **Check route patterns**: Ensure all routes follow standard patterns
2. **Verify parameter extraction**: Test parameter extraction still works
3. **Test wildcard routes**: Ensure wildcard patterns work as expected
4. **Monitor performance**: Verify the performance improvement

### Rollback Plan

If issues arise, rollback is simple:

```rust
// Rollback to Base mode
let router = Router::new()
    .with_mode(RouterMode::Base) // Add this line back
    .get("/users/:id", get_user);
```

---

**This routing guide covers all aspects of Ignitia's powerful routing system. For more specific examples and use cases, refer to the [Examples Documentation](EXAMPLES.md) and [API Reference](https://docs.rs/ignitia).**

**🔥 Ready to ignite your web development with high-performance routing? Start building with Ignitia today!**
