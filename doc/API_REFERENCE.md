# API Reference

Complete API documentation for the Ignitia web framework.

## Table of Contents

- [Core Types](#core-types)
- [Router](#router)
- [Server](#server)
- [Request & Response](#request--response)
- [Extractors](#extractors)
- [Middleware](#middleware)
- [Error Handling](#error-handling)
- [WebSocket Support](#websocket-support)
- [Multipart & File Uploads](#multipart--file-uploads)
- [Cookie Management](#cookie-management)
- [State & Extensions](#state--extensions)
- [TLS & HTTPS](#tls--https)
- [Configuration](#configuration)

***

## Core Types

### Router

The main routing component for defining application routes and middleware.

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
    pub fn middleware<M: Middleware>(self, middleware: M) -> Self
    pub fn nest(self, path: &str, router: Router) -> Self
    pub fn state<T: Clone + Send + Sync + 'static>(self, state: T) -> Self
    pub fn not_found<H, T>(self, handler: H) -> Self
}
```

#### Route Registration Methods

- **`get(path, handler)`** - Register GET route
- **`post(path, handler)`** - Register POST route
- **`put(path, handler)`** - Register PUT route
- **`delete(path, handler)`** - Register DELETE route
- **`patch(path, handler)`** - Register PATCH route
- **`head(path, handler)`** - Register HEAD route
- **`options(path, handler)`** - Register OPTIONS route

#### Advanced Methods

- **`middleware(middleware)`** - Add global middleware
- **`nest(path, router)`** - Mount sub-router at path
- **`state(state)`** - Add shared application state
- **`not_found(handler)`** - Set 404 handler

#### WebSocket Support

```rust
#[cfg(feature = "websocket")]
impl Router {
    pub fn websocket<H: WebSocketHandler>(self, path: &str, handler: H) -> Self
    pub fn websocket_fn<F, Fut>(self, path: &str, f: F) -> Self
}
```

### Server

HTTP server configuration and startup.

```rust
impl Server {
    pub fn new(router: Router, addr: SocketAddr) -> Self
    pub fn with_config(self, config: ServerConfig) -> Self
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

HTTP request representation with extractors support.

```rust
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

HTTP response builder with convenience methods.

```rust
impl Response {
    pub fn new(status: StatusCode) -> Self
    pub fn with_status(self, status: StatusCode) -> Self
    pub fn with_status_code(self, status_code: u16) -> Self
    pub fn with_body(self, body: impl Into<Bytes>) -> Self
    pub fn ok() -> Self
    pub fn not_found() -> Self
    pub fn internal_error() -> Self
}
```

#### Content Type Methods

```rust
impl Response {
    pub fn json<T: Serialize>(data: T) -> Result<Self>
    pub fn text(text: impl Into<String>) -> Self
    pub fn html(html: impl Into<String>) -> Self
}
```

#### Error Response Methods

```rust
impl Response {
    pub fn error_json<E: Into<Error>>(error: E) -> Result<Self>
    pub fn validation_error(messages: Vec<String>) -> Result<Self>
}
```

#### Cookie Methods

```rust
impl Response {
    pub fn add_cookie(self, cookie: Cookie) -> Self
    pub fn add_cookies(self, cookies: Vec<Cookie>) -> Self
    pub fn remove_cookie(self, name: impl Into<String>) -> Self
}
```

***

## Extractors

Type-safe request data extraction for handlers.

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
async fn get_user(Path(user_id): Path<u32>) -> Result<Response> {
    // user_id is extracted from URL path
}

// Multiple parameters
#[derive(Deserialize)]
struct UserPost {
    user_id: u32,
    post_id: u32,
}

async fn get_post(Path(params): Path<UserPost>) -> Result<Response> {
    // params.user_id and params.post_id extracted from /users/:user_id/posts/:post_id
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
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_users(Query(params): Query<Pagination>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
}
```

### JSON Body

```rust
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T
}
```

**Usage:**
```rust
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    // user is deserialized from JSON body
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

**Usage:**
```rust
#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(Form(form): Form<LoginForm>) -> Result<Response> {
    // form data from application/x-www-form-urlencoded
}
```

### Other Extractors

```rust
// Raw request body
pub struct Body(pub Bytes);

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

Composable middleware system for cross-cutting concerns.

### Middleware Trait

```rust
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, req: &mut Request) -> Result<()>
    async fn after(&self, req: &Request, res: &mut Response) -> Result<()>
}
```

### Built-in Middleware

#### Logger Middleware

```rust
pub struct LoggerMiddleware;

// Usage
let router = Router::new()
    .middleware(LoggerMiddleware)
    .get("/", handler);
```

#### CORS Middleware

```rust
impl Cors {
    pub fn new() -> Self
    pub fn allow_any_origin(self) -> Self
    pub fn allowed_origin(self, origin: &str) -> Self
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

**Usage:**
```rust
let cors = Cors::new()
    .allowed_origins(&["https://example.com"])
    .allowed_methods(&[Method::GET, Method::POST])
    .allow_credentials()
    .build()?;

let router = Router::new()
    .middleware(cors)
    .get("/api/data", handler);
```

#### Authentication Middleware

```rust
impl AuthMiddleware {
    pub fn new(token: impl Into<String>) -> Self
    pub fn protect_path(self, path: impl Into<String>) -> Self
    pub fn protect_paths(self, paths: Vec<impl Into<String>>) -> Self
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
    pub fn with_hsts(self, enabled: bool) -> Self
    pub fn with_csp(self, config: CspConfig) -> Self
    pub fn with_rate_limit(self, max_requests: u32, window: Duration) -> Self
    pub fn for_api() -> Self
    pub fn for_web() -> Self
    pub fn high_security() -> Self
}
```

#### Compression Middleware

```rust
impl CompressionMiddleware {
    pub fn new() -> Self
    pub fn with_threshold(self, threshold: usize) -> Self
    pub fn with_level(self, level: CompressionLevel) -> Self
    pub fn with_gzip(self, enabled: bool) -> Self
    pub fn with_brotli(self, enabled: bool) -> Self
    pub fn for_api() -> Self
    pub fn for_web() -> Self
}
```

***

## Error Handling

Comprehensive error handling with custom error types.

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
    pub fn to_response(&self, include_timestamp: bool) -> ErrorResponse
}
```

### Custom Error Trait

```rust
pub trait CustomError: fmt::Debug + fmt::Display + Send + Sync + 'static {
    fn status_code(&self) -> StatusCode;
    fn error_type(&self) -> &'static str;
    fn error_code(&self) -> Option<String> { None }
    fn metadata(&self) -> Option<serde_json::Value> { None }
}
```

### Error Response Format

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
    pub error_type: Option<String>,
    pub error_code: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: Option<String>,
}
```

### Error Helper Macro

```rust
define_error! {
    MyError {
        NotFound(StatusCode::NOT_FOUND, "not_found"),
        InvalidInput(StatusCode::BAD_REQUEST, "invalid_input", "INVALID_INPUT"),
        DatabaseError(StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
    }
}
```

***

## WebSocket Support

Real-time WebSocket communication support.

### WebSocket Handler Trait

```rust
#[async_trait::async_trait]
pub trait WebSocketHandler: Send + Sync {
    async fn handle_connection(&self, websocket: WebSocketConnection) -> Result<()>;
    async fn on_message(&self, websocket: &WebSocketConnection, message: Message) -> Result<()>;
    async fn on_connect(&self, websocket: &WebSocketConnection) -> Result<()>;
    async fn on_disconnect(&self, websocket: &WebSocketConnection, reason: Option<&str>) -> Result<()>;
}
```

### WebSocket Connection

```rust
impl WebSocketConnection {
    pub async fn send(&self, message: Message) -> Result<()>
    pub async fn recv(&self) -> Option<Message>
    pub async fn recv_timeout(&self, timeout: Duration) -> Option<Message>
    pub async fn send_text(&self, text: String) -> Result<()>
    pub async fn send_bytes(&self, data: Bytes) -> Result<()>
    pub async fn send_json<T: Serialize>(&self, data: &T) -> Result<()>
    pub async fn close(&self) -> Result<()>
    pub async fn close_with_reason(&self, code: u16, reason: String) -> Result<()>
    pub async fn ping(&self, data: impl Into<Bytes>) -> Result<()>
    pub async fn pong(&self, data: impl Into<Bytes>) -> Result<()>
    pub async fn send_batch(&self, messages: Vec<Message>) -> Result<()>
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

### Handler Creation

```rust
// Simple function handler
pub fn websocket_handler<F, Fut>(f: F) -> WebSocketHandlerFn
where
    F: Fn(WebSocketConnection) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static;

// Message-based handler
pub fn websocket_message_handler<F, Fut>(f: F) -> impl WebSocketHandler
where
    F: Fn(WebSocketConnection, Message) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static;

// Batch handler for high throughput
pub fn websocket_batch_handler<F, Fut>(f: F, batch_size: usize, timeout_ms: u64) -> impl WebSocketHandler
where
    F: Fn(WebSocketConnection, Vec<Message>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static;
```

### Router Integration

```rust
let router = Router::new()
    .websocket("/ws", websocket_handler(|ws| async move {
        while let Some(message) = ws.recv().await {
            match message {
                Message::Text(text) => {
                    ws.send_text(format!("Echo: {}", text)).await?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    }))
    .websocket_fn("/chat", |ws| async move {
        // Handle chat connection
        Ok(())
    });
```

***

## Multipart & File Uploads

Advanced file upload handling with multipart form data.

### Multipart Extractor

```rust
impl FromRequest for Multipart {
    fn from_request(req: &Request) -> Result<Self>
}
```

### Multipart Parser

```rust
impl Multipart {
    pub fn new(body: Bytes, boundary: String, config: MultipartConfig) -> Self
    pub async fn next_field(&mut self) -> Result<Option<Field>, MultipartError>
    pub async fn collect_all(self) -> Result<MultipartData, MultipartError>
}
```

### Field Types

```rust
impl Field {
    pub fn name(&self) -> &str
    pub fn file_name(&self) -> Option<&str>
    pub fn content_type(&self) -> Option<&str>
    pub fn headers(&self) -> &HashMap<String, String>
    pub fn is_file(&self) -> bool
    pub async fn bytes(self) -> Result<Bytes, MultipartError>
    pub async fn text(self) -> Result<String, MultipartError>
    pub async fn save_to_file<P: AsRef<Path>>(self, path: P) -> Result<FileField, MultipartError>
}
```

### File Field

```rust
#[derive(Debug, Clone)]
pub struct FileField {
    pub name: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub path: PathBuf,
    pub size: u64,
}

impl FileField {
    pub async fn bytes(&self) -> Result<Bytes, MultipartError>
    pub async fn text(&self) -> Result<String, MultipartError>
    pub async fn persist<P: AsRef<Path>>(self, new_path: P) -> Result<FileField, MultipartError>
}
```

### Configuration

```rust
#[derive(Debug, Clone)]
pub struct MultipartConfig {
    pub max_request_size: usize,    // Default: 10MB
    pub max_field_size: usize,      // Default: 1MB
    pub file_size_threshold: usize, // Default: 256KB
    pub max_fields: usize,          // Default: 100
}
```

**Usage:**
```rust
async fn upload_handler(mut multipart: Multipart) -> Result<Response> {
    let mut files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            let filename = field.file_name().unwrap_or("unknown");
            let file_field = field.save_to_file(format!("./uploads/{}", filename)).await?;
            files.push(file_field);
        }
    }

    Response::json(serde_json::json!({
        "uploaded": files.len(),
        "files": files
    }))
}
```

***

## Cookie Management

Full-featured cookie handling with security attributes.

### Cookie

```rust
impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self
    pub fn path(self, path: impl Into<String>) -> Self
    pub fn domain(self, domain: impl Into<String>) -> Self
    pub fn max_age(self, seconds: u64) -> Self
    pub fn expires(self, time: SystemTime) -> Self
    pub fn secure(self) -> Self
    pub fn http_only(self) -> Self
    pub fn same_site(self, same_site: SameSite) -> Self
    pub fn removal(name: impl Into<String>) -> Self
    pub fn to_header_value(&self) -> String
}
```

### SameSite Options

```rust
#[derive(Debug, Clone)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}
```

### Cookie Jar

```rust
impl CookieJar {
    pub fn new() -> Self
    pub fn from_header(cookie_header: &str) -> Self
    pub fn get(&self, name: &str) -> Option<&String>
    pub fn contains(&self, name: &str) -> bool
    pub fn all(&self) -> &HashMap<String, String>
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool
}
```

**Usage:**
```rust
// Setting cookies
let cookie = Cookie::new("session_id", "abc123")
    .path("/")
    .http_only()
    .secure()
    .same_site(SameSite::Strict)
    .max_age(3600); // 1 hour

let response = Response::ok()
    .add_cookie(cookie);

// Reading cookies
async fn handler(cookies: Cookies) -> Result<Response> {
    if let Some(session_id) = cookies.get("session_id") {
        // Handle authenticated user
    }
    Response::ok()
}
```

***

## State & Extensions

Shared application state and request extensions.

### Application State

```rust
// Define state
#[derive(Clone)]
struct AppState {
    db_pool: DatabasePool,
    config: AppConfig,
}

// Add to router
let router = Router::new()
    .state(app_state)
    .get("/users", get_users);

// Use in handler
async fn get_users(State(state): State<AppState>) -> Result<Response> {
    let users = state.db_pool.get_users().await?;
    Response::json(users)
}
```

### Extensions

```rust
impl Extensions {
    pub fn new() -> Self
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T)
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>>
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T>
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool
    pub fn clear(&mut self)
}

// Extension extractor
async fn handler(Extension(user): Extension<User>) -> Result<Response> {
    Response::json(user)
}
```

***

## TLS & HTTPS

Built-in TLS support with certificate management.

### TLS Configuration

```rust
#[cfg(feature = "tls")]
impl TlsConfig {
    pub fn new(cert_file: impl Into<String>, key_file: impl Into<String>) -> Self
    pub fn with_alpn_protocols(self, protocols: Vec<&str>) -> Self
    pub fn enable_client_cert_verification(self) -> Self
    pub fn tls_versions(self, min: TlsVersion, max: TlsVersion) -> Self
    pub fn build(&self) -> Result<TlsAcceptor, TlsError>
}
```

### TLS Versions

```rust
#[derive(Debug, Clone)]
pub enum TlsVersion {
    TlsV12,
    TlsV13,
}
```

### Self-Signed Certificates

```rust
#[cfg(feature = "self-signed")]
impl TlsConfig {
    pub fn generate_self_signed(domain: &str) -> Result<(String, String), TlsError>
}
```

**Usage:**
```rust
// Production HTTPS
let server = Server::new(router, "0.0.0.0:8443".parse()?)
    .enable_https("production.crt", "production.key")?
    .ignitia()
    .await?;

// Development with self-signed
let server = Server::new(router, "127.0.0.1:8443".parse()?)
    .with_self_signed_cert("localhost")?
    .ignitia()
    .await?;

// Custom TLS configuration
let tls_config = TlsConfig::new("cert.pem", "key.pem")
    .with_alpn_protocols(vec!["h2", "http/1.1"])
    .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13);

let server = Server::new(router, addr)
    .with_tls(tls_config)?
    .ignitia()
    .await?;
```

***

## Configuration

Server and HTTP protocol configuration.

### Server Configuration

```rust
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub http1_enabled: bool,
    pub http2: Http2Config,
    pub auto_protocol_detection: bool,
    pub tls: Option<TlsConfig>,
    pub redirect_http_to_https: bool,
    pub https_port: Option<u16>,
}

impl ServerConfig {
    pub fn with_tls(self, tls_config: TlsConfig) -> Self
    pub fn redirect_to_https(self, https_port: u16) -> Self
    pub fn is_tls_enabled(&self) -> bool
}
```

### HTTP/2 Configuration

```rust
#[derive(Debug, Clone)]
pub struct Http2Config {
    pub enabled: bool,
    pub enable_prior_knowledge: bool,
    pub max_concurrent_streams: Option<u32>,
    pub initial_connection_window_size: Option<u32>,
    pub initial_stream_window_size: Option<u32>,
    pub max_frame_size: Option<u32>,
    pub keep_alive_interval: Option<Duration>,
    pub keep_alive_timeout: Option<Duration>,
    pub adaptive_window: bool,
    pub max_header_list_size: Option<u32>,
}
```

**Usage:**
```rust
let config = ServerConfig {
    http2: Http2Config {
        enabled: true,
        max_concurrent_streams: Some(1000),
        initial_connection_window_size: Some(1024 * 1024), // 1MB
        adaptive_window: true,
        ..Default::default()
    },
    ..Default::default()
};

let server = Server::new(router, addr)
    .with_config(config)
    .ignitia()
    .await?;
```

***

## Handler Signatures

Ignitia supports various handler signatures through automatic extraction:

```rust
// No parameters
async fn handler() -> Result<Response> { ... }

// Single extractor
async fn handler(Path(id): Path<u32>) -> Result<Response> { ... }

// Multiple extractors
async fn handler(
    Path(id): Path<u32>,
    Query(params): Query<SearchParams>,
    Json(data): Json<CreateData>
) -> Result<Response> { ... }

// With state
async fn handler(
    State(state): State<AppState>,
    Path(id): Path<u32>
) -> Result<Response> { ... }

// Maximum 8 extractors supported
async fn handler(
    State(state): State<AppState>,
    Path(params): Path<PathParams>,
    Query(query): Query<QueryParams>,
    Json(body): Json<RequestBody>,
    Headers(headers): Headers,
    Method(method): Method,
    Uri(uri): Uri,
    Extension(user): Extension<User>
) -> Result<Response> { ... }
```

***

## Feature Flags

Configure Ignitia features through Cargo.toml:

```toml
[dependencies]
ignitia = {
    version = "0.2.0",
    features = ["tls", "websocket", "self-signed"]
}
```

### Available Features

- **`tls`** - Enables HTTPS/TLS support with certificate management and ALPN
- **`websocket`** - Enables WebSocket protocol support with connection management
- **`self-signed`** - Enables self-signed certificate generation (development only)

***

## Result Type

All fallible operations return `Result<T, Error>`:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

This ensures consistent error handling across the framework.
