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
- [Multiple Extractors](#multiple-extractors)
- [Custom Extractors](#custom-extractors)
- [Error Handling](#error-handling)
- [Performance Tips](#performance-tips)
- [Best Practices](#best-practices)

## Overview

Extractors implement the `FromRequest` trait and automatically extract data from incoming HTTP requests. They're used as parameters in handler functions:

```rust
use ignitia::{Router, Response, Json, Path, Query};
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

async fn get_user(Path(user_id): Path<u32>, Query(params): Query<UserQuery>) -> Result<Response, ignitia::Error> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "limit": params.limit.unwrap_or(10),
        "offset": params.offset.unwrap_or(0)
    }))?)
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response, ignitia::Error> {
    Ok(Response::json(serde_json::json!({
        "message": "User created",
        "name": user.name,
        "email": user.email
    }))?)
}

let router = Router::new()
    .get("/users/:user_id", get_user)
    .post("/users", create_user);
```

## Core Concept

The `FromRequest` trait defines how extractors work:

```rust
pub trait FromRequest: Sized {
    fn from_request(req: &Request) -> Result<Self>;
}
```

Extractors are applied in order as handler parameters, and if any extraction fails, the request is rejected with an appropriate error.

## Built-in Extractors

### Path Parameters

Extract typed parameters from URL paths using `Path<T>`:

```rust
use ignitia::{Path, Response};
use serde::Deserialize;

// Single parameter
async fn get_user(Path(user_id): Path<u32>) -> Result<Response, ignitia::Error> {
    Ok(Response::text(format!("User ID: {}", user_id)))
}

// Multiple parameters
#[derive(Deserialize)]
struct PostPath {
    user_id: u32,
    post_id: u32,
}

async fn get_post(Path(params): Path<PostPath>) -> Result<Response, ignitia::Error> {
    Ok(Response::text(format!(
        "User {} Post {}",
        params.user_id, params.post_id
    )))
}

let router = Router::new()
    .get("/users/:user_id", get_user)
    .get("/users/:user_id/posts/:post_id", get_post);
```

**Supported Types:**
- Primitive types: `u32`, `i64`, `String`, `bool`, etc.
- Custom structs implementing `Deserialize`
- `Option<T>` for optional parameters

### Query Parameters

Extract query parameters using `Query<T>`:

```rust
use ignitia::{Query, Response};
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
    tags: Option<Vec<String>>,
}

async fn search(Query(params): Query<SearchQuery>) -> Result<Response, ignitia::Error> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);

    Ok(Response::json(serde_json::json!({
        "query": params.q,
        "page": page,
        "limit": limit,
        "tags": params.tags.unwrap_or_default()
    }))?)
}

// URL: /search?q=rust&page=2&limit=20&tags=web&tags=framework
let router = Router::new()
    .get("/search", search);
```

**Query Parameter Features:**
- Automatic type conversion
- Support for arrays/vectors
- Optional parameters with `Option<T>`
- Boolean parsing (`true`, `false`, `1`, `0`)

### JSON Body

Extract JSON request bodies using `Json<T>`:

```rust
use ignitia::{Json, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Serialize)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
}

async fn create_user(Json(user): Json<CreateUser>) -> Result<Response, ignitia::Error> {
    // Validate email
    if !user.email.contains('@') {
        return Err(ignitia::Error::BadRequest("Invalid email address".into()));
    }

    let response = UserResponse {
        id: 123,
        name: user.name,
        email: user.email,
    };

    Ok(Response::json(response)?)
}

let router = Router::new()
    .post("/users", create_user);
```

**Requirements:**
- Content-Type must be `application/json`
- Request body must be valid JSON
- JSON structure must match the target type

### Form Data

Extract form-encoded data using `Form<T>`:

```rust
use ignitia::{Form, Response};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
    remember: Option<bool>,
}

async fn login(Form(form): Form<LoginForm>) -> Result<Response, ignitia::Error> {
    // Authenticate user
    if form.username == "admin" && form.password == "secret" {
        Ok(Response::text("Login successful"))
    } else {
        Err(ignitia::Error::Unauthorized)
    }
}

let router = Router::new()
    .post("/login", login);
```

**Requirements:**
- Content-Type must be `application/x-www-form-urlencoded`
- Supports URL encoding/decoding
- Boolean values: `true`, `false`, `1`, `0`

### Headers

Access request headers using `Headers`:

```rust
use ignitia::{Headers, Response};

async fn check_auth(Headers(headers): Headers) -> Result<Response, ignitia::Error> {
    if let Some(auth) = headers.get("authorization") {
        if auth.starts_with("Bearer ") {
            Ok(Response::text("Authorized"))
        } else {
            Err(ignitia::Error::Unauthorized)
        }
    } else {
        Err(ignitia::Error::Unauthorized)
    }
}

let router = Router::new()
    .get("/protected", check_auth);
```

### Cookies

Access cookies using `Cookies`:

```rust
use ignitia::{Cookies, Response, Cookie};

async fn get_session(Cookies(cookies): Cookies) -> Result<Response, ignitia::Error> {
    if let Some(session_id) = cookies.get("session_id") {
        Ok(Response::text(format!("Session: {}", session_id)))
    } else {
        // Set a new session cookie
        let response = Response::text("New session created")
            .add_cookie(Cookie::new("session_id", "new-session-123")
                .path("/")
                .http_only()
                .max_age(3600)); // 1 hour
        Ok(response)
    }
}

let router = Router::new()
    .get("/session", get_session);
```

### Raw Body

Access the raw request body using `Body`:

```rust
use ignitia::{Body, Response};

async fn upload_file(Body(body): Body) -> Result<Response, ignitia::Error> {
    // Process raw binary data
    let file_size = body.len();

    if file_size > 10 * 1024 * 1024 { // 10MB limit
        return Err(ignitia::Error::BadRequest("File too large".into()));
    }

    // Save file or process data
    tokio::fs::write("uploaded_file.bin", &body).await
        .map_err(|e| ignitia::Error::Internal(e.to_string()))?;

    Ok(Response::json(serde_json::json!({
        "status": "uploaded",
        "size": file_size
    }))?)
}

let router = Router::new()
    .post("/upload", upload_file);
```

### HTTP Method

Access the HTTP method using `Method`:

```rust
use ignitia::{Method, Response};

async fn method_info(Method(method): Method) -> Result<Response, ignitia::Error> {
    let method_name = method.as_str();
    Ok(Response::text(format!("Request method: {}", method_name)))
}

let router = Router::new()
    .route("/method-info", http::Method::GET, method_info.clone())
    .route("/method-info", http::Method::POST, method_info);
```

### URI

Access the request URI using `Uri`:

```rust
use ignitia::{Uri, Response};

async fn uri_info(Uri(uri): Uri) -> Result<Response, ignitia::Error> {
    Ok(Response::json(serde_json::json!({
        "path": uri.path(),
        "query": uri.query(),
        "scheme": uri.scheme_str(),
        "host": uri.host()
    }))?)
}

let router = Router::new()
    .get("/uri-info", uri_info);
```

### Application State

Access shared application state using `State<T>`:

```rust
use ignitia::{State, Response, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    counter: Arc<RwLock<u64>>,
}

async fn increment_counter(State(state): State<AppState>) -> Result<Response, ignitia::Error> {
    let mut counter = state.counter.write().await;
    *counter += 1;

    Ok(Response::json(serde_json::json!({
        "counter": *counter
    }))?)
}

async fn get_counter(State(state): State<AppState>) -> Result<Response, ignitia::Error> {
    let counter = state.counter.read().await;

    Ok(Response::json(serde_json::json!({
        "counter": *counter
    }))?)
}

let state = AppState {
    counter: Arc::new(RwLock::new(0)),
};

let router = Router::new()
    .state(state)
    .post("/counter/increment", increment_counter)
    .get("/counter", get_counter);
```

### Extensions

Access request extensions using `Extension<T>`:

```rust
use ignitia::{Extension, Response, Request, Result, Error};

#[derive(Clone)]
struct UserId(u32);

#[derive(Clone)]
struct UserRole(String);

// Middleware to extract user info
async fn auth_middleware(mut req: Request) -> Result<Request> {
    // Extract user info from token/session
    let user_id = UserId(123);
    let user_role = UserRole("admin".to_string());

    req.insert_extension(user_id);
    req.insert_extension(user_role);

    Ok(req)
}

async fn protected_handler(
    Extension(user_id): Extension<UserId>,
    Extension(role): Extension<UserRole>
) -> Result<Response> {
    Ok(Response::json(serde_json::json!({
        "user_id": user_id.0,
        "role": role.0
    }))?)
}
```

## Multiple Extractors

You can use multiple extractors in a single handler:

```rust
use ignitia::{Path, Query, Json, Headers, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Deserialize)]
struct UpdateQuery {
    force: Option<bool>,
}

async fn update_user(
    Path(user_id): Path<u32>,
    Query(params): Query<UpdateQuery>,
    Headers(headers): Headers,
    Json(update): Json<UpdateUser>
) -> Result<Response, ignitia::Error> {
    // Check authorization
    let auth_header = headers.get("authorization")
        .ok_or_else(|| ignitia::Error::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ignitia::Error::Unauthorized);
    }

    // Process update
    let force_update = params.force.unwrap_or(false);

    Ok(Response::json(serde_json::json!({
        "user_id": user_id,
        "updated": true,
        "force": force_update,
        "changes": {
            "name": update.name,
            "email": update.email
        }
    }))?)
}

let router = Router::new()
    .patch("/users/:user_id", update_user);
```

## Custom Extractors

Create custom extractors by implementing `FromRequest`:

```rust
use ignitia::{Request, Result, Error};
use serde::Deserialize;

// Custom extractor for API keys
#[derive(Debug)]
struct ApiKey(String);

impl FromRequest for ApiKey {
    fn from_request(req: &Request) -> Result<Self> {
        // Try header first
        if let Some(key) = req.header("x-api-key") {
            return Ok(ApiKey(key.to_string()));
        }

        // Try query parameter
        if let Some(key) = req.query("api_key") {
            return Ok(ApiKey(key.clone()));
        }

        Err(Error::BadRequest("Missing API key".into()))
    }
}

// Custom extractor for user agent parsing
struct UserAgent {
    browser: String,
    version: String,
    platform: String,
}

impl FromRequest for UserAgent {
    fn from_request(req: &Request) -> Result<Self> {
        let user_agent = req.header("user-agent")
            .ok_or_else(|| Error::BadRequest("Missing User-Agent header".into()))?;

        // Simple parsing (in real apps, use a proper parser)
        let parts: Vec<&str> = user_agent.split(' ').collect();

        Ok(UserAgent {
            browser: parts.get(0).unwrap_or(&"Unknown").to_string(),
            version: parts.get(1).unwrap_or(&"Unknown").to_string(),
            platform: parts.last().unwrap_or(&"Unknown").to_string(),
        })
    }
}

async fn protected_endpoint(
    _api_key: ApiKey, // Authentication required
    user_agent: UserAgent
) -> Result<Response, ignitia::Error> {
    Ok(Response::json(serde_json::json!({
        "browser": user_agent.browser,
        "version": user_agent.version,
        "platform": user_agent.platform
    }))?)
}
```

### Advanced Custom Extractor with Validation

```rust
use ignitia::{Request, Result, Error};
use serde::Deserialize;

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
    fn from_request(req: &Request) -> Result<Self> {
        // Extract query parameters
        let query_value = crate::handler::extractor::convert_string_map_to_json_value(&req.query_params);
        let params: PaginationQuery = serde_json::from_value(query_value)
            .map_err(|e| Error::BadRequest(format!("Invalid pagination params: {}", e)))?;

        // Validate and apply defaults
        let page = params.page.unwrap_or(1);
        let limit = params.limit.unwrap_or(10);

        // Validation
        if page == 0 {
            return Err(Error::BadRequest("Page must be >= 1".into()));
        }

        if limit == 0 || limit > 100 {
            return Err(Error::BadRequest("Limit must be between 1 and 100".into()));
        }

        let offset = (page - 1) * limit;

        Ok(ValidatedPagination {
            page,
            limit,
            offset,
        })
    }
}

async fn list_items(pagination: ValidatedPagination) -> Result<Response, ignitia::Error> {
    // Use validated pagination parameters
    Ok(Response::json(serde_json::json!({
        "page": pagination.page,
        "limit": pagination.limit,
        "offset": pagination.offset,
        "items": []
    }))?)
}
```

## Error Handling

Extractors can fail and return appropriate HTTP errors:

```rust
use ignitia::{Json, Response, Error, Result};
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
            return Err(Error::Validation("Product name cannot be empty".into()));
        }

        if self.price <= 0.0 {
            return Err(Error::Validation("Price must be positive".into()));
        }

        if self.category_id == 0 {
            return Err(Error::Validation("Invalid category ID".into()));
        }

        Ok(())
    }
}

async fn create_product(Json(product): Json<CreateProduct>) -> Result<Response> {
    // Validate the extracted data
    product.validate()?;

    // Process the valid product
    Ok(Response::json(serde_json::json!({
        "message": "Product created",
        "name": product.name,
        "price": product.price
    }))?)
}
```

### Custom Error Responses

```rust
use ignitia::{Error, Request, Result};
use serde::Deserialize;

struct ValidatedJson<T>(T);

impl<T> FromRequest for ValidatedJson<T>
where
    T: for<'de> Deserialize<'de> + Validate,
{
    fn from_request(req: &Request) -> Result<Self> {
        // Extract JSON
        let data: T = req.json()?;

        // Validate
        data.validate()
            .map_err(|e| Error::Validation(format!("Validation failed: {}", e)))?;

        Ok(ValidatedJson(data))
    }
}

trait Validate {
    fn validate(&self) -> std::result::Result<(), String>;
}

#[derive(Deserialize)]
struct User {
    email: String,
    age: u32,
}

impl Validate for User {
    fn validate(&self) -> std::result::Result<(), String> {
        if !self.email.contains('@') {
            return Err("Invalid email format".into());
        }

        if self.age > 150 {
            return Err("Age must be realistic".into());
        }

        Ok(())
    }
}

async fn create_user(ValidatedJson(user): ValidatedJson<User>) -> Result<Response> {
    // User is guaranteed to be valid
    Ok(Response::json(serde_json::json!({
        "message": "User created",
        "email": user.email
    }))?)
}
```

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
}
```

### 2. Use References Where Possible

```rust
// For read-only access, consider implementing custom extractors
// that return references instead of owned data
struct HeaderRef<'a>(&'a str);

impl<'a> FromRequest for HeaderRef<'a> {
    fn from_request(req: &'a Request) -> Result<Self> {
        req.header("authorization")
            .map(HeaderRef)
            .ok_or_else(|| Error::BadRequest("Missing authorization header".into()))
    }
}
```

### 3. Avoid Unnecessary Cloning

```rust
// Instead of extracting all headers
async fn bad_handler(Headers(all_headers): Headers) -> Result<Response> {
    let auth = all_headers.get("authorization"); // Only need one header
    // ...
}

// Create a specific extractor
struct AuthHeader(String);

impl FromRequest for AuthHeader {
    fn from_request(req: &Request) -> Result<Self> {
        req.header("authorization")
            .map(|h| AuthHeader(h.to_string()))
            .ok_or_else(|| Error::Unauthorized)
    }
}

async fn good_handler(AuthHeader(auth): AuthHeader) -> Result<Response> {
    // Only extracted what we need
    // ...
}
```

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
}

// Avoid: Manual parameter parsing
async fn get_user_bad(req: Request) -> Result<Response> {
    let user_id = req.param("user_id")
        .ok_or_else(|| Error::BadRequest("Missing user_id".into()))?
        .parse::<u32>()
        .map_err(|_| Error::BadRequest("Invalid user_id".into()))?;
    // ...
}
```

### 2. Validate Early

```rust
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
    // Email is already validated by the time we get here
    Ok(Response::json(serde_json::json!({
        "message": "User created",
        "email": user.email
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

    Ok(Response::json(serde_json::json!({
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
    let user = auth_service.authenticate(&headers).await?;

    // Process request
    let updated_item = update_item(user, params, filters, update).await?;

    Ok(Response::json(updated_item)?)
}
```

### 5. Handle Large Payloads

```rust
async fn upload_handler(Body(body): Body) -> Result<Response> {
    // Check size before processing
    if body.len() > 10 * 1024 * 1024 { // 10MB
        return Err(Error::BadRequest("Payload too large".into()));
    }

    // Process the body
    process_upload(&body).await?;

    Ok(Response::text("Upload successful"))
}
```

Extractors provide a powerful and type-safe way to handle request data in Ignitia. They help reduce boilerplate code while ensuring data validation and proper error handling. Use them to create clean, maintainable handlers that clearly express their requirements.
