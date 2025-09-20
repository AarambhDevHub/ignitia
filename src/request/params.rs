//! # HTTP Request Parameter Handling
//!
//! This module provides comprehensive parameter handling for HTTP requests in the Ignitia
//! web framework. It supports both path parameters (from route patterns) and query
//! parameters (from URL query strings) with type-safe parsing and validation.
//!
//! ## Features
//!
//! - **Type-Safe Parsing**: Automatic conversion to Rust types with error handling
//! - **Path Parameters**: Extract parameters from route patterns like `/users/:id`
//! - **Query Parameters**: Parse URL query strings with multiple value support
//! - **Validation**: Built-in validation and error reporting
//! - **Performance**: Efficient HashMap-based storage and lookup
//!
//! ## Parameter Types
//!
//! ### Path Parameters
//! Extracted from route patterns using colons (e.g., `/users/:id/:action`)
//! - Always strings that can be parsed to other types
//! - Guaranteed to exist when route matches
//! - Case-sensitive matching
//!
//! ### Query Parameters
//! Parsed from URL query strings (e.g., `?page=1&limit=10&active=true`)
//! - Optional parameters with default values
//! - Support for multiple data types
//! - URL-decoded automatically
//!
//! ## Usage Examples
//!
//! ### Basic Parameter Access
//! ```
//! use ignitia::request::Params;
//! use std::collections::HashMap;
//!
//! // Create params from a HashMap
//! let mut param_map = HashMap::new();
//! param_map.insert("id".to_string(), "123".to_string());
//! param_map.insert("action".to_string(), "edit".to_string());
//!
//! let params = Params::from(param_map);
//!
//! // Access parameters
//! assert_eq!(params.get("id"), Some(&"123".to_string()));
//! assert_eq!(params.get("action"), Some(&"edit".to_string()));
//! assert_eq!(params.get("missing"), None);
//! ```
//!
//! ### Type-Safe Parameter Parsing
//! ```
//! use ignitia::request::Params;
//! use std::collections::HashMap;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut param_map = HashMap::new();
//! param_map.insert("user_id".to_string(), "42".to_string());
//! param_map.insert("page".to_string(), "5".to_string());
//! param_map.insert("active".to_string(), "true".to_string());
//!
//! let params = Params::from(param_map);
//!
//! // Parse to specific types
//! let user_id: u32 = params.get_parsed("user_id")?;
//! let page: i32 = params.get_parsed("page")?;
//! let active: bool = params.get_parsed("active")?;
//!
//! assert_eq!(user_id, 42);
//! assert_eq!(page, 5);
//! assert_eq!(active, true);
//! # Ok(())
//! # }
//! ```
//!
//! ## Integration with Request Handlers
//!
//! ### RESTful API Handler
//! ```
//! use ignitia::{Request, Response, Result};
//!
//! // Route: GET /users/:id
//! async fn get_user_handler(req: Request) -> Result<Response> {
//!     // Extract user ID from path parameters
//!     let user_id: u32 = req.param("id")
//!         .ok_or_else(|| ignitia::Error::BadRequest("Missing user ID".into()))?
//!         .parse()
//!         .map_err(|_| ignitia::Error::BadRequest("Invalid user ID format".into()))?;
//!
//!     // Simulate database lookup
//!     if user_id == 0 {
//!         return Err(ignitia::Error::BadRequest("User ID cannot be zero".into()));
//!     }
//!
//!     Ok(Response::json(serde_json::json!({
//!         "id": user_id,
//!         "name": "John Doe",
//!         "email": "john@example.com"
//!     }))?)
//! }
//! ```
//!
//! ### Pagination Handler with Query Parameters
//! ```
//! use ignitia::{Request, Response, Result};
//! use serde_json::json;
//!
//! // Route: GET /users?page=1&limit=10&sort=name
//! async fn list_users_handler(req: Request) -> Result<Response> {
//!     // Parse query parameters with defaults
//!     let page: u32 = req.query("page")
//!         .and_then(|p| p.parse().ok())
//!         .unwrap_or(1);
//!
//!     let limit: u32 = req.query("limit")
//!         .and_then(|l| l.parse().ok())
//!         .unwrap_or(10);
//!
//!     let sort = req.query("sort")
//!         .unwrap_or("id");
//!
//!     // Validate parameters
//!     if page == 0 {
//!         return Err(ignitia::Error::BadRequest("Page must be greater than 0".into()));
//!     }
//!
//!     if limit > 100 {
//!         return Err(ignitia::Error::BadRequest("Limit cannot exceed 100".into()));
//!     }
//!
//!     let response = json!({
//!         "users": [],
//!         "pagination": {
//!             "page": page,
//!             "limit": limit,
//!             "total": 0
//!         },
//!         "sort": sort
//!     });
//!
//!     Response::json(response)
//! }
//! ```
//!
//! ### Search Handler with Multiple Parameters
//! ```
//! use ignitia::{Request, Response, Result};
//! use serde_json::json;
//!
//! // Route: GET /search?q=rust&category=programming&min_price=10&max_price=100
//! async fn search_handler(req: Request) -> Result<Response> {
//!     // Required parameter
//!     let query = req.query("q")
//!         .ok_or_else(|| ignitia::Error::BadRequest("Search query is required".into()))?;
//!
//!     // Optional parameters
//!     let category = req.query("category");
//!     let min_price: Option<f32> = req.query("min_price")
//!         .and_then(|p| p.parse().ok());
//!     let max_price: Option<f32> = req.query("max_price")
//!         .and_then(|p| p.parse().ok());
//!
//!     // Validate price range
//!     if let (Some(min), Some(max)) = (min_price, max_price) {
//!         if min > max {
//!             return Err(ignitia::Error::BadRequest(
//!                 "Min price cannot be greater than max price".into()
//!             ));
//!         }
//!     }
//!
//!     let response = json!({
//!         "query": query,
//!         "category": category,
//!         "price_range": {
//!             "min": min_price,
//!             "max": max_price
//!         },
//!         "results": []
//!     });
//!
//!     Response::json(response)
//! }
//! ```
//!
//! ## Advanced Parameter Handling
//!
//! ### Custom Parameter Validation
//! ```
//! use ignitia::{Request, Response, Result, Error};
//!
//! // Helper function for validating UUIDs
//! fn validate_uuid(param: &str) -> Result<String> {
//!     if param.len() == 36 && param.chars().filter(|&c| c == '-').count() == 4 {
//!         Ok(param.to_string())
//!     } else {
//!         Err(Error::BadRequest("Invalid UUID format".into()))
//!     }
//! }
//!
//! // Route: GET /api/v1/orders/:order_id
//! async fn get_order_handler(req: Request) -> Result<Response> {
//!     let order_id = req.param("order_id")
//!         .ok_or_else(|| Error::BadRequest("Missing order ID".into()))?;
//!
//!     let validated_id = validate_uuid(order_id)?;
//!
//!     Ok(Response::json(serde_json::json!({
//!         "order_id": validated_id,
//!         "status": "completed"
//!     }))?)
//! }
//! ```
//!
//! ### Parameter Conversion with Error Handling
//! ```
//! use ignitia::{Request, Response, Result, Error};
//! use std::str::FromStr;
//!
//! // Generic parameter parser with better error messages
//! fn parse_param<T: FromStr>(req: &Request, name: &str) -> Result<T>
//! where
//!     T::Err: std::fmt::Display,
//! {
//!     req.param(name)
//!         .ok_or_else(|| Error::BadRequest(format!("Missing parameter: {}", name)))?
//!         .parse()
//!         .map_err(|e| Error::BadRequest(format!("Invalid {}: {}", name, e)))
//! }
//!
//! // Route: GET /users/:id/posts/:post_id
//! async fn get_user_post_handler(req: Request) -> Result<Response> {
//!     let user_id: u32 = parse_param(&req, "id")?;
//!     let post_id: u32 = parse_param(&req, "post_id")?;
//!
//!     Ok(Response::json(serde_json::json!({
//!         "user_id": user_id,
//!         "post_id": post_id,
//!         "title": "Sample Post"
//!     }))?)
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Efficient Parameter Access
//! - Parameters are stored in HashMap for O(1) lookup
//! - String parsing is done on-demand to avoid unnecessary conversions
//! - Pre-allocated capacity reduces memory allocations
//!
//! ### Caching Parsed Values
//! ```
//! use ignitia::{Request, Response, Result};
//! use std::collections::HashMap;
//!
//! async fn optimized_handler(req: Request) -> Result<Response> {
//!     // Parse commonly used parameters once
//!     let page: u32 = req.query("page")
//!         .and_then(|p| p.parse().ok())
//!         .unwrap_or(1);
//!
//!     let limit: u32 = req.query("limit")
//!         .and_then(|l| l.parse().ok())
//!         .unwrap_or(10);
//!
//!     // Use parsed values multiple times without re-parsing
//!     let offset = (page - 1) * limit;
//!     let end = offset + limit;
//!
//!     Ok(Response::text(format!(
//!         "Showing items {} to {} (page {} of {} items each)",
//!         offset + 1, end, page, limit
//!     )))
//! }
//! ```
//!
//! ## Security Considerations
//!
//! ### Input Validation
//! - Always validate parameter values before using in business logic
//! - Set reasonable bounds on numeric parameters
//! - Sanitize string parameters to prevent injection attacks
//! - Use allowlists for enum-like parameters
//!
//! ### Example Secure Parameter Handling
//! ```
//! use ignitia::{Request, Response, Result, Error};
//!
//! // Secure parameter validation
//! async fn secure_handler(req: Request) -> Result<Response> {
//!     // Validate user ID
//!     let user_id: u32 = req.param("id")
//!         .ok_or_else(|| Error::BadRequest("Missing user ID".into()))?
//!         .parse()
//!         .map_err(|_| Error::BadRequest("Invalid user ID".into()))?;
//!
//!     if user_id == 0 || user_id > 1_000_000 {
//!         return Err(Error::BadRequest("User ID out of valid range".into()));
//!     }
//!
//!     // Validate sort parameter with allowlist
//!     let sort = req.query("sort").unwrap_or("id");
//!     let allowed_sorts = ["id", "name", "created_at", "updated_at"];
//!     if !allowed_sorts.contains(&sort) {
//!         return Err(Error::BadRequest("Invalid sort parameter".into()));
//!     }
//!
//!     // Validate and sanitize search query
//!     if let Some(query) = req.query("q") {
//!         if query.len() > 100 {
//!             return Err(Error::BadRequest("Search query too long".into()));
//!         }
//!
//!         // Remove potentially dangerous characters
//!         let sanitized_query = query
//!             .chars()
//!             .filter(|&c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_')
//!             .collect::<String>();
//!
//!         if sanitized_query.trim().is_empty() {
//!             return Err(Error::BadRequest("Invalid search query".into()));
//!         }
//!     }
//!
//!     Ok(Response::text("Parameters validated successfully"))
//! }
//! ```
//!
//! ## Testing Parameter Handling
//!
//! ### Unit Tests
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use std::collections::HashMap;
//!
//!     #[test]
//!     fn test_params_creation() {
//!         let mut map = HashMap::new();
//!         map.insert("id".to_string(), "123".to_string());
//!         map.insert("name".to_string(), "test".to_string());
//!
//!         let params = Params::from(map);
//!
//!         assert_eq!(params.get("id"), Some(&"123".to_string()));
//!         assert_eq!(params.get("name"), Some(&"test".to_string()));
//!         assert_eq!(params.get("missing"), None);
//!     }
//!
//!     #[test]
//!     fn test_type_conversion() {
//!         let mut map = HashMap::new();
//!         map.insert("number".to_string(), "42".to_string());
//!         map.insert("flag".to_string(), "true".to_string());
//!
//!         let params = Params::from(map);
//!
//!         assert_eq!(params.get_parsed::<u32>("number").unwrap(), 42);
//!         assert_eq!(params.get_parsed::<bool>("flag").unwrap(), true);
//!     }
//!
//!     #[test]
//!     fn test_error_handling() {
//!         let params = Params::new();
//!
//!         // Test missing parameter
//!         assert!(params.get_parsed::<u32>("missing").is_err());
//!
//!         // Test invalid conversion
//!         let mut map = HashMap::new();
//!         map.insert("invalid".to_string(), "not_a_number".to_string());
//!         let params = Params::from(map);
//!
//!         assert!(params.get_parsed::<u32>("invalid").is_err());
//!     }
//! }
//! ```

use crate::error::{Error, Result};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

/// HTTP request parameter container with type-safe access methods.
///
/// The `Params` struct provides a convenient interface for accessing and parsing
/// HTTP request parameters, whether they come from path parameters (route patterns)
/// or query parameters (URL query strings). It supports automatic type conversion
/// with comprehensive error handling.
///
/// # Internal Structure
/// - **inner**: HashMap storing parameter key-value pairs as strings
///
/// # Features
/// - Type-safe parameter parsing with `FromStr` trait
/// - Comprehensive error reporting for missing or invalid parameters
/// - Efficient HashMap-based storage for fast lookups
/// - Iterator support for processing all parameters
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::request::Params;
/// use std::collections::HashMap;
///
/// let mut param_map = HashMap::new();
/// param_map.insert("id".to_string(), "123".to_string());
/// param_map.insert("active".to_string(), "true".to_string());
///
/// let params = Params::from(param_map);
///
/// assert_eq!(params.get("id"), Some(&"123".to_string()));
/// assert!(params.contains_key("active"));
/// ```
///
/// ## Type Conversion
/// ```
/// use ignitia::request::Params;
/// use std::collections::HashMap;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut param_map = HashMap::new();
/// param_map.insert("count".to_string(), "42".to_string());
/// param_map.insert("rate".to_string(), "3.14".to_string());
///
/// let params = Params::from(param_map);
///
/// let count: u32 = params.get_parsed("count")?;
/// let rate: f64 = params.get_parsed("rate")?;
///
/// assert_eq!(count, 42);
/// assert_eq!(rate, 3.14);
/// # Ok(())
/// # }
/// ```
pub struct Params {
    inner: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl Params {
    /// Creates a new empty Params container.
    ///
    /// This constructor creates an empty parameter container that can be
    /// populated manually or used as a default when no parameters are present.
    ///
    /// # Returns
    /// A new empty Params instance
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    ///
    /// let params = Params::new();
    /// assert_eq!(params.len(), 0);
    /// assert!(params.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Inserts a parameter key-value pair.
    ///
    /// This method adds a new parameter or updates an existing one.
    /// Both key and value are stored as strings.
    ///
    /// # Parameters
    /// - `key`: Parameter name
    /// - `value`: Parameter value
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    ///
    /// let mut params = Params::new();
    /// params.insert("id".to_string(), "123".to_string());
    /// params.insert("action".to_string(), "edit".to_string());
    ///
    /// assert_eq!(params.get("id"), Some(&"123".to_string()));
    /// assert_eq!(params.len(), 2);
    /// ```
    ///
    /// ## Building Parameters Programmatically
    /// ```
    /// use ignitia::request::Params;
    ///
    /// let mut params = Params::new();
    ///
    /// // Add parameters from various sources
    /// params.insert("user_id".to_string(), "456".to_string());
    /// params.insert("timestamp".to_string(), "1640995200".to_string());
    /// params.insert("format".to_string(), "json".to_string());
    ///
    /// assert_eq!(params.len(), 3);
    /// ```
    pub fn insert(&mut self, key: &'static str, value: &'static str) {
        self.inner.insert(Cow::Borrowed(key), Cow::Borrowed(value));
    }

    /// Gets a parameter value by key.
    ///
    /// This method returns a reference to the parameter value if it exists.
    /// The value is always returned as a string and must be parsed if a
    /// different type is needed.
    ///
    /// # Parameters
    /// - `key`: The parameter name to look up
    ///
    /// # Returns
    /// - `Some(&String)`: The parameter value if found
    /// - `None`: If the parameter doesn't exist
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let mut param_map = HashMap::new();
    /// param_map.insert("username".to_string(), "alice".to_string());
    /// param_map.insert("role".to_string(), "admin".to_string());
    ///
    /// let params = Params::from(param_map);
    ///
    /// assert_eq!(params.get("username"), Some(&"alice".to_string()));
    /// assert_eq!(params.get("role"), Some(&"admin".to_string()));
    /// assert_eq!(params.get("missing"), None);
    /// ```
    ///
    /// ## Safe Access Pattern
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let params = Params::from(HashMap::new());
    ///
    /// // Safe access with default values
    /// let page = params.get("page").unwrap_or(&"1".to_string());
    /// let limit = params.get("limit").unwrap_or(&"10".to_string());
    ///
    /// println!("Page: {}, Limit: {}", page, limit);
    /// ```
    pub fn get(&self, key: &'static str) -> Option<&Cow<str>> {
        self.inner.get(key)
    }

    /// Gets and parses a parameter value to the specified type.
    ///
    /// This method retrieves a parameter value and attempts to parse it
    /// to the target type using the `FromStr` trait. It provides comprehensive
    /// error handling for both missing parameters and parsing failures.
    ///
    /// # Type Parameters
    /// - `T`: The target type (must implement `FromStr`)
    ///
    /// # Parameters
    /// - `key`: The parameter name to parse
    ///
    /// # Returns
    /// - `Ok(T)`: Successfully parsed value
    /// - `Err(Error::BadRequest)`: Missing parameter or parsing error
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut param_map = HashMap::new();
    /// param_map.insert("id".to_string(), "42".to_string());
    /// param_map.insert("price".to_string(), "19.99".to_string());
    /// param_map.insert("active".to_string(), "true".to_string());
    ///
    /// let params = Params::from(param_map);
    ///
    /// let id: u32 = params.get_parsed("id")?;
    /// let price: f64 = params.get_parsed("price")?;
    /// let active: bool = params.get_parsed("active")?;
    ///
    /// assert_eq!(id, 42);
    /// assert_eq!(price, 19.99);
    /// assert_eq!(active, true);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Error Handling
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let mut param_map = HashMap::new();
    /// param_map.insert("invalid_number".to_string(), "not_a_number".to_string());
    ///
    /// let params = Params::from(param_map);
    ///
    /// // Missing parameter error
    /// match params.get_parsed::<u32>("missing") {
    ///     Err(e) => println!("Expected error: {}", e),
    ///     _ => panic!("Should have failed"),
    /// }
    ///
    /// // Parsing error
    /// match params.get_parsed::<u32>("invalid_number") {
    ///     Err(e) => println!("Expected parsing error: {}", e),
    ///     _ => panic!("Should have failed"),
    /// }
    /// ```
    ///
    /// ## Custom Types
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    /// use std::str::FromStr;
    ///
    /// // Custom enum that implements FromStr
    /// #[derive(Debug, PartialEq)]
    /// enum Status {
    ///     Active,
    ///     Inactive,
    ///     Pending,
    /// }
    ///
    /// impl FromStr for Status {
    ///     type Err = String;
    ///
    ///     fn from_str(s: &str) -> Result<Self, Self::Err> {
    ///         match s.to_lowercase().as_str() {
    ///             "active" => Ok(Status::Active),
    ///             "inactive" => Ok(Status::Inactive),
    ///             "pending" => Ok(Status::Pending),
    ///             _ => Err(format!("Invalid status: {}", s)),
    ///         }
    ///     }
    /// }
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut param_map = HashMap::new();
    /// param_map.insert("status".to_string(), "active".to_string());
    ///
    /// let params = Params::from(param_map);
    /// let status: Status = params.get_parsed("status")?;
    ///
    /// assert_eq!(status, Status::Active);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_parsed<T: FromStr>(&self, key: &str) -> Result<T> {
        self.inner
            .get(key)
            .ok_or_else(|| Error::BadRequest(format!("Missing parameter: {}", key)))?
            .parse()
            .map_err(|_| Error::BadRequest(format!("Invalid parameter format: {}", key)))
    }

    /// Checks if a parameter with the given key exists.
    ///
    /// This method provides a quick way to check for parameter existence
    /// without retrieving the value.
    ///
    /// # Parameters
    /// - `key`: The parameter name to check
    ///
    /// # Returns
    /// - `true`: If the parameter exists
    /// - `false`: If the parameter doesn't exist
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let mut param_map = HashMap::new();
    /// param_map.insert("filter".to_string(), "active".to_string());
    ///
    /// let params = Params::from(param_map);
    ///
    /// assert!(params.contains_key("filter"));
    /// assert!(!params.contains_key("missing"));
    /// ```
    ///
    /// ## Conditional Processing
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// fn process_request(params: &Params) -> String {
    ///     let mut result = String::new();
    ///
    ///     if params.contains_key("debug") {
    ///         result.push_str("Debug mode enabled. ");
    ///     }
    ///
    ///     if params.contains_key("verbose") {
    ///         result.push_str("Verbose output enabled. ");
    ///     }
    ///
    ///     if result.is_empty() {
    ///         result.push_str("Standard processing.");
    ///     }
    ///
    ///     result
    /// }
    /// ```
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Returns an iterator over all parameter key-value pairs.
    ///
    /// This method provides access to all parameters for batch processing,
    /// validation, or debugging purposes.
    ///
    /// # Returns
    /// Iterator over `(&String, &String)` pairs
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let mut param_map = HashMap::new();
    /// param_map.insert("name".to_string(), "Alice".to_string());
    /// param_map.insert("age".to_string(), "30".to_string());
    /// param_map.insert("city".to_string(), "New York".to_string());
    ///
    /// let params = Params::from(param_map);
    ///
    /// for (key, value) in params.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    ///
    /// ## Parameter Validation
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// fn validate_all_params(params: &Params) -> Vec<String> {
    ///     let mut errors = Vec::new();
    ///
    ///     for (key, value) in params.iter() {
    ///         if value.is_empty() {
    ///             errors.push(format!("Parameter '{}' cannot be empty", key));
    ///         }
    ///
    ///         if value.len() > 100 {
    ///             errors.push(format!("Parameter '{}' is too long", key));
    ///         }
    ///     }
    ///
    ///     errors
    /// }
    /// ```
    ///
    /// ## Debug Information
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// fn debug_params(params: &Params) -> String {
    ///     let mut debug_info = String::from("Parameters:\n");
    ///
    ///     for (key, value) in params.iter() {
    ///         debug_info.push_str(&format!("  {}: {} ({} chars)\n",
    ///             key, value, value.len()));
    ///     }
    ///
    ///     debug_info
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&Cow<str>, &Cow<str>)> {
        self.inner.iter()
    }

    /// Returns the number of parameters.
    ///
    /// This method returns the total count of parameters in the container.
    ///
    /// # Returns
    /// The number of parameters
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let mut param_map = HashMap::new();
    /// param_map.insert("a".to_string(), "1".to_string());
    /// param_map.insert("b".to_string(), "2".to_string());
    ///
    /// let params = Params::from(param_map);
    /// assert_eq!(params.len(), 2);
    ///
    /// let empty_params = Params::new();
    /// assert_eq!(empty_params.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Checks if the parameter container is empty.
    ///
    /// This method returns `true` if no parameters are present.
    ///
    /// # Returns
    /// - `true`: If no parameters exist
    /// - `false`: If at least one parameter exists
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let empty_params = Params::new();
    /// assert!(empty_params.is_empty());
    ///
    /// let mut param_map = HashMap::new();
    /// param_map.insert("key".to_string(), "value".to_string());
    /// let params = Params::from(param_map);
    /// assert!(!params.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl From<HashMap<Cow<'static, str>, Cow<'static, str>>> for Params {
    /// Creates a Params instance from a HashMap.
    ///
    /// This conversion is commonly used when creating Params from parsed
    /// query strings or route parameters.
    ///
    /// # Parameters
    /// - `map`: HashMap containing parameter key-value pairs
    ///
    /// # Returns
    /// A new Params instance containing all the HashMap entries
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("user_id".to_string(), "123".to_string());
    /// map.insert("session".to_string(), "abc123".to_string());
    ///
    /// let params = Params::from(map);
    ///
    /// assert_eq!(params.get("user_id"), Some(&"123".to_string()));
    /// assert_eq!(params.get("session"), Some(&"abc123".to_string()));
    /// ```
    fn from(map: HashMap<Cow<'static, str>, Cow<'static, str>>) -> Self {
        Self { inner: map }
    }
}

impl Default for Params {
    /// Creates an empty Params instance.
    ///
    /// This is equivalent to calling `Params::new()`.
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Params;
    ///
    /// let params = Params::default();
    /// assert!(params.is_empty());
    /// assert_eq!(params.len(), 0);
    /// ```
    fn default() -> Self {
        Self::new()
    }
}
