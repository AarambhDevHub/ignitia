# Static Files

This guide covers serving static files (CSS, JavaScript, images, etc.) with Ignitia.

## Table of Contents

- [Quick Start](#quick-start)
- [Static File Middleware](#static-file-middleware)
- [Manual File Serving](#manual-file-serving)
- [Configuration](#configuration)
- [Security Considerations](#security-considerations)
- [Performance Optimization](#performance-optimization)
- [Content Types](#content-types)
- [Caching](#caching)
- [Examples](#examples)

## Quick Start

Here's how to serve static files from a directory:

```rust
use ignitia::{Router, Server, Response};
use std::path::Path;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", || async { Ok(Response::html(include_str!("../static/index.html"))) })
        // Serve static files from /static/* routes
        .get("/static/*path", |Path(path): Path<String>| async move {
            serve_static_file(&path).await
        });

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn serve_static_file(path: &str) -> ignitia::Result<Response> {
    let file_path = format!("static/{}", path);

    // Security: Prevent directory traversal
    if path.contains("..") {
        return Err(ignitia::Error::forbidden());
    }

    match fs::read(&file_path).await {
        Ok(contents) => {
            let content_type = guess_content_type(&file_path);
            Ok(Response::new(ignitia::StatusCode::OK)
                .with_body(contents)
                .header("content-type", content_type))
        }
        Err(_) => Err(ignitia::Error::not_found(&file_path)),
    }
}
```

## Static File Middleware

Create reusable middleware for serving static files:

```rust
use ignitia::{middleware::Middleware, Request, Response, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct StaticFileMiddleware {
    root_dir: PathBuf,
    url_prefix: String,
    index_file: Option<String>,
    cache_control: Option<String>,
}

impl StaticFileMiddleware {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            url_prefix: "/static".to_string(),
            index_file: Some("index.html".to_string()),
            cache_control: Some("public, max-age=3600".to_string()),
        }
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.url_prefix = prefix.to_string();
        self
    }

    pub fn with_index_file(mut self, index: &str) -> Self {
        self.index_file = Some(index.to_string());
        self
    }

    pub fn with_cache_control(mut self, cache_control: &str) -> Self {
        self.cache_control = Some(cache_control.to_string());
        self
    }

    async fn serve_file(&self, file_path: &Path) -> Result<Response> {
        let contents = fs::read(file_path).await
            .map_err(|_| ignitia::Error::not_found(file_path.to_string_lossy()))?;

        let content_type = guess_content_type(file_path.to_string_lossy().as_ref());
        let mut response = Response::new(ignitia::StatusCode::OK)
            .with_body(contents)
            .header("content-type", content_type);

        if let Some(cache_control) = &self.cache_control {
            response = response.header("cache-control", cache_control);
        }

        Ok(response)
    }
}

#[async_trait::async_trait]
impl Middleware for StaticFileMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let path = req.uri.path();

        if !path.starts_with(&self.url_prefix) {
            return Ok(());
        }

        // Remove prefix and get relative path
        let relative_path = &path[self.url_prefix.len()..];
        let relative_path = relative_path.trim_start_matches('/');

        // Security check
        if relative_path.contains("..") || relative_path.contains("//") {
            return Err(ignitia::Error::forbidden());
        }

        let mut file_path = self.root_dir.join(relative_path);

        // Handle directory requests with index file
        if file_path.is_dir() {
            if let Some(index) = &self.index_file {
                file_path = file_path.join(index);
            }
        }

        if file_path.exists() && file_path.is_file() {
            let response = self.serve_file(&file_path).await?;
            // In a real middleware implementation, you'd return the response
            // This is a simplified example
        }

        Ok(())
    }
}

// Usage
let router = Router::new()
    .middleware(StaticFileMiddleware::new("./assets")
        .with_prefix("/assets")
        .with_cache_control("public, max-age=86400"))
    .get("/", || async { Ok(Response::text("Hello World")) });
```

## Manual File Serving

For more control, serve files manually:

```rust
use ignitia::{Router, Response, Path};
use std::path::PathBuf;

async fn serve_css(Path(filename): Path<String>) -> ignitia::Result<Response> {
    let file_path = PathBuf::from("assets/css").join(&filename);

    // Validate file extension
    if !filename.ends_with(".css") {
        return Err(ignitia::Error::bad_request("Invalid file type"));
    }

    let contents = tokio::fs::read(&file_path).await
        .map_err(|_| ignitia::Error::not_found(&filename))?;

    Ok(Response::new(ignitia::StatusCode::OK)
        .with_body(contents)
        .header("content-type", "text/css; charset=utf-8")
        .header("cache-control", "public, max-age=31536000"))
}

async fn serve_js(Path(filename): Path<String>) -> ignitia::Result<Response> {
    let file_path = PathBuf::from("assets/js").join(&filename);

    if !filename.ends_with(".js") {
        return Err(ignitia::Error::bad_request("Invalid file type"));
    }

    let contents = tokio::fs::read(&file_path).await
        .map_err(|_| ignitia::Error::not_found(&filename))?;

    Ok(Response::new(ignitia::StatusCode::OK)
        .with_body(contents)
        .header("content-type", "application/javascript; charset=utf-8")
        .header("cache-control", "public, max-age=31536000"))
}

// Router setup
let router = Router::new()
    .get("/css/:filename", serve_css)
    .get("/js/:filename", serve_js)
    .get("/images/:filename", serve_image);
```

## Configuration

### Static File Server Configuration

```rust
#[derive(Debug, Clone)]
pub struct StaticConfig {
    /// Root directory for static files
    pub root_dir: PathBuf,
    /// URL prefix (e.g., "/static", "/assets")
    pub url_prefix: String,
    /// Default file to serve for directories
    pub index_file: Option<String>,
    /// Maximum file size to serve (bytes)
    pub max_file_size: usize,
    /// Cache control header value
    pub cache_control: Option<String>,
    /// Enable gzip compression for text files
    pub enable_compression: bool,
    /// List of allowed file extensions
    pub allowed_extensions: Option<Vec<String>>,
    /// Enable directory listing
    pub directory_listing: bool,
}

impl Default for StaticConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("./static"),
            url_prefix: "/static".to_string(),
            index_file: Some("index.html".to_string()),
            max_file_size: 10 * 1024 * 1024, // 10MB
            cache_control: Some("public, max-age=3600".to_string()),
            enable_compression: true,
            allowed_extensions: None, // Allow all by default
            directory_listing: false,
        }
    }
}

impl StaticConfig {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.url_prefix = prefix.to_string();
        self
    }

    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn allow_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.allowed_extensions = Some(extensions.into_iter().map(String::from).collect());
        self
    }
}
```

## Security Considerations

### Path Traversal Prevention

```rust
use std::path::{Path, Component};

fn is_safe_path(path: &str) -> bool {
    let path = Path::new(path);

    // Check for directory traversal attempts
    for component in path.components() {
        match component {
            Component::ParentDir => return false,
            Component::CurDir => return false,
            Component::Normal(_) => continue,
            _ => return false,
        }
    }

    true
}

async fn secure_file_handler(Path(requested_path): Path<String>) -> ignitia::Result<Response> {
    // Validate path
    if !is_safe_path(&requested_path) {
        return Err(ignitia::Error::forbidden());
    }

    // Normalize path
    let file_path = std::path::PathBuf::from("static")
        .join(&requested_path)
        .canonicalize()
        .map_err(|_| ignitia::Error::not_found(&requested_path))?;

    // Ensure file is within allowed directory
    let static_dir = std::path::PathBuf::from("static")
        .canonicalize()
        .map_err(|_| ignitia::Error::internal("Static directory not found"))?;

    if !file_path.starts_with(&static_dir) {
        return Err(ignitia::Error::forbidden());
    }

    serve_file(&file_path).await
}
```

### File Extension Validation

```rust
const ALLOWED_EXTENSIONS: &[&str] = &[
    "html", "css", "js", "png", "jpg", "jpeg", "gif", "svg",
    "ico", "woff", "woff2", "ttf", "eot", "json", "xml", "txt"
];

fn is_allowed_extension(path: &str) -> bool {
    if let Some(extension) = Path::new(path).extension() {
        if let Some(ext_str) = extension.to_str() {
            return ALLOWED_EXTENSIONS.contains(&ext_str.to_lowercase().as_str());
        }
    }
    false
}
```

## Performance Optimization

### Content Compression

```rust
use ignitia::middleware::CompressionMiddleware;

let router = Router::new()
    .middleware(CompressionMiddleware::new()
        .with_threshold(1024)
        .with_compressible_types(vec![
            "text/css",
            "text/javascript",
            "application/javascript",
            "text/html",
            "image/svg+xml"
        ]))
    .get("/static/*path", static_file_handler);
```

### Efficient File Reading

```rust
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

async fn efficient_file_read(file_path: &Path) -> ignitia::Result<bytes::Bytes> {
    let file = File::open(file_path).await
        .map_err(|_| ignitia::Error::not_found(file_path.to_string_lossy()))?;

    let metadata = file.metadata().await
        .map_err(|_| ignitia::Error::internal("Failed to read file metadata"))?;

    // Use buffered reading for large files
    if metadata.len() > 1024 * 1024 { // 1MB
        let mut reader = BufReader::new(file);
        let mut contents = Vec::with_capacity(metadata.len() as usize);
        reader.read_to_end(&mut contents).await
            .map_err(|_| ignitia::Error::internal("Failed to read file"))?;
        Ok(bytes::Bytes::from(contents))
    } else {
        let contents = tokio::fs::read(file_path).await
            .map_err(|_| ignitia::Error::not_found(file_path.to_string_lossy()))?;
        Ok(bytes::Bytes::from(contents))
    }
}
```

## Content Types

### MIME Type Detection

```rust
use mime_guess::from_path;

fn guess_content_type(file_path: &str) -> &'static str {
    match from_path(file_path).first_or_octet_stream().as_ref() {
        "text/html" => "text/html; charset=utf-8",
        "text/css" => "text/css; charset=utf-8",
        "application/javascript" => "application/javascript; charset=utf-8",
        "application/json" => "application/json; charset=utf-8",
        "text/plain" => "text/plain; charset=utf-8",
        other => other,
    }
}

// Custom content type mapping
fn get_content_type(file_path: &str) -> &'static str {
    match std::path::Path::new(file_path).extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase().as_str())
    {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("eot") => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}
```

## Caching

### HTTP Caching Headers

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn add_cache_headers(response: Response, file_path: &Path) -> ignitia::Result<Response> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|_| ignitia::Error::internal("Failed to read file metadata"))?;

    let modified = metadata.modified()
        .map_err(|_| ignitia::Error::internal("Failed to get modification time"))?;

    let timestamp = modified.duration_since(UNIX_EPOCH)
        .map_err(|_| ignitia::Error::internal("Invalid modification time"))?
        .as_secs();

    // Generate ETag
    let etag = format!("\"{}\"", timestamp);

    // Set cache headers
    Ok(response
        .header("etag", &etag)
        .header("last-modified", httpdate::fmt_http_date(modified))
        .header("cache-control", "public, max-age=3600"))
}

async fn cached_file_handler(
    req: &Request,
    file_path: &Path
) -> ignitia::Result<Response> {
    let metadata = tokio::fs::metadata(file_path).await
        .map_err(|_| ignitia::Error::not_found(file_path.to_string_lossy()))?;

    let modified = metadata.modified()
        .map_err(|_| ignitia::Error::internal("Failed to get modification time"))?;

    // Check If-Modified-Since header
    if let Some(if_modified_since) = req.header("if-modified-since") {
        if let Ok(client_time) = httpdate::parse_http_date(if_modified_since) {
            if modified <= client_time {
                return Ok(Response::new(ignitia::StatusCode::NOT_MODIFIED));
            }
        }
    }

    // Check If-None-Match header (ETag)
    let etag = format!("\"{}\"", modified.duration_since(UNIX_EPOCH).unwrap().as_secs());
    if let Some(if_none_match) = req.header("if-none-match") {
        if if_none_match == etag {
            return Ok(Response::new(ignitia::StatusCode::NOT_MODIFIED));
        }
    }

    // Serve file with cache headers
    let contents = tokio::fs::read(file_path).await
        .map_err(|_| ignitia::Error::not_found(file_path.to_string_lossy()))?;

    let content_type = guess_content_type(&file_path.to_string_lossy());

    Ok(Response::new(ignitia::StatusCode::OK)
        .with_body(contents)
        .header("content-type", content_type)
        .header("etag", &etag)
        .header("last-modified", httpdate::fmt_http_date(modified))
        .header("cache-control", "public, max-age=3600"))
}
```

## Examples

### Complete Static File Server

```rust
use ignitia::{Router, Server, Response, Path, Request};
use std::path::PathBuf;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        // Serve main page
        .get("/", || async {
            Ok(Response::html(include_str!("../static/index.html")))
        })

        // API routes
        .get("/api/health", || async {
            Ok(Response::json(serde_json::json!({"status": "ok"}))?)
        })

        // Static assets with caching
        .get("/assets/*path", |req: Request, Path(path): Path<String>| async move {
            serve_static_with_cache(req, &path).await
        })

        // Favicon
        .get("/favicon.ico", || async {
            serve_favicon().await
        });

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}

async fn serve_static_with_cache(req: Request, path: &str) -> ignitia::Result<Response> {
    // Security validation
    if !is_safe_path(path) {
        return Err(ignitia::Error::forbidden());
    }

    let file_path = PathBuf::from("static/assets").join(path);

    // Check if file exists
    if !file_path.exists() || !file_path.is_file() {
        return Err(ignitia::Error::not_found(path));
    }

    // Serve with caching
    cached_file_handler(&req, &file_path).await
}

async fn serve_favicon() -> ignitia::Result<Response> {
    let favicon_path = "static/favicon.ico";
    let contents = fs::read(favicon_path).await
        .map_err(|_| ignitia::Error::not_found("favicon.ico"))?;

    Ok(Response::new(ignitia::StatusCode::OK)
        .with_body(contents)
        .header("content-type", "image/x-icon")
        .header("cache-control", "public, max-age=86400"))
}
```

### SPA (Single Page Application) Support

```rust
async fn spa_handler(Path(path): Path<String>) -> ignitia::Result<Response> {
    let file_path = PathBuf::from("dist").join(&path);

    // Try to serve the requested file
    if file_path.exists() && file_path.is_file() {
        let contents = fs::read(&file_path).await
            .map_err(|_| ignitia::Error::not_found(&path))?;

        let content_type = guess_content_type(&path);
        return Ok(Response::new(ignitia::StatusCode::OK)
            .with_body(contents)
            .header("content-type", content_type));
    }

    // Fallback to index.html for SPA routing
    let index_path = PathBuf::from("dist/index.html");
    let contents = fs::read(&index_path).await
        .map_err(|_| ignitia::Error::not_found("index.html"))?;

    Ok(Response::html(String::from_utf8_lossy(&contents).to_string()))
}

// Router setup for SPA
let router = Router::new()
    .get("/api/*path", api_handler)
    .get("/*path", spa_handler);
```

### Development vs Production

```rust
#[cfg(debug_assertions)]
async fn dev_static_handler(Path(path): Path<String>) -> ignitia::Result<Response> {
    // In development: serve files directly with no caching
    let file_path = PathBuf::from("src/static").join(&path);
    let contents = fs::read(&file_path).await
        .map_err(|_| ignitia::Error::not_found(&path))?;

    let content_type = guess_content_type(&path);
    Ok(Response::new(ignitia::StatusCode::OK)
        .with_body(contents)
        .header("content-type", content_type)
        .header("cache-control", "no-cache"))
}

#[cfg(not(debug_assertions))]
async fn prod_static_handler(req: Request, Path(path): Path<String>) -> ignitia::Result<Response> {
    // In production: use aggressive caching
    serve_static_with_cache(req, &path).await
        .map(|response| response.header("cache-control", "public, max-age=31536000"))
}
```

This documentation provides comprehensive coverage of static file serving in Ignitia, from basic examples to advanced features like caching, security, and performance optimization.
