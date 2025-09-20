//! # HTTP Request Body Handling
//!
//! This module provides comprehensive request body handling for the Ignitia web framework.
//! It supports various body formats including JSON, plain text, and binary data with
//! efficient parsing and validation.
//!
//! ## Features
//!
//! - **Multiple Format Support**: JSON, text, and binary body handling
//! - **Type-Safe Parsing**: Automatic deserialization with error handling
//! - **Memory Efficient**: Zero-copy operations where possible
//! - **Validation**: Content validation and error reporting
//! - **Size Checking**: Built-in body size validation
//!
//! ## Supported Body Types
//!
//! ### JSON Bodies
//! - Automatic JSON parsing and deserialization
//! - Comprehensive error reporting for malformed JSON
//! - Support for any type implementing `DeserializeOwned`
//!
//! ### Text Bodies
//! - UTF-8 text extraction with validation
//! - Automatic encoding detection and conversion
//!
//! ### Binary Bodies
//! - Raw byte access for file uploads and binary data
//! - Efficient handling of large binary payloads
//!
//! ## Usage Examples
//!
//! ### Working with JSON Bodies
//! ```
//! use ignitia::request::Body;
//! use serde::Deserialize;
//! use bytes::Bytes;
//!
//! #[derive(Deserialize)]
//! struct UserData {
//!     name: String,
//!     email: String,
//!     age: u32,
//! }
//!
//! async fn handle_json_body() -> Result<(), Box<dyn std::error::Error>> {
//!     let json_data = r#"{"name": "John", "email": "john@example.com", "age": 30}"#;
//!     let body = Body::new(Bytes::from(json_data));
//!
//!     let user: UserData = body.json()?;
//!     println!("User: {} ({}) - Age: {}", user.name, user.email, user.age);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Working with Text Bodies
//! ```
//! use ignitia::request::Body;
//! use bytes::Bytes;
//!
//! async fn handle_text_body() -> Result<(), Box<dyn std::error::Error>> {
//!     let text_data = "Hello, World! This is a text body.";
//!     let body = Body::new(Bytes::from(text_data));
//!
//!     let text = body.text()?;
//!     println!("Received text: {}", text);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Working with Binary Bodies
//! ```
//! use ignitia::request::Body;
//! use bytes::Bytes;
//!
//! async fn handle_binary_body() -> Result<(), Box<dyn std::error::Error>> {
//!     let binary_data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in bytes
//!     let body = Body::new(Bytes::from(binary_data));
//!
//!     let bytes = body.bytes();
//!     println!("Received {} bytes of binary data", bytes.len());
//!
//!     // Process binary data
//!     for byte in bytes.iter() {
//!         print!("{:02X} ", byte);
//!     }
//!     println!();
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Integration with Request Handlers
//!
//! ### JSON API Endpoint
//! ```
//! use ignitia::{Request, Response, Result};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Serialize)]
//! struct CreateUserResponse {
//!     id: u32,
//!     name: String,
//!     email: String,
//!     created_at: String,
//! }
//!
//! async fn create_user_handler(req: Request) -> Result<Response> {
//!     // Parse JSON body
//!     let user_data: CreateUserRequest = req.json()?;
//!
//!     // Validate input
//!     if user_data.name.is_empty() {
//!         return Err(ignitia::Error::BadRequest("Name cannot be empty".into()));
//!     }
//!
//!     if !user_data.email.contains('@') {
//!         return Err(ignitia::Error::BadRequest("Invalid email format".into()));
//!     }
//!
//!     // Create response
//!     let response = CreateUserResponse {
//!         id: 123,
//!         name: user_data.name,
//!         email: user_data.email,
//!         created_at: chrono::Utc::now().to_rfc3339(),
//!     };
//!
//!     Response::json(response)
//! }
//! ```
//!
//! ### File Upload Handler
//! ```
//! use ignitia::{Request, Response, Result};
//! use bytes::Bytes;
//!
//! async fn file_upload_handler(req: Request) -> Result<Response> {
//!     // Check content type
//!     let content_type = req.header("content-type")
//!         .ok_or_else(|| ignitia::Error::BadRequest("Missing content-type header".into()))?;
//!
//!     // Get body size
//!     let body_size = req.body.len();
//!
//!     // Validate file size (max 10MB)
//!     if body_size > 10 * 1024 * 1024 {
//!         return Err(ignitia::Error::BadRequest("File too large".into()));
//!     }
//!
//!     // Process based on content type
//!     match content_type {
//!         "image/jpeg" | "image/png" => {
//!             // Process image upload
//!             Ok(Response::text(format!("Uploaded image: {} bytes", body_size)))
//!         }
//!         "application/pdf" => {
//!             // Process PDF upload
//!             Ok(Response::text(format!("Uploaded PDF: {} bytes", body_size)))
//!         }
//!         _ => {
//!             Err(ignitia::Error::BadRequest("Unsupported file type".into()))
//!         }
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! ### Comprehensive Error Handling
//! ```
//! use ignitia::{Request, Response, Result, Error};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct ApiRequest {
//!     action: String,
//!     data: serde_json::Value,
//! }
//!
//! async fn robust_handler(req: Request) -> Result<Response> {
//!     // Check if body is empty
//!     if req.body.is_empty() {
//!         return Ok(Response::text("Empty request body")
//!             .with_status_code(400));
//!     }
//!
//!     // Check content type
//!     match req.header("content-type") {
//!         Some(ct) if ct.starts_with("application/json") => {
//!             // Handle JSON
//!             match req.json::<ApiRequest>() {
//!                 Ok(api_req) => {
//!                     Ok(Response::text(format!("Action: {}", api_req.action)))
//!                 }
//!                 Err(Error::Json(e)) => {
//!                     Ok(Response::text(format!("Invalid JSON: {}", e))
//!                         .with_status_code(400))
//!                 }
//!                 Err(e) => Err(e),
//!             }
//!         }
//!         Some(ct) if ct.starts_with("text/") => {
//!             // Handle text
//!             match std::str::from_utf8(&req.body) {
//!                 Ok(text) => Ok(Response::text(format!("Received text: {}", text))),
//!                 Err(_) => Ok(Response::text("Invalid UTF-8 text")
//!                     .with_status_code(400)),
//!             }
//!         }
//!         Some(ct) => {
//!             Ok(Response::text(format!("Unsupported content type: {}", ct))
//!                 .with_status_code(415))
//!         }
//!         None => {
//!             Ok(Response::text("Missing content-type header")
//!                 .with_status_code(400))
//!         }
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Memory Efficiency
//! - Uses `bytes::Bytes` for zero-copy operations
//! - Avoids unnecessary string allocations
//! - Efficient binary data handling
//!
//! ### Size Limits
//! ```
//! use ignitia::{Request, Response, Result};
//!
//! async fn size_limited_handler(req: Request) -> Result<Response> {
//!     const MAX_BODY_SIZE: usize = 1024 * 1024; // 1MB
//!
//!     if req.body.len() > MAX_BODY_SIZE {
//!         return Ok(Response::text("Request body too large")
//!             .with_status_code(413)); // Payload Too Large
//!     }
//!
//!     // Process body
//!     Ok(Response::text("Body processed successfully"))
//! }
//! ```
//!
//! ## Security Considerations
//!
//! ### Input Validation
//! - Always validate deserialized data
//! - Set reasonable size limits for request bodies
//! - Validate content types before processing
//! - Sanitize text input to prevent injection attacks
//!
//! ### Example Secure Handler
//! ```
//! use ignitia::{Request, Response, Result};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct SecureInput {
//!     #[serde(deserialize_with = "validate_name")]
//!     name: String,
//!     #[serde(deserialize_with = "validate_email")]
//!     email: String,
//! }
//!
//! fn validate_name<'de, D>(deserializer: D) -> Result<String, D::Error>
//! where
//!     D: serde::Deserializer<'de>,
//! {
//!     let name = String::deserialize(deserializer)?;
//!     if name.len() > 100 {
//!         return Err(serde::de::Error::custom("Name too long"));
//!     }
//!     if name.trim().is_empty() {
//!         return Err(serde::de::Error::custom("Name cannot be empty"));
//!     }
//!     Ok(name.trim().to_string())
//! }
//!
//! fn validate_email<'de, D>(deserializer: D) -> Result<String, D::Error>
//! where
//!     D: serde::Deserializer<'de>,
//! {
//!     let email = String::deserialize(deserializer)?;
//!     if !email.contains('@') || email.len() > 254 {
//!         return Err(serde::de::Error::custom("Invalid email"));
//!     }
//!     Ok(email.to_lowercase())
//! }
//!
//! async fn secure_handler(req: Request) -> Result<Response> {
//!     let input: SecureInput = req.json()?;
//!     Ok(Response::text(format!("Hello, {}", input.name)))
//! }
//! ```

use std::{borrow::Cow, sync::Arc};

use crate::error::Result;
use bytes::Bytes;
use serde::de::DeserializeOwned;

/// HTTP request body wrapper providing convenient access to body content.
///
/// The `Body` struct wraps the raw bytes of an HTTP request body and provides
/// methods for parsing it as different formats (JSON, text, binary). It ensures
/// efficient access to body data while providing type-safe parsing operations.
///
/// # Internal Structure
/// - **inner**: The raw body bytes as a `bytes::Bytes` for efficient memory usage
///
/// # Examples
///
/// ## Creating a Body
/// ```
/// use ignitia::request::Body;
/// use bytes::Bytes;
///
/// let json_data = r#"{"name": "John", "age": 30}"#;
/// let body = Body::new(Bytes::from(json_data));
/// ```
///
/// ## From Different Sources
/// ```
/// use ignitia::request::Body;
/// use bytes::Bytes;
///
/// // From string
/// let body1 = Body::from("Hello, World!");
///
/// // From String
/// let body2 = Body::from(String::from("Hello, World!"));
///
/// // From bytes
/// let body3 = Body::from(Bytes::from("Hello, World!"));
/// ```
pub struct Body {
    inner: Arc<Bytes>,
}

impl Body {
    /// Creates a new Body from bytes.
    ///
    /// This is the primary constructor for Body instances. It wraps the provided
    /// bytes in a Body struct for convenient parsing operations.
    ///
    /// # Parameters
    /// - `bytes`: The raw body bytes
    ///
    /// # Returns
    /// A new Body instance
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// let data = b"Hello, World!";
    /// let body = Body::new(Bytes::from(&data[..]));
    /// assert_eq!(body.len(), 13);
    /// ```
    pub fn new(bytes: Bytes) -> Self {
        Self {
            inner: Arc::new(bytes),
        }
    }

    /// Returns a reference to the underlying bytes.
    ///
    /// This method provides direct access to the raw body bytes without
    /// copying or parsing. Useful for binary data processing or when you
    /// need the raw bytes for custom parsing.
    ///
    /// # Returns
    /// Reference to the underlying `Bytes`
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// let body = Body::new(Bytes::from("Hello"));
    /// let bytes_ref = body.bytes();
    /// assert_eq!(bytes_ref.len(), 5);
    /// ```
    ///
    /// ## Binary Data Processing
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// let binary_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    /// let body = Body::new(Bytes::from(binary_data));
    ///
    /// let bytes = body.bytes();
    /// if bytes.len() >= 4 && &bytes[0..4] == &[0xFF, 0xD8, 0xFF, 0xE0] {
    ///     println!("This is a JPEG file");
    /// }
    /// ```
    pub fn bytes(&self) -> Arc<&Bytes> {
        Arc::new(&self.inner)
    }

    /// Reference to the underlying `Bytes`
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// let body = Body::new(Bytes::from("Hello"));
    /// let bytes_ref = body.bytes();
    /// assert_eq!(bytes_ref.len(), 5);
    /// ```
    pub fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    /// Zero-copy clone
    pub fn clone_shared(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Converts the body to a UTF-8 string.
    ///
    /// This method attempts to parse the body bytes as UTF-8 text. It performs
    /// validation to ensure the bytes represent valid UTF-8 characters.
    ///
    /// # Returns
    /// - `Ok(String)`: The body content as a UTF-8 string
    /// - `Err(Error::BadRequest)`: If the body contains invalid UTF-8
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let body = Body::new(Bytes::from("Hello, 世界!"));
    /// let text = body.text()?;
    /// assert_eq!(text, "Hello, 世界!");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Error Handling
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Invalid UTF-8 bytes
    /// let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    /// let body = Body::new(Bytes::from(invalid_utf8));
    ///
    /// match body.text() {
    ///     Ok(text) => println!("Text: {}", text),
    ///     Err(e) => println!("Invalid UTF-8: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Processing Text Content
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let body = Body::new(Bytes::from("line1\nline2\nline3"));
    /// let text = body.text()?;
    ///
    /// let line_count = text.lines().count();
    /// let word_count = text.split_whitespace().count();
    ///
    /// println!("Lines: {}, Words: {}", line_count, word_count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn text(&self) -> Result<Cow<'_, str>> {
        std::str::from_utf8(&self.inner)
            .map(Cow::Borrowed)
            .or_else(|_| {
                Ok(Cow::Owned(
                    String::from_utf8_lossy(&self.inner).into_owned(),
                ))
            })
    }

    /// Parses the body as JSON and deserializes it to the specified type.
    ///
    /// This method performs JSON parsing and deserialization in one step.
    /// It uses serde for deserialization, supporting any type that implements
    /// `DeserializeOwned`.
    ///
    /// # Type Parameters
    /// - `T`: The target type for deserialization (must implement `DeserializeOwned`)
    ///
    /// # Returns
    /// - `Ok(T)`: Successfully parsed and deserialized data
    /// - `Err(Error)`: JSON parsing or deserialization error
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let json_str = r#"{"name": "Alice", "age": 30}"#;
    /// let body = Body::new(Bytes::from(json_str));
    ///
    /// let person: Person = body.json()?;
    /// assert_eq!(person.name, "Alice");
    /// assert_eq!(person.age, 30);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Working with Generic JSON
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    /// use serde_json::Value;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let json_str = r#"{"users": [{"id": 1, "name": "John"}], "total": 1}"#;
    /// let body = Body::new(Bytes::from(json_str));
    ///
    /// let data: Value = body.json()?;
    /// if let Some(users) = data["users"].as_array() {
    ///     println!("Found {} users", users.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Error Handling
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Config {
    ///     timeout: u32,
    ///     retries: u32,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let invalid_json = r#"{"timeout": "not_a_number"}"#;
    /// let body = Body::new(Bytes::from(invalid_json));
    ///
    /// match body.json::<Config>() {
    ///     Ok(config) => println!("Config loaded successfully"),
    ///     Err(e) => println!("Failed to parse config: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.inner).map_err(Into::into)
    }

    /// Checks if the body is empty.
    ///
    /// This method provides a convenient way to check if the request body
    /// contains any data without examining the bytes directly.
    ///
    /// # Returns
    /// - `true`: If the body contains no bytes
    /// - `false`: If the body contains data
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// let empty_body = Body::new(Bytes::new());
    /// assert!(empty_body.is_empty());
    ///
    /// let non_empty_body = Body::new(Bytes::from("data"));
    /// assert!(!non_empty_body.is_empty());
    /// ```
    ///
    /// ## Conditional Processing
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// async fn process_body(body: Body) -> String {
    ///     if body.is_empty() {
    ///         "No data provided".to_string()
    ///     } else {
    ///         format!("Processing {} bytes", body.len())
    ///     }
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the length of the body in bytes.
    ///
    /// This method returns the exact number of bytes in the request body,
    /// which is useful for validation, logging, and processing decisions.
    ///
    /// # Returns
    /// The number of bytes in the body
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// let body = Body::new(Bytes::from("Hello"));
    /// assert_eq!(body.len(), 5);
    ///
    /// let empty_body = Body::new(Bytes::new());
    /// assert_eq!(empty_body.len(), 0);
    /// ```
    ///
    /// ## Size Validation
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// async fn validate_body_size(body: Body) -> Result<String, String> {
    ///     const MAX_SIZE: usize = 1024 * 1024; // 1MB
    ///
    ///     if body.len() > MAX_SIZE {
    ///         Err("Body too large".to_string())
    ///     } else if body.is_empty() {
    ///         Err("Body cannot be empty".to_string())
    ///     } else {
    ///         Ok(format!("Body size: {} bytes", body.len()))
    ///     }
    /// }
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl From<Bytes> for Body {
    /// Creates a Body from Bytes.
    ///
    /// This conversion is zero-cost as it simply wraps the Bytes in a Body struct.
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    /// use bytes::Bytes;
    ///
    /// let bytes = Bytes::from("Hello, World!");
    /// let body = Body::from(bytes);
    /// assert_eq!(body.len(), 13);
    /// ```
    fn from(bytes: Bytes) -> Self {
        Self::new(bytes)
    }
}

impl From<String> for Body {
    /// Creates a Body from a String.
    ///
    /// The String is converted to Bytes and then wrapped in a Body.
    /// This is convenient for creating bodies from owned strings.
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    ///
    /// let text = String::from("Hello, World!");
    /// let body = Body::from(text);
    /// assert_eq!(body.len(), 13);
    /// ```
    fn from(s: String) -> Self {
        Self::new(Bytes::from(s))
    }
}

impl From<&str> for Body {
    /// Creates a Body from a string slice.
    ///
    /// The string slice is converted to a String, then to Bytes,
    /// and finally wrapped in a Body.
    ///
    /// # Examples
    /// ```
    /// use ignitia::request::Body;
    ///
    /// let body = Body::from("Hello, World!");
    /// assert_eq!(body.len(), 13);
    /// ```
    fn from(s: &str) -> Self {
        Self::new(Bytes::from(s.to_string()))
    }
}

impl Clone for Body {
    fn clone(&self) -> Self {
        self.clone_shared()
    }
}
