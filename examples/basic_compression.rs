use ignitia::{CompressionMiddleware, LoggerMiddleware, Response, Result, Router, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt().init();

    let router = Router::new()
        .middleware(LoggerMiddleware)
        // Add compression middleware
        .middleware(CompressionMiddleware::new())
        // Basic text endpoint
        .get("/", || async {
            Ok(Response::text("Hello from Ignitia! 🔥"))
        })
        // JSON endpoint that will benefit from compression
        .get("/api/data", || async {
            let large_data = serde_json::json!({
                "message": "This is a large JSON response that will be compressed",
                "data": vec![1; 1000], // Large array to demonstrate compression
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "metadata": {
                    "version": "1.0",
                    "compressed": true,
                    "framework": "Ignitia"
                }
            });
            Response::json(large_data)
        })
        // Large text response
        .get("/large-text", || async {
            let large_text =
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
            Ok(Response::text(large_text))
        })
        // Small response (should not be compressed due to threshold)
        .get("/small", || async { Ok(Response::text("Small")) });

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let server = Server::new(router, addr);

    println!(
        "🔥 Ignitia server with compression running on http://{}",
        addr
    );
    println!("📊 Test compression with:");
    println!("   curl -H 'Accept-Encoding: gzip' http://localhost:8080/api/data -v");
    println!("   curl -H 'Accept-Encoding: br' http://localhost:8080/large-text -v");

    server
        .ignitia()
        .await
        .map_err(|e| ignitia::Error::Internal(e.to_string()))
}
