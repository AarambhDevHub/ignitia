use http::{HeaderName, HeaderValue}; // Add this import
use ignitia::{
    async_trait,
    middleware::{LoggerMiddleware, Middleware},
    Error, Request, Response, Result, Router, Server,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing_subscriber;

// Custom Rate Limiting Middleware
#[derive(Clone)]
struct RateLimitMiddleware {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_seconds),
        }
    }

    fn is_rate_limited(&self, client_ip: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        // Get or create request history for this IP
        let client_requests = requests
            .entry(client_ip.to_string())
            .or_insert_with(Vec::new);

        // Remove old requests outside the window
        client_requests.retain(|&timestamp| now.duration_since(timestamp) < self.window);

        // Check if we've exceeded the limit
        if client_requests.len() >= self.max_requests {
            return true;
        }

        // Add current request
        client_requests.push(now);
        false
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, _req: &mut Request) -> Result<()> {
        // Extract client IP (in real app, you'd get this from headers or connection)
        let client_ip = "127.0.0.1"; // Simplified for example

        if self.is_rate_limited(client_ip) {
            return Err(Error::BadRequest(
                "Rate limit exceeded. Try again later.".into(),
            ));
        }

        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // Add rate limit headers - FIXED
        res.headers.insert(
            HeaderName::from_static("x-ratelimit-limit"),
            HeaderValue::from_str(&format!("{}", self.max_requests)).unwrap(),
        );
        res.headers.insert(
            HeaderName::from_static("x-ratelimit-window"),
            HeaderValue::from_str(&format!("{}s", self.window.as_secs())).unwrap(),
        );
        Ok(())
    }
}

// Custom Request Validation Middleware
struct RequestValidationMiddleware;

#[async_trait]
impl Middleware for RequestValidationMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Validate Content-Type for POST requests
        if req.method == ignitia::Method::POST {
            if let Some(content_type) = req.header("Content-Type") {
                if !content_type.contains("application/json") {
                    return Err(Error::BadRequest(
                        "Content-Type must be application/json for POST requests".into(),
                    ));
                }
            } else {
                return Err(Error::BadRequest(
                    "Content-Type header required for POST requests".into(),
                ));
            }
        }

        // Validate User-Agent header exists
        if req.header("User-Agent").is_none() {
            return Err(Error::BadRequest("User-Agent header is required".into()));
        }

        // Add custom request metadata
        println!("✅ Request validated: {} {}", req.method, req.uri.path());
        Ok(())
    }
}

// Custom Security Headers Middleware
struct SecurityHeadersMiddleware;

#[async_trait]
impl Middleware for SecurityHeadersMiddleware {
    async fn after(&self, res: &mut Response) -> Result<()> {
        // Add security headers - FIXED
        let security_headers = [
            ("x-frame-options", "DENY"),
            ("x-content-type-options", "nosniff"),
            ("x-xss-protection", "1; mode=block"),
            ("referrer-policy", "strict-origin-when-cross-origin"),
            ("content-security-policy", "default-src 'self'"),
        ];

        for (name, value) in security_headers.iter() {
            res.headers.insert(
                HeaderName::from_static(name),
                HeaderValue::from_static(value),
            );
        }

        Ok(())
    }
}

// Custom Request/Response Time Tracking Middleware
struct TimingMiddleware {
    start_times: Arc<Mutex<HashMap<String, Instant>>>,
}

impl TimingMiddleware {
    pub fn new() -> Self {
        Self {
            start_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn generate_request_id(&self) -> String {
        // Simple request ID generation
        format!(
            "req_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }
}

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let request_id = self.generate_request_id();
        let start_time = Instant::now();

        // Store start time
        self.start_times
            .lock()
            .unwrap()
            .insert(request_id.clone(), start_time);

        // Add request ID to headers for tracking - FIXED
        req.headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(&request_id).unwrap(),
        );

        println!(
            "🕐 Request started: {} {} [ID: {}]",
            req.method,
            req.uri.path(),
            request_id
        );
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        // Clone the header value to avoid holding the immutable borrow
        let maybe_req_id = res.headers.get("x-request-id").cloned();

        if let Some(req_id_header) = maybe_req_id {
            if let Ok(request_id) = req_id_header.to_str() {
                let mut start_times = self.start_times.lock().unwrap();
                if let Some(start_time) = start_times.remove(request_id) {
                    let duration = start_time.elapsed();

                    // Now we can safely mutate headers
                    res.headers.insert(
                        HeaderName::from_static("x-response-time"),
                        HeaderValue::from_str(&format!("{}ms", duration.as_millis())).unwrap(),
                    );

                    println!(
                        "⏱️  Request completed in {}ms [ID: {}]",
                        duration.as_millis(),
                        request_id
                    );
                }
            }
        }
        Ok(())
    }
}

// Custom API Key Authentication Middleware
struct ApiKeyMiddleware {
    valid_keys: Vec<String>,
    protected_paths: Vec<String>,
}

impl ApiKeyMiddleware {
    pub fn new() -> Self {
        Self {
            valid_keys: vec![
                "api_key_123".to_string(),
                "api_key_456".to_string(),
                "api_key_789".to_string(),
            ],
            protected_paths: Vec::new(),
        }
    }

    pub fn protect_path(mut self, path: impl Into<String>) -> Self {
        self.protected_paths.push(path.into());
        self
    }

    fn requires_auth(&self, path: &str) -> bool {
        self.protected_paths.iter().any(|p| path.starts_with(p))
    }
}

#[async_trait]
impl Middleware for ApiKeyMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let path = req.uri.path();

        // Only check API key for protected paths
        if !self.requires_auth(path) {
            return Ok(());
        }

        // Check for API key in query params first
        if let Some(api_key) = req.query("api_key") {
            if self.valid_keys.contains(api_key) {
                println!("✅ Valid API key from query param");
                return Ok(());
            }
        }

        // Check for API key in headers
        if let Some(api_key) = req.header("X-API-Key") {
            if self.valid_keys.contains(&api_key.to_string()) {
                println!("✅ Valid API key from header");
                return Ok(());
            }
        }

        println!("❌ Invalid or missing API key");
        Err(Error::Unauthorized)
    }
}

// Main application
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        // Add custom middleware in order
        .middleware(LoggerMiddleware)
        .middleware(TimingMiddleware::new())
        .middleware(SecurityHeadersMiddleware)
        .middleware(RateLimitMiddleware::new(5, 60)) // 5 requests per minute
        .middleware(RequestValidationMiddleware)
        .middleware(
            ApiKeyMiddleware::new()
                .protect_path("/api/")
                .protect_path("/admin/"),
        )
        // Public routes
        .get("/", home)
        .get("/health", health_check)
        .post("/echo", echo_post)
        // Protected routes (require API key)
        .get("/api/data", api_data)
        .get("/admin/stats", admin_stats)
        // Custom 404 handler
        .not_found(not_found);

    let addr: SocketAddr = "127.0.0.1:3004".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🚀 Custom Middleware Server running on http://{}", addr);
    println!("📋 Available endpoints:");
    println!("   GET  /           - Home page (public)");
    println!("   GET  /health     - Health check (public)");
    println!("   POST /echo       - Echo service (public, requires JSON)");
    println!("   GET  /api/data   - API data (requires API key)");
    println!("   GET  /admin/stats - Admin stats (requires API key)");
    println!();
    println!("🔑 Valid API keys: api_key_123, api_key_456, api_key_789");
    println!("⚡ Rate limit: 5 requests per minute");

    server.run().await.unwrap();
    Ok(())
}

// Handler functions
async fn home(_req: Request) -> Result<Response> {
    Ok(Response::html(
        r#"
        <h1>🔧 Custom Middleware Demo</h1>
        <h2>Available Endpoints:</h2>
        <ul>
            <li><a href="/health">GET /health</a> - Health check</li>
            <li>POST /echo - Echo service (requires JSON and User-Agent)</li>
            <li><a href="/api/data?api_key=api_key_123">GET /api/data</a> - API data (with API key)</li>
            <li><a href="/admin/stats?api_key=api_key_123">GET /admin/stats</a> - Admin stats (with API key)</li>
        </ul>
        <h2>Middleware Features:</h2>
        <ul>
            <li>🔒 API Key Authentication</li>
            <li>⚡ Rate Limiting (5 req/min)</li>
            <li>🛡️ Security Headers</li>
            <li>⏱️ Request Timing</li>
            <li>✅ Request Validation</li>
        </ul>
    "#,
    ))
}

async fn health_check(_req: Request) -> Result<Response> {
    let health_data = serde_json::json!({
        "status": "healthy",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "version": "1.0.0"
    });
    Response::json(health_data)
}

async fn echo_post(req: Request) -> Result<Response> {
    let body_text = String::from_utf8(req.body.to_vec())
        .map_err(|_| Error::BadRequest("Invalid UTF-8 body".into()))?;

    let response_data = serde_json::json!({
        "echo": body_text,
        "method": req.method.to_string(),
        "path": req.uri.path(),
        "headers": req.headers.len()
    });

    Response::json(response_data)
}

async fn api_data(_req: Request) -> Result<Response> {
    let api_data = serde_json::json!({
        "data": [
            {"id": 1, "name": "Item 1", "value": 100},
            {"id": 2, "name": "Item 2", "value": 200},
            {"id": 3, "name": "Item 3", "value": 300}
        ],
        "total": 3,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    Response::json(api_data)
}

async fn admin_stats(_req: Request) -> Result<Response> {
    let stats_data = serde_json::json!({
        "stats": {
            "total_requests": 1337,
            "active_users": 42,
            "uptime": "72h 35m",
            "memory_usage": "45.2MB"
        },
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    Response::json(stats_data)
}

async fn not_found(_req: Request) -> Result<Response> {
    Ok(Response::html(
        r#"
        <h1>🔍 404 - Not Found</h1>
        <p>The requested resource could not be found.</p>
        <a href="/">← Go back to home</a>
    "#,
    ))
}
