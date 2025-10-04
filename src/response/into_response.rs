//! Type conversions for HTTP responses
//!
//! This module provides the `IntoResponse` trait and its implementations,
//! which allow various types to be automatically converted into HTTP responses.
//! This enables handlers to return different types without manually constructing
//! `Response` objects every time.
//!
//! # Design Philosophy
//!
//! The trait is designed for:
//! - **Zero-copy where possible**: Uses `Bytes::from_static()` for static data
//! - **Minimal allocations**: Pre-allocated headers and optimized serialization
//! - **Developer ergonomics**: Rich set of implementations for common types
//! - **Type safety**: Compile-time guarantees for response conversions
//!
//! # Examples
//!
//! ## Simple String Response
//!
//! ```
//! async fn handler() -> &'static str {
//!     "Hello, World!" // Zero-copy static string
//! }
//! ```
//!
//! ## JSON Response
//!
//! ```
//! use ignitia::Json;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//!
//! async fn get_user() -> Json<User> {
//!     Json(User { id: 1, name: "Alice".to_string() })
//! }
//! ```
//!
//! ## Error Handling
//!
//! ```
//! async fn handler() -> Result<String, Error> {
//!     Ok("Success".to_string()) // Both Ok and Err convert to Response
//! }
//! ```

use bytes::Bytes;
use http::{HeaderValue, StatusCode};
use serde::Serialize;

use crate::{Json, Response};

/// Trait for converting a type into an HTTP response.
///
/// This trait enables flexible return types from handlers. Any type implementing `IntoResponse`
/// can be returned from a handler function, and the framework will automatically convert it
/// to a [`Response`].
///
/// # Type Conversion
///
/// The conversion is performed automatically by the handler system when a handler returns.
/// This allows for cleaner handler signatures and better type safety.
///
/// # Implementing IntoResponse
///
/// To make a custom type returnable from handlers, implement this trait:
///
/// ```
/// use ignitia::Response;
/// use ignitia::response::IntoResponse;
/// use http::StatusCode;
///
/// struct CustomResponse {
///     status: u16,
///     body: String,
/// }
///
/// impl IntoResponse for CustomResponse {
///     fn into_response(self) -> Response {
///         Response::new(
///             StatusCode::from_u16(self.status).unwrap_or(StatusCode::OK)
///         )
///         .with_body(self.body)
///     }
/// }
/// ```
///
/// # Examples
///
/// ## Basic Usage in Handlers
///
/// ```
/// use ignitia::prelude::*;
///
/// // Handler returning &str
/// async fn text_handler() -> &'static str {
///     "Plain text response"
/// }
///
/// // Handler returning Response
/// async fn json_handler() -> Response {
///     Response::json(json!({"key": "value"}))
/// }
///
/// // Handler returning StatusCode
/// async fn status_handler() -> StatusCode {
///     StatusCode::ACCEPTED
/// }
/// ```
///
/// ## Error Handling with Results
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn fallible_handler() -> Result<String> {
///     if some_condition() {
///         Ok("Success!".to_string())
///     } else {
///         Err(Error::bad_request("Invalid input"))
///     }
/// }
/// ```
pub trait IntoResponse {
    /// Convert this type into an HTTP response.
    ///
    /// This method consumes `self` and returns a [`Response`]. Implementations should
    /// handle the conversion appropriately, setting status codes, headers, and body
    /// content as needed.
    ///
    /// # Returns
    ///
    /// Returns a [`Response`] ready to be sent to the client.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::Response;
    /// use ignitia::response::IntoResponse;
    ///
    /// let response = "Hello".to_string().into_response();
    /// assert_eq!(response.status, http::StatusCode::OK);
    /// ```
    fn into_response(self) -> Response;
}

/// Implementation for [`Response`] (pass-through).
///
/// Since [`Response`] is already the target type, this implementation simply returns
/// itself without any conversion.
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn handler() -> Response {
///     Response::json(json!({"status": "ok"}))
/// }
/// ```
impl IntoResponse for Response {
    #[inline(always)]
    fn into_response(self) -> Response {
        self
    }
}

/// Implementation for [`String`] (owned string).
///
/// Converts an owned string into a plain text response with `Content-Type: text/plain; charset=utf-8`
/// and status code 200 OK.
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn handler() -> String {
///     format!("Hello, {}!", "World")
/// }
///
/// async fn dynamic_handler() -> String {
///     let name = get_user_name().await;
///     format!("Welcome, {}", name)
/// }
/// ```
impl IntoResponse for String {
    #[inline]
    fn into_response(self) -> Response {
        Response::text(self)
    }
}

/// Implementation for [`&'static str`] (static string slice).
///
/// Converts a static string slice into a plain text response with `Content-Type: text/plain; charset=utf-8`
/// and status code 200 OK.
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn handler() -> &'static str {
///     "Hello, World!"
/// }
///
/// async fn status_handler() -> &'static str {
///     "Server is running"
/// }
/// ```
impl IntoResponse for &'static str {
    #[inline]
    fn into_response(self) -> Response {
        Response::text(self)
    }
}

/// Convert a Cow (Clone-on-Write) string into a text/plain response
///
/// Optimizes for both static and owned strings using Cow semantics:
/// - `Cow::Borrowed`: Zero-copy for static strings
/// - `Cow::Owned`: Efficient conversion for owned strings
///
/// # Example
///
/// ```
/// use std::borrow::Cow;
///
/// async fn handler(dynamic: bool) -> Cow<'static, str> {
///     if dynamic {
///         Cow::Owned(format!("Dynamic: {}", value))
///     } else {
///         Cow::Borrowed("Static message")
///     }
/// }
/// ```
impl IntoResponse for std::borrow::Cow<'static, str> {
    #[inline]
    fn into_response(self) -> Response {
        let mut response = Response::new(StatusCode::OK);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.body = match self {
            std::borrow::Cow::Borrowed(s) => Bytes::from_static(s.as_bytes()),
            std::borrow::Cow::Owned(s) => Bytes::from(s),
        };
        response
    }
}

// ========== Bytes types - zero-copy ==========

/// Convert Bytes into an application/octet-stream response
///
/// Bytes is already an efficient, reference-counted buffer type.
/// Cloning is cheap (just a refcount bump).
///
/// # Example
///
/// ```
/// async fn handler() -> Bytes {
///     Bytes::from(vec![0x01, 0x02, 0x03])
/// }
/// ```
impl IntoResponse for Bytes {
    #[inline]
    fn into_response(self) -> Response {
        let mut response = Response::new(StatusCode::OK);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        response.body = self;
        response
    }
}

/// Convert a Vec<u8> into an application/octet-stream response
///
/// The vector is consumed and converted to Bytes efficiently.
///
/// # Example
///
/// ```
/// async fn handler() -> Vec<u8> {
///     vec![0x89, 0x50, 0x4E, 0x47] // PNG magic bytes
/// }
/// ```
impl IntoResponse for Vec<u8> {
    #[inline]
    fn into_response(self) -> Response {
        let mut response = Response::new(StatusCode::OK);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        response.body = Bytes::from(self);
        response
    }
}

/// Convert a static byte slice into an application/octet-stream response
///
/// **Zero-copy**: Uses `Bytes::from_static` for static byte arrays.
///
/// # Example
///
/// ```
/// async fn handler() -> &'static [u8] {
///     b"Binary data"
/// }
/// ```
impl IntoResponse for &'static [u8] {
    #[inline]
    fn into_response(self) -> Response {
        let mut response = Response::new(StatusCode::OK);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        response.body = Bytes::from_static(self); // Zero-copy!
        response
    }
}

// ========== Option types ==========

/// Convert Option<T> into a response
///
/// Automatically handles presence/absence:
/// - `Some(T)`: Converts T to response
/// - `None`: Returns 404 Not Found
///
/// Perfect for database queries that may not find a record.
///
/// # Example
///
/// ```
/// async fn find_user(id: u64) -> Option<Json<User>> {
///     database.find_user(id).await.map(Json)
/// }
/// ```
///
/// # Response
///
/// - `Some(value)`: HTTP 200 with value
/// - `None`: HTTP 404 with "Not Found" body
impl<T> IntoResponse for Option<T>
where
    T: IntoResponse,
{
    #[inline]
    fn into_response(self) -> Response {
        match self {
            Some(val) => val.into_response(),
            None => {
                Response::new(StatusCode::NOT_FOUND).with_body(Bytes::from_static(b"Not Found"))
            }
        }
    }
}

// ========== Tuple types (status + body) ==========

/// Convert (StatusCode, T) into a response with custom status
///
/// Allows overriding the status code of any IntoResponse type.
///
/// # Example
///
/// ```
/// async fn create() -> (StatusCode, Json<User>) {
///     (StatusCode::CREATED, Json(new_user))
/// }
/// ```
impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    #[inline]
    fn into_response(self) -> Response {
        let mut response = self.1.into_response();
        response.status = self.0;
        response
    }
}

/// Convert (StatusCode, &'static str, T) into a response
///
/// Allows setting both status code and Content-Type header.
///
/// # Example
///
/// ```
/// async fn handler() -> (StatusCode, &'static str, String) {
///     (StatusCode::OK, "text/plain", "Hello".to_string())
/// }
/// ```
impl<T> IntoResponse for (StatusCode, &'static str, T)
where
    T: IntoResponse,
{
    #[inline]
    fn into_response(self) -> Response {
        let mut response = self.2.into_response();
        response.status = self.0;
        if let Ok(value) = HeaderValue::from_str(self.1) {
            response.headers.insert(http::header::CONTENT_TYPE, value);
        }
        response
    }
}

// ========== JSON wrapper - optimized serialization ==========

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    #[inline]
    fn into_response(self) -> Response {
        Response::json(self.0)
    }
}

// ========== HTML wrapper ==========

/// HTML response wrapper
///
/// Wraps any type that can be converted to a String and sets the
/// Content-Type to text/html.
///
/// # Example
///
/// ```
/// async fn handler() -> Html<String> {
///     Html("<h1>Hello</h1>".to_string())
/// }
/// ```
#[derive(Debug)]
pub struct Html<T>(pub T);

/// Convert Html<T> into an HTML response
///
/// Sets Content-Type to text/html; charset=utf-8 and converts the
/// inner value to bytes.
///
/// # Example
///
/// ```
/// Html("<html><body>Hello</body></html>")
/// ```
impl<T> IntoResponse for Html<T>
where
    T: Into<String>,
{
    #[inline]
    fn into_response(self) -> Response {
        Response::html(self.0.into())
    }
}

// ========== Empty/Unit type ==========

/// Convert () (unit type) into an empty 200 OK response
///
/// Useful for handlers that perform actions but don't need to return data.
///
/// # Example
///
/// ```
/// async fn delete_user(id: u64) -> Result<()> {
///     database.delete(id).await?;
///     Ok(())
/// }
/// ```
impl IntoResponse for () {
    #[inline]
    fn into_response(self) -> Response {
        Response::ok()
    }
}

// ========== Boolean convenience ==========

/// Convert bool into a JSON boolean response
///
/// Returns `true` or `false` as valid JSON with application/json content type.
///
/// # Example
///
/// ```
/// async fn is_healthy() -> bool {
///     true
/// }
/// ```
///
/// # Response
///
/// - `true`: `{"result": true}` or just `true`
/// - `false`: `{"result": false}` or just `false`
impl IntoResponse for bool {
    #[inline]
    fn into_response(self) -> Response {
        let mut response = Response::new(StatusCode::OK);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response.body = if self {
            Bytes::from_static(b"true")
        } else {
            Bytes::from_static(b"false")
        };
        response
    }
}

// ========== Numeric types ==========

/// Implements IntoResponse for all numeric primitive types
///
/// Numbers are converted to their string representation and returned
/// as text/plain responses.
///
/// # Supported Types
///
/// - Signed integers: i8, i16, i32, i64, i128, isize
/// - Unsigned integers: u8, u16, u32, u64, u128, usize
/// - Floats: f32, f64
///
/// # Example
///
/// ```
/// async fn count_users() -> i32 {
///     42
/// }
/// ```
///
/// # Response
///
/// Returns "42" with Content-Type: text/plain; charset=utf-8
macro_rules! impl_into_response_for_number {
    ($($t:ty),+ $(,)?) => {
        $(
            #[doc = concat!("Convert `", stringify!($t), "` to a text/plain response")]
            impl IntoResponse for $t {
                #[inline]
                fn into_response(self) -> Response {
                    let body = self.to_string();
                    let mut response = Response::new(StatusCode::OK);
                    response.headers.insert(
                        http::header::CONTENT_TYPE,
                        HeaderValue::from_static("text/plain; charset=utf-8"),
                    );
                    response.body = Bytes::from(body);
                    response
                }
            }
        )+
    };
}

// Generate implementations for all numeric types
impl_into_response_for_number!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

/// Implementation for [`http::StatusCode`].
///
/// Converts a status code into an empty response with that status code. This is useful
/// for endpoints that only need to communicate status without a response body.
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
///
/// // Return 204 No Content
/// async fn delete_handler() -> StatusCode {
///     // Perform deletion...
///     StatusCode::NO_CONTENT
/// }
///
/// // Return 202 Accepted
/// async fn async_task_handler() -> StatusCode {
///     // Queue task for processing...
///     StatusCode::ACCEPTED
/// }
///
/// // Return 201 Created
/// async fn create_handler() -> StatusCode {
///     // Create resource...
///     StatusCode::CREATED
/// }
/// ```
impl IntoResponse for http::StatusCode {
    #[inline]
    fn into_response(self) -> Response {
        Response::new(self)
    }
}

/// Implementation for [`Result<T, E>`] where both `T` and `E` implement [`IntoResponse`].
///
/// This is the "magic" implementation that enables automatic error handling in handlers.
/// When a handler returns `Result<T, E>`, both the success value (`T`) and error value (`E`)
/// are automatically converted to responses.
///
/// This means:
/// - On `Ok(value)`, `value.into_response()` is called
/// - On `Err(error)`, `error.into_response()` is called
///
/// Since [`Error`] implements [`IntoResponse`], errors are automatically converted to
/// properly formatted error responses with appropriate status codes and JSON bodies.
///
/// # Type Parameters
///
/// * `T` - The success type, must implement [`IntoResponse`]
/// * `E` - The error type, must implement [`IntoResponse`]
///
/// # Examples
///
/// ## Basic Result Handling
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn handler() -> Result<Response> {
///     if is_valid() {
///         Ok(Response::json(json!({"status": "success"})))
///     } else {
///         Err(Error::bad_request("Invalid input"))
///     }
/// }
/// ```
///
/// ## Returning Different Success Types
///
/// ```
/// use ignitia::prelude::*;
///
/// // Return Result<String>
/// async fn text_handler() -> Result<String> {
///     validate_request()?;
///     Ok("Success".to_string())
/// }
///
/// // Return Result<StatusCode>
/// async fn status_handler() -> Result<StatusCode> {
///     process_data()?;
///     Ok(StatusCode::NO_CONTENT)
/// }
/// ```
///
/// ## Error Propagation with ?
///
/// ```
/// use ignitia::prelude::*;
///
/// async fn complex_handler() -> Result<Response> {
///     // Errors are automatically propagated
///     let user = get_user().await?;
///     let data = process_data(&user).await?;
///     validate(&data)?;
///
///     Ok(Response::json(data))
/// }
/// ```
///
/// ## Custom Error Types
///
/// ```
/// use ignitia::prelude::*;
/// use ignitia::response::IntoResponse;
///
/// enum MyError {
///     NotFound,
///     Unauthorized,
/// }
///
/// impl IntoResponse for MyError {
///     fn into_response(self) -> Response {
///         match self {
///             MyError::NotFound => {
///                 Response::text("Not found").with_status(StatusCode::NOT_FOUND)
///             }
///             MyError::Unauthorized => {
///                 Response::text("Unauthorized").with_status(StatusCode::UNAUTHORIZED)
///             }
///         }
///     }
/// }
///
/// async fn handler() -> Result<String, MyError> {
///     if !is_authenticated() {
///         return Err(MyError::Unauthorized);
///     }
///     Ok("Success".to_string())
/// }
/// ```
impl<T, E> IntoResponse for std::result::Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
