# Request Handling Guide

This guide covers comprehensive HTTP request handling in the Ignitia web framework, including extractors, body parsing, headers, cookies, and advanced request processing techniques.

## Table of Contents

- [Request Structure](#request-structure)
- [Request Extractors](#request-extractors)
- [Body Handling](#body-handling)
- [Headers and Metadata](#headers-and-metadata)
- [Parameters and Query Strings](#parameters-and-query-strings)
- [Cookies](#cookies)
- [File Uploads and Multipart Forms](#file-uploads-and-multipart-forms)
- [Advanced Request Handling](#advanced-request-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Request Structure

The `Request` struct in Ignitia contains all information about an incoming HTTP request:

```rust
use ignitia::{Request, Response, Result};

// Basic request structure
pub struct Request {
    pub method: Method,           // HTTP method (GET, POST, etc.)
    pub uri: Uri,                // Request URI
    pub version: Version,        // HTTP version
    pub headers: HeaderMap,      // Request headers
    pub body: Bytes,            // Raw request body
    pub params: HashMap<String, String>,      // Route parameters
    pub query_params: HashMap<String, String>, // Query parameters
    pub extensions: Extensions,  // Request extensions
}
```

### Basic Request Information

```rust
async fn handler(req: Request) -> Result<Response> {
    println!("Method: {}", req.method);
    println!("URI: {}", req.uri);
    println!("Version: {:?}", req.version);
    println!("Path: {}", req.uri.path());

    Ok(Response::text("Request processed"))
}
```

## Request Extractors

Ignitia provides powerful extractors that automatically parse request data into type-safe Rust types. Extractors implement the `FromRequest` trait and can be used directly in handler parameters.

### Available Extractors

```rust
use ignitia::{Path, Query, Json, Headers, Cookies, Body, Method, Uri, Form, State};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserQuery {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    age: Option<u32>,
}

// Handler using multiple extractors
async fn advanced_handler(
    Path(user_id): Path<u32>,           // Extract path parameter
    Query(query): Query<UserQuery>,     // Extract query parameters
    Json(data): Json<CreateUser>,       // Extract JSON body
    headers: Headers,                   // Extract all headers
    cookies: Cookies,                   // Extract cookies
    body: Body,                        // Raw body access
    method: Method,                    // HTTP method
    uri: Uri,                         // Request URI
) -> Result<Response> {
    // Use extracted data
    let page = query.page.unwrap_or(1);
    let user_agent = headers.get("user-agent").unwrap_or("unknown");
    let session = cookies.get("session_id");

    Response::json(serde_json::json!({
        "user_id": user_id,
        "data": data,
        "page": page,
        "user_agent": user_agent,
        "has_session": session.is_some(),
        "method": method.as_str(),
        "path": uri.path()
    }))
}
```

### Path Parameters

Extract typed parameters from URL paths:

```rust
use ignitia::{Router, Path};

let router = Router::new()
    // Single parameter
    .get("/users/:id", |Path(id): Path<u32>| async move {
        Ok(Response::json(serde_json::json!({
            "user_id": id
        }))?)
    })

    // Multiple parameters
    .get("/users/:user_id/posts/:post_id",
        |Path((user_id, post_id)): Path<(u32, u32)>| async move {
            Ok(Response::json(serde_json::json!({
                "user_id": user_id,
                "post_id": post_id
            }))?)
        }
    )

    // Custom struct for parameters
    .get("/api/:version/users/:id", |Path(params): Path<ApiParams>| async move {
        Ok(Response::json(params)?)
    });

#[derive(Deserialize)]
struct ApiParams {
    version: String,
    id: u32,
}
```

### Query Parameters

Extract and validate query parameters:

```rust
use ignitia::Query;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,                    // Required parameter
    page: Option<u32>,           // Optional with default
    limit: Option<u32>,          // Optional
    #[serde(default)]
    include_deleted: bool,       // Boolean with default false
}

async fn search_handler(Query(params): Query<SearchQuery>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100); // Cap at 100

    // Perform search with params.q, page, limit, params.include_deleted

    Response::json(serde_json::json!({
        "query": params.q,
        "page": page,
        "limit": limit,
        "include_deleted": params.include_deleted
    }))
}

// Usage: GET /search?q=rust&page=2&limit=50&include_deleted=true
```

## Body Handling

### JSON Body Parsing

```rust
use ignitia::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreatePost {
    title: String,
    content: String,
    tags: Vec<String>,
    published: Option<bool>,
}

#[derive(Serialize)]
struct PostResponse {
    id: u32,
    title: String,
    status: String,
}

async fn create_post(Json(post): Json<CreatePost>) -> Result<Response> {
    // Validate input
    if post.title.is_empty() {
        return Ok(Response::new(StatusCode::BAD_REQUEST)
            .with_body("Title cannot be empty"));
    }

    // Process post creation
    let post_id = save_post_to_database(post).await?;

    Response::json(PostResponse {
        id: post_id,
        title: post.title,
        status: "created".to_string(),
    })
}
```

### Form Data Handling

```rust
use ignitia::Form;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

async fn login_handler(Form(login): Form<LoginForm>) -> Result<Response> {
    // Validate credentials
    if authenticate_user(&login.username, &login.password).await? {
        let session_id = create_session(&login.username).await?;

        let mut response = Response::text("Login successful");

        // Set session cookie
        let mut cookie = Cookie::new("session_id", session_id)
            .http_only()
            .secure()
            .path("/");

        if login.remember_me.unwrap_or(false) {
            cookie = cookie.max_age(30 * 24 * 60 * 60); // 30 days
        }

        Ok(response.add_cookie(cookie))
    } else {
        Ok(Response::new(StatusCode::UNAUTHORIZED)
            .with_body("Invalid credentials"))
    }
}
```

### Raw Body Access

```rust
use ignitia::Body;

async fn upload_handler(Body(body): Body) -> Result<Response> {
    // Check content length
    if body.len() > 10 * 1024 * 1024 {  // 10MB limit
        return Ok(Response::new(StatusCode::PAYLOAD_TOO_LARGE)
            .with_body("File too large"));
    }

    // Process raw binary data
    let file_hash = sha256_hash(&body);
    let file_path = save_file(&body, &file_hash).await?;

    Response::json(serde_json::json!({
        "hash": file_hash,
        "path": file_path,
        "size": body.len()
    }))
}
```

## Headers and Metadata

### Working with Headers

```rust
use ignitia::{Headers, Method, Uri};

async fn headers_handler(
    headers: Headers,
    method: Method,
    uri: Uri,
) -> Result<Response> {
    // Access specific headers
    let user_agent = headers.get("user-agent").unwrap_or("Unknown");
    let content_type = headers.get("content-type");
    let auth_header = headers.get("authorization");

    // Check for API key
    let api_key = headers.get("x-api-key")
        .ok_or_else(|| Error::Unauthorized)?;

    // Validate API key
    if !is_valid_api_key(api_key) {
        return Err(Error::Unauthorized);
    }

    Response::json(serde_json::json!({
        "method": method.as_str(),
        "path": uri.path(),
        "user_agent": user_agent,
        "content_type": content_type,
        "authenticated": true
    }))
}

// Direct header access from request
async fn manual_header_check(req: Request) -> Result<Response> {
    let accept_language = req.header("accept-language")
        .unwrap_or("en");

    let prefer_json = req.header("accept")
        .map(|accept| accept.contains("application/json"))
        .unwrap_or(false);

    if prefer_json {
        Response::json(serde_json::json!({"language": accept_language}))
    } else {
        Response::text(format!("Language: {}", accept_language))
    }
}
```

### Content Negotiation

```rust
async fn content_negotiation_handler(headers: Headers) -> Result<Response> {
    let accept = headers.get("accept").unwrap_or("*/*");

    let data = serde_json::json!({
        "message": "Hello, World!",
        "timestamp": chrono::Utc::now(),
    });

    if accept.contains("application/json") {
        Response::json(data)
    } else if accept.contains("text/html") {
        Response::html(format!(
            "<html><body><h1>{}</h1><p>Time: {}</p></body></html>",
            data["message"], data["timestamp"]
        ))
    } else {
        Response::text(format!("{} at {}", data["message"], data["timestamp"]))
    }
}
```

## Parameters and Query Strings

### Complex Query Processing

```rust
use ignitia::Query;

#[derive(Deserialize)]
struct ProductFilter {
    category: Option<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
    in_stock: Option<bool>,
    sort_by: Option<String>,
    #[serde(default)]
    ascending: bool,
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

async fn list_products(Query(filter): Query<ProductFilter>) -> Result<Response> {
    // Validate parameters
    let limit = filter.limit.min(100);  // Cap at 100
    let page = filter.page.max(1);      // Minimum page 1

    if let (Some(min), Some(max)) = (filter.min_price, filter.max_price) {
        if min > max {
            return Ok(Response::new(StatusCode::BAD_REQUEST)
                .with_body("min_price cannot be greater than max_price"));
        }
    }

    // Build database query
    let mut query_builder = ProductQuery::new();

    if let Some(category) = &filter.category {
        query_builder = query_builder.category(category);
    }

    if let Some(min_price) = filter.min_price {
        query_builder = query_builder.min_price(min_price);
    }

    if let Some(max_price) = filter.max_price {
        query_builder = query_builder.max_price(max_price);
    }

    if let Some(in_stock) = filter.in_stock {
        query_builder = query_builder.in_stock(in_stock);
    }

    // Execute query
    let products = query_builder
        .sort_by(filter.sort_by.as_deref().unwrap_or("name"), filter.ascending)
        .page(page)
        .limit(limit)
        .execute()
        .await?;

    Response::json(serde_json::json!({
        "products": products,
        "page": page,
        "limit": limit,
        "total": products.len()
    }))
}
```

### Manual Parameter Extraction

```rust
async fn manual_params(req: Request) -> Result<Response> {
    // Access route parameters directly
    let user_id: u32 = req.param("user_id")
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| Error::BadRequest("Invalid user_id".into()))?;

    // Access query parameters directly
    let include_deleted = req.query("include_deleted")
        .map(|val| val == "true")
        .unwrap_or(false);

    let page: u32 = req.query("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);

    Response::json(serde_json::json!({
        "user_id": user_id,
        "page": page,
        "include_deleted": include_deleted
    }))
}
```

## Cookies

### Reading Cookies

```rust
use ignitia::{Cookies, Cookie, CookieJar};

async fn cookie_handler(cookies: Cookies) -> Result<Response> {
    // Get specific cookie
    let session_id = cookies.get("session_id");
    let user_prefs = cookies.get("user_preferences");

    // Check all cookies
    let cookie_count = cookies.len();
    let has_session = cookies.contains("session_id");

    Response::json(serde_json::json!({
        "has_session": has_session,
        "session_id": session_id,
        "user_preferences": user_prefs,
        "total_cookies": cookie_count
    }))
}

// Manual cookie access
async fn manual_cookies(req: Request) -> Result<Response> {
    let cookies = req.cookies();

    // Get authentication cookie
    let auth_token = req.cookie("auth_token")
        .ok_or_else(|| Error::Unauthorized)?;

    // Validate token
    let user = validate_auth_token(&auth_token).await?;

    Response::json(user)
}
```

### Setting Cookies

```rust
async fn set_cookie_handler() -> Result<Response> {
    let mut response = Response::text("Cookies set!");

    // Simple cookie
    response = response.add_cookie(
        Cookie::new("simple", "value")
    );

    // Secure session cookie
    response = response.add_cookie(
        Cookie::new("session", "abc123")
            .http_only()
            .secure()
            .same_site(SameSite::Strict)
            .path("/")
            .max_age(3600) // 1 hour
    );

    // Persistent user preference
    response = response.add_cookie(
        Cookie::new("theme", "dark")
            .path("/")
            .max_age(30 * 24 * 60 * 60) // 30 days
    );

    Ok(response)
}
```

## File Uploads and Multipart Forms

### Basic File Upload

```rust
use ignitia::{Multipart, Field, FileField};

async fn upload_handler(mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().to_string();

        if field.is_file() {
            // Handle file upload
            let file_name = field.file_name()
                .unwrap_or("unknown")
                .to_string();

            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            // Save file
            let file_path = format!("uploads/{}", file_name);
            let saved_file = field.save_to_file(&file_path).await?;

            uploaded_files.push(serde_json::json!({
                "field": field_name,
                "filename": file_name,
                "content_type": content_type,
                "size": saved_file.size,
                "path": file_path
            }));
        } else {
            // Handle text field
            let text_value = field.text().await?;
            println!("Field '{}': {}", field_name, text_value);
        }
    }

    Response::json(serde_json::json!({
        "uploaded_files": uploaded_files,
        "count": uploaded_files.len()
    }))
}
```

### Advanced Multipart Processing

```rust
use ignitia::{Multipart, MultipartConfig};

async fn advanced_upload(mut multipart: Multipart) -> Result<Response> {
    let mut form_data = std::collections::HashMap::new();
    let mut files = Vec::new();

    // Process all fields
    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().to_string();

        if field.is_file() {
            // Validate file type
            let content_type = field.content_type().unwrap_or("");
            if !is_allowed_file_type(content_type) {
                return Ok(Response::new(StatusCode::BAD_REQUEST)
                    .with_body("File type not allowed"));
            }

            // Process file
            let bytes = field.bytes().await?;

            // Generate unique filename
            let original_name = field.file_name().unwrap_or("unnamed");
            let extension = std::path::Path::new(original_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            let unique_name = format!("{}_{}.{}",
                chrono::Utc::now().timestamp(),
                uuid::Uuid::new_v4().to_simple(),
                extension
            );

            let file_path = format!("uploads/{}", unique_name);
            tokio::fs::write(&file_path, &bytes).await?;

            files.push(serde_json::json!({
                "original_name": original_name,
                "stored_name": unique_name,
                "size": bytes.len(),
                "content_type": content_type,
                "path": file_path
            }));
        } else {
            // Handle text field
            let text_value = field.text().await?;
            form_data.insert(field_name, text_value);
        }
    }

    Response::json(serde_json::json!({
        "form_data": form_data,
        "files": files,
        "total_files": files.len()
    }))
}

fn is_allowed_file_type(content_type: &str) -> bool {
    matches!(content_type,
        "image/jpeg" | "image/png" | "image/gif" |
        "text/plain" | "application/pdf"
    )
}
```

## Advanced Request Handling

### Request State and Extensions

```rust
use ignitia::{State, Extension};

#[derive(Clone)]
struct AppState {
    db_pool: DatabasePool,
    redis_client: RedisClient,
    config: AppConfig,
}

#[derive(Clone)]
struct RequestContext {
    user_id: Option<u32>,
    request_id: String,
    start_time: std::time::Instant,
}

async fn stateful_handler(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Response> {
    // Use application state
    let user = if let Some(user_id) = ctx.user_id {
        app_state.db_pool.get_user(user_id).await?
    } else {
        return Err(Error::Unauthorized);
    };

    // Use request context
    let elapsed = ctx.start_time.elapsed();

    Response::json(serde_json::json!({
        "user": user,
        "request_id": ctx.request_id,
        "processing_time_ms": elapsed.as_millis()
    }))
}
```

### Custom Extractors

```rust
use ignitia::{FromRequest, Request, Result, Error};

// Custom extractor for API versioning
struct ApiVersion(String);

impl FromRequest for ApiVersion {
    fn from_request(req: &Request) -> Result<Self> {
        // Try header first
        if let Some(version) = req.header("api-version") {
            return Ok(ApiVersion(version.to_string()));
        }

        // Try query parameter
        if let Some(version) = req.query("version") {
            return Ok(ApiVersion(version.clone()));
        }

        // Try path prefix
        let path = req.uri.path();
        if path.starts_with("/v1/") {
            return Ok(ApiVersion("1.0".to_string()));
        }
        if path.starts_with("/v2/") {
            return Ok(ApiVersion("2.0".to_string()));
        }

        // Default version
        Ok(ApiVersion("1.0".to_string()))
    }
}

async fn versioned_handler(ApiVersion(version): ApiVersion) -> Result<Response> {
    match version.as_str() {
        "1.0" => Response::json(serde_json::json!({"version": "1.0", "deprecated": true})),
        "2.0" => Response::json(serde_json::json!({"version": "2.0", "current": true})),
        _ => Ok(Response::new(StatusCode::BAD_REQUEST)
            .with_body("Unsupported API version"))
    }
}
```

### Request Validation

```rust
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 1, max = 50))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 13, max = 120))]
    age: u8,

    #[validate(custom = "validate_password")]
    password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("Password must be at least 8 characters"));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("Password must contain uppercase letter"));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(ValidationError::new("Password must contain number"));
    }

    Ok(())
}

async fn create_user_validated(Json(user): Json<CreateUserRequest>) -> Result<Response> {
    // Validate input
    if let Err(errors) = user.validate() {
        let error_messages: Vec<String> = errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!("{}: {}", field, error.message.as_deref().unwrap_or("Invalid"))
                })
            })
            .collect();

        return Response::validation_error(error_messages);
    }

    // Process valid user creation
    let created_user = create_user_in_db(user).await?;

    Response::json(serde_json::json!({
        "user": created_user,
        "status": "created"
    }))
}
```

## Best Practices

### 1. Error Handling in Extractors

```rust
use ignitia::{Path, Query, Json};

async fn robust_handler(
    path_result: Result<Path<u32>, Error>,
    query_result: Result<Query<SearchParams>, Error>,
    json_result: Result<Json<UserData>, Error>,
) -> Result<Response> {
    // Handle extractor errors gracefully
    let Path(id) = path_result.map_err(|_| {
        Error::BadRequest("Invalid ID parameter".into())
    })?;

    let Query(search) = query_result.unwrap_or_else(|_| {
        Query(SearchParams::default())
    });

    let Json(data) = json_result.map_err(|e| {
        Error::BadRequest(format!("Invalid JSON: {}", e))
    })?;

    // Process with validated data
    process_request(id, search, data).await
}
```

### 2. Request Size Limits

```rust
use ignitia::{Body, Headers};

async fn size_limited_handler(
    headers: Headers,
    Body(body): Body,
) -> Result<Response> {
    // Check content length header
    if let Some(length_str) = headers.get("content-length") {
        if let Ok(length) = length_str.parse::<usize>() {
            if length > 5 * 1024 * 1024 {  // 5MB limit
                return Ok(Response::new(StatusCode::PAYLOAD_TOO_LARGE)
                    .with_body("Request too large"));
            }
        }
    }

    // Also check actual body size
    if body.len() > 5 * 1024 * 1024 {
        return Ok(Response::new(StatusCode::PAYLOAD_TOO_LARGE)
            .with_body("Request too large"));
    }

    // Process within limits
    process_large_request(body).await
}
```

### 3. Content Type Validation

```rust
async fn typed_handler(headers: Headers, Body(body): Body) -> Result<Response> {
    let content_type = headers.get("content-type")
        .ok_or_else(|| Error::BadRequest("Missing Content-Type header".into()))?;

    match content_type {
        ct if ct.starts_with("application/json") => {
            let data: serde_json::Value = serde_json::from_slice(&body)?;
            process_json_data(data).await
        },
        ct if ct.starts_with("application/xml") => {
            let xml_data = String::from_utf8(body.to_vec())
                .map_err(|_| Error::BadRequest("Invalid UTF-8 in XML".into()))?;
            process_xml_data(xml_data).await
        },
        ct if ct.starts_with("text/plain") => {
            let text = String::from_utf8(body.to_vec())
                .map_err(|_| Error::BadRequest("Invalid UTF-8 in text".into()))?;
            process_text_data(text).await
        },
        _ => Err(Error::BadRequest("Unsupported content type".into()))
    }
}
```

## Examples

### Complete REST API Handler

```rust
use ignitia::{Router, Path, Query, Json, Headers, Method};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ListParams {
    page: Option<u32>,
    limit: Option<u32>,
    filter: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct User {
    id: Option<u32>,
    name: String,
    email: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn user_handler(
    Path(path_parts): Path<Vec<String>>,
    Query(params): Query<ListParams>,
    Json(user_data): Json<Option<User>>,
    headers: Headers,
    method: Method,
) -> Result<Response> {
    // Check authentication
    let api_key = headers.get("authorization")
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .ok_or_else(|| Error::Unauthorized)?;

    validate_api_key(api_key)?;

    match (method.as_str(), path_parts.as_slice()) {
        ("GET", []) => {
            // List users
            let page = params.page.unwrap_or(1);
            let limit = params.limit.unwrap_or(20).min(100);

            let users = list_users(page, limit, params.filter).await?;
            Response::json(users)
        },

        ("GET", [id_str]) => {
            // Get specific user
            let id: u32 = id_str.parse()
                .map_err(|_| Error::BadRequest("Invalid user ID".into()))?;

            let user = get_user(id).await?
                .ok_or_else(|| Error::NotFound("User not found".into()))?;

            Response::json(user)
        },

        ("POST", []) => {
            // Create user
            let user_data = user_data
                .ok_or_else(|| Error::BadRequest("Missing user data".into()))?;

            let created_user = create_user(user_data).await?;
            Response::json(created_user).with_status(StatusCode::CREATED)
        },

        ("PUT", [id_str]) => {
            // Update user
            let id: u32 = id_str.parse()
                .map_err(|_| Error::BadRequest("Invalid user ID".into()))?;

            let mut user_data = user_data
                .ok_or_else(|| Error::BadRequest("Missing user data".into()))?;
            user_data.id = Some(id);

            let updated_user = update_user(user_data).await?;
            Response::json(updated_user)
        },

        ("DELETE", [id_str]) => {
            // Delete user
            let id: u32 = id_str.parse()
                .map_err(|_| Error::BadRequest("Invalid user ID".into()))?;

            delete_user(id).await?;
            Ok(Response::new(StatusCode::NO_CONTENT))
        },

        _ => Err(Error::MethodNotAllowed(format!("{} not allowed", method)))
    }
}

// Router setup
fn create_router() -> Router {
    Router::new()
        .get("/users", user_handler)
        .get("/users/:id", user_handler)
        .post("/users", user_handler)
        .put("/users/:id", user_handler)
        .delete("/users/:id", user_handler)
}
```

### File Upload with Progress

```rust
use ignitia::{Multipart, Headers};

async fn upload_with_progress(
    headers: Headers,
    mut multipart: Multipart,
) -> Result<Response> {
    // Check total content length for progress tracking
    let total_size = headers.get("content-length")
        .and_then(|len| len.parse::<usize>().ok())
        .unwrap_or(0);

    let mut processed_size = 0;
    let mut results = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let bytes = field.bytes().await?;

            processed_size += bytes.len();

            // Save file with metadata
            let file_path = save_uploaded_file(&filename, &bytes).await?;
            let file_hash = calculate_file_hash(&bytes);

            results.push(serde_json::json!({
                "filename": filename,
                "size": bytes.len(),
                "path": file_path,
                "hash": file_hash,
                "progress": if total_size > 0 {
                    (processed_size as f64 / total_size as f64) * 100.0
                } else { 0.0 }
            }));

            // Log progress
            tracing::info!(
                "Uploaded file '{}' ({} bytes, {:.1}% complete)",
                filename,
                bytes.len(),
                if total_size > 0 {
                    (processed_size as f64 / total_size as f64) * 100.0
                } else { 0.0 }
            );
        }
    }

    Response::json(serde_json::json!({
        "uploaded_files": results,
        "total_files": results.len(),
        "total_size": processed_size
    }))
}
```

This comprehensive guide covers all aspects of HTTP request handling in Ignitia. The framework's type-safe extractors, flexible body handling, and comprehensive request parsing make it easy to build robust web applications and APIs.
