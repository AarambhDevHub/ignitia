use ignite::{handler_fn, Request, Response, Result, Router, Server};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing_subscriber;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create router
    let router = Router::new()
        .get("/", handler_fn(home))
        .get("/hello/:name", handler_fn(hello))
        .get("/users", handler_fn(list_users))
        .get("/users/:id", handler_fn(get_user))
        .post("/users", handler_fn(create_user))
        .not_found(handler_fn(not_found));

    // Create and run server
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let server = Server::new(router, addr);

    println!("Server running on http://{}", addr);
    server.run().await.unwrap();

    Ok(())
}

async fn home(_req: Request) -> Result<Response> {
    Ok(Response::html(
        r#"
        <h1>Welcome to Mini Web Framework!</h1>
        <p>Try these endpoints:</p>
        <ul>
            <li><a href="/hello/World">GET /hello/:name</a></li>
            <li><a href="/users">GET /users</a></li>
            <li>GET /users/:id</li>
            <li>POST /users</li>
        </ul>
    "#,
    ))
}

async fn hello(req: Request) -> Result<Response> {
    // let name = req.param("name").unwrap_or(&"Unknown".to_string());
    let name = req.param("name").map(|s| s.as_str()).unwrap_or("Unknown");
    Ok(Response::text(format!("Hello, {}!", name)))
}

async fn list_users(_req: Request) -> Result<Response> {
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
    Response::json(users)
}

async fn get_user(req: Request) -> Result<Response> {
    let id: u32 = req.param("id").and_then(|s| s.parse().ok()).unwrap_or(0);

    let user = User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    };

    Response::json(user)
}

async fn create_user(req: Request) -> Result<Response> {
    let user: User = req.json()?;
    println!("Creating user: {:?}", user.name);
    Response::json(user)
}

async fn not_found(_req: Request) -> Result<Response> {
    Ok(Response::html(
        r#"
        <h1>404 - Page Not Found</h1>
        <p>The page you're looking for doesn't exist.</p>
        <a href="/">Go back home</a>
    "#,
    ))
}
