//! Request extractor types for the Ignitia web framework.
//!
//! This module provides extractors that automatically parse and extract data from HTTP requests.
//! Extractors implement the [`FromRequest`] trait, allowing them to be used as handler parameters.
//!
//! # Overview
//!
//! Extractors enable declarative request handling by automatically extracting typed data from various
//! parts of the request (path parameters, query strings, JSON bodies, headers, etc.).
//!
//! # Available Extractors
//!
//! - [`Path`] - Extract path parameters
//! - [`Query`] - Extract query string parameters
//! - [`Json`] - Parse JSON request body
//! - [`Body`] - Access raw request body
//! - [`Headers`] - Extract HTTP headers
//! - [`Cookies`] - Access request cookies
//! - [`Method`] - Extract HTTP method
//! - [`Uri`] - Extract request URI
//! - [`State`] - Access application state
//! - [`Extension`] - Access request extensions
//! - [`Form`] - Parse URL-encoded form data
//!
//! # Examples
//!
//! ## Basic Path Parameter Extraction
//!
//! ```
//! use ignitia::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserParams {
//!     id: u32,
//! }
//!
//! async fn get_user(Path(params): Path<UserParams>) -> String {
//!     format!("User ID: {}", params.id)
//! }
//!
//! let router = Router::new()
//!     .get("/users/:id", get_user);
//! ```
//!
//! ## Multiple Extractors
//!
//! ```
//! use ignitia::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct QueryParams {
//!     notify: Option<bool>,
//! }
//!
//! async fn create_user(
//!     Path(id): Path<String>,
//!     Query(params): Query<QueryParams>,
//!     Json(user): Json<CreateUser>,
//! ) -> Result<Response> {
//!     // Process user creation...
//!     Ok(Response::json(json!({"id": id, "name": user.name})))
//! }
//! ```
//!
//! ## Using State
//!
//! ```
//! use ignitia::prelude::*;
//! use std::sync::Arc;
//!
//! #[derive(Clone)]
//! struct AppState {
//!     db_pool: Arc<DatabasePool>,
//! }
//!
//! async fn handler(State(state): State<AppState>) -> Result<Response> {
//!     // Access shared state
//!     let data = state.db_pool.query().await?;
//!     Ok(Response::json(data))
//! }
//!
//! let state = AppState { db_pool: Arc::new(create_pool()) };
//! let router = Router::new()
//!     .state(state)
//!     .get("/data", handler);
//! ```

use crate::extension::Extension;
use crate::response::IntoResponse;
use crate::{Request, Response};
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

/// Core trait for extracting typed data from HTTP requests.
///
/// Types implementing this trait can be used as handler parameters, enabling
/// automatic extraction and validation of request data. The framework will
/// automatically call `from_request` for each extractor parameter before
/// invoking the handler function.
///
/// # Type Parameters
///
/// The trait has an associated `Error` type that must implement `Into<ExtractionError>`,
/// allowing custom error handling for failed extractions.
///
/// # Examples
///
/// ## Implementing a Custom Extractor
///
/// ```
/// use ignitia::handler::extractor::FromRequest;
/// use ignitia::Request;
///
/// struct ApiKey(String);
///
/// impl FromRequest for ApiKey {
///     type Error = ExtractionError;
///
///     fn from_request(req: &Request) -> Result<Self, Self::Error> {
///         req.header("x-api-key")
///             .map(|key| ApiKey(key.to_string()))
///             .ok_or_else(|| ExtractionError::unauthorized("Missing API key"))
///     }
/// }
///
/// async fn protected_handler(ApiKey(key): ApiKey) -> String {
///     format!("Authenticated with key: {}", key)
/// }
/// ```
pub trait FromRequest: Sized {
    /// The error type returned when extraction fails.
    type Error: IntoResponse;

    /// Extract this type from an HTTP request.
    ///
    /// # Arguments
    ///
    /// * `req` - Reference to the HTTP request
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if extraction succeeds, or `Err(Self::Error)` if it fails.
    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error>;
}

/// Error type for request extraction failures.
///
/// This type represents all possible errors that can occur during request data extraction,
/// providing structured error information that can be converted into HTTP error responses.
///
/// # Fields
///
/// * `message` - Human-readable error message
/// * `status` - HTTP status code for the error response
/// * `error_type` - Machine-readable error type identifier
#[derive(Debug)]
pub struct ExtractionError {
    /// Human-readable error message describing what went wrong
    pub message: String,
    /// HTTP status code to return to the client
    pub status: http::StatusCode,
    /// Machine-readable error type (e.g., "bad_request", "unauthorized")
    pub error_type: String,
}

impl ExtractionError {
    /// Create a new extraction error with custom details.
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    /// * `status` - HTTP status code
    /// * `error_type` - Error type identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::handler::extractor::ExtractionError;
    /// use http::StatusCode;
    ///
    /// let error = ExtractionError::new(
    ///     "Invalid input",
    ///     StatusCode::BAD_REQUEST,
    ///     "validation_error",
    /// );
    /// ```
    pub fn new(message: impl Into<String>, status: http::StatusCode, error_type: &str) -> Self {
        Self {
            message: message.into(),
            status,
            error_type: error_type.to_string(),
        }
    }

    /// Create a bad request error (400).
    ///
    /// Used when the request contains invalid or malformed data.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of what was invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::handler::extractor::ExtractionError;
    ///
    /// let error = ExtractionError::bad_request("Missing required field 'email'");
    /// ```
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(message, http::StatusCode::BAD_REQUEST, "bad_request")
    }

    /// Create an unauthorized error (401).
    ///
    /// Used when authentication is required but not provided or invalid.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of the authentication failure
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::handler::extractor::ExtractionError;
    ///
    /// let error = ExtractionError::unauthorized("Invalid or missing token");
    /// ```
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(message, http::StatusCode::UNAUTHORIZED, "unauthorized")
    }

    /// Create an internal server error (500).
    ///
    /// Used when extraction fails due to server-side issues.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of the internal error
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::handler::extractor::ExtractionError;
    ///
    /// let error = ExtractionError::internal("Failed to deserialize state");
    /// ```
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            message,
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )
    }
}

impl IntoResponse for ExtractionError {
    fn into_response(self) -> Response {
        let error_body = serde_json::json!({
            "error": self.error_type,
            "message": self.message,
            "status": self.status.as_u16(),
        });

        Response::json(error_body).with_status(self.status)
    }
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExtractionError {}

/// Extension extractor implementation.
///
/// This allows extracting request extensions that were previously set by middleware
/// or other parts of the application.
impl<T> FromRequest for Extension<T>
where
    T: Send + Sync + Clone + 'static,
{
    type Error = ExtractionError;
    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
        req.get_extension::<T>()
            .map(|arc_value| Extension((*arc_value).clone()))
            .ok_or_else(|| {
                ExtractionError::internal(format!(
                    "Extension of type {} not found",
                    std::any::type_name::<T>()
                ))
            })
    }
}

/// Application state extractor that provides type-safe access to shared application data.
///
/// This extractor provides a semantic wrapper around `Extension<T>` specifically designed
/// for application state. The state must be added to the router using `.state()` methods.
///
/// # Requirements
///
/// The state type `T` must implement:
/// - `Clone` - For efficient sharing across handlers
/// - `Send + Sync` - For thread safety in async context
/// - `'static` - For lifetime requirements
///
/// # Example
///
/// ```
/// use ignitia::{State, Router, Response, Result};
/// use std::sync::Arc;
///
/// #[derive(Clone)]
/// struct AppState {
///     db_pool: Arc<DatabasePool>,
///     config: AppConfig,
/// }
///
/// // Handler using application state with destructuring
/// async fn get_users(State(state): State<AppState>) -> Result<Response> {
///     let users = state.db_pool.get_all_users().await?;
///     Response::ok().json(users)
/// }
///
/// // Alternative: without destructuring
/// async fn get_posts(state: State<AppState>) -> Result<Response> {
///     let posts = state.db_pool.get_all_posts().await?;
///     Response::ok().json(posts)
/// }
///
/// // Setting up the application
/// let app = Router::new()
///     .route("/users", get_users)
///     .route("/posts", get_posts)
///     .state(app_state);
/// ```
///
/// # Multiple State Types
///
/// ```
/// async fn api_handler(
///     State(app_state): State<AppState>,
///     State(metrics): State<MetricsCollector>,
/// ) -> Result<Response> {
///     metrics.increment_requests();
///     let data = app_state.db_pool.query("SELECT * FROM api_data").await?;
///     Response::ok().json(data)
/// }
/// ```
#[derive(Debug)]
pub struct State<T>(pub T);

impl<T> State<T> {
    /// Extract the inner value from the State wrapper.
    ///
    /// # Example
    ///
    /// ```
    /// # use ignitia::State;
    /// # let state = State("example".to_string());
    /// let inner = state.into_inner();
    /// println!("Inner: {}", inner);
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for State<T> {
    type Target = T;

    /// Allows direct access to the inner value.
    ///
    /// # Example
    ///
    /// ```
    /// # use ignitia::State;
    /// # let state = State("example".to_string());
    /// // Direct access without calling into_inner()
    /// println!("Value: {}", *state);
    /// ```
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = ExtractionError;
    /// Extract application state from the request.
    ///
    /// This delegates to the existing `Extension<T>` extractor internally,
    /// providing a semantic wrapper for application state.
    ///
    /// # Errors
    ///
    /// - `Error::Internal` - State type hasn't been added to the router
    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
        // Delegate to Extension<T> and wrap the result
        Extension::<T>::from_request(req).map(|Extension(inner)| State(inner))
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
        if req.params.is_empty() {
            return Err(ExtractionError::bad_request(
                "No path parameters found in request",
            ));
        }

        if req.params.len() == 1 {
            if let Some((_, value)) = req.params.iter().next() {
                // Try direct deserialization from string for types like Uuid
                if let Ok(extracted) =
                    serde_json::from_value(serde_json::Value::String(value.clone()))
                {
                    return Ok(Path(extracted));
                }
            }
        }

        let params_value = convert_string_map_to_json_value(&req.params);

        let extracted = T::deserialize(params_value).map_err(|e| {
            ExtractionError::bad_request(format!(
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
        let query_value = convert_string_map_to_json_value(&req.query_params);

        let extracted = T::deserialize(query_value).map_err(|e| {
            ExtractionError::bad_request(format!(
                "Failed to extract query parameters: {} (from query_params: {:?})",
                e, req.query_params
            ))
        })?;

        Ok(Query(extracted))
    }
}

/// JSON wrapper type for both extraction and response
///
/// # As an Extractor
///
/// Extract and deserialize JSON from request bodies:
///
/// ```
/// use ignitia::Json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// async fn create_user(Json(user): Json<CreateUser>) -> Json<User> {
///     // user is already deserialized
///     let created = save_user(user).await;
///     Json(created)
/// }
/// ```
///
/// # As a Response
///
/// Serialize types to JSON responses:
///
/// ```
/// use ignitia::Json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// async fn get_user() -> impl IntoResponse {
///     Json(User {
///         id: 1,
///         name: "Alice".to_string(),
///     })
/// }
/// ```
///
/// # Error Handling
///
/// - Returns 400 Bad Request if JSON parsing fails
/// - Returns 500 Internal Server Error if serialization fails
/// - Automatically sets `Content-Type: application/json` header
///
/// # Performance
///
/// - Zero-copy where possible using `Bytes`
/// - Minimal allocations during serialization
/// - Efficient error handling with detailed messages
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    /// Create a new JSON wrapper
    ///
    /// # Example
    ///
    /// ```
    /// let response = Json::new(User { id: 1, name: "Alice" });
    /// ```
    #[inline]
    pub fn new(value: T) -> Self {
        Json(value)
    }
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
    #[inline]
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
        match serde_json::from_slice(&req.body) {
            Ok(data) => Ok(Json(data)),
            Err(err) => Err(ExtractionError::bad_request(format!(
                "Invalid JSON: {}",
                err
            ))),
        }
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
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
    type Error = ExtractionError;

    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
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

/// Form data extractor for `application/x-www-form-urlencoded` content.
///
/// This extractor parses form-encoded request bodies into strongly-typed structs.
/// The target type must implement `serde::de::DeserializeOwned`.
///
/// # Content Type
///
/// This extractor expects the request to have a `Content-Type` header of
/// `application/x-www-form-urlencoded`. If the content type is missing or different,
/// it will return a `BadRequest` error.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use ignitia::{Form, Response, Result};
///
/// #[derive(Deserialize)]
/// struct LoginForm {
///     username: String,
///     password: String,
///     remember_me: Option<bool>,
/// }
///
/// async fn login(Form(form): Form<LoginForm>) -> Result<Response> {
///     println!("Username: {}", form.username);
///     println!("Password: {}", form.password);
///     println!("Remember me: {:?}", form.remember_me);
///
///     // Process login...
///     Response::ok().json("Login successful")
/// }
/// ```
///
/// # Form Data Format
///
/// The extractor parses standard form-encoded data:
/// ```
/// username=john_doe&password=secret123&remember_me=true
/// ```
///
/// # Nested Fields
///
/// Basic nested field support using dot notation:
/// ```
/// user.name=John&user.email=john@example.com&user.age=30
/// ```
///
/// # Error Handling
///
/// - Returns `Error::BadRequest` if Content-Type is not form data
/// - Returns `Error::BadRequest` if the body contains invalid UTF-8
/// - Returns `Error::BadRequest` if deserialization fails
///
/// # Performance
///
/// This extractor clones the request body for parsing. For large form data,
/// consider using streaming approaches or the `Body` extractor directly.
#[derive(Debug)]
pub struct Form<T>(pub T);

impl<T> Form<T> {
    /// Extract the inner value from the Form wrapper.
    ///
    /// # Example
    ///
    /// ```
    /// # use serde::Deserialize;
    /// # #[derive(Deserialize)]
    /// # struct MyForm { name: String }
    /// # let form = Form(MyForm { name: "test".to_string() });
    /// let inner = form.into_inner();
    /// println!("Name: {}", inner.name);
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Form<T> {
    type Target = T;

    /// Allows direct access to the inner value.
    ///
    /// # Example
    ///
    /// ```
    /// # use serde::Deserialize;
    /// # #[derive(Deserialize)]
    /// # struct MyForm { name: String }
    /// # let form = Form(MyForm { name: "test".to_string() });
    /// // Direct field access without calling into_inner()
    /// println!("Name: {}", form.name);
    /// ```
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for Form<T>
where
    T: serde::de::DeserializeOwned,
{
    type Error = ExtractionError;

    /// Extract form data from the request.
    ///
    /// # Errors
    ///
    /// - `Error::BadRequest` - Missing or invalid Content-Type header
    /// - `Error::BadRequest` - Body contains invalid UTF-8
    /// - `Error::BadRequest` - Form data deserialization failed
    fn from_request(req: &Request) -> std::result::Result<Self, Self::Error> {
        // Check Content-Type header
        let content_type = req
            .header("content-type")
            .ok_or_else(|| ExtractionError::bad_request("Missing Content-Type header"))?;

        if !content_type.starts_with("application/x-www-form-urlencoded") {
            return Err(ExtractionError::bad_request(
                "Expected 'application/x-www-form-urlencoded' content type",
            ));
        }

        // Convert body to UTF-8 string
        let body_str = String::from_utf8(req.body.to_vec())
            .map_err(|_| ExtractionError::bad_request("Request body contains invalid UTF-8"))?;

        // Parse form data using existing utility
        let form_data = parse_form_data(&body_str);

        // Convert to JSON value for serde deserialization
        let form_value = convert_string_map_to_json_value(&form_data);

        // Deserialize into target type
        let extracted = T::deserialize(form_value).map_err(|e| {
            ExtractionError::bad_request(format!(
                "Failed to deserialize form data: {} (from form: {:?})",
                e, form_data
            ))
        })?;

        Ok(Form(extracted))
    }
}

/// Parse URL-encoded form data into a HashMap.
///
/// This function handles standard form encoding with proper URL decoding.
///
/// # Format
///
/// Supports the standard `key=value&key2=value2` format with URL encoding.
///
/// # Examples
///
/// ```
/// let data = parse_form_data("name=John%20Doe&age=30&active=true");
/// assert_eq!(data.get("name"), Some(&"John Doe".to_string()));
/// assert_eq!(data.get("age"), Some(&"30".to_string()));
/// ```
///
/// # Nested Fields
///
/// Basic support for dot-notation nested fields:
/// ```
/// let data = parse_form_data("user.name=John&user.age=30");
/// // Results in: {"user.name": "John", "user.age": "30"}
/// ```
fn parse_form_data(body: &str) -> std::collections::HashMap<String, String> {
    let mut form_data = std::collections::HashMap::new();

    if body.is_empty() {
        return form_data;
    }

    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // URL decode key and value
            let decoded_key = urlencoding::decode(key)
                .unwrap_or_else(|_| std::borrow::Cow::Borrowed(key))
                .into_owned();

            let decoded_value = urlencoding::decode(value)
                .unwrap_or_else(|_| std::borrow::Cow::Borrowed(value))
                .into_owned();

            form_data.insert(decoded_key, decoded_value);
        } else {
            // Handle keys without values (e.g., "submit" button)
            let decoded_key = urlencoding::decode(pair)
                .unwrap_or_else(|_| std::borrow::Cow::Borrowed(pair))
                .into_owned();

            form_data.insert(decoded_key, String::new());
        }
    }

    form_data
}
