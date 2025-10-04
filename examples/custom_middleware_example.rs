use http::{HeaderName, HeaderValue, Method, StatusCode};
use ignitia::{
    handler::extractor::{Headers, Json, Path, Query},
    middleware::{LoggerMiddleware, Middleware, Next},
    response::IntoResponse,
    Error, Request, Response, Result, Router, Server,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

// ============================================================================
// CUSTOM MIDDLEWARE IMPLEMENTATIONS (Updated with Next pattern)
// ============================================================================

// Enhanced Rate Limiting Middleware with per-endpoint limits
#[derive(Clone)]
struct RateLimitMiddleware {
    global_requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    endpoint_requests: Arc<Mutex<HashMap<String, HashMap<String, Vec<Instant>>>>>,
    max_requests: usize,
    window: Duration,
    endpoint_limits: Arc<HashMap<String, usize>>,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            global_requests: Arc::new(Mutex::new(HashMap::new())),
            endpoint_requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_seconds),
            endpoint_limits: Arc::new(HashMap::new()),
        }
    }

    pub fn with_endpoint_limit(mut self, endpoint: &str, limit: usize) -> Self {
        Arc::make_mut(&mut self.endpoint_limits).insert(endpoint.to_string(), limit);
        self
    }

    fn get_client_ip(&self, req: &Request) -> String {
        req.header("X-Real-IP")
            .or_else(|| req.header("X-Forwarded-For"))
            .map(|ip| ip.split(',').next().unwrap_or("unknown").trim().to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string())
    }

    async fn check_rate_limit(&self, client_ip: &str, endpoint: &str) -> Result<()> {
        let now = Instant::now();

        // Check global rate limit
        {
            let mut global_requests = self.global_requests.lock().await;
            let client_requests = global_requests
                .entry(client_ip.to_string())
                .or_insert_with(Vec::new);

            client_requests.retain(|timestamp| now.duration_since(*timestamp) < self.window);

            if client_requests.len() >= self.max_requests {
                warn!("Global rate limit exceeded for IP: {}", client_ip);
                return Err(Error::BadRequest(
                    "Global rate limit exceeded. Try again later.".into(),
                ));
            }

            client_requests.push(now);
        }

        // Check endpoint-specific rate limit
        if let Some(&endpoint_limit) = self.endpoint_limits.get(endpoint) {
            let mut endpoint_requests = self.endpoint_requests.lock().await;
            let endpoint_map = endpoint_requests
                .entry(endpoint.to_string())
                .or_insert_with(HashMap::new);
            let client_requests = endpoint_map
                .entry(client_ip.to_string())
                .or_insert_with(Vec::new);

            client_requests.retain(|timestamp| now.duration_since(*timestamp) < self.window);

            if client_requests.len() >= endpoint_limit {
                warn!(
                    "Endpoint rate limit exceeded for IP: {} on endpoint: {}",
                    client_ip, endpoint
                );
                return Err(Error::BadRequest(format!(
                    "Rate limit exceeded for endpoint {}. Try again later.",
                    endpoint
                )));
            }

            client_requests.push(now);
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let client_ip = self.get_client_ip(&req);
        let endpoint = req.uri.path().to_string();

        // Check rate limit
        if let Err(e) = self.check_rate_limit(&client_ip, &endpoint).await {
            return e.into_response();
        }

        info!(
            "Rate limit check passed for IP: {} on endpoint: {}",
            client_ip, endpoint
        );

        // Process request
        let mut response = next.run(req).await;

        // Add rate limit headers
        response.headers.insert(
            HeaderName::from_static("x-ratelimit-limit"),
            HeaderValue::from_str(&format!("{}", self.max_requests)).unwrap(),
        );
        response.headers.insert(
            HeaderName::from_static("x-ratelimit-window"),
            HeaderValue::from_str(&format!("{}s", self.window.as_secs())).unwrap(),
        );

        response
    }
}

// Enhanced Request Validation Middleware
#[derive(Clone)]
struct RequestValidationMiddleware {
    max_body_size: usize,
    required_headers: Arc<Vec<String>>,
}

impl RequestValidationMiddleware {
    pub fn new() -> Self {
        Self {
            max_body_size: 10 * 1024 * 1024, // 10MB default
            required_headers: Arc::new(vec!["User-Agent".to_string()]),
        }
    }

    pub fn max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    pub fn require_header(mut self, header: &str) -> Self {
        Arc::make_mut(&mut self.required_headers).push(header.to_string());
        self
    }
}

#[async_trait::async_trait]
impl Middleware for RequestValidationMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Check body size
        if req.body.len() > self.max_body_size {
            error!("Request body too large: {} bytes", req.body.len());
            return Error::BadRequest("Request body too large".into()).into_response();
        }

        // Validate Content-Type for POST/PUT/PATCH requests
        match req.method {
            Method::POST | Method::PUT | Method::PATCH => {
                if let Some(content_type) = req.header("Content-Type") {
                    if !content_type.contains("application/json")
                        && !content_type.contains("application/x-www-form-urlencoded")
                        && !content_type.contains("multipart/form-data")
                    {
                        return Error::BadRequest(
                            "Unsupported Content-Type. Expected application/json, application/x-www-form-urlencoded, or multipart/form-data".into(),
                        ).into_response();
                    }
                } else {
                    return Error::BadRequest(
                        "Content-Type header required for POST/PUT/PATCH requests".into(),
                    )
                    .into_response();
                }
            }
            _ => {}
        }

        // Check required headers
        for header in self.required_headers.as_ref() {
            if req.header(header).is_none() {
                return Error::BadRequest(format!("{} header is required", header)).into_response();
            }
        }

        info!(
            "Request validation passed: {} {}",
            req.method,
            req.uri.path()
        );

        next.run(req).await
    }
}

// Enhanced Security Headers Middleware
#[derive(Clone)]
struct SecurityHeadersMiddleware {
    csp_policy: String,
    hsts_max_age: u64,
}

impl SecurityHeadersMiddleware {
    pub fn new() -> Self {
        Self {
            csp_policy: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string(),
            hsts_max_age: 31536000, // 1 year
        }
    }

    pub fn _csp_policy(mut self, policy: &str) -> Self {
        self.csp_policy = policy.to_string();
        self
    }
}

#[async_trait::async_trait]
impl Middleware for SecurityHeadersMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let mut response = next.run(req).await;

        // Add security headers
        let security_headers = [
            ("x-frame-options", "DENY"),
            ("x-content-type-options", "nosniff"),
            ("x-xss-protection", "1; mode=block"),
            ("referrer-policy", "strict-origin-when-cross-origin"),
            ("x-permitted-cross-domain-policies", "none"),
            ("x-download-options", "noopen"),
        ];

        for (name, value) in security_headers.iter() {
            response.headers.insert(
                HeaderName::from_static(name),
                HeaderValue::from_static(value),
            );
        }

        response.headers.insert(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_str(&self.csp_policy).unwrap(),
        );

        response.headers.insert(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_str(&format!("max-age={}; includeSubDomains", self.hsts_max_age))
                .unwrap(),
        );

        response
    }
}

// Enhanced Timing Middleware with detailed metrics
#[derive(Clone)]
struct TimingMiddleware {
    request_data: Arc<Mutex<HashMap<String, RequestMetrics>>>,
}

#[derive(Debug, Clone)]
struct RequestMetrics {
    _start_time: Instant,
    client_ip: String,
    user_agent: Option<String>,
    endpoint: String,
    method: String,
}

impl TimingMiddleware {
    pub fn new() -> Self {
        Self {
            request_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn generate_request_id(&self) -> String {
        format!(
            "req_{}_{:x}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            simple_random()
        )
    }
}

#[async_trait::async_trait]
impl Middleware for TimingMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        let request_id = self.generate_request_id();
        let start_time = Instant::now();

        let metrics = RequestMetrics {
            _start_time: start_time,
            client_ip: req
                .header("X-Real-IP")
                .or_else(|| req.header("X-Forwarded-For"))
                .unwrap_or("127.0.0.1")
                .to_string(),
            user_agent: req.header("User-Agent").map(|s| s.to_string()),
            endpoint: req.uri.path().to_string(),
            method: req.method.to_string(),
        };

        self.request_data
            .lock()
            .await
            .insert(request_id.clone(), metrics.clone());

        req.headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(&request_id).unwrap(),
        );

        info!(
            "🕐 Request started: {} {} [ID: {}]",
            req.method,
            req.uri.path(),
            request_id
        );

        // Process request
        let mut response = next.run(req).await;

        // Add timing info
        let duration = start_time.elapsed();
        response.headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(&request_id).unwrap(),
        );
        response.headers.insert(
            HeaderName::from_static("x-response-time"),
            HeaderValue::from_str(&format!("{}ms", duration.as_millis())).unwrap(),
        );

        info!(
            "⏱️ Request completed: {} {} in {}ms [IP: {}, UA: {}] [ID: {}]",
            metrics.method,
            metrics.endpoint,
            duration.as_millis(),
            metrics.client_ip,
            metrics.user_agent.unwrap_or_else(|| "Unknown".to_string()),
            request_id
        );

        // Cleanup
        self.request_data.lock().await.remove(&request_id);

        response
    }
}

// Enhanced API Key Middleware with key management
#[derive(Clone)]
struct ApiKeyMiddleware {
    valid_keys: Arc<Mutex<HashMap<String, ApiKeyInfo>>>,
    protected_paths: Arc<Vec<String>>,
}

#[derive(Debug, Clone)]
struct ApiKeyInfo {
    name: String,
    permissions: Vec<String>,
    _rate_limit: Option<usize>,
    last_used: Option<Instant>,
}

impl ApiKeyMiddleware {
    pub fn new() -> Self {
        let mut keys = HashMap::new();

        keys.insert(
            "api_key_123".to_string(),
            ApiKeyInfo {
                name: "Development Key".to_string(),
                permissions: vec!["read".to_string(), "write".to_string()],
                _rate_limit: Some(100),
                last_used: None,
            },
        );

        keys.insert(
            "api_key_456".to_string(),
            ApiKeyInfo {
                name: "Production Key".to_string(),
                permissions: vec!["read".to_string()],
                _rate_limit: Some(1000),
                last_used: None,
            },
        );

        keys.insert(
            "admin_key_789".to_string(),
            ApiKeyInfo {
                name: "Admin Key".to_string(),
                permissions: vec!["read".to_string(), "write".to_string(), "admin".to_string()],
                _rate_limit: None,
                last_used: None,
            },
        );

        Self {
            valid_keys: Arc::new(Mutex::new(keys)),
            protected_paths: Arc::new(Vec::new()),
        }
    }

    pub fn protect_path(mut self, path: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.protected_paths).push(path.into());
        self
    }

    fn requires_auth(&self, path: &str) -> bool {
        self.protected_paths.iter().any(|p| path.starts_with(p))
    }

    async fn validate_key(&self, key: &str, path: &str) -> Result<ApiKeyInfo> {
        let mut keys = self.valid_keys.lock().await;

        if let Some(key_info) = keys.get_mut(key) {
            key_info.last_used = Some(Instant::now());

            if path.contains("/admin/") && !key_info.permissions.contains(&"admin".to_string()) {
                return Err(Error::Unauthorized("Insufficient permissions".to_string()));
            }

            Ok(key_info.clone())
        } else {
            Err(Error::Unauthorized("Invalid API key".to_string()))
        }
    }
}

#[async_trait::async_trait]
impl Middleware for ApiKeyMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        let path = req.uri.path().to_string();

        if !self.requires_auth(&path) {
            return next.run(req).await;
        }

        // Check for API key in query params first
        if let Some(api_key) = req.query("api_key") {
            match self.validate_key(api_key, &path).await {
                Ok(key_info) => {
                    info!("✅ Valid API key from query param: {}", key_info.name);
                    req.insert_extension(key_info);
                    return next.run(req).await;
                }
                Err(_) => {}
            }
        }

        // Check for API key in headers
        if let Some(api_key) = req.header("X-API-Key") {
            match self.validate_key(api_key, &path).await {
                Ok(key_info) => {
                    info!("✅ Valid API key from header: {}", key_info.name);
                    req.insert_extension(key_info);
                    return next.run(req).await;
                }
                Err(_) => {}
            }
        }

        error!("❌ Invalid or missing API key for path: {}", path);
        Error::Unauthorized("Invalid API key".to_string()).into_response()
    }
}

// CORS Middleware with more options
#[derive(Clone)]
struct EnhancedCorsMiddleware {
    allow_origins: Arc<Vec<String>>,
    allow_methods: Arc<Vec<Method>>,
    allow_headers: Arc<Vec<String>>,
    expose_headers: Arc<Vec<String>>,
    allow_credentials: bool,
    max_age: Option<u64>,
}

impl EnhancedCorsMiddleware {
    pub fn new() -> Self {
        Self {
            allow_origins: Arc::new(vec!["*".to_string()]),
            allow_methods: Arc::new(vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::HEAD,
            ]),
            allow_headers: Arc::new(vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-API-Key".to_string(),
                "X-Requested-With".to_string(),
            ]),
            expose_headers: Arc::new(vec![
                "x-request-id".to_string(),
                "x-response-time".to_string(),
            ]),
            allow_credentials: false,
            max_age: Some(86400), // 24 hours
        }
    }

    pub fn allow_origin(mut self, origin: &str) -> Self {
        let origins = Arc::make_mut(&mut self.allow_origins);
        if origins.contains(&"*".to_string()) {
            origins.clear();
        }
        origins.push(origin.to_string());
        self
    }

    pub fn allow_credentials(mut self) -> Self {
        self.allow_credentials = true;
        self
    }
}

#[async_trait::async_trait]
impl Middleware for EnhancedCorsMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Handle preflight OPTIONS request
        if req.method == Method::OPTIONS {
            info!("Handling CORS preflight request");
            let mut response = Response::new(StatusCode::NO_CONTENT);
            self.add_cors_headers(&mut response);
            return response;
        }

        // Process request
        let mut response = next.run(req).await;

        // Add CORS headers
        self.add_cors_headers(&mut response);

        response
    }
}

impl EnhancedCorsMiddleware {
    fn add_cors_headers(&self, res: &mut Response) {
        res.headers.insert(
            HeaderName::from_static("access-control-allow-origin"),
            HeaderValue::from_str(&self.allow_origins.join(", ")).unwrap(),
        );

        let methods: Vec<String> = self.allow_methods.iter().map(|m| m.to_string()).collect();
        res.headers.insert(
            HeaderName::from_static("access-control-allow-methods"),
            HeaderValue::from_str(&methods.join(", ")).unwrap(),
        );

        res.headers.insert(
            HeaderName::from_static("access-control-allow-headers"),
            HeaderValue::from_str(&self.allow_headers.join(", ")).unwrap(),
        );

        if !self.expose_headers.is_empty() {
            res.headers.insert(
                HeaderName::from_static("access-control-expose-headers"),
                HeaderValue::from_str(&self.expose_headers.join(", ")).unwrap(),
            );
        }

        if self.allow_credentials {
            res.headers.insert(
                HeaderName::from_static("access-control-allow-credentials"),
                HeaderValue::from_static("true"),
            );
        }

        if let Some(max_age) = self.max_age {
            res.headers.insert(
                HeaderName::from_static("access-control-max-age"),
                HeaderValue::from_str(&max_age.to_string()).unwrap(),
            );
        }
    }
}

// ============================================================================
// DATA STRUCTURES FOR HANDLERS
// ============================================================================

#[derive(Debug, Deserialize)]
struct UserQuery {
    name: Option<String>,
    age: Option<u32>,
    page: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct UserPath {
    id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Debug, Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
    age: Option<u32>,
    created_at: u64,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
    timestamp: u64,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl ApiResponse<()> {
    fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: message.into(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

// ============================================================================
// HANDLER FUNCTIONS WITH EXTRACTORS
// ============================================================================

async fn home() -> Result<Response> {
    Ok(Response::html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>🔧 Enhanced Custom Middleware Demo</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; margin-top: 0; }
        .endpoint { background: #f8f9fa; padding: 15px; margin: 10px 0; border-radius: 5px; border-left: 4px solid #007bff; }
        .method { font-weight: bold; color: #007bff; }
        .feature { background: #e8f5e9; padding: 10px; margin: 5px 0; border-radius: 5px; }
        .warning { background: #fff3cd; padding: 10px; margin: 10px 0; border-radius: 5px; border-left: 4px solid #ffc107; }
        code { background: #f1f1f1; padding: 2px 6px; border-radius: 3px; font-family: 'Monaco', 'Menlo', monospace; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔧 Enhanced Custom Middleware Demo (Next Pattern)</h1>

        <h2>📋 Available Endpoints:</h2>

        <div class="endpoint">
            <div class="method">GET /public/</div>
            <div>Home page (public, no authentication required)</div>
        </div>

        <div class="endpoint">
            <div class="method">GET /public/health</div>
            <div>Health check endpoint with system metrics</div>
        </div>

        <div class="endpoint">
            <div class="method">POST /public/echo</div>
            <div>Echo service - returns your request data (requires JSON and User-Agent header)</div>
        </div>

        <div class="endpoint">
            <div class="method">GET /public/users?name=john&age=25&page=1</div>
            <div>List users with query parameters</div>
        </div>

        <div class="endpoint">
            <div class="method">GET /protected/api/users/{id}</div>
            <div>Get specific user by ID (requires API key)</div>
        </div>

        <div class="endpoint">
            <div class="method">POST /protected/api/users</div>
            <div>Create new user (requires API key and JSON body)</div>
        </div>

        <div class="endpoint">
            <div class="method">GET /protected/admin/stats</div>
            <div>Admin statistics (requires admin API key)</div>
        </div>

        <div class="endpoint">
            <div class="method">GET /protected/admin/keys</div>
            <div>List API keys (requires admin API key)</div>
        </div>

        <h2>🔑 API Keys:</h2>
        <div class="warning">
            <strong>Development Key:</strong> <code>api_key_123</code> (read, write permissions, 100 req/min)<br>
            <strong>Production Key:</strong> <code>api_key_456</code> (read permissions, 1000 req/min)<br>
            <strong>Admin Key:</strong> <code>admin_key_789</code> (full permissions, unlimited)
        </div>

        <h2>🛡️ Middleware Features (Axum-Style Next Pattern):</h2>
        <div class="feature">🔒 Enhanced API Key Authentication with permissions</div>
        <div class="feature">⚡ Smart Rate Limiting (global + per-endpoint)</div>
        <div class="feature">🛡️ Comprehensive Security Headers</div>
        <div class="feature">⏱️ Detailed Request Timing & Metrics</div>
        <div class="feature">✅ Advanced Request Validation</div>
        <div class="feature">🌐 Enhanced CORS Support</div>
        <div class="feature">📝 Structured Logging</div>
        <div class="feature">🔄 Middleware chaining with Next pattern</div>

        <h2>🧪 Test Commands:</h2>
        <div class="warning">
            <code>curl -H "User-Agent: TestBot/1.0" http://localhost:3004/public/health</code><br><br>
            <code>curl -X POST -H "Content-Type: application/json" -H "User-Agent: TestBot/1.0" -d '{"test":"data"}' http://localhost:3004/public/echo</code><br><br>
            <code>curl -H "X-API-Key: api_key_123" http://localhost:3004/protected/api/users/1</code><br><br>
            <code>curl -H "X-API-Key: admin_key_789" http://localhost:3004/protected/admin/stats</code>
        </div>
    </div>
</body>
</html>"#,
    ))
}

async fn health_check() -> Result<Response> {
    let health_data = ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "version": "2.0.0",
        "middleware_pattern": "Axum-style Next",
        "system": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        },
        "middleware": {
            "rate_limiting": "active",
            "security_headers": "active",
            "api_key_auth": "active",
            "cors": "active",
            "request_timing": "active",
            "validation": "active"
        }
    }));

    Ok(Response::json(health_data))
}

async fn echo_post(Json(body): Json<serde_json::Value>, headers: Headers) -> Result<Response> {
    let response_data = ApiResponse::success(serde_json::json!({
        "received_body": body,
        "headers_count": headers.len(),
        "user_agent": headers.get("user-agent").unwrap_or(&"Unknown".to_string()),
        "content_type": headers.get("content-type").unwrap_or(&"Unknown".to_string()),
        "echo_timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }));

    Ok(Response::json(response_data))
}

async fn list_users(Query(query): Query<UserQuery>) -> Result<Response> {
    let page = query.page.unwrap_or(1);
    let limit = 10;
    let offset = (page - 1) * limit;

    let users: Vec<User> = (1..=50)
        .skip(offset as usize)
        .take(limit as usize)
        .filter(|&id| {
            if let Some(ref name) = query.name {
                format!("User {}", id)
                    .to_lowercase()
                    .contains(&name.to_lowercase())
            } else {
                true
            }
        })
        .filter(|&id| {
            if let Some(age) = query.age {
                (id % 50) + 18 == age
            } else {
                true
            }
        })
        .map(|id| User {
            id,
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
            age: Some((id % 50) + 18),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - (id as u64 * 86400),
        })
        .collect();

    let response_data = ApiResponse::success(serde_json::json!({
        "users": users,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": 50,
            "total_pages": 5
        }
    }));

    Ok(Response::json(response_data))
}

async fn get_user(Path(path): Path<UserPath>) -> Result<Response> {
    if path.id > 100 {
        let error_response = ApiResponse::<()>::error("User not found");
        let mut response = Response::json(error_response);
        response.status = StatusCode::NOT_FOUND;
        return Ok(response);
    }

    let user = User {
        id: path.id,
        name: format!("User {}", path.id),
        email: format!("user{}@example.com", path.id),
        age: Some((path.id % 50) + 18),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (path.id as u64 * 86400),
    };

    let response_data = ApiResponse::success(user);
    Ok(Response::json(response_data))
}

async fn create_user(Json(user_data): Json<CreateUser>) -> Result<Response> {
    let new_user = User {
        id: simple_random() % 10000 + 1000,
        name: user_data.name,
        email: user_data.email,
        age: user_data.age,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let response_data = ApiResponse::success(new_user);
    let mut response = Response::json(response_data);
    response.status = StatusCode::CREATED;
    Ok(response)
}

async fn admin_stats(req: Request) -> Result<Response> {
    let key_info = req.get_extension::<ApiKeyInfo>();

    let stats_data = ApiResponse::success(serde_json::json!({
        "system_stats": {
            "total_requests": simple_random() % 10000 + 1000,
            "active_connections": simple_random() % 100 + 1,
        },
        "request_info": {
            "api_key_used": key_info.as_ref().map(|k| k.name.clone()).unwrap_or_else(|| "Unknown".to_string()),
            "permissions": key_info.as_ref().map(|k| k.permissions.clone()).unwrap_or_else(|| vec![])
        }
    }));

    Ok(Response::json(stats_data))
}

async fn list_api_keys(req: Request) -> Result<Response> {
    let key_info = req.get_extension::<ApiKeyInfo>();

    if let Some(info) = key_info {
        if !info.permissions.contains(&"admin".to_string()) {
            let error_response = ApiResponse::<()>::error("Admin permissions required");
            let mut response = Response::json(error_response);
            response.status = StatusCode::FORBIDDEN;
            return Ok(response);
        }
    }

    let keys_data = ApiResponse::success(serde_json::json!({
        "api_keys": [
            {
                "name": "Development Key",
                "permissions": ["read", "write"],
                "rate_limit": 100
            },
            {
                "name": "Production Key",
                "permissions": ["read"],
                "rate_limit": 1000
            },
            {
                "name": "Admin Key",
                "permissions": ["read", "write", "admin"],
                "rate_limit": null
            }
        ]
    }));

    Ok(Response::json(keys_data))
}

async fn not_found() -> Result<Response> {
    let error_response = ApiResponse::<()>::error("Resource not found");
    let mut response = Response::json(error_response);
    response.status = StatusCode::NOT_FOUND;
    Ok(response)
}

// ============================================================================
// MAIN APPLICATION
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    info!("🚀 Starting Enhanced Custom Middleware Server (Next Pattern)...");

    // Create public routes
    let public_routes = Router::new()
        .get("/", home)
        .get("/health", health_check)
        .post("/echo", echo_post)
        .get("/users", list_users);

    // Create protected API routes
    let api_routes = Router::new()
        .get("/users/{id}", get_user)
        .post("/users", create_user);

    // Create admin routes
    let admin_routes = Router::new()
        .get("/stats", admin_stats)
        .get("/keys", list_api_keys);

    // Create protected routes container
    let protected_routes = Router::new()
        .nest("/api", api_routes)
        .nest("/admin", admin_routes);

    // Main router with comprehensive middleware stack using Next pattern
    let router = Router::new()
        .middleware(LoggerMiddleware)
        .middleware(TimingMiddleware::new())
        .middleware(
            EnhancedCorsMiddleware::new()
                .allow_origin("http://localhost:3000")
                .allow_credentials(),
        )
        .middleware(SecurityHeadersMiddleware::new())
        .middleware(
            RateLimitMiddleware::new(10, 60)
                .with_endpoint_limit("/protected/api/users", 5)
                .with_endpoint_limit("/protected/admin/", 2),
        )
        .middleware(
            RequestValidationMiddleware::new()
                .max_body_size(1024 * 1024)
                .require_header("User-Agent"),
        )
        .middleware(ApiKeyMiddleware::new().protect_path("/protected/"))
        .nest("/public", public_routes)
        .nest("/protected", protected_routes)
        .not_found(not_found);

    let addr: SocketAddr = "127.0.0.1:3004".parse().unwrap();
    let server = Server::new(router, addr);

    info!(
        "🔥 Enhanced Custom Middleware Server blazing on http://{}",
        addr
    );
    info!("📋 Available endpoints:");
    info!("   GET  /public/ - Home page (public)");
    info!("   GET  /public/health - Health check (public)");
    info!("   POST /public/echo - Echo service (public, requires JSON + User-Agent)");
    info!("   GET  /public/users?name=john&age=25 - List users with filters (public)");
    info!("   GET  /protected/api/users/{{id}} - Get user by ID (requires API key)");
    info!("   POST /protected/api/users - Create user (requires API key + JSON)");
    info!("   GET  /protected/admin/stats - Admin statistics (requires admin key)");
    info!("   GET  /protected/admin/keys - List API keys (requires admin key)");
    info!("");
    info!("🔑 Valid API keys:");
    info!("   Development: api_key_123 (read, write permissions, 100 req/min)");
    info!("   Production:  api_key_456 (read permissions, 1000 req/min)");
    info!("   Admin:       admin_key_789 (full permissions, unlimited)");
    info!("");
    info!("⚡ Rate limits:");
    info!("   Global: 10 requests per minute");
    info!("   User creation: 5 requests per minute");
    info!("   Admin endpoints: 2 requests per minute");

    server.ignitia().await.unwrap();

    Ok(())
}

fn simple_random() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    hasher.finish() as u32
}
