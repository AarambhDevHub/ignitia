use ignitia::{
    middleware::{RateLimitConfig, RateLimitingMiddleware},
    raw_handler, Json, Path, Query, Request, Response, Result, Router, Server,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use tracing_subscriber;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize, Debug)]
struct UserParams {
    id: u32, // This MUST match the ":id" in your route exactly
}

#[derive(Deserialize, Debug)]
struct NameParams {
    name: String, // This MUST match the ":name" in your route exactly
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct UserQuery {
    limit: Option<u32>,
    page: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserQueryResponse {
    users: Vec<User>,
    query_params: UserQuery,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize rate limiting middleware
    let rate_limiting_middleware = RateLimitConfig::new(10, Duration::from_secs(60));

    // Create router using the new handler system
    let router = Router::new()
        .middleware(RateLimitingMiddleware::new(rate_limiting_middleware))
        .with_mode(ignitia::router::RouterMode::Radix)
        .get("/", home) // No extractors - works directly
        .get("/hello/:name", hello) // Uses Path extractor
        .get("/users", list_users) // Uses Query extractor
        .get("/users/:id", get_user) // Uses Path extractor
        .post("/users", create_user) // Uses Json extractor
        .get("/old-style", raw_handler(old_style_handler)); // For Request access
                                                            // .not_found(not_found);

    // Create and run server
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🔥 Server running on http://{}", addr);
    println!("Try these endpoints:");
    println!("  GET  /");
    println!("  GET  /hello/World");
    println!("  GET  /users");
    println!("  GET  /users/1");
    println!("  POST /users (with JSON body)");
    println!("  GET  /old-style");

    server.ignitia().await.unwrap();
    Ok(())
}

// Handler with no parameters - new style
async fn home() -> Result<Response> {
    Ok(Response::html(
        r#"
        <h1>🔥 Welcome to Mini Web Framework!</h1>
        <p>Try these endpoints:</p>
        <ul>
            <li><a href="/hello/World">GET /hello/World</a></li>
            <li><a href="/users">GET /users</a></li>
            <li><a href="/users/1">GET /users/1</a></li>
            <li><a href="/users?limit=5">GET /users?limit=5</a></li>
            <li>POST /users (send JSON body)</li>
            <li><a href="/old-style">GET /old-style (old Request style)</a></li>
        </ul>
    "#,
    ))
}

// Handler with path extractor - new style
async fn hello(Path(params): Path<NameParams>) -> Result<Response> {
    println!("Hello handler - extracted name: {}", params.name);
    Ok(Response::text(format!("Hello, {}!", params.name)))
}

// Handler with optional query parameters - new style
async fn list_users(Query(query): Query<UserQuery>) -> Result<Response> {
    println!("List users - query params: {:?}", query);

    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    // Apply pagination if specified
    let limit = query.limit.unwrap_or(users.len() as u32) as usize;
    let limited_users: Vec<User> = users.into_iter().take(limit).collect();

    Response::json(UserQueryResponse {
        users: limited_users,
        query_params: query,
    })
}

// Handler with path parameter extraction - new style
async fn get_user(Path(params): Path<UserParams>) -> Result<Response> {
    println!("Get user - extracted ID: {}", params.id);

    let user = User {
        id: params.id,
        name: format!("User {}", params.id),
        email: format!("user{}@example.com", params.id),
    };

    Response::json(user)
}

// Handler with JSON body extraction - new style
async fn create_user(Json(user): Json<User>) -> Result<Response> {
    println!("Creating user: {:?}", user.name);

    Response::json(serde_json::json!({
        "message": "User created successfully",
        "user": user
    }))
}

// Old-style handler that needs raw Request access
async fn old_style_handler(req: Request) -> Result<Response> {
    let path = req.uri.path();
    let method = req.method.to_string();

    Response::json(serde_json::json!({
        "message": "This is an old-style handler",
        "method": method,
        "path": path,
        "headers_count": req.headers.len()
    }))
}

// Handler with no parameters - new style
async fn not_found() -> Result<Response> {
    Ok(Response::html(
        r#"
        <h1>404 - Page Not Found</h1>
        <p>The page you're looking for doesn't exist.</p>
        <a href="/">Go back home</a>
    "#,
    ))
}
