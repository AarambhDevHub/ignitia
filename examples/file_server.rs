use mime_guess;
use mini_web_framework::{
    Error, Request, Response, ResponseBuilder, Result, Router, Server, handler_fn,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::fs;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .get("/", handler_fn(serve_index))
        .get("/*path", handler_fn(serve_file)); // This now works!

    let addr: SocketAddr = "127.0.0.1:3003".parse().unwrap();
    let server = Server::new(router, addr);

    println!("File server running on http://{}", addr);
    println!("Serving files from ./static directory");
    println!("Try: http://127.0.0.1:3003/example.html");

    server.run().await.unwrap();
    Ok(())
}

async fn serve_index(_req: Request) -> Result<Response> {
    match serve_static_file("index.html").await {
        Ok(response) => Ok(response),
        Err(_) => Ok(Response::html(
            r#"
                <h1>File Server</h1>
                <p>Static file server is running!</p>
                <p>Place files in the <code>./static</code> directory to serve them.</p>
                <p>Try: <a href="/example.html">example.html</a></p>
                <p>Or: <a href="/css/style.css">css/style.css</a></p>
            "#,
        )),
    }
}

async fn serve_file(req: Request) -> Result<Response> {
    let path = req.param("path").map(|s| s.as_str()).unwrap_or("");

    match serve_static_file(path).await {
        Ok(response) => Ok(response),
        Err(_) => Ok(Response::not_found()),
    }
}

async fn serve_static_file(file_path: &str) -> Result<Response> {
    // Handle empty path
    if file_path.is_empty() {
        return Err(Error::NotFound);
    }

    let mut path = PathBuf::from("static");
    path.push(file_path);

    // Security: prevent directory traversal
    let canonical = path.canonicalize().map_err(|_| Error::NotFound)?;
    let static_dir = std::env::current_dir().unwrap().join("static");
    if !canonical.starts_with(&static_dir) {
        return Err(Error::BadRequest("Invalid path".into()));
    }

    // Check if it's a directory
    if canonical.is_dir() {
        // Try to serve index.html from the directory
        let index_path = canonical.join("index.html");
        if index_path.exists() {
            let content = fs::read(&index_path).await.map_err(|_| Error::NotFound)?;
            let mime_type = "text/html";
            return Ok(ResponseBuilder::new()
                .header("Content-Type", mime_type)
                .body(content)
                .build());
        } else {
            return Err(Error::NotFound);
        }
    }

    let content = fs::read(&canonical).await.map_err(|_| Error::NotFound)?;
    let mime_type = mime_guess::from_path(&canonical)
        .first_or_octet_stream()
        .to_string();

    Ok(ResponseBuilder::new()
        .header("Content-Type", mime_type)
        .body(content)
        .build())
}
