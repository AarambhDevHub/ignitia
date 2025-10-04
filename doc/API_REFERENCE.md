# API Reference

Complete API documentation for the Ignitia web framework v0.2.4+.

## Table of Contents

- [Core Types](#core-types)
- [Router](#router)
- [Server](#server)
- [Request & Response](#request--response)
- [IntoResponse Trait](#intoresponse-trait)
- [Extractors](#extractors)
- [Middleware](#middleware)
- [Error Handling](#error-handling)
- [WebSocket Support](#websocket-support)
- [Multipart & File Uploads](#multipart--file-uploads)
- [Cookie Management](#cookie-management)
- [State & Extensions](#state--extensions)
- [TLS & HTTPS](#tls--https)
- [Configuration](#configuration)
- [Performance](#performance)
- [Routing](#routing)
- [Handler Signatures](#handler-signatures)
- [Feature Flags](#feature-flags)
- [Result Type](#result-type)

***

## Core Types

### Router

The main routing component for defining application routes and middleware. **v0.2.4+ uses exclusively high-performance Radix tree routing.**

```rust
impl Router {
    pub fn new() -> Self
    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    pub fn post<H, T>(self, path: &str, handler: H) -> Self
    pub fn put<H, T>(self, path: &str, handler: H) -> Self
    pub fn delete<H, T>(self, path: &str, handler: H) -> Self
    pub fn patch<H, T>(self, path: &str, handler: H) -> Self
    pub fn head<H, T>(self, path: &str, handler: H) -> Self
    pub fn options<H, T>(self, path: &str, handler: H) -> Self
    pub fn any<H, T>(self, path: &str, handler: H) -> Self
    pub fn middleware<M: Middleware>(self, middleware: M) -> Self
    pub fn nest(self, path: &str, router: Router) -> Self
    pub fn merge(self, other: Router) -> Self
    pub fn state<T: Clone + Send + Sync + 'static>(self, state: T) -> Self
    pub fn state_arc<T>(self, state: Arc<T>) -> Self
    pub fn extension<T: Send + Sync + 'static>(self, ext: T) -> Self
    pub fn not_found<H, T>(self, handler: H) -> Self
    pub fn stats(&self) -> Option<RadixStats>
    pub fn print_tree(&self)
}
```

#### Routing Mode (v0.2.4+)

**Breaking Change**: Router now uses Radix tree exclusively. `RouterMode` enum and `with_mode()` method have been removed.

```rust
// ❌ Removed in v0.2.4
pub fn with_mode(self, mode: RouterMode) -> Self

// ✅ v0.2.4: Always uses Radix
let router = Router::new();  // Automatically uses Radix tree
```

#### WebSocket Support

**Enhanced in v0.2.4+**: Full extractor support for WebSocket handlers.

```rust
#[cfg(feature = "websocket")]
impl Router {
    // Universal handler with extractor support (0-7 extractors)
    pub fn websocket<H, T>(self, path: &str, handler: H) -> Self
    where
        H: UniversalWebSocketHandler<T>,
        T: Send + Sync + 'static

    // Convenience method for simple closures
    pub fn websocket_fn<F, Fut, R>(self, path: &str, f: F) -> Self
    where
        F: Fn(WebSocketConnection) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse
}
```

**Example Usage:**

```rust
// Basic WebSocket
router.websocket("/ws", |mut ws: WebSocketConnection| async move {
    ws.send_text("Hello!").await.ok();
    Response::ok()
})

// WebSocket with extractors
router.websocket("/ws/{room}", |
    Path(room): Path<String>,
    State(state): State<AppState>,
    mut ws: WebSocketConnection
| async move {
    ws.send_text(format!("Welcome to {}", room)).await.ok();
    "Connection closed"
})
```

### Server

HTTP server configuration and startup with enhanced performance features.

```rust
impl Server {
    pub fn new(router: Router, addr: SocketAddr) -> Self
    pub fn with_server_config(self, config: ServerConfig) -> Self
    pub fn with_performance_config(self, config: PerformanceConfig) -> Self
    pub fn with_pool_config(self, config: PoolConfig) -> Self
    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>>
}
```

#### TLS/HTTPS Methods

```rust
#[cfg(feature = "tls")]
impl Server {
    pub fn with_tls(self, tls_config: TlsConfig) -> Result<Self, TlsError>
    pub fn enable_https(self, cert_file: impl Into<String>, key_file: impl Into<String>) -> Result<Self, TlsError>
    pub fn redirect_to_https(self, https_port: u16) -> Self
}
```

#### Self-Signed Certificates (Development)

```rust
#[cfg(all(feature = "tls", feature = "self-signed"))]
impl Server {
    pub fn with_self_signed_cert(self, domain: &str) -> Result<Self, TlsError>
}
```

***

## Request & Response

### Request

Enhanced HTTP request representation with optimized body handling.

**v0.2.4 Breaking Change**: Body type changed from `Arc<Bytes>` to `Bytes`.

```rust
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Bytes,  // ✅ Changed from Arc<Bytes> in v0.2.4
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub extensions: Extensions,
}

impl Request {
    pub fn new(method: Method, uri: Uri, version: Version, headers: HeaderMap, body: Bytes) -> Self
    pub fn json<T: DeserializeOwned>(&self) -> Result<T>
    pub fn param(&self, key: &str) -> Option<&String>
    pub fn query(&self, key: &str) -> Option<&String>
    pub fn header(&self, key: &str) -> Option<&str>
    pub fn cookies(&self) -> CookieJar
    pub fn cookie(&self, name: &str) -> Option<String>
}
```

#### Extension Methods

```rust
impl Request {
    pub fn insert_extension<T: Send + Sync + 'static>(&mut self, value: T)
    pub fn get_extension<T: Send + Sync + Clone + 'static>(&self) -> Option<Arc<T>>
    pub fn remove_extension<T: Send + Sync + 'static>(&mut self) -> Option<T>
    pub fn has_extension<T: Send + Sync + 'static>(&self) -> bool
}
```

### Response

Enhanced response builder with optimized body handling.

**v0.2.4 Breaking Changes**:
- Body type changed from `Arc<Bytes>` to `Bytes`
- Content type methods now infallible (return `Response` directly)

```rust
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,  // ✅ Changed from Arc<Bytes> in v0.2.4
    pub cache_control: Option<CacheControl>,
}

impl Response {
    pub fn new(status: StatusCode) -> Self
    pub fn with_status(self, status: StatusCode) -> Self
    pub fn with_body(self, body: impl Into<Bytes>) -> Self
    pub fn with_header(self, name: impl Into<String>, value: impl Into<String>) -> Self
    pub fn ok() -> Self
    pub fn created() -> Self
    pub fn accepted() -> Self
    pub fn no_content() -> Self
    pub fn bad_request(message: impl Into<String>) -> Self
    pub fn unauthorized(message: impl Into<String>) -> Self
    pub fn forbidden(message: impl Into<String>) -> Self
    pub fn not_found() -> Self
    pub fn internal_error() -> Self
    pub fn service_unavailable() -> Self
}
```

#### Content Type Methods (v0.2.4+: Infallible)

```rust
impl Response {
    // ✅ Returns Response directly (not Result) in v0.2.4
    pub fn json<T: Serialize>(data: T) -> Self
    pub fn text(text: impl Into<String>) -> Self
    pub fn html(html: impl Into<String>) -> Self
}
```

**Migration Note (v0.2.4):**
```rust
// ❌ Old (v0.2.3 and earlier)
async fn handler() -> Result<Response> {
    Response::json(data)?  // Double Result unwrap
}

// ✅ New (v0.2.4+)
async fn handler() -> Result<Response> {
    Ok(Response::json(data))  // Single Result wrap
}

// ✅ Or with IntoResponse
async fn handler() -> impl IntoResponse {
    Response::json(data)  // No Result needed!
}
```

#### Redirect Methods

```rust
impl Response {
    pub fn redirect(location: impl Into<String>) -> Self
    pub fn permanent_redirect(location: impl Into<String>) -> Self
    pub fn redirect_with_status(status: StatusCode, location: impl Into<String>) -> Self
    pub fn see_other(location: impl Into<String>) -> Self
    pub fn temporary_redirect(location: impl Into<String>) -> Self
    pub fn permanent_redirect_308(location: impl Into<String>) -> Self
    pub fn redirect_to_login(login_path: impl Into<String>) -> Self
    pub fn redirect_after_post(location: impl Into<String>) -> Self
    pub fn redirect_moved(new_location: impl Into<String>) -> Self
}
```

#### Caching Support

```rust
impl Response {
    pub fn is_cacheable(&self) -> bool
    pub fn cache_max_age(&self) -> u64
    pub fn cache_key(&self, request_uri: &str) -> String
    pub fn with_cache_control(self, max_age: u64) -> Self
}
```

***

## IntoResponse Trait

**New in v0.2.4**: Unified response conversion system for ergonomic handler returns.

### Trait Definition

```rust
pub trait IntoResponse {
    fn into_response(self) -> Response;
}
```

### Automatic Implementations

The framework provides automatic implementations for common types:

```rust
// Strings
impl IntoResponse for String
impl IntoResponse for &'static str
impl IntoResponse for Cow<'static, str>

// Status codes
impl IntoResponse for StatusCode
impl<T: IntoResponse> IntoResponse for (StatusCode, T)

// Bytes
impl IntoResponse for Bytes
impl IntoResponse for Vec<u8>
impl IntoResponse for &'static [u8]

// Result and Option
impl<T: IntoResponse, E: IntoResponse> IntoResponse for Result<T, E>
impl<T: IntoResponse> IntoResponse for Option<T>

// Primitives (v0.2.4+)
impl IntoResponse for bool
impl IntoResponse for i8, i16, i32, i64, i128, isize
impl IntoResponse for u8, u16, u32, u64, u128, usize
impl IntoResponse for f32, f64

// Special types
impl IntoResponse for ()
impl IntoResponse for Response
impl<T: Serialize> IntoResponse for Json<T>
impl<T: Into<String>> IntoResponse for Html<T>
```

### Usage Examples

```rust
// Simple string response
async fn handler() -> impl IntoResponse {
    "Hello, World!"
}

// With status code
async fn handler() -> impl IntoResponse {
    (StatusCode::CREATED, "Resource created")
}

// JSON response
async fn handler() -> impl IntoResponse {
    Json(MyData { field: "value" })
}

// Result with IntoResponse
async fn handler() -> Result<impl IntoResponse, Error> {
    let data = fetch_data().await?;
    Ok(Json(data))
}

// Boolean (v0.2.4+)
async fn is_healthy() -> bool {
    true  // Returns JSON "true"
}

// Numbers (v0.2.4+)
async fn count() -> i32 {
    42  // Returns "42" as text/plain
}

// Option (v0.2.4+)
async fn find_user(id: u64) -> Option<Json<User>> {
    database.find(id).await.map(Json)  // Auto 404 if None
}

// Custom type implementing IntoResponse
impl IntoResponse for User {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
```

***

## Extractors

Type-safe request data extraction for handlers with improved performance.

### Path Parameters

```rust
#[derive(Debug)]
pub struct Path<T>(pub T);

impl<T> Path<T> {
    pub fn into_inner(self) -> T
}

impl<T> std::ops::Deref for Path<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target
}
```

**Usage:**
```rust
// Single parameter
async fn get_user(Path(user_id): Path<u32>) -> impl IntoResponse {
    format!("User ID: {}", user_id)
}

// Multiple parameters as tuple
async fn get_post(
    Path((user_id, post_id)): Path<(u32, u32)>
) -> impl IntoResponse {
    format!("User {}, Post {}", user_id, post_id)
}

// Named struct
#[derive(Deserialize)]
struct UserPost {
    user_id: u32,
    post_id: u32,
}

async fn get_post(Path(params): Path<UserPost>) -> impl IntoResponse {
    Json(params)
}
```

### Query Parameters

```rust
#[derive(Debug)]
pub struct Query<T>(pub T);

impl<T> Query<T> {
    pub fn into_inner(self) -> T
}
```

**Usage:**
```rust
#[derive(Deserialize)]
struct Pagination {
    page: u32,
    per_page: u32,
}

async fn list_items(Query(pagination): Query<Pagination>) -> impl IntoResponse {
    format!("Page {} of {}", pagination.page, pagination.per_page)
}
```

### JSON Body

**v0.2.4: Unified `Json<T>` type for both extraction and response.**

```rust
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn new(value: T) -> Self
    pub fn into_inner(self) -> T
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target
}

// Extractor
impl<T: DeserializeOwned> FromRequest for Json<T>

// Response
impl<T: Serialize> IntoResponse for Json<T>
```

**Usage:**
```rust
#[derive(Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

// ✅ Single Json type for both input and output!
async fn create_user(Json(user): Json<CreateUser>) -> Json<User> {
    let created_user = save_user(user).await;
    Json(created_user)
}
```

### Form Data

```rust
#[derive(Debug)]
pub struct Form<T>(pub T);

impl<T> Form<T> {
    pub fn into_inner(self) -> T
}
```

### Other Extractors

```rust
// Raw request body
pub struct Body(pub Bytes);  // ✅ Changed from Arc<Bytes> in v0.2.4

// HTTP headers
pub struct Headers(pub HashMap<String, String>);

// Cookies
pub struct Cookies(pub CookieJar);

// HTTP method
pub struct Method(pub http::Method);

// Request URI
pub struct Uri(pub http::Uri);

// Application state
pub struct State<T>(pub T);

// Extensions
pub struct Extension<T>(pub T);
```

***

## Middleware

**v0.2.4: Axum-inspired middleware system with `handle(req, next)` pattern.**

### Middleware Trait

```rust
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, req: Request, next: Next) -> Response;
}
```

### Next Type

```rust
pub struct Next { /* ... */ }

impl Next {
    pub async fn run(self, req: Request) -> Response
}
```

### from_fn Helper

**New in v0.2.4**: Create middleware from closures (inspired by Axum).

```rust
pub fn from_fn<F, Fut>(f: F) -> impl Middleware
where
    F: Fn(Request, Next) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static;
```

**Usage:**
```rust
use ignitia::middleware::from_fn;

// Simple logging middleware
let logger = from_fn(|req, next| async move {
    println!("Request: {} {}", req.method, req.uri.path());
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    println!("Took: {:?}", start.elapsed());
    response
});

// Authentication middleware
let auth = from_fn(|req, next| async move {
    if let Some(token) = req.headers.get("Authorization") {
        if verify_token(token).await {
            return next.run(req).await;
        }
    }
    Response::new(StatusCode::UNAUTHORIZED)
});

let router = Router::new()
    .middleware(logger)
    .middleware(auth)
    .get("/", handler);
```

### Built-in Middleware

All built-in middleware updated to v0.2.4 API.

#### Logger Middleware

```rust
pub struct LoggerMiddleware;

impl LoggerMiddleware {
    pub fn new() -> Self
}
```

#### CORS Middleware

```rust
impl CorsMiddleware {
    pub fn new() -> CorsBuilder
}

impl CorsBuilder {
    pub fn allowed_origins(self, origins: &[&str]) -> Self
    pub fn allowed_origin_regex(self, pattern: &str) -> Self
    pub fn allowed_methods(self, methods: &[Method]) -> Self
    pub fn allowed_headers(self, headers: &[&str]) -> Self
    pub fn expose_headers(self, headers: &[&str]) -> Self
    pub fn max_age(self, seconds: u64) -> Self
    pub fn allow_credentials(self) -> Self
    pub fn build(self) -> Result<CorsMiddleware>
}
```

#### Rate Limiting Middleware

```rust
impl RateLimitingMiddleware {
    pub fn new(config: RateLimitConfig) -> Self
    pub fn per_minute(max_requests: usize) -> Self
    pub fn per_second(max_requests: usize) -> Self
    pub fn per_hour(max_requests: usize) -> Self
}

impl RateLimitConfig {
    pub fn new(max_requests: usize, window: Duration) -> Self
    pub fn with_key_extractor<F>(self, extractor: F) -> Self
    pub fn with_error_message(self, message: impl Into<String>) -> Self
    pub fn with_headers(self, include: bool) -> Self
    pub fn with_burst(self, multiplier: f32) -> Self
}
```

#### Security Middleware

```rust
impl SecurityMiddleware {
    pub fn new() -> Self
    pub fn with_hsts_config(self, max_age: u64, include_subdomains: bool, preload: bool) -> Self
    pub fn with_csp(self, config: CspConfig) -> Self
    pub fn with_frame_options(self, options: &str) -> Self
    pub fn with_content_type_nosniff(self) -> Self
    pub fn with_xss_protection(self) -> Self
    pub fn high_security() -> Self
    pub fn for_api() -> Self
    pub fn for_web() -> Self
}
```

#### Request ID Middleware

```rust
impl RequestIdMiddleware {
    pub fn new() -> Self
    pub fn with_generator(self, generator: IdGenerator) -> Self
    pub fn with_header_name(self, header_name: &str) -> Self
}

#[derive(Debug, Clone)]
pub enum IdGenerator {
    UuidV4,
    NanoId { length: usize },
}
```

#### Other Middleware

```rust
// Compression
impl CompressionMiddleware {
    pub fn new() -> Self
    pub fn with_threshold(self, threshold: usize) -> Self
    pub fn with_level(self, level: CompressionLevel) -> Self
}

// Body Size Limit
impl BodySizeLimitMiddleware {
    pub fn new(limit: usize) -> Self
}
```

***

## Error Handling

Comprehensive error handling with automatic response conversion.

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Route not found: {0}")]
    NotFound(String),
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Custom error: {0}")]
    Custom(Box<dyn CustomError>),
}
```

### Error Methods

```rust
impl Error {
    pub fn status_code(&self) -> StatusCode
    pub fn error_type(&self) -> &'static str
    pub fn not_found(path: &str) -> Self
    pub fn bad_request(msg: impl Into<String>) -> Self
    pub fn validation(msg: impl Into<String>) -> Self
    pub fn unauthorized() -> Self
    pub fn forbidden() -> Self
    pub fn internal(msg: impl Into<String>) -> Self
}
```

### IntoResponse for Errors

```rust
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Automatic JSON error response with status code
    }
}
```

**Usage:**
```rust
async fn handler() -> Result<impl IntoResponse, Error> {
    let data = fetch_data().await?;  // Errors automatically convert
    Ok(Json(data))
}
```

***

## WebSocket Support

**Enhanced in v0.2.4+**: Full extractor support for WebSocket handlers.

### WebSocket Handler Traits

```rust
/// Core WebSocket handler trait
#[async_trait::async_trait]
pub trait WebSocketHandler: Send + Sync {
    async fn handle(&self, req: Request, websocket: WebSocketConnection) -> Response;
}

/// Universal handler with extractor support (0-7 extractors)
#[async_trait::async_trait]
pub trait UniversalWebSocketHandler<T>: Clone + Send + Sync + 'static {
    async fn call(&self, req: Request, websocket: WebSocketConnection) -> Response;
}
```

### WebSocket Connection

```rust
impl WebSocketConnection {
    pub async fn send(&self, message: Message) -> Result<()>
    pub async fn recv(&self) -> Option<Message>
    pub async fn send_text(&self, text: impl Into<String>) -> Result<()>
    pub async fn send_binary(&self, data: impl Into<Bytes>) -> Result<()>
    pub async fn send_json<T: Serialize>(&self, data: &T) -> Result<()>
    pub async fn send_ping(&self, data: impl Into<Bytes>) -> Result<()>
    pub async fn send_pong(&self, data: impl Into<Bytes>) -> Result<()>
    pub async fn close(&self, frame: Option<CloseFrame>) -> Result<()>
    pub fn clone(&self) -> Self
}
```

### WebSocket Message

```rust
#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Binary(Bytes),
    Ping(Bytes),
    Pong(Bytes),
    Close(Option<CloseFrame>),
}

#[derive(Debug, Clone)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

impl Message {
    pub fn text(text: impl Into<String>) -> Self
    pub fn binary(data: impl Into<Bytes>) -> Self
    pub fn ping(data: impl Into<Bytes>) -> Self
    pub fn pong(data: impl Into<Bytes>) -> Self
    pub fn close() -> Self
    pub fn close_with_reason(code: u16, reason: impl Into<String>) -> Self
    pub fn json<T: Serialize>(value: &T) -> Result<Self, serde_json::Error>
    pub fn parse_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error>
}
```

### WebSocket Helper Functions

```rust
/// Create a basic WebSocket handler from a closure
pub fn websocket_handler<F, Fut, R>(f: F) -> impl UniversalWebSocketHandler<()>
where
    F: Fn(WebSocketConnection) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse;

/// Create a per-message handler
pub fn websocket_message_handler<F, Fut, R>(f: F) -> OptimizedMessageHandler<F>
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse;

/// Create a batch message handler for high throughput
pub fn websocket_batch_handler<F, Fut, R>(
    f: F,
    batch_size: usize,
    timeout_ms: u64,
) -> BatchMessageHandler<F>
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse;
```

### WebSocket with Extractors

**New in v0.2.4+**: WebSocket handlers support all standard extractors.

```rust
// No extractors
router.websocket("/ws", |mut ws: WebSocketConnection| async move {
    ws.send_text("Hello!").await.ok();
    Response::ok()
})

// With State
router.websocket("/ws", |
    State(state): State<AppState>,
    mut ws: WebSocketConnection
| async move {
    // Access shared state
    ws.send_text("Connected!").await.ok();
    "Success"
})

// With Path parameters
router.websocket("/ws/{room_id}", |
    Path(room_id): Path<String>,
    mut ws: WebSocketConnection
| async move {
    ws.send_text(format!("Room: {}", room_id)).await.ok();
    Response::ok()
})

// With Query parameters
#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

router.websocket("/ws", |
    Query(query): Query<WsQuery>,
    mut ws: WebSocketConnection
| async move {
    if !validate_token(&query.token).await {
        return Response::unauthorized("Invalid token");
    }
    ws.send_text("Authenticated!").await.ok();
    Response::ok()
})

// Multiple extractors (up to 7)
router.websocket("/ws/{user_id}", |
    Path(user_id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    Headers(headers): Headers,
    Extension(db): Extension<Arc<Database>>,
    // Cookies(cookies): Cookies,  // 6th extractor
    // RequestId(id): RequestId,   // 7th extractor
    mut ws: WebSocketConnection
| async move {
    // All extractors available before WebSocket handling
    ws.send_text(format!("Hello, {}", user_id)).await.ok();

    while let Some(msg) = ws.recv().await {
        // Handle messages with full context...
    }

    (StatusCode::OK, "Session ended")
})
```

### Protocol Support

**HTTP/1.1 WebSocket**: ✅ Fully supported (RFC 6455)
- Standard upgrade handshake
- Works over HTTP (`ws://`) and HTTPS (`wss://`)
- 101 Switching Protocols response
- Full TLS encryption support

**HTTP/2 WebSocket**: ⏳ Not yet supported (RFC 8441)
- Requires Hyper/h2 enhancements
- Browser support varies
- WebSocket automatically uses HTTP/1.1 even when server supports HTTP/2

**Secure WebSocket (wss://)**: ✅ Fully supported
- TLS encryption for all WebSocket traffic
- Certificate validation
- Works over HTTPS with HTTP/1.1

### WebSocket Return Types

WebSocket handlers must return types implementing `IntoResponse`:

```rust
// ✅ Valid return types
Response::ok()
"Connection closed"
Json(json!({"status": "completed"}))
(StatusCode::OK, "Success")
Result<Response>
Result<String>

// ❌ Invalid
Ok(())  // Empty tuple doesn't implement IntoResponse
```

***

## Multipart & File Uploads

```rust
impl Multipart {
    pub async fn next_field(&mut self) -> Result<Option<Field>>
}

impl Field {
    pub fn name(&self) -> Option<&str>
    pub fn file_name(&self) -> Option<&str>
    pub fn content_type(&self) -> Option<&str>
    pub async fn bytes(&mut self) -> Result<Bytes>
    pub async fn text(&mut self) -> Result<String>
    pub async fn save_to_file(&mut self, path: impl AsRef<Path>) -> Result<u64>
}
```

***

## Cookie Management

```rust
impl CookieJar {
    pub fn get(&self, name: &str) -> Option<Cookie>
    pub fn add(&mut self, cookie: Cookie)
    pub fn remove(&mut self, name: &str)
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self
    pub fn with_domain(self, domain: impl Into<String>) -> Self
    pub fn with_path(self, path: impl Into<String>) -> Self
    pub fn with_max_age(self, max_age: Duration) -> Self
    pub fn with_expires(self, expires: SystemTime) -> Self
    pub fn with_secure(self, secure: bool) -> Self
    pub fn with_http_only(self, http_only: bool) -> Self
    pub fn with_same_site(self, same_site: SameSite) -> Self
}
```

***

## State & Extensions

### State Management

```rust
// Add state to router
let router = Router::new()
    .state(AppState::new());

// Extract state in handlers
async fn handler(State(state): State<AppState>) -> impl IntoResponse {
    // Use state...
}

// Extract state in WebSocket handlers
router.websocket("/ws", |
    State(state): State<AppState>,
    mut ws: WebSocketConnection
| async move {
    // Use state in WebSocket...
    Response::ok()
})
```

### Extensions

```rust
// Add extensions to router
let router = Router::new()
    .extension(Arc::new(Database::new()));

// Extract extensions in handlers
async fn handler(Extension(db): Extension<Arc<Database>>) -> impl IntoResponse {
    // Use database...
}

// Extract extensions in WebSocket handlers
router.websocket("/ws", |
    Extension(db): Extension<Arc<Database>>,
    mut ws: WebSocketConnection
| async move {
    // Use database in WebSocket...
    Response::ok()
})
```

***

## TLS & HTTPS

### TLS Configuration

```rust
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub alpn_protocols: Vec<Vec<u8>>,
}

impl TlsConfig {
    pub fn new(cert: Vec<u8>, key: Vec<u8>) -> Self
    pub fn from_files(cert_path: impl AsRef<Path>, key_path: impl AsRef<Path>) -> Result<Self>
    pub fn with_alpn(self, protocols: Vec<Vec<u8>>) -> Self
}
```

### Usage

```rust
// Development (self-signed)
let server = Server::new(router, addr)
    .with_self_signed_cert("localhost")?;

// Production (proper certificates)
let server = Server::new(router, addr)
    .with_server_config(ServerConfig {
        tls: Some(TlsConfig::from_files("cert.pem", "key.pem")?),
        ..Default::default()
    });
```

***

## Configuration

### ServerConfig

```rust
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub http1_enabled: bool,
    pub http2: Http2Config,
    pub tls: Option<TlsConfig>,
    pub graceful_shutdown_timeout: Duration,
}
```

### Http2Config

```rust
#[derive(Debug, Clone)]
pub struct Http2Config {
    pub enabled: bool,
    pub enable_connect_protocol: bool,  // RFC 8441 (not yet functional)
    pub max_concurrent_streams: Option<u32>,
    pub initial_connection_window_size: Option<u32>,
    pub initial_stream_window_size: Option<u32>,
    pub max_frame_size: Option<u32>,
    pub keepalive_interval: Option<Duration>,
    pub keepalive_timeout: Option<Duration>,
    pub adaptive_window: bool,
    pub max_header_list_size: Option<u32>,
}
```

***

## Performance

### Benchmarks (v0.2.4, wrk -c100 -d30s)

```
Throughput:    51,574 req/sec
Latency:       1.90ms avg, 10.60ms max
Transfer:      7.97 MB/sec
Consistency:   70.54% within 1 std dev
```

### Performance Optimizations (v0.2.4)

- **Middleware**: 94% latency reduction (30ms → 1.9ms)
- **Body handling**: 40% reduction in allocations
- **Radix routing**: O(log n) lookups, zero-copy matching
- **Response building**: Zero-copy for static content
- **WebSocket**: Zero-copy message passing where possible

### Performance Configuration

```rust
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub reuse_port: bool,
    pub tcp_nodelay: bool,
    pub reuse_addr: bool,
    pub keepalive: Option<Duration>,
    pub send_buffer_size: Option<usize>,
    pub recv_buffer_size: Option<usize>,
    pub backlog: u32,
    pub cpu_affinity: bool,
    pub worker_threads: usize,
    pub fast_path: bool,
    pub zero_copy: bool,
}

impl PerformanceConfig {
    pub fn max_rps() -> Self
    pub fn high_throughput_api() -> Self
    pub fn default() -> Self
}
```

***

## Routing

### Path Patterns

```rust
// Static paths
router.get("/users", handler)

// Path parameters
router.get("/users/{id}", handler)
router.get("/users/{user_id}/posts/{post_id}", handler)

// Wildcard (catches all)
router.get("/files/*path", handler)

// WebSocket routes
router.websocket("/ws", handler)
router.websocket("/ws/{room_id}", handler)
```

### Route Priority

1. Exact matches (highest priority)
2. Path parameters
3. Wildcards (lowest priority)

***

## Handler Signatures

Ignitia supports flexible handler signatures with up to 16 extractors for HTTP and up to 7 extractors for WebSocket handlers:

### HTTP Handlers

```rust
// No parameters
async fn handler() -> impl IntoResponse {
    "Hello"
}

// Single extractor
async fn handler(Path(id): Path<u32>) -> impl IntoResponse {
    format!("ID: {}", id)
}

// Multiple extractors (up to 16)
async fn handler(
    Path(id): Path<u32>,
    Query(params): Query<SearchParams>,
    Json(data): Json<CreateData>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    Json(data)
}

// Result with IntoResponse
async fn handler() -> Result<impl IntoResponse, Error> {
    let data = fetch_data().await?;
    Ok(Json(data))
}
```

### WebSocket Handlers

```rust
// No extractors
router.websocket("/ws", |mut ws: WebSocketConnection| async move {
    ws.send_text("Hello!").await.ok();
    Response::ok()
})

// With extractors (up to 7)
router.websocket("/ws/{room}", |
    Path(room): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    Extension(db): Extension<Arc<Database>>,
    Headers(headers): Headers,
    Cookies(cookies): Cookies,
    // RequestId(id): RequestId,  // 7th extractor
    mut ws: WebSocketConnection
| async move {
    // Handle with full context
    ws.send_text("Connected!").await.ok();
    Response::ok()
})
```

***

## Feature Flags

```toml
[dependencies]
ignitia = {
    version = "0.2.4",
    features = ["tls", "websocket", "self-signed"]
}
```

### Available Features

- **`tls`** - Enables HTTPS/TLS support
- **`websocket`** - Enables WebSocket protocol support (HTTP/1.1)
- **`self-signed`** - Enables self-signed certificate generation for development

***

## Result Type

The framework uses a standard `Result` type:

```
pub type Result<T, E = Error> = std::result::Result<T, E>;
```

***

## Version 0.2.4 Changes Summary

### Breaking Changes

1. **Router Mode**: Removed `RouterMode` enum and `with_mode()` - now exclusively Radix
2. **Body Type**: Changed from `Arc<Bytes>` to `Bytes` in Request and Response
3. **Response Methods**: `json()`, `html()`, `text()` now return `Response` directly (not `Result`)
4. **Json Type**: Unified into single type for extraction and response

### New Features

1. **IntoResponse Trait**: Ergonomic response conversion
2. **WebSocket Extractors**: Full extractor support (0-7 parameters) for WebSocket handlers
3. **Extended Type Support**: bool, numbers, Option, Cow strings
4. **Better Error Handling**: Automatic error-to-response conversion

### Performance

- Middleware: 94% latency reduction
- Body handling: 40% allocation reduction
- Throughput: 51,574 req/sec
- WebSocket: Zero-copy optimizations

### Protocol Support

- HTTP/1.1: ✅ Fully supported
- HTTP/2: ✅ Supported for regular HTTP requests
- HTTP/1.1 WebSocket: ✅ Fully supported (ws:// and wss://)
- HTTP/2 WebSocket: ⏳ Not yet supported (RFC 8441)

See [CHANGELOG.md](CHANGELOG.md) for complete migration guide and [WEB_SOCKETS.md](WEB_SOCKETS.md) for detailed WebSocket documentation.

***

**🔥 Ready to build blazing fast web applications with real-time capabilities? Get started with Ignitia today!**
