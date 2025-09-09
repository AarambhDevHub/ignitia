//! # Request Data Extractors
//!
//! This module provides a comprehensive system for extracting typed data from HTTP requests.
//! Extractors allow handler functions to automatically receive parsed and validated data
//! from various parts of the request, including path parameters, query strings, JSON bodies,
//! headers, cookies, and more.
//!
//! ## Features
//!
//! - **Type Safety**: All extractions are compile-time type-checked
//! - **Automatic Parsing**: Data is automatically parsed and validated
//! - **Error Handling**: Clear error messages for extraction failures
//! - **Performance**: Optimized extraction with minimal allocations
//! - **Extensibility**: Easy to create custom extractors
//! - **Serde Integration**: Seamless integration with serde for JSON/form data
//!
//! ## Available Extractors
//!
//! - **Path**: Extract typed path parameters
//! - **Query**: Extract typed query parameters
//! - **Json**: Extract and deserialize JSON request bodies
//! - **Headers**: Access to request headers
//! - **Cookies**: Access to request cookies
//! - **Body**: Raw request body access
//! - **Method**: HTTP method extraction
//! - **Uri**: URI information extraction
//! - **Extension**: Custom request extensions
//!
//! ## Basic Usage
//!
//! ```
//! use ignitia::{Path, Query, Json, Response, Result};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserParams {
//!     id: u64,
//! }
//!
//! #[derive(Deserialize)]
//! struct QueryParams {
//!     format: Option<String>,
//!     limit: Option<u32>,
//! }
//!
//! #[derive(Deserialize)]
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! // Extract path parameters
//! async fn get_user(Path(params): Path<UserParams>) -> Result<Response> {
//!     Ok(Response::text(format!("User ID: {}", params.id)))
//! }
//!
//! // Extract query parameters
//! async fn list_users(Query(query): Query<QueryParams>) -> Result<Response> {
//!     let format = query.format.unwrap_or_else(|| "json".to_string());
//!     let limit = query.limit.unwrap_or(10);
//!     Ok(Response::text(format!("Format: {}, Limit: {}", format, limit)))
//! }
//!
//! // Extract JSON body
//! async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
//!     // Process user creation
//!     Ok(Response::text(format!("Creating user: {}", user.name)))
//! }
//!
//! // Multiple extractors
//! async fn update_user(
//!     Path(params): Path<UserParams>,
//!     Json(user): Json<CreateUser>,
//! ) -> Result<Response> {
//!     Ok(Response::text(format!("Updating user {}: {}", params.id, user.name)))
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ### Custom Validation
//! ```
//! use ignitia::{Path, Error, Response, Result};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserParams {
//!     id: u64,
//! }
//!
//! async fn get_user(Path(params): Path<UserParams>) -> Result<Response> {
//!     if params.id == 0 {
//!         return Err(Error::BadRequest("User ID cannot be zero".into()));
//!     }
//!
//!     if params.id > 1000000 {
//!         return Err(Error::BadRequest("User ID too large".into()));
//!     }
//!
//!     Ok(Response::text(format!("Valid user ID: {}", params.id)))
//! }
//! ```
//!
//! ### Optional Parameters
//! ```
//! use ignitia::{Query, Response, Result};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct SearchParams {
//!     q: Option<String>,
//!     page: Option<u32>,
//!     per_page: Option<u32>,
//! }
//!
//! async fn search(Query(params): Query<SearchParams>) -> Result<Response> {
//!     let query = params.q.unwrap_or_else(|| "*".to_string());
//!     let page = params.page.unwrap_or(1);
//!     let per_page = params.per_page.unwrap_or(10);
//!
//!     Ok(Response::json(serde_json::json!({
//!         "query": query,
//!         "page": page,
//!         "per_page": per_page
//!     }))?)
//! }
//! ```

use crate::extension::Extension;
use crate::{Error, Request, Result};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Helper function to convert HashMap<String, String> to serde_json::Value with intelligent type conversion.
///
/// This function attempts to parse string values into their most appropriate JSON types:
/// - Integers are parsed as numbers
/// - Floats are parsed as numbers
/// - "true"/"false" are parsed as booleans
/// - Everything else remains as strings
///
/// # Parameters
/// - `map`: The HashMap to convert
///
/// # Returns
/// A serde_json::Value::Object with converted values
///
/// # Examples
/// ```
/// let mut map = HashMap::new();
/// map.insert("id".to_string(), "123".to_string());
/// map.insert("active".to_string(), "true".to_string());
/// map.insert("name".to_string(), "John".to_string());
/// map.insert("score".to_string(), "95.5".to_string());
///
/// let json_value = convert_string_map_to_json_value(&map);
/// // Results in: {"id": 123, "active": true, "name": "John", "score": 95.5}
/// ```
fn convert_string_map_to_json_value(map: &HashMap<String, String>) -> serde_json::Value {
    let mut json_map = serde_json::Map::new();

    for (key, value) in map {
        // Try to parse as number first, fall back to string
        let json_value = if let Ok(num) = value.parse::<i64>() {
            serde_json::Value::Number(serde_json::Number::from(num))
        } else if let Ok(num) = value.parse::<f64>() {
            serde_json::Value::Number(
                serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0)),
            )
        } else if value == "true" {
            serde_json::Value::Bool(true)
        } else if value == "false" {
            serde_json::Value::Bool(false)
        } else {
            serde_json::Value::String(value.clone())
        };
        json_map.insert(key.clone(), json_value);
    }

    serde_json::Value::Object(json_map)
}

/// Trait for types that can be extracted from HTTP requests.
///
/// This trait defines the interface for extracting typed data from requests.
/// It's implemented by all the extractor types and can be implemented by
/// custom types to create custom extractors.
///
/// # Examples
///
/// ## Custom Extractor Implementation
/// ```
/// use ignitia::{FromRequest, Request, Result, Error};
/// use std::net::IpAddr;
///
/// struct ClientIp(IpAddr);
///
/// impl FromRequest for ClientIp {
///     fn from_request(req: &Request) -> Result<Self> {
///         // Try to get IP from X-Forwarded-For header first
///         if let Some(forwarded) = req.header("x-forwarded-for") {
///             if let Some(ip) = forwarded.split(',').next() {
///                 if let Ok(addr) = ip.trim().parse() {
///                     return Ok(ClientIp(addr));
///                 }
///             }
///         }
///
///         // Try to get IP from X-Real-IP header
///         if let Some(real_ip) = req.header("x-real-ip") {
///             if let Ok(addr) = real_ip.parse() {
///                 return Ok(ClientIp(addr));
///             }
///         }
///
///         // Fallback to a default or return an error
///         Err(Error::BadRequest("Could not determine client IP".into()))
///     }
/// }
///
/// // Usage in handler
/// async fn handler(ClientIp(ip): ClientIp) -> Result<ignitia::Response> {
///     Ok(ignitia::Response::text(format!("Your IP: {}", ip)))
/// }
/// ```
pub trait FromRequest: Sized {
    /// Extract this type from the given request.
    ///
    /// # Parameters
    /// - `req`: The HTTP request to extract data from
    ///
    /// # Returns
    /// - `Ok(Self)` if extraction succeeds
    /// - `Err(Error)` if extraction fails
    ///
    /// # Errors
    /// Common errors include:
    /// - `BadRequest`: Invalid or missing data
    /// - `Internal`: Parsing or conversion errors
    fn from_request(req: &Request) -> Result<Self>;
}

/// Extension extractor implementation.
///
/// This allows extracting request extensions that were previously set by middleware
/// or other parts of the application.
impl<T> FromRequest for Extension<T>
where
    T: Send + Sync + Clone + 'static,
{
    fn from_request(req: &Request) -> Result<Self> {
        req.get_extension::<T>()
            .map(|arc_value| Extension((*arc_value).clone()))
            .ok_or_else(|| {
                Error::Internal(format!(
                    "Extension of type {} not found",
                    std::any::type_name::<T>()
                ))
            })
    }
}

/// Extractor for typed path parameters.
///
/// This extractor deserializes path parameters (like `/users/:id`) into a typed struct.
/// It uses serde for deserialization, allowing for automatic type conversion and validation.
///
/// # Type Requirements
/// The extracted type `T` must implement `DeserializeOwned` from serde.
///
/// # Examples
///
/// ## Single Parameter
/// ```
/// use ignitia::{Path, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UserParams {
///     id: u64,
/// }
///
/// async fn get_user(Path(params): Path<UserParams>) -> Result<Response> {
///     Ok(Response::text(format!("User ID: {}", params.id)))
/// }
/// ```
///
/// ## Multiple Parameters
/// ```
/// use ignitia::{Path, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct PostParams {
///     user_id: u64,
///     post_id: u64,
/// }
///
/// async fn get_post(Path(params): Path<PostParams>) -> Result<Response> {
///     Ok(Response::text(format!("User {} Post {}", params.user_id, params.post_id)))
/// }
/// ```
///
/// ## With Validation
/// ```
/// use ignitia::{Path, Response, Result, Error};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UserParams {
///     id: u64,
/// }
///
/// async fn get_user(Path(params): Path<UserParams>) -> Result<Response> {
///     if params.id == 0 {
///         return Err(Error::BadRequest("Invalid user ID".into()));
///     }
///
///     Ok(Response::text(format!("User ID: {}", params.id)))
/// }
/// ```
///
/// ## String Parameters
/// ```
/// use ignitia::{Path, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct SlugParams {
///     category: String,
///     slug: String,
/// }
///
/// async fn get_article(Path(params): Path<SlugParams>) -> Result<Response> {
///     Ok(Response::text(format!("Article: {}/{}", params.category, params.slug)))
/// }
/// ```
#[derive(Debug)]
pub struct Path<T>(pub T);

impl<T> Path<T> {
    /// Unwraps the path parameters, consuming the wrapper and returning the inner value.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Path;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Params { id: u64 }
    ///
    /// let path = Path(Params { id: 123 });
    /// let params = path.into_inner();
    /// assert_eq!(params.id, 123);
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Path<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for Path<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        if req.params.is_empty() {
            return Err(Error::BadRequest(
                "No path parameters found in request".into(),
            ));
        }

        let params_value = convert_string_map_to_json_value(&req.params);

        let extracted = T::deserialize(params_value).map_err(|e| {
            Error::BadRequest(format!(
                "Failed to extract path parameters: {} (from params: {:?})",
                e, req.params
            ))
        })?;

        Ok(Path(extracted))
    }
}

/// Extractor for typed query parameters.
///
/// This extractor deserializes URL query parameters (like `?page=1&limit=10`) into a typed struct.
/// It supports optional parameters and automatic type conversion.
///
/// # Type Requirements
/// The extracted type `T` must implement `DeserializeOwned` from serde.
///
/// # Examples
///
/// ## Basic Query Parameters
/// ```
/// use ignitia::{Query, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct SearchParams {
///     q: String,
///     page: Option<u32>,
///     limit: Option<u32>,
/// }
///
/// async fn search(Query(params): Query<SearchParams>) -> Result<Response> {
///     let page = params.page.unwrap_or(1);
///     let limit = params.limit.unwrap_or(10);
///
///     Ok(Response::text(format!("Search: '{}' page {} limit {}", params.q, page, limit)))
/// }
/// ```
///
/// ## Optional Parameters
/// ```
/// use ignitia::{Query, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct FilterParams {
///     category: Option<String>,
///     min_price: Option<f64>,
///     max_price: Option<f64>,
///     sort_by: Option<String>,
/// }
///
/// async fn filter_products(Query(params): Query<FilterParams>) -> Result<Response> {
///     let category = params.category.unwrap_or_else(|| "all".to_string());
///     let sort = params.sort_by.unwrap_or_else(|| "name".to_string());
///
///     Ok(Response::json(serde_json::json!({
///         "category": category,
///         "price_range": {
///             "min": params.min_price,
///             "max": params.max_price
///         },
///         "sort_by": sort
///     }))?)
/// }
/// ```
///
/// ## Boolean and Number Parameters
/// ```
/// use ignitia::{Query, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct ListParams {
///     active: Option<bool>,
///     page: Option<u32>,
///     per_page: Option<u32>,
///     include_deleted: Option<bool>,
/// }
///
/// async fn list_items(Query(params): Query<ListParams>) -> Result<Response> {
///     let active = params.active.unwrap_or(true);
///     let include_deleted = params.include_deleted.unwrap_or(false);
///
///     Ok(Response::json(serde_json::json!({
///         "filters": {
///             "active": active,
///             "include_deleted": include_deleted
///         },
///         "pagination": {
///             "page": params.page.unwrap_or(1),
///             "per_page": params.per_page.unwrap_or(20)
///         }
///     }))?)
/// }
/// ```
#[derive(Debug)]
pub struct Query<T>(pub T);

impl<T> Query<T> {
    /// Unwraps the query parameters, consuming the wrapper and returning the inner value.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Query;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Params { page: Option<u32> }
    ///
    /// let query = Query(Params { page: Some(5) });
    /// let params = query.into_inner();
    /// assert_eq!(params.page, Some(5));
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Query<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        let query_value = convert_string_map_to_json_value(&req.query_params);

        let extracted = T::deserialize(query_value).map_err(|e| {
            Error::BadRequest(format!(
                "Failed to extract query parameters: {} (from query_params: {:?})",
                e, req.query_params
            ))
        })?;

        Ok(Query(extracted))
    }
}

/// Extractor for JSON request bodies.
///
/// This extractor reads and deserializes the request body as JSON into a typed struct.
/// It automatically validates the Content-Type header and handles JSON parsing errors.
///
/// # Type Requirements
/// The extracted type `T` must implement `DeserializeOwned` from serde.
///
/// # Examples
///
/// ## Basic JSON Extraction
/// ```
/// use ignitia::{Json, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
///     age: Option<u32>,
/// }
///
/// async fn create_user(Json(user): Json<CreateUser>) -> Result<Response> {
///     // Validate the data
///     if user.name.is_empty() {
///         return Err(ignitia::Error::BadRequest("Name is required".into()));
///     }
///
///     if !user.email.contains('@') {
///         return Err(ignitia::Error::BadRequest("Invalid email format".into()));
///     }
///
///     // Process user creation
///     Ok(Response::json(serde_json::json!({
///         "message": "User created successfully",
///         "user": {
///             "name": user.name,
///             "email": user.email,
///             "age": user.age
///         }
///     }))?)
/// }
/// ```
///
/// ## Complex Nested JSON
/// ```
/// use ignitia::{Json, Response, Result};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Address {
///     street: String,
///     city: String,
///     country: String,
/// }
///
/// #[derive(Deserialize)]
/// struct CreateUserWithAddress {
///     name: String,
///     email: String,
///     address: Address,
///     tags: Vec<String>,
/// }
///
/// async fn create_user_with_address(Json(user): Json<CreateUserWithAddress>) -> Result<Response> {
///     Ok(Response::text(format!(
///         "Creating user {} in {}, {} with {} tags",
///         user.name, user.address.city, user.address.country, user.tags.len()
///     )))
/// }
/// ```
///
/// ## API Request/Response Pattern
/// ```
/// use ignitia::{Json, Response, Result};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct UpdateProductRequest {
///     name: Option<String>,
///     price: Option<f64>,
///     description: Option<String>,
/// }
///
/// #[derive(Serialize)]
/// struct UpdateProductResponse {
///     id: u64,
///     updated_fields: Vec<String>,
///     success: bool,
/// }
///
/// async fn update_product(
///     ignitia::Path(id): ignitia::Path<u64>,
///     Json(update): Json<UpdateProductRequest>,
/// ) -> Result<Response> {
///     let mut updated_fields = Vec::new();
///
///     if update.name.is_some() { updated_fields.push("name".to_string()); }
///     if update.price.is_some() { updated_fields.push("price".to_string()); }
///     if update.description.is_some() { updated_fields.push("description".to_string()); }
///
///     let response = UpdateProductResponse {
///         id,
///         updated_fields,
///         success: true,
///     };
///
///     Response::json(response)
/// }
/// ```
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    /// Unwraps the JSON data, consuming the wrapper and returning the inner value.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Json;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct User { name: String }
    ///
    /// let json = Json(User { name: "Alice".to_string() });
    /// let user = json.into_inner();
    /// assert_eq!(user.name, "Alice");
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        let extracted = req.json()?;
        Ok(Json(extracted))
    }
}

/// Extractor for HTTP headers.
///
/// This extractor provides access to all HTTP headers as a HashMap of strings.
/// It automatically converts header names to lowercase and values to UTF-8 strings.
///
/// # Examples
///
/// ## Basic Header Access
/// ```
/// use ignitia::{Headers, Response, Result};
///
/// async fn debug_headers(Headers(headers): Headers) -> Result<Response> {
///     let mut debug_info = Vec::new();
///
///     for (name, value) in headers.iter() {
///         debug_info.push(format!("{}: {}", name, value));
///     }
///
///     Ok(Response::text(debug_info.join("\n")))
/// }
/// ```
///
/// ## Content Type Handling
/// ```
/// use ignitia::{Headers, Response, Result, Error};
///
/// async fn handle_content_type(Headers(headers): Headers) -> Result<Response> {
///     let content_type = headers.get("content-type")
///         .ok_or_else(|| Error::BadRequest("Content-Type header required".into()))?;
///
///     match content_type.as_str() {
///         "application/json" => Ok(Response::text("JSON content detected")),
///         "application/xml" => Ok(Response::text("XML content detected")),
///         "text/plain" => Ok(Response::text("Plain text content detected")),
///         _ => Err(Error::BadRequest(format!("Unsupported content type: {}", content_type))),
///     }
/// }
/// ```
///
/// ## Custom Authorization
/// ```
/// use ignitia::{Headers, Response, Result, Error};
///
/// async fn check_auth(Headers(headers): Headers) -> Result<Response> {
///     let auth_header = headers.get("authorization")
///         .ok_or_else(|| Error::Unauthorized)?;
///
///     if auth_header.starts_with("Bearer ") {
///         let token = &auth_header[7..];
///         if is_valid_token(token) {
///             Ok(Response::text("Access granted"))
///         } else {
///             Err(Error::Unauthorized)
///         }
///     } else {
///         Err(Error::BadRequest("Bearer token required".into()))
///     }
/// }
///
/// # fn is_valid_token(_token: &str) -> bool { true }
/// ```
#[derive(Debug)]
pub struct Headers(pub HashMap<String, String>);

impl std::ops::Deref for Headers {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Headers {
    fn from_request(req: &Request) -> Result<Self> {
        let mut headers = HashMap::new();
        for (key, value) in req.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }
        Ok(Headers(headers))
    }
}

/// Extractor for HTTP cookies.
///
/// This extractor provides access to all cookies sent with the request through
/// the framework's CookieJar type, which offers convenient methods for cookie access.
///
/// # Examples
///
/// ## Basic Cookie Access
/// ```
/// use ignitia::{Cookies, Response, Result};
///
/// async fn handle_cookies(Cookies(cookies): Cookies) -> Result<Response> {
///     if let Some(session_id) = cookies.get("session_id") {
///         Ok(Response::text(format!("Session ID: {}", session_id)))
///     } else {
///         Ok(Response::text("No session found"))
///     }
/// }
/// ```
///
/// ## Multiple Cookie Access
/// ```
/// use ignitia::{Cookies, Response, Result};
///
/// async fn user_preferences(Cookies(cookies): Cookies) -> Result<Response> {
///     let theme = cookies.get("theme").unwrap_or(&"light".to_string()).clone();
///     let lang = cookies.get("language").unwrap_or(&"en".to_string()).clone();
///     let timezone = cookies.get("timezone").unwrap_or(&"UTC".to_string()).clone();
///
///     Ok(Response::json(serde_json::json!({
///         "preferences": {
///             "theme": theme,
///             "language": lang,
///             "timezone": timezone
///         }
///     }))?)
/// }
/// ```
///
/// ## Session Management
/// ```
/// use ignitia::{Cookies, Response, Result, Error};
///
/// async fn protected_route(Cookies(cookies): Cookies) -> Result<Response> {
///     let session_id = cookies.get("session_id")
///         .ok_or_else(|| Error::Unauthorized)?;
///
///     if !is_valid_session(session_id) {
///         return Err(Error::Unauthorized);
///     }
///
///     Ok(Response::text("Access granted to protected resource"))
/// }
///
/// # fn is_valid_session(_session_id: &str) -> bool { true }
/// ```
#[derive(Debug)]
pub struct Cookies(pub crate::CookieJar);

impl std::ops::Deref for Cookies {
    type Target = crate::CookieJar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Cookies {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Cookies(req.cookies()))
    }
}

/// Extractor for raw request body.
///
/// This extractor provides access to the raw request body as bytes, without any
/// parsing or interpretation. Useful for handling binary data, custom formats,
/// or when you need full control over body processing.
///
/// # Examples
///
/// ## Binary Data Handling
/// ```
/// use ignitia::{Body, Response, Result};
///
/// async fn upload_image(Body(body): Body) -> Result<Response> {
///     if body.is_empty() {
///         return Ok(Response::text("No image data received"));
///     }
///
///     // Check if it's a valid image format (simple check)
///     let is_jpeg = body.starts_with(b"\xFF\xD8\xFF");
///     let is_png = body.starts_with(b"\x89PNG");
///
///     if !is_jpeg && !is_png {
///         return Err(ignitia::Error::BadRequest("Invalid image format".into()));
///     }
///
///     // Process image upload
///     Ok(Response::text(format!("Received {} bytes of image data", body.len())))
/// }
/// ```
///
/// ## Text Processing
/// ```
/// use ignitia::{Body, Response, Result, Error};
///
/// async fn process_text(Body(body): Body) -> Result<Response> {
///     let text = String::from_utf8(body.to_vec())
///         .map_err(|_| Error::BadRequest("Invalid UTF-8 text".into()))?;
///
///     let word_count = text.split_whitespace().count();
///     let char_count = text.chars().count();
///     let line_count = text.lines().count();
///
///     Ok(Response::json(serde_json::json!({
///         "stats": {
///             "words": word_count,
///             "characters": char_count,
///             "lines": line_count
///         }
///     }))?)
/// }
/// ```
///
/// ## Custom Format Parsing
/// ```
/// use ignitia::{Body, Response, Result, Error};
///
/// async fn parse_csv(Body(body): Body) -> Result<Response> {
///     let text = String::from_utf8(body.to_vec())
///         .map_err(|_| Error::BadRequest("Invalid UTF-8 in CSV".into()))?;
///
///     let mut records = Vec::new();
///     let mut lines = text.lines();
///
///     // Skip header
///     if let Some(_header) = lines.next() {
///         for line in lines {
///             let fields: Vec<&str> = line.split(',').collect();
///             records.push(fields);
///         }
///     }
///
///     Ok(Response::text(format!("Parsed {} CSV records", records.len())))
/// }
/// ```
#[derive(Debug)]
pub struct Body(pub bytes::Bytes);

impl std::ops::Deref for Body {
    type Target = bytes::Bytes;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Body {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Body(req.body.clone()))
    }
}

/// Extractor for HTTP method.
///
/// This extractor provides access to the HTTP method of the request.
/// Useful for handlers that need to behave differently based on the method.
///
/// # Examples
///
/// ## Method-Based Logic
/// ```
/// use ignitia::{Method, Response, Result, Error};
/// use http::Method as HttpMethod;
///
/// async fn handle_method(Method(method): Method) -> Result<Response> {
///     match *method {
///         HttpMethod::GET => Ok(Response::text("GET request received")),
///         HttpMethod::POST => Ok(Response::text("POST request received")),
///         HttpMethod::PUT => Ok(Response::text("PUT request received")),
///         HttpMethod::DELETE => Ok(Response::text("DELETE request received")),
///         _ => Err(Error::BadRequest(format!("Unsupported method: {}", method))),
///     }
/// }
/// ```
///
/// ## CORS Preflight Handling
/// ```
/// use ignitia::{Method, Response, Result};
/// use http::Method as HttpMethod;
///
/// async fn cors_handler(Method(method): Method) -> Result<Response> {
///     if *method == HttpMethod::OPTIONS {
///         Ok(Response::ok()
///             .with_header("Access-Control-Allow-Origin", "*")
///             .with_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
///             .with_header("Access-Control-Allow-Headers", "Content-Type, Authorization"))
///     } else {
///         Ok(Response::text(format!("Method: {}", method)))
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Method(pub http::Method);

impl std::ops::Deref for Method {
    type Target = http::Method;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Method {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Method(req.method.clone()))
    }
}

/// Extractor for request URI information.
///
/// This extractor provides access to the complete URI information from the request,
/// including path, query string, and other URI components.
///
/// # Examples
///
/// ## URI Analysis
/// ```
/// use ignitia::{Uri, Response, Result};
///
/// async fn analyze_uri(Uri(uri): Uri) -> Result<Response> {
///     let path = uri.path();
///     let query = uri.query().unwrap_or("none");
///     let scheme = uri.scheme_str().unwrap_or("unknown");
///     let host = uri.host().unwrap_or("unknown");
///
///     Ok(Response::json(serde_json::json!({
///         "uri_info": {
///             "path": path,
///             "query": query,
///             "scheme": scheme,
///             "host": host
///         }
///     }))?)
/// }
/// ```
///
/// ## Path Parsing
/// ```
/// use ignitia::{Uri, Response, Result};
///
/// async fn path_info(Uri(uri): Uri) -> Result<Response> {
///     let path = uri.path();
///     let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
///
///     Ok(Response::json(serde_json::json!({
///         "path_analysis": {
///             "full_path": path,
///             "segments": segments,
///             "depth": segments.len()
///         }
///     }))?)
/// }
/// ```
///
/// ## Query String Access
/// ```
/// use ignitia::{Uri, Response, Result};
///
/// async fn query_info(Uri(uri): Uri) -> Result<Response> {
///     match uri.query() {
///         Some(query) => {
///             let params: Vec<&str> = query.split('&').collect();
///             Ok(Response::text(format!("Query has {} parameters: {}", params.len(), query)))
///         }
///         None => Ok(Response::text("No query string present"))
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Uri(pub http::Uri);

impl std::ops::Deref for Uri {
    type Target = http::Uri;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Uri {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Uri(req.uri.clone()))
    }
}

// // Request extractor (for cases where you still need the full request)
// impl FromRequest for Request {
//     fn from_request(req: &Request) -> Result<Self> {
//         // We can't move out of a reference, so we'll need to clone
//         // This is a limitation - in a real implementation, you'd want to avoid this
//         Ok(Request {
//             method: req.method.clone(),
//             uri: req.uri.clone(),
//             version: req.version,
//             headers: req.headers.clone(),
//             body: req.body.clone(),
//             params: req.params.clone(),
//             query_params: req.query_params.clone(),
//             extensions: req.extensions.clone(),
//         })
//     }
// }
