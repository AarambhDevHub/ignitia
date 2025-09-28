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
- [Performance](#performance)
- [Routing](#routing)
- [Handler Signatures](#handler-signatures)
- [Feature Flags](#feature-flags)
- [Result Type](#result-type)

***

## Core Types

### Router

The main routing component for defining application routes and middleware. Ignitia now features a high-performance Radix tree router as the default routing engine.

```rust
impl Router {
    pub fn new() -> Self
    pub fn with_mode(mut self, mode: RouterMode) -> Self
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
    pub fn state_arc<T>(self, state: Arc<T>) -> Self
    pub fn state_factory<T, F>(self, factory: F) -> Self
    pub fn not_found<H, T>(self, handler: H) -> Self
    pub fn mode(self) -> RouterMode
    pub fn clear_cache(self)
    pub fn stats(self) -> Option<RadixStats>
    pub fn print_tree(self)
}
```

#### RouterMode

Choose between routing implementations:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterMode {
    /// Radix tree router (default) - High performance with O(1) lookup
    Radix,
    /// Original regex-based router - Legacy support
    Base,
}
```

#### WebSocket Support

```rust
#[cfg(feature = "websocket")]
impl Router {
    pub fn websocket<H: WebSocketHandler>(self, path: &str, handler: H) -> Self
    pub fn websocket_fn<F, Fut>(self, path: &str, f: F) -> Self
    pub fn get_websocket_handlers(self) -> DashMap<String, Arc<dyn WebSocketHandler>>
}
```

### Server

HTTP server configuration and startup with enhanced performance features.

```rust
impl Server {
    pub fn new(router: Router, addr: SocketAddr) -> Self
    pub fn with_config(self, config: ServerConfig) -> Self
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

Enhanced HTTP request representation with optimized parameter parsing and extension support.

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

Enhanced response builder with caching support and performance optimizations.

```rust
impl Response {
    pub fn new(status: StatusCode) -> Self
    pub fn with_status(self, status: StatusCode) -> Self
    pub fn with_status_code(self, status_code: u16) -> Self
    pub fn with_body(self, body: impl Into<Bytes>) -> Self
    pub fn body_shared(self) -> Arc<Bytes>
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
    pub fn with_cache_control(mut self, max_age: u64) -> Self
}
```

#### Error Response Methods

```rust
impl Response {
    pub fn error_json<E: Into<Error>>(error: E) -> Result<Self>
    pub fn validation_error(messages: Vec<String>) -> Result<Self>
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

### JSON Body

```rust
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T
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

Enhanced composable middleware system with performance optimizations.

### Middleware Trait

```rust
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, req: &mut Request) -> Result<()>;
    async fn after(&self, req: &Request, res: &mut Response) -> Result<()>;
}
```

### Built-in Middleware

#### Logger Middleware

```rust
pub struct LoggerMiddleware;

impl LoggerMiddleware {
    pub fn new() -> Self
    pub fn with_format(self, format: LogFormat) -> Self
}
```

#### CORS Middleware

Enhanced CORS middleware with regex origin matching:

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

#### Authentication Middleware

```rust
impl AuthMiddleware {
    pub fn new(token: impl Into<String>) -> Self
    pub fn protect_path(self, path: impl Into<String>) -> Self
    pub fn protect_paths(self, paths: Vec<impl Into<String>>) -> Self
}
```

#### Rate Limiting Middleware

Advanced rate limiting with configurable windows:

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

Comprehensive security middleware:

```rust
impl SecurityMiddleware {
    pub fn new() -> Self
    pub fn with_hsts_config(self, max_age: u64, include_subdomains: bool, preload: bool) -> Self
    pub fn with_csp(self, config: CspConfig) -> Self
    pub fn with_rate_limit(self, max_requests: u32, window: Duration) -> Self
    pub fn for_api() -> Self
    pub fn for_web() -> Self
    pub fn high_security() -> Self
    pub fn for_development() -> Self
}
```

#### Request ID Middleware

Enhanced request ID generation with customizable generators:

```rust
impl RequestIdMiddleware {
    pub fn new() -> Self
    pub fn with_generator(mut self, generator: IdGenerator) -> Self
    pub fn with_header_name(mut self, header_name: &str) -> Self
    pub fn with_validation(mut self, validate: bool) -> Self
    pub fn with_logging(mut self, enable: bool) -> Self
    pub fn for_microservices() -> Self
}

#[derive(Debug, Clone)]
pub enum IdGenerator {
    UuidV4,
    NanoId { length: usize },
    Custom(Arc<dyn Fn() -> String + Send + Sync>),
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

#### Body Size Limit Middleware

```rust
impl BodySizeLimitMiddleware {
    pub fn new(limit: usize) -> Self
    pub fn with_error_handler<F>(self, handler: F) -> Self
}
```

#### Error Handler Middleware

```rust
impl ErrorHandlerMiddleware {
    pub fn new() -> Self
    pub fn with_details(mut self, include: bool) -> Self
    pub fn with_stack_trace(mut self, include: bool) -> Self
    pub fn with_custom_error_page(mut self, status: StatusCode, html: String) -> Self
    pub fn with_logging(mut self, log_errors: bool) -> Self
}
```

***

## Error Handling

Comprehensive error handling with custom error types and enhanced error responses.

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
    #[error("Database error: {0}")]
    Database(String),
    #[error("External service error: {0}")]
    ExternalService(String),
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

### Error Extension Trait

Convenient error conversion methods:

```rust
pub trait ErrorExt<T> {
    fn bad_request(self) -> Result<T>;
    fn unauthorized(self) -> Result<T>;
    fn forbidden(self) -> Result<T>;
    fn internal_error(self) -> Result<T>;
    fn validation_error(self) -> Result<T>;
}
```

***

## WebSocket Support

Enhanced WebSocket support with batch processing and optimized message handlers.

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

Enhanced connection with batch operations:

```rust
impl WebSocketConnection {
    pub async fn send(&self, message: Message) -> Result<()>
    pub async fn recv(&self) -> Option<Message>
    pub async fn recv_timeout(&self, timeout: Duration) -> Option<Message>
    pub async fn send_text(&self, text: String) -> Result<()>
    pub async fn send_bytes(&self, data: Bytes) -> Result<()>
    pub async fn send_json<T: Serialize>(&self, data: &T) -> Result<()>
    pub async fn send_batch(&self, messages: Vec<Message>) -> Result<()>
    pub async fn close(&self) -> Result<()>
    pub async fn close_with_reason(&self, code: u16, reason: String) -> Result<()>
    pub async fn ping(&self, data: impl Into<Bytes>) -> Result<()>
    pub async fn pong(&self, data: impl Into<Bytes>) -> Result<()>
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

Enhanced handler creation with batch support:

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

***

## Multipart & File Uploads

Advanced file upload handling with enhanced configuration and temporary file management.

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

Enhanced field handling:

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

### Enhanced Configuration

```rust
#[derive(Debug, Clone)]
pub struct MultipartConfig {
    pub max_request_size: usize,      // Default: 10MB
    pub max_field_size: usize,        // Default: 1MB
    pub file_size_threshold: usize,   // Default: 256KB
    pub max_fields: usize,            // Default: 100
    pub temp_dir: Option<PathBuf>,    // Custom temp directory
    pub preserve_filename: bool,      // Preserve original filename
}
```

***

## Cookie Management

Enhanced cookie handling with security attributes and SameSite support.

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

***

## State & Extensions

Enhanced state management and request extensions.

### Application State

```rust
// Multiple state registration methods
impl Router {
    pub fn state<T: Clone + Send + Sync + 'static>(self, state: T) -> Self
    pub fn state_arc<T>(self, state: Arc<T>) -> Self
    pub fn state_factory<T, F>(self, factory: F) -> Self
    pub fn has_state<T: Send + Sync + Clone + 'static>(&self) -> bool
    pub fn get_state<T: Clone + Send + Sync + 'static>(&self) -> Option<T>
}
```

### Extensions

Enhanced extension system:

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
```

***

## TLS & HTTPS

Enhanced TLS support with advanced configuration options.

### TLS Configuration

```rust
#[cfg(feature = "tls")]
impl TlsConfig {
    pub fn new(cert_file: impl Into<String>, key_file: impl Into<String>) -> Self
    pub fn with_alpn_protocols(mut self, protocols: Vec<&str>) -> Self
    pub fn enable_client_cert_verification(mut self) -> Self
    pub fn tls_versions(mut self, min: TlsVersion, max: TlsVersion) -> Self
    pub fn build(&self) -> Result<TlsAcceptor, TlsError>
}
```

### Self-Signed Certificate Generation

```rust
#[cfg(feature = "self-signed")]
impl TlsConfig {
    pub fn generate_self_signed(domain: &str) -> Result<(String, String), TlsError>
}
```

***

## Configuration

Enhanced server configuration with performance tuning options.

### Server Configuration

```rust
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub http1_enabled: bool,
    pub http2: Http2Config,
    pub auto_protocol_detection: bool,
    #[cfg(feature = "tls")]
    pub tls: Option<TlsConfig>,
    pub redirect_http_to_https: bool,
    pub https_port: Option<u16>,
    pub max_request_body_size: usize,
}
```

### HTTP/2 Configuration

Enhanced HTTP/2 configuration:

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

***

## Performance

Advanced performance configuration and optimization features.

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

### Connection Pool Configuration

```rust
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub keep_alive_timeout: Duration,
    pub pool_idle_timeout: Duration,
    pub max_idle_connections: usize,
    pub connection_reuse: bool,
}
```

### Performance Metrics

Built-in performance monitoring:

```rust
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub requests_total: AtomicU64,
    pub requests_per_second: AtomicU64,
    pub active_connections: AtomicUsize,
    pub response_times: RwLock<Vec<Duration>>,
    pub memory_usage: AtomicUsize,
    pub cpu_usage: AtomicU64,
    pub error_count: AtomicU64,
    pub last_update: Mutex<Instant>,
}
```

***

## Routing

Enhanced routing system with Radix tree implementation for high-performance lookups.

### Radix Tree Router

```rust
pub struct RadixRouter {
    root: RadixNode,
}

impl RadixRouter {
    pub fn new() -> Self
    pub fn insert(&mut self, path: String, method: Method, handler: HandlerFn)
    pub fn lookup(&self, method: &Method, path: &str) -> Option<(HandlerFn, HashMap<String, String>)>
    pub fn stats(&self) -> RadixStats
    pub fn print_tree(&self)
}
```

### Radix Statistics

```rust
#[derive(Debug, Clone)]
pub struct RadixStats {
    pub total_nodes: usize,
    pub total_routes: usize,
    pub max_depth: usize,
    pub param_routes: usize,
    pub wildcard_routes: usize,
    pub static_routes: usize,
}
```

### Route Caching

Built-in route caching for improved performance:

```rust
impl Router {
    pub fn clear_cache(&self)
    pub fn cache_stats(&self) -> CacheStats
}
```

***

## Handler Signatures

Ignitia supports various handler signatures through automatic extraction (up to 8 extractors):

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

// Maximum extractors (8)
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

### Raw Request Handlers

```rust
// Direct request access
async fn raw_handler(req: Request) -> Result<Response> { ... }

// Using RawRequest marker
async fn handler(RawRequest(req): RawRequest) -> Result<Response> { ... }
```

***

## Feature Flags

Configure Ignitia features through Cargo.toml:

```toml
[dependencies]
ignitia = {
    version = "0.2.3",
    features = ["tls", "websocket", "self-signed"]
}
```

### Available Features

- **`tls`** - Enables HTTPS/TLS support with certificate management and ALPN
- **`websocket`** - Enables WebSocket protocol support with connection management
- **`self-signed`** - Enables self-signed certificate generation (development only)

### Default Features

- **`websocket`** - WebSocket support is enabled by default

***

## Result Type

All fallible operations return `Result<T, Error>`:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

This ensures consistent error handling across the framework.

***

## Framework Information

### Version Information

```rust
/// Current framework version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Framework name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Get framework version string
pub fn version() -> String

/// Get build information
pub fn build_info() -> BuildInfo
```

### Prelude Module

Common imports for quick setup:

```rust
use ignitia::prelude::*;
```

This includes all commonly used types and traits for rapid development.
