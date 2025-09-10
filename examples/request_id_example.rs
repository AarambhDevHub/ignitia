use ignitia::{
    CompressionMiddleware, Error, LoggerMiddleware, RequestIdMiddleware, Response, Result, Router,
    Server,
};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .init();

    let router = Router::new()
        // Request ID must come first for proper propagation
        .middleware(RequestIdMiddleware::new())
        // Logger middleware will automatically include request IDs
        .middleware(LoggerMiddleware)
        // Other middleware can access request ID from context
        .middleware(CompressionMiddleware::new())
        // Basic endpoint demonstrating request correlation
        .get("/", || async {
            info!("Processing home request");
            Ok(Response::text("Hello from Ignitia with Request ID! 🔥"))
        })
        // API endpoint that demonstrates distributed tracing
        .get("/api/users", || async {
            info!("Fetching users from database");

            // Simulate database call
            let users = fetch_users_from_db().await;

            info!(user_count = users.len(), "Users fetched successfully");
            Response::json(serde_json::json!({
                "users": users,
                "metadata": {
                    "total": users.len(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            }))
        })
        // Endpoint that demonstrates error correlation
        .get("/api/error", || async {
            info!("Simulating error scenario");
            Err(Error::Internal("Simulated error for tracing".to_string()))
        })
        // Health check with minimal overhead
        .get("/health", || async { Ok(Response::text("OK")) });

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let server = Server::new(router, addr);

    println!(
        "🔥 Ignitia server with Request ID tracing on http://{}",
        addr
    );
    println!("🔍 Test request correlation with:");
    println!("  curl -H 'X-Request-ID: my-custom-123' http://localhost:8080/api/users -v");
    println!("  curl http://localhost:8080/api/error -v");

    server
        .ignitia()
        .await
        .map_err(|e| ignitia::Error::Internal(e.to_string()))
}

async fn fetch_users_from_db() -> Vec<serde_json::Value> {
    // Simulate async database operation
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    info!("Database query executed");

    vec![
        serde_json::json!({"id": 1, "name": "Alice", "email": "alice@example.com"}),
        serde_json::json!({"id": 2, "name": "Bob", "email": "bob@example.com"}),
    ]
}
