# Extractors Guide

Request extractors in Ignitia allow you to declaratively extract data from HTTP requests in your handler functions. They provide a clean, type-safe way to access request data without manual parsing.

## Table of Contents

- [Overview](#overview)
- [Core Concept](#core-concept)
- [Built-in Extractors](#built-in-extractors)
  - [Path Parameters](#path-parameters)
  - [Query Parameters](#query-parameters)
  - [JSON Body](#json-body)
  - [Form Data](#form-data)
  - [Headers](#headers)
  - [Cookies](#cookies)
  - [Raw Body](#raw-body)
  - [HTTP Method](#http-method)
  - [URI](#uri)
  - [Application State](#application-state)
  - [Extensions](#extensions)
  - [Multipart Form Data](#multipart-form-data)
- [Multiple Extractors](#multiple-extractors)
- [Custom Extractors](#custom-extractors)
- [Error Handling](#error-handling)
- [Performance Tips](#performance-tips)
- [Best Practices](#best-practices)

---

## Overview

Extractors implement the `FromRequest` trait and automatically extract data from incoming HTTP requests. They're used as parameters in handler functions:

```rust
use ignitia::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct UserQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn get_user(
    Path(user_id): Path<u32>,
    Query(params): Query<UserQuery>
) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "user_id": user_id,
        "limit": params.limit.unwrap_or(10),
        "offset": params.offset.unwrap_or(0)
    }))?)
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "message": "User created",
        "name": user.name,
        "email": user.email
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/users/{user_id}", get_user)
        .post("/users", create_user);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

---

## Core Concept

The `FromRequest` trait defines how extractors work:

```rust
pub trait FromRequest: Sized {
    type Error;
    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error>;
}
```

Extractors are applied in order as handler parameters. If any extraction fails, the request is rejected with an appropriate error response.

### Extractor Execution Order

```rust
// Extractors are executed left-to-right
async fn handler(
    Path(id): Path<u32>,           // 1. Extract path parameter
    Query(params): Query<Filters>,  // 2. Extract query parameters
    Headers(headers): Headers,      // 3. Extract headers
    Json(body): Json<CreateData>   // 4. Extract and parse JSON body
) -> Result<Response> {
    // All extractors succeeded - safe to use the data
    Ok(Response::json(&serde_json::json!({
        "id": id,
        "filters": params,
        "body": body
    }))?)
}
```

---

## Built-in Extractors

### Path Parameters

Extract typed parameters from URL paths using `Path<T>`:

```rust
use ignitia::prelude::*;
use serde::Deserialize;

// Single parameter
async fn get_user(Path(user_id): Path<u32>) -> Result<Response> {
    Ok(Response::text(&format!("User ID: {}", user_id)))
}

// Multiple parameters using tuple
async fn get_user_post(
    Path((user_id, post_id)): Path<(u32, u32)>
) -> Result<Response> {
    Ok(Response::text(&format!(
        "User {} Post {}",
        user_id, post_id
    )))
}

// Multiple parameters using struct
#[derive(Deserialize)]
struct PostPath {
    user_id: u32,
    post_id: u32,
}

async fn get_post(Path(params): Path<PostPath>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "user_id": params.user_id,
        "post_id": params.post_id
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/users/{user_id}", get_user)
        .get("/users/{user_id}/posts/{post_id}", get_post);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Supported Types:**
- Primitive types: `u32`, `i64`, `String`, `bool`, `f64`, etc.
- Tuples: `(u32, String)`, `(u32, u32, String)`
- Custom structs implementing `Deserialize`
- `Option<T>` for optional parameters

**Path Parameter Features:**
- Automatic type conversion with validation
- Detailed error messages on conversion failures
- Support for URL-encoded values
- Zero-copy extraction where possible

---

### Query Parameters

Extract query parameters using `Query<T>`:

```rust
use ignitia::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
    tags: Option<Vec<String>>,
    sort_by: Option<String>,
}

async fn search(Query(params): Query<SearchQuery>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    Ok(Response::json(&serde_json::json!({
        "query": params.q,
        "page": page,
        "limit": limit,
        "tags": params.tags.unwrap_or_default(),
        "sort_by": params.sort_by
    }))?)
}

// Using HashMap for dynamic query parameters
async fn dynamic_search(
    Query(params): Query<std::collections::HashMap<String, String>>
) -> Result<Response> {
    Ok(Response::json(&params)?)
}

// URL examples:
// /search?q=rust&page=2&limit=20&tags=web&tags=framework&sort_by=date
// /search?filter=active&category=tools&min_price=10

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/search", search)
        .get("/dynamic-search", dynamic_search);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Query Parameter Features:**
- Automatic type conversion
- Support for arrays/vectors (repeated parameters)
- Optional parameters with `Option<T>`
- Boolean parsing (`true`, `false`, `1`, `0`)
- URL decoding handled automatically
- Case-sensitive parameter names

---

### JSON Body

Extract JSON request bodies using `Json<T>`:

```rust
use ignitia::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    age: Option<u32>,
    roles: Vec<String>,
}

#[derive(Serialize)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
    created_at: String,
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    // Validate email
    if !user.email.contains('@') {
        return Err(Error::bad_request("Invalid email address"));
    }

    // Validate age
    if let Some(age) = user.age {
        if age > 150 {
            return Err(Error::validation("Age must be realistic"));
        }
    }

    let response = UserResponse {
        id: 123,
        name: user.name,
        email: user.email,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Response::json(&response)?)
}

// Nested JSON structures
#[derive(Deserialize)]
struct ComplexData {
    user: CreateUser,
    metadata: serde_json::Value,
    settings: Settings,
}

#[derive(Deserialize)]
struct Settings {
    theme: String,
    notifications: bool,
}

async fn handle_complex(Json(data): Json<ComplexData>) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "user": data.user.name,
        "theme": data.settings.theme,
        "notifications": data.settings.notifications
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .post("/users", create_user)
        .post("/complex", handle_complex);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Requirements:**
- Content-Type must be `application/json`
- Request body must be valid JSON
- JSON structure must match the target type
- Empty bodies are rejected with 400 Bad Request

**JSON Extractor Features:**
- Pre-checks Content-Type header
- Zero-copy deserialization where possible
- Detailed error messages for parsing failures
- Support for nested structures
- Support for generic `serde_json::Value` for dynamic JSON

---

### Form Data

Extract form-encoded data using `Form<T>`:

```rust
use ignitia::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember: Option<bool>,
}

async fn login(Form(form): Form<LoginForm>) -> Result<Response> {
    // Authenticate user
    if form.username == "admin" && form.password == "secret" {
        let session_cookie = Cookie::new("session_id", "abc123")
            .path("/")
            .http_only(true)
            .max_age(3600); // 1 hour

        Ok(Response::json(&serde_json::json!({
            "status": "success",
            "message": "Login successful"
        }))?
        .with_cookie(session_cookie))
    } else {
        Err(Error::unauthorized("Invalid credentials"))
    }
}

// Contact form example
#[derive(Deserialize)]
struct ContactForm {
    name: String,
    email: String,
    subject: String,
    message: String,
}

async fn submit_contact(Form(form): Form<ContactForm>) -> Result<Response> {
    // Validate
    if form.message.len() < 10 {
        return Err(Error::validation("Message is too short"));
    }

    // Process form
    println!("Contact from {}: {}", form.email, form.subject);

    Ok(Response::json(&serde_json::json!({
        "status": "success",
        "message": "Your message has been sent"
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .post("/login", login)
        .post("/contact", submit_contact);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Requirements:**
- Content-Type must be `application/x-www-form-urlencoded`
- Supports URL encoding/decoding
- Boolean values: `true`, `false`, `1`, `0`

---

### Headers

Access request headers using `Headers`:

```rust
use ignitia::prelude::*;

async fn check_auth(Headers(headers): Headers) -> Result<Response> {
    if let Some(auth) = headers.get("authorization") {
        if auth.starts_with("Bearer ") {
            let token = &auth[7..];
            // Validate token...
            Ok(Response::json(&serde_json::json!({
                "status": "authorized",
                "token_length": token.len()
            }))?)
        } else {
            Err(Error::unauthorized("Invalid authorization format"))
        }
    } else {
        Err(Error::unauthorized("Missing authorization header"))
    }
}

async fn user_agent_info(Headers(headers): Headers) -> Result<Response> {
    let user_agent = headers.get("user-agent").unwrap_or("Unknown");
    let accept = headers.get("accept").unwrap_or("*/*");
    let accept_language = headers.get("accept-language").unwrap_or("en");

    Ok(Response::json(&serde_json::json!({
        "user_agent": user_agent,
        "accept": accept,
        "language": accept_language
    }))?)
}

async fn custom_headers(Headers(headers): Headers) -> Result<Response> {
    let api_key = headers.get("x-api-key")
        .ok_or_else(|| Error::unauthorized("Missing API key"))?;

    let client_id = headers.get("x-client-id")
        .ok_or_else(|| Error::bad_request("Missing client ID"))?;

    Ok(Response::json(&serde_json::json!({
        "api_key_length": api_key.len(),
        "client_id": client_id
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/protected", check_auth)
        .get("/info", user_agent_info)
        .post("/api/data", custom_headers);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Headers Extractor Features:**
- Case-insensitive header name lookup
- Returns `HashMap<String, String>`
- Only includes headers with valid UTF-8 values
- Common headers are pre-cached for performance

---

### Cookies

Access cookies using `Cookies`:

```rust
use ignitia::prelude::*;

async fn get_session(Cookies(cookies): Cookies) -> Result<Response> {
    if let Some(session_id) = cookies.get("session_id") {
        Ok(Response::json(&serde_json::json!({
            "session_id": session_id.value(),
            "authenticated": true
        }))?)
    } else {
        // Set a new session cookie
        let session_cookie = Cookie::new("session_id", "new-session-123")
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(3600); // 1 hour

        Ok(Response::json(&serde_json::json!({
            "message": "New session created",
            "authenticated": false
        }))?
        .with_cookie(session_cookie))
    }
}

async fn user_preferences(Cookies(cookies): Cookies) -> Result<Response> {
    let theme = cookies.get("theme")
        .map(|c| c.value())
        .unwrap_or("light");

    let language = cookies.get("language")
        .map(|c| c.value())
        .unwrap_or("en");

    Ok(Response::json(&serde_json::json!({
        "theme": theme,
        "language": language
    }))?)
}

async fn logout(Cookies(cookies): Cookies) -> Result<Response> {
    // Remove session cookie
    let remove_cookie = Cookie::build("session_id", "")
        .path("/")
        .max_age(0)
        .finish();

    Ok(Response::json(&serde_json::json!({
        "message": "Logged out successfully"
    }))?
    .with_cookie(remove_cookie))
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/session", get_session)
        .get("/preferences", user_preferences)
        .post("/logout", logout);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Cookie Features:**
- Automatic cookie parsing from headers
- Support for cookie attributes (HttpOnly, Secure, SameSite)
- Easy cookie creation and deletion
- Built-in cookie jar for multiple cookies

---

### Raw Body

Access the raw request body using `Body`:

```rust
use ignitia::prelude::*;

async fn upload_file(Body(body): Body) -> Result<Response> {
    let file_size = body.len();

    // Check size limit
    if file_size > 10 * 1024 * 1024 { // 10MB limit
        return Err(Error::bad_request("File too large"));
    }

    // Save file
    tokio::fs::write("uploaded_file.bin", &*body).await
        .map_err(|e| Error::internal(e.to_string()))?;

    Ok(Response::json(&serde_json::json!({
        "status": "uploaded",
        "size": file_size
    }))?)
}

async fn webhook_handler(
    Headers(headers): Headers,
    Body(body): Body
) -> Result<Response> {
    // Verify webhook signature
    let signature = headers.get("x-webhook-signature")
        .ok_or_else(|| Error::unauthorized("Missing signature"))?;

    // Compute HMAC of body and verify
    // ... verification logic ...

    // Process webhook payload
    let payload = String::from_utf8(body.to_vec())
        .map_err(|_| Error::bad_request("Invalid UTF-8"))?;

    println!("Webhook received: {}", payload);

    Ok(Response::json(&serde_json::json!({
        "status": "processed"
    }))?)
}

async fn binary_upload(Body(body): Body) -> Result<Response> {
    // Process binary data (e.g., image, video)
    let bytes: &[u8] = &body;

    // Check file magic numbers
    if bytes.len() < 4 {
        return Err(Error::bad_request("File too small"));
    }

    let file_type = match &bytes[0..4] {
        [0xFF, 0xD8, 0xFF, _] => "JPEG",
        [0x89, 0x50, 0x4E, 0x47] => "PNG",
        [0x47, 0x49, 0x46, 0x38] => "GIF",
        _ => "Unknown"
    };

    Ok(Response::json(&serde_json::json!({
        "file_type": file_type,
        "size": bytes.len()
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .post("/upload", upload_file)
        .post("/webhook", webhook_handler)
        .post("/binary", binary_upload);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Body Extractor Features:**
- Zero-copy access with `Arc<Bytes>`
- Efficient for large payloads
- Works with any content type
- Useful for binary data, webhooks, custom parsing

---

### HTTP Method

Access the HTTP method using `Method`:

```rust
use ignitia::prelude::*;

async fn method_info(Method(method): Method) -> Result<Response> {
    let method_name = method.as_str();
    let is_safe = matches!(method, Method::GET | Method::HEAD | Method::OPTIONS);

    Ok(Response::json(&serde_json::json!({
        "method": method_name,
        "is_safe": is_safe
    }))?)
}

async fn conditional_handler(Method(method): Method) -> Result<Response> {
    match method {
        Method::GET => Ok(Response::text("Getting resource")),
        Method::POST => Ok(Response::text("Creating resource")),
        Method::PUT => Ok(Response::text("Updating resource")),
        Method::DELETE => Ok(Response::text("Deleting resource")),
        _ => Err(Error::bad_request("Method not supported"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/method-info", method_info)
        .post("/method-info", method_info)
        .get("/conditional", conditional_handler)
        .post("/conditional", conditional_handler);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

---

### URI

Access the request URI using `Uri`:

```rust
use ignitia::prelude::*;

async fn uri_info(Uri(uri): Uri) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "path": uri.path(),
        "query": uri.query(),
        "scheme": uri.scheme_str(),
        "host": uri.host(),
        "full_uri": uri.to_string()
    }))?)
}

async fn redirect_based_on_host(Uri(uri): Uri) -> Result<Response> {
    if let Some(host) = uri.host() {
        if host.contains("old-domain.com") {
            let new_uri = format!("https://new-domain.com{}", uri.path());
            return Ok(Response::permanent_redirect(&new_uri));
        }
    }

    Ok(Response::text("Welcome to the new domain!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/uri-info", uri_info)
        .get("/redirect", redirect_based_on_host);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

---

### Application State

Access shared application state using `State<T>`:

```rust
use ignitia::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    counter: Arc<RwLock<u64>>,
    config: Arc<AppConfig>,
}

#[derive(Clone)]
struct AppConfig {
    app_name: String,
    max_connections: u32,
}

async fn increment_counter(State(state): State<AppState>) -> Result<Response> {
    let mut counter = state.counter.write().await;
    *counter += 1;

    Ok(Response::json(&serde_json::json!({
        "counter": *counter,
        "app_name": state.config.app_name
    }))?)
}

async fn get_counter(State(state): State<AppState>) -> Result<Response> {
    let counter = state.counter.read().await;

    Ok(Response::json(&serde_json::json!({
        "counter": *counter,
        "max_connections": state.config.max_connections
    }))?)
}

async fn reset_counter(State(state): State<AppState>) -> Result<Response> {
    let mut counter = state.counter.write().await;
    *counter = 0;

    Ok(Response::json(&serde_json::json!({
        "message": "Counter reset",
        "counter": 0
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        counter: Arc::new(RwLock::new(0)),
        config: Arc::new(AppConfig {
            app_name: "My App".to_string(),
            max_connections: 1000,
        }),
    };

    let router = Router::new()
        .state(state)
        .post("/counter/increment", increment_counter)
        .get("/counter", get_counter)
        .post("/counter/reset", reset_counter);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    println!("🔥 Server running on http://127.0.0.1:3000");
    server.ignitia().await?;
    Ok(())
}
```

**State Features:**
- Type-safe shared state across handlers
- Works with any `Clone + Send + Sync` type
- Efficient `Arc` sharing internally
- Commonly used with `Arc<Mutex<T>>` or `Arc<RwLock<T>>`

---

### Extensions

Access request extensions using `Extension<T>`:

```rust
use ignitia::prelude::*;

#[derive(Clone)]
struct UserId(u32);

#[derive(Clone)]
struct UserRole(String);

// In a real app, this would be middleware
async fn auth_middleware(mut req: Request, next: Next) -> Result<Response> {
    // Extract user info from token/session
    let user_id = UserId(123);
    let user_role = UserRole("admin".to_string());

    req.insert_extension(user_id);
    req.insert_extension(user_role);

    next.run(req).await
}

async fn protected_handler(
    Extension(user_id): Extension<UserId>,
    Extension(role): Extension<UserRole>
) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "user_id": user_id.0,
        "role": role.0,
        "message": "Access granted"
    }))?)
}

async fn admin_only(
    Extension(role): Extension<UserRole>
) -> Result<Response> {
    if role.0 != "admin" {
        return Err(Error::forbidden("Admin access required"));
    }

    Ok(Response::json(&serde_json::json!({
        "message": "Welcome, admin!"
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .middleware(auth_middleware)
        .get("/protected", protected_handler)
        .get("/admin", admin_only);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Extension Features:**
- Type-safe request-scoped data
- Set by middleware, used in handlers
- Efficient `Arc` sharing
- Must be `Clone + Send + Sync`

---

### Multipart Form Data

Extract multipart form data (file uploads) using `Multipart`:

```rust
use ignitia::prelude::*;
use std::path::PathBuf;

async fn handle_file_upload(mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            let filename = field.file_name()
                .unwrap_or("unknown")
                .to_string();

            let content_type = field.content_type()
                .unwrap_or("application/octet-stream");

            let file_path = PathBuf::from("uploads").join(&filename);

            let file_field = field.save_to_file(&file_path).await?;

            uploaded_files.push(serde_json::json!({
                "filename": filename,
                "content_type": content_type,
                "size": file_field.size
            }));
        } else {
            // Text field
            let name = field.name().unwrap_or("unknown");
            let value = field.text().await?;
            println!("Field {}: {}", name, value);
        }
    }

    Ok(Response::json(&serde_json::json!({
        "status": "success",
        "uploaded_files": uploaded_files
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    tokio::fs::create_dir_all("uploads").await?;

    let router = Router::new()
        .post("/upload", handle_file_upload);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

---

## Multiple Extractors

You can use multiple extractors in a single handler. They're executed in order from left to right:

```rust
use ignitia::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Deserialize)]
struct UpdateQuery {
    force: Option<bool>,
    notify: Option<bool>,
}

async fn update_user(
    Path(user_id): Path<u32>,
    Query(params): Query<UpdateQuery>,
    Headers(headers): Headers,
    Json(update): Json<UpdateUser>
) -> Result<Response> {
    // Check authorization
    let auth_header = headers.get("authorization")
        .ok_or_else(|| Error::unauthorized("Missing authorization"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(Error::unauthorized("Invalid authorization format"));
    }

    // Process update
    let force_update = params.force.unwrap_or(false);
    let notify = params.notify.unwrap_or(true);

    Ok(Response::json(&serde_json::json!({
        "user_id": user_id,
        "updated": true,
        "force": force_update,
        "notify": notify,
        "changes": {
            "name": update.name,
            "email": update.email
        }
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .patch("/users/{user_id}", update_user);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

**Best Practices for Multiple Extractors:**
1. Order extractors from cheapest to most expensive
2. Put path/query extractors before body extractors
3. Extract headers before body if you need to validate them first
4. Limit to 4-5 extractors per handler for readability

---

## Custom Extractors

Create custom extractors by implementing `FromRequest`:

```rust
use ignitia::prelude::*;
use serde::Deserialize;

// Custom extractor for API keys
#[derive(Debug)]
struct ApiKey(String);

impl FromRequest for ApiKey {
    type Error = Error;

    fn from_request(req: &Request) -> Result<Self> {
        // Try header first
        if let Some(key) = req.header("x-api-key") {
            return Ok(ApiKey(key.to_string()));
        }

        // Try query parameter
        if let Some(key) = req.query("api_key") {
            return Ok(ApiKey(key.clone()));
        }

        Err(Error::unauthorized("Missing API key"))
    }
}

// Custom extractor with validation
#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

struct ValidatedPagination {
    page: u32,
    limit: u32,
    offset: u32,
}

impl FromRequest for ValidatedPagination {
    type Error = Error;

    fn from_request(req: &Request) -> Result<Self> {
        // Convert query params to JSON value for deserialization
        let query_json = serde_json::to_value(&req.query_params)?;
        let params: PaginationQuery = serde_json::from_value(query_json)?;

        // Validate and apply defaults
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(10);

        // Validation
        if page == 0 {
            return Err(Error::validation("Page must be >= 1"));
        }

        if limit == 0 || limit > 100 {
            return Err(Error::validation("Limit must be between 1 and 100"));
        }

        let offset = (page - 1) * limit;

        Ok(ValidatedPagination {
            page,
            limit,
            offset,
        })
    }
}

async fn list_items(
    _api_key: ApiKey,
    pagination: ValidatedPagination
) -> Result<Response> {
    Ok(Response::json(&serde_json::json!({
        "page": pagination.page,
        "limit": pagination.limit,
        "offset": pagination.offset,
        "items": []
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/items", list_items);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

### Advanced Custom Extractor Example

```rust
use ignitia::prelude::*;
use serde::de::DeserializeOwned;

// Generic validated JSON extractor
struct ValidatedJson<T>(T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for ValidatedJson<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

trait Validate {
    fn validate(&self) -> std::result::Result<(), String>;
}

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
{
    type Error = Error;

    fn from_request(req: &Request) -> Result<Self> {
        // Extract JSON
        let data: T = serde_json::from_slice(&req.body)?;

        // Validate
        data.validate()
            .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

        Ok(ValidatedJson(data))
    }
}

// Example usage
#[derive(serde::Deserialize)]
struct CreateUserRequest {
    email: String,
    age: u32,
    username: String,
}

impl Validate for CreateUserRequest {
    fn validate(&self) -> std::result::Result<(), String> {
        if !self.email.contains('@') {
            return Err("Invalid email format".into());
        }

        if self.age > 150 {
            return Err("Age must be realistic".into());
        }

        if self.username.len() < 3 {
            return Err("Username must be at least 3 characters".into());
        }

        Ok(())
    }
}

async fn create_user(
    ValidatedJson(user): ValidatedJson<CreateUserRequest>
) -> Result<Response> {
    // User is guaranteed to be valid
    Ok(Response::json(&serde_json::json!({
        "message": "User created",
        "email": user.email,
        "username": user.username
    }))?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .post("/users", create_user);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

---

## Error Handling

Extractors can fail and return appropriate HTTP errors:

```rust
use ignitia::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateProduct {
    name: String,
    price: f64,
    category_id: u32,
}

impl CreateProduct {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::validation("Product name cannot be empty"));
        }

        if self.price <= 0.0 {
            return Err(Error::validation("Price must be positive"));
        }

        if self.category_id == 0 {
            return Err(Error::validation("Invalid category ID"));
        }

        Ok(())
    }
}

async fn create_product(Json(product): Json<CreateProduct>) -> Result<Response> {
    // Validate the extracted data
    product.validate()?;

    // Process the valid product
    Ok(Response::json(&serde_json::json!({
        "message": "Product created",
        "name": product.name,
        "price": product.price
    }))?)
}

// Comprehensive error response
async fn handle_with_detailed_errors(
    Path(id): Path<u32>,
    Json(data): Json<serde_json::Value>
) -> Result<Response> {
    if id == 0 {
        return Err(Error::validation("ID cannot be zero"));
    }

    if !data.is_object() {
        return Err(Error::bad_request("Expected JSON object"));
    }

    Ok(Response::json(&data)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .post("/products", create_product)
        .post("/data/{id}", handle_with_detailed_errors);

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());
    server.ignitia().await?;
    Ok(())
}
```

### Common Error Types

- **Bad Request (400)**: Invalid data format, parsing errors
- **Unauthorized (401)**: Missing or invalid authentication
- **Forbidden (403)**: Valid auth but insufficient permissions
- **Not Found (404)**: Resource doesn't exist
- **Validation (400)**: Data fails validation rules
- **Internal Server Error (500)**: Unexpected errors

---

## Performance Tips

### 1. Extractor Order Matters

Place cheaper extractors first to fail fast:

```rust
// Good: Check simple path parameter before expensive JSON parsing
async fn update_item(
    Path(item_id): Path<u32>,        // Fast: simple parsing
    Json(update): Json<ItemUpdate>   // Slower: JSON parsing
) -> Result<Response> {
    // ...
    Ok(Response::text("Updated"))
}

// Bad: Expensive operations first
async fn bad_order(
    Json(large_data): Json<LargeData>,  // Parse entire body first
    Path(id): Path<u32>                  // Then check simple parameter
) -> Result<Response> {
    // ...
    Ok(Response::text("Processed"))
}
```

### 2. Use Specific Extractors

```rust
// Good: Extract only what you need
async fn good_handler(
    Path(id): Path<u32>
) -> Result<Response> {
    Ok(Response::text(&format!("ID: {}", id)))
}

// Bad: Extract everything
async fn bad_handler(
    Headers(all_headers): Headers,  // Extracts all headers
    req: Request                     // Clones entire request
) -> Result<Response> {
    let id = req.param("id").unwrap();  // Only needed one thing
    Ok(Response::text(&format!("ID: {}", id)))
}
```

### 3. Avoid Unnecessary Cloning

```rust
// Custom extractor that returns a reference (when possible)
#[derive(Debug)]
struct AuthHeader<'a>(&'a str);

// For owned data, consider using Arc
#[derive(Debug, Clone)]
struct SharedState(Arc<AppState>);
```

### 4. Pre-validate in Extractors

```rust
// Good: Validation happens during extraction
struct ValidatedId(u32);

impl FromRequest for ValidatedId {
    type Error = Error;

    fn from_request(req: &Request) -> Result<Self> {
        let id: u32 = req.param("id")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Error::bad_request("Invalid ID"))?;

        if id == 0 {
            return Err(Error::validation("ID cannot be zero"));
        }

        Ok(ValidatedId(id))
    }
}

// Handler receives validated data
async fn handler(ValidatedId(id): ValidatedId) -> Result<Response> {
    // No need to re-validate
    Ok(Response::text(&format!("Valid ID: {}", id)))
}
```

---

## Best Practices

### 1. Use Type-Safe Extractors

```rust
// Good: Type-safe parameter extraction
#[derive(Deserialize)]
struct UserPath {
    user_id: u32,
}

async fn get_user(Path(params): Path<UserPath>) -> Result<Response> {
    // params.user_id is guaranteed to be u32
    Ok(Response::text(&format!("User: {}", params.user_id)))
}

// Avoid: Manual parameter parsing
async fn get_user_bad(req: Request) -> Result<Response> {
    let user_id = req.param("user_id")
        .ok_or_else(|| Error::bad_request("Missing user_id"))?
        .parse::<u32>()
        .map_err(|_| Error::bad_request("Invalid user_id"))?;
    // More boilerplate...
    Ok(Response::text(&format!("User: {}", user_id)))
}
```

### 2. Validate Early

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    #[serde(deserialize_with = "validate_email")]
    email: String,
    name: String,
}

fn validate_email<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let email = String::deserialize(deserializer)?;
    if email.contains('@') {
        Ok(email)
    } else {
        Err(serde::de::Error::custom("Invalid email format"))
    }
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
    // Email is already validated
    Ok(Response::json(&serde_json::json!({
        "email": user.email,
        "name": user.name
    }))?)
}
```

### 3. Use Optional Parameters Wisely

```rust
#[derive(Deserialize)]
struct SearchQuery {
    q: String,                    // Required
    page: Option<u32>,           // Optional with default
    limit: Option<u32>,          // Optional with default
    sort_by: Option<String>,     // Optional, no default
}

async fn search(Query(params): Query<SearchQuery>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10).min(100); // Cap at 100

    Ok(Response::json(&serde_json::json!({
        "query": params.q,
        "page": page,
        "limit": limit,
        "sort_by": params.sort_by
    }))?)
}
```

### 4. Combine Extractors Effectively

```rust
async fn complex_handler(
    // Authentication
    State(auth_service): State<AuthService>,

    // Request identification
    Headers(headers): Headers,

    // URL parameters
    Path(params): Path<ItemParams>,

    // Query parameters
    Query(filters): Query<ItemFilters>,

    // Request body
    Json(update): Json<ItemUpdate>
) -> Result<Response> {
    // All request data is now available and validated

    // Authenticate
    let user = auth_service.authenticate(&headers)?;

    // Process request
    let updated_item = update_item(user, params, filters, update).await?;

    Ok(Response::json(&updated_item)?)
}
```

### 5. Document Your Extractors

```rust
/// Extracts and validates API authentication
///
/// Checks for API key in:
/// 1. `X-API-Key` header
/// 2. `api_key` query parameter
///
/// Returns 401 Unauthorized if missing or invalid
struct ApiKey(String);

impl FromRequest for ApiKey {
    type Error = Error;

    fn from_request(req: &Request) -> Result<Self> {
        // Implementation...
        Ok(ApiKey("key".to_string()))
    }
}
```

---

## Summary

Extractors provide a **powerful** and **type-safe** way to handle request data in Ignitia. They help reduce boilerplate code while ensuring data validation and proper error handling.

### Key Takeaways

1. **Type Safety**: Extractors provide compile-time guarantees about data types
2. **Composability**: Mix and match multiple extractors in handlers
3. **Performance**: Extractors are optimized for zero-copy and minimal allocations
4. **Extensibility**: Create custom extractors for domain-specific needs
5. **Error Handling**: Automatic error responses with detailed messages

Use extractors to create clean, maintainable handlers that clearly express their requirements and handle errors gracefully.

---

For more information, check out:
- [API Documentation](https://docs.rs/ignitia)
- [Examples](https://github.com/AarambhDevHub/ignitia/tree/main/examples)
- [GitHub Repository](https://github.com/AarambhDevHub/ignitia)
