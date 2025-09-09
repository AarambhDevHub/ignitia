//! # Extension System Module
//!
//! This module provides a type-safe extension system for the Ignitia web framework.
//! It allows storing and retrieving arbitrary typed data in requests, responses,
//! and other framework components using a type map pattern.
//!
//! ## Features
//!
//! - **Type Safety**: All extensions are stored and retrieved with full type safety
//! - **Thread Safety**: Extensions can be safely shared across threads
//! - **Zero-Cost Abstractions**: Minimal runtime overhead for type storage and retrieval
//! - **Flexible Storage**: Store any type that implements `Send + Sync + 'static`
//! - **Handler Integration**: Seamless integration with request handlers via extractors
//! - **Memory Efficient**: Automatic cleanup when extensions are dropped
//!
//! ## Common Use Cases
//!
//! - **User Authentication**: Store user information for the request lifecycle
//! - **Database Connections**: Share database pools or connections
//! - **Request Context**: Store request-specific metadata and state
//! - **Middleware Data**: Pass data between middleware layers
//! - **Custom Application State**: Store application-specific data structures
//!
//! ## Quick Start
//!
//! ### Basic Usage
//!
//! ```
//! use ignitia::{Extensions, Extension};
//!
//! #[derive(Clone)]
//! struct UserId(u64);
//!
//! #[derive(Clone)]
//! struct UserRole(String);
//!
//! let mut extensions = Extensions::new();
//!
//! // Insert typed data
//! extensions.insert(UserId(12345));
//! extensions.insert(UserRole("admin".to_string()));
//!
//! // Retrieve typed data
//! if let Some(user_id) = extensions.get::<UserId>() {
//!     println!("User ID: {}", user_id.0);
//! }
//! ```
//!
//! ### With Request Handlers
//!
//! ```
//! use ignitia::{Request, Response, Extension, Result};
//!
//! #[derive(Clone)]
//! struct DatabasePool(/* your pool type */);
//!
//! async fn handler(Extension(db_pool): Extension<DatabasePool>) -> Result<Response> {
//!     // Use the database pool
//!     // let connection = db_pool.get_connection().await?;
//!     Ok(Response::text("Data retrieved"))
//! }
//! ```
//!
//! ### In Middleware
//!
//! ```
//! use ignitia::{Request, Middleware, Result};
//!
//! #[derive(Clone)]
//! struct RequestId(String);
//!
//! struct RequestIdMiddleware;
//!
//! #[async_trait::async_trait]
//! impl Middleware for RequestIdMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         let request_id = RequestId(uuid::Uuid::new_v4().to_string());
//!         req.insert_extension(request_id);
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Type Requirements
//!
//! All types stored as extensions must implement:
//! - `Send + Sync`: For thread safety
//! - `'static`: For stable type identification
//! - `Clone`: For extraction from shared references (when using `get()`)
//!
//! ## Performance Considerations
//!
//! - Extension lookup is O(1) using `TypeId` as the key
//! - Cloning occurs when extracting values (consider using `Arc<T>` for expensive-to-clone types)
//! - Memory usage scales linearly with the number of unique types stored

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A thread-safe type map for storing arbitrary extensions.
///
/// `Extensions` provides a way to store heterogeneous data in a type-safe manner.
/// It uses Rust's type system to ensure that data can only be retrieved with
/// the correct type, preventing runtime type errors.
///
/// ## Thread Safety
///
/// This type is thread-safe and can be safely shared across threads. All operations
/// use appropriate locking to ensure data consistency.
///
/// ## Type Storage
///
/// Extensions are stored using their `TypeId` as the key, which means:
/// - Each type can only have one instance stored at a time
/// - Generic types with different parameters are considered different types
/// - Type aliases refer to the same underlying type
///
/// ## Examples
///
/// ### Basic Storage and Retrieval
/// ```
/// use ignitia::Extensions;
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct UserId(u64);
///
/// #[derive(Clone, Debug)]
/// struct UserName(String);
///
/// let mut extensions = Extensions::new();
///
/// // Store different types
/// extensions.insert(UserId(42));
/// extensions.insert(UserName("Alice".to_string()));
///
/// // Retrieve with type safety
/// let user_id = extensions.get::<UserId>().unwrap();
/// assert_eq!(user_id.0, 42);
///
/// let user_name = extensions.get::<UserName>().unwrap();
/// assert_eq!(user_name.0, "Alice");
/// ```
///
/// ### Working with Optional Data
/// ```
/// use ignitia::Extensions;
///
/// #[derive(Clone)]
/// struct OptionalData(String);
///
/// let extensions = Extensions::new();
///
/// // Safe handling of missing data
/// match extensions.get::<OptionalData>() {
///     Some(data) => println!("Found: {}", data.0),
///     None => println!("Data not found"),
/// }
/// ```
///
/// ### Using Arc for Expensive Types
/// ```
/// use ignitia::Extensions;
/// use std::sync::Arc;
///
/// #[derive(Debug)]
/// struct ExpensiveData {
///     large_vector: Vec<u8>,
/// }
///
/// let mut extensions = Extensions::new();
///
/// // Store as Arc to avoid expensive clones
/// let expensive_data = Arc::new(ExpensiveData {
///     large_vector: vec![0; 1000000],
/// });
/// extensions.insert(expensive_data);
///
/// // Retrieval returns Arc<Arc<ExpensiveData>>, clone is cheap
/// if let Some(data) = extensions.get::<Arc<ExpensiveData>>() {
///     println!("Data length: {}", data.large_vector.len());
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Extensions {
    map: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl Extensions {
    /// Creates a new empty Extensions map.
    ///
    /// The map starts with no stored extensions and allocates memory as needed.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// let extensions = Extensions::new();
    /// assert!(extensions.is_empty());
    /// assert_eq!(extensions.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inserts a value into the extensions map.
    ///
    /// If a value of the same type was already present, it will be replaced
    /// and the old value will be dropped.
    ///
    /// # Type Requirements
    /// - `T: Send + Sync + 'static`: The type must be thread-safe and have static lifetime
    ///
    /// # Parameters
    /// - `value`: The value to store in the extensions map
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Config {
    ///     debug: bool,
    ///     timeout: u64,
    /// }
    ///
    /// let mut extensions = Extensions::new();
    /// let config = Config { debug: true, timeout: 30 };
    ///
    /// extensions.insert(config.clone());
    /// assert_eq!(extensions.len(), 1);
    ///
    /// // Inserting the same type replaces the old value
    /// let new_config = Config { debug: false, timeout: 60 };
    /// extensions.insert(new_config.clone());
    /// assert_eq!(extensions.len(), 1);
    ///
    /// let retrieved = extensions.get::<Config>().unwrap();
    /// assert_eq!(retrieved.timeout, 60);
    /// ```
    ///
    /// # Thread Safety
    /// This method acquires a write lock on the internal map, so it may block
    /// if other threads are currently reading or writing.
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        let mut map = self.map.write().unwrap();
        map.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Gets a reference to a value from the extensions map.
    ///
    /// This method returns an `Arc<T>` for shared ownership of the retrieved value.
    /// The value is cloned from the stored instance, so the original type must
    /// implement `Clone`.
    ///
    /// # Type Requirements
    /// - `T: Send + Sync + 'static + Clone`: The type must be thread-safe, have static lifetime, and be cloneable
    ///
    /// # Returns
    /// - `Some(Arc<T>)` if a value of type T exists
    /// - `None` if no value of type T is stored
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct DatabaseUrl(String);
    ///
    /// let mut extensions = Extensions::new();
    /// let db_url = DatabaseUrl("postgresql://localhost/mydb".to_string());
    /// extensions.insert(db_url);
    ///
    /// // Successful retrieval
    /// let retrieved = extensions.get::<DatabaseUrl>().unwrap();
    /// assert_eq!(retrieved.0, "postgresql://localhost/mydb");
    ///
    /// // Type safety - different type returns None
    /// #[derive(Clone)]
    /// struct ApiKey(String);
    /// assert!(extensions.get::<ApiKey>().is_none());
    /// ```
    ///
    /// # Performance
    /// This method acquires a read lock and performs a hash map lookup by `TypeId`.
    /// The lookup is O(1) on average, but the clone operation depends on the
    /// complexity of the stored type.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>>
    where
        T: Clone,
    {
        let map = self.map.read().unwrap();
        map.get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .map(|value| Arc::new(value.clone()))
    }

    /// Removes a value from the extensions map.
    ///
    /// This method takes ownership of the stored value and returns it if it exists.
    /// After removal, subsequent calls to `get()` or `contains()` for this type
    /// will return `None`/`false` until a new value is inserted.
    ///
    /// # Type Requirements
    /// - `T: Send + Sync + 'static`: The type must be thread-safe and have static lifetime
    ///
    /// # Returns
    /// - `Some(T)` if a value of type T existed and was removed
    /// - `None` if no value of type T was stored
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct SessionId(String);
    ///
    /// let mut extensions = Extensions::new();
    /// let session_id = SessionId("sess_123".to_string());
    /// extensions.insert(session_id.clone());
    ///
    /// assert!(extensions.contains::<SessionId>());
    ///
    /// // Remove the value
    /// let removed = extensions.remove::<SessionId>().unwrap();
    /// assert_eq!(removed.0, "sess_123");
    ///
    /// // Value is no longer present
    /// assert!(!extensions.contains::<SessionId>());
    /// assert!(extensions.get::<SessionId>().is_none());
    /// ```
    ///
    /// # Thread Safety
    /// This method acquires a write lock on the internal map, so it may block
    /// if other threads are currently reading or writing.
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        let mut map = self.map.write().unwrap();
        map.remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok())
            .map(|boxed| *boxed)
    }

    /// Checks if the extensions map contains a value of the specified type.
    ///
    /// This is a more efficient alternative to `get().is_some()` when you only
    /// need to check for presence without retrieving the value.
    ///
    /// # Type Requirements
    /// - `T: Send + Sync + 'static`: The type must be thread-safe and have static lifetime
    ///
    /// # Returns
    /// - `true` if a value of type T is stored
    /// - `false` if no value of type T is stored
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// #[derive(Clone)]
    /// struct Feature1Enabled(bool);
    ///
    /// #[derive(Clone)]
    /// struct Feature2Enabled(bool);
    ///
    /// let mut extensions = Extensions::new();
    /// extensions.insert(Feature1Enabled(true));
    ///
    /// assert!(extensions.contains::<Feature1Enabled>());
    /// assert!(!extensions.contains::<Feature2Enabled>());
    ///
    /// // Conditional logic based on presence
    /// if extensions.contains::<Feature1Enabled>() {
    ///     println!("Feature 1 is configured");
    /// }
    /// ```
    ///
    /// # Performance
    /// This method only requires a read lock and performs a simple hash map
    /// key existence check, making it more efficient than `get()` when you
    /// don't need the actual value.
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        let map = self.map.read().unwrap();
        map.contains_key(&TypeId::of::<T>())
    }

    /// Returns the number of extensions stored in the map.
    ///
    /// This counts the number of different types stored, not the total
    /// memory usage or the number of values within each type.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// #[derive(Clone)]
    /// struct TypeA(i32);
    ///
    /// #[derive(Clone)]
    /// struct TypeB(String);
    ///
    /// let mut extensions = Extensions::new();
    /// assert_eq!(extensions.len(), 0);
    ///
    /// extensions.insert(TypeA(42));
    /// assert_eq!(extensions.len(), 1);
    ///
    /// extensions.insert(TypeB("hello".to_string()));
    /// assert_eq!(extensions.len(), 2);
    ///
    /// // Inserting the same type again doesn't increase count
    /// extensions.insert(TypeA(100));
    /// assert_eq!(extensions.len(), 2);
    /// ```
    ///
    /// # Performance
    /// This operation requires a read lock and calls `HashMap::len()`,
    /// which is O(1).
    pub fn len(&self) -> usize {
        let map = self.map.read().unwrap();
        map.len()
    }

    /// Checks if the extensions map is empty.
    ///
    /// Returns `true` if no extensions are stored, `false` otherwise.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// #[derive(Clone)]
    /// struct SomeData(String);
    ///
    /// let mut extensions = Extensions::new();
    /// assert!(extensions.is_empty());
    ///
    /// extensions.insert(SomeData("test".to_string()));
    /// assert!(!extensions.is_empty());
    ///
    /// extensions.remove::<SomeData>();
    /// assert!(extensions.is_empty());
    /// ```
    ///
    /// # Performance
    /// This is equivalent to `self.len() == 0` but may be slightly more
    /// efficient depending on the `HashMap` implementation.
    pub fn is_empty(&self) -> bool {
        let map = self.map.read().unwrap();
        map.is_empty()
    }

    /// Clears all extensions from the map.
    ///
    /// After calling this method, the map will be empty and all stored
    /// extensions will be dropped.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extensions;
    ///
    /// #[derive(Clone)]
    /// struct Data1(i32);
    ///
    /// #[derive(Clone)]
    /// struct Data2(String);
    ///
    /// let mut extensions = Extensions::new();
    /// extensions.insert(Data1(42));
    /// extensions.insert(Data2("hello".to_string()));
    ///
    /// assert_eq!(extensions.len(), 2);
    ///
    /// extensions.clear();
    /// assert!(extensions.is_empty());
    /// assert!(!extensions.contains::<Data1>());
    /// assert!(!extensions.contains::<Data2>());
    /// ```
    ///
    /// # Thread Safety
    /// This method acquires a write lock on the internal map. All stored
    /// values will be dropped while holding the lock.
    pub fn clear(&mut self) {
        let mut map = self.map.write().unwrap();
        map.clear();
    }
}

/// A wrapper for extension values that can be extracted in handlers.
///
/// `Extension<T>` is used as an extractor in request handlers to automatically
/// retrieve typed data from the request's extension map. It implements the
/// `FromRequest` trait, allowing it to be used as a handler parameter.
///
/// ## Usage in Handlers
///
/// The `Extension<T>` extractor automatically looks up the specified type in
/// the request's extensions and provides it to your handler. If the extension
/// doesn't exist, the extraction will fail with an error.
///
/// ## Examples
///
/// ### Basic Handler Usage
/// ```
/// use ignitia::{Extension, Response, Result};
///
/// #[derive(Clone)]
/// struct DatabasePool {
///     // Your database pool implementation
/// }
///
/// #[derive(Clone)]
/// struct UserId(u64);
///
/// async fn get_user_profile(
///     Extension(db_pool): Extension<DatabasePool>,
///     Extension(user_id): Extension<UserId>,
/// ) -> Result<Response> {
///     // Use the extracted extensions
///     println!("Getting profile for user {} using database pool", user_id.0);
///
///     Ok(Response::json(serde_json::json!({
///         "user_id": user_id.0,
///         "profile": "user profile data"
///     }))?)
/// }
/// ```
///
/// ### Multiple Extensions
/// ```
/// use ignitia::{Extension, Response, Result};
///
/// #[derive(Clone)]
/// struct RequestId(String);
///
/// #[derive(Clone)]
/// struct UserPermissions(Vec<String>);
///
/// #[derive(Clone)]
/// struct AppConfig {
///     debug_mode: bool,
/// }
///
/// async fn protected_handler(
///     Extension(request_id): Extension<RequestId>,
///     Extension(permissions): Extension<UserPermissions>,
///     Extension(config): Extension<AppConfig>,
/// ) -> Result<Response> {
///     if config.debug_mode {
///         println!("Request {} with permissions: {:?}", request_id.0, permissions.0);
///     }
///
///     if permissions.0.contains(&"admin".to_string()) {
///         Ok(Response::text("Admin access granted"))
///     } else {
///         Ok(Response::text("Regular user access"))
///     }
/// }
/// ```
///
/// ### Error Handling
/// ```
/// use ignitia::{Extension, Response, Result, Error};
///
/// #[derive(Clone)]
/// struct OptionalFeature(String);
///
/// async fn feature_handler(
///     // This will return an error if OptionalFeature is not present
///     Extension(feature): Extension<OptionalFeature>,
/// ) -> Result<Response> {
///     Ok(Response::text(format!("Feature: {}", feature.0)))
/// }
/// ```
///
/// ## Setting Extensions in Middleware
///
/// Extensions are typically set by middleware before reaching handlers:
///
/// ```
/// use ignitia::{Middleware, Request, Result};
///
/// #[derive(Clone)]
/// struct ProcessingTime(std::time::Instant);
///
/// struct TimingMiddleware;
///
/// #[async_trait::async_trait]
/// impl Middleware for TimingMiddleware {
///     async fn before(&self, req: &mut Request) -> Result<()> {
///         req.insert_extension(ProcessingTime(std::time::Instant::now()));
///         Ok(())
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Extension<T>(pub T);

impl<T> Extension<T> {
    /// Unwraps the extension, consuming the wrapper and returning the inner value.
    ///
    /// This is useful when you want to take ownership of the extracted value
    /// rather than working with it through the wrapper.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extension;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct UserId(u64);
    ///
    /// let extension = Extension(UserId(42));
    /// let user_id = extension.into_inner();
    /// assert_eq!(user_id.0, 42);
    /// ```
    ///
    /// ### In Handler Context
    /// ```
    /// use ignitia::{Extension, Response, Result};
    ///
    /// #[derive(Clone)]
    /// struct Config {
    ///     api_key: String,
    ///     timeout: u64,
    /// }
    ///
    /// async fn handler(Extension(config): Extension<Config>) -> Result<Response> {
    ///     // config is directly the Config struct, not Extension<Config>
    ///     let api_key = config.api_key;
    ///     let timeout = config.timeout;
    ///
    ///     Ok(Response::text(format!("Using API key: {}", api_key)))
    /// }
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Extension<T> {
    type Target = T;

    /// Provides automatic dereferencing to the inner value.
    ///
    /// This allows you to call methods on the wrapped type directly
    /// without explicitly accessing the inner field.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extension;
    ///
    /// #[derive(Clone)]
    /// struct Counter {
    ///     value: u64,
    /// }
    ///
    /// impl Counter {
    ///     fn increment(&mut self) {
    ///         self.value += 1;
    ///     }
    ///
    ///     fn get_value(&self) -> u64 {
    ///         self.value
    ///     }
    /// }
    ///
    /// let extension = Extension(Counter { value: 5 });
    /// // Can call methods directly due to Deref
    /// let value = extension.get_value();
    /// assert_eq!(value, 5);
    /// ```
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Extension<T> {
    /// Provides automatic mutable dereferencing to the inner value.
    ///
    /// This allows you to call mutable methods on the wrapped type directly.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Extension;
    ///
    /// #[derive(Clone)]
    /// struct Counter {
    ///     value: u64,
    /// }
    ///
    /// impl Counter {
    ///     fn increment(&mut self) {
    ///         self.value += 1;
    ///     }
    ///
    ///     fn get_value(&self) -> u64 {
    ///         self.value
    ///     }
    /// }
    ///
    /// let mut extension = Extension(Counter { value: 5 });
    /// // Can call mutable methods directly due to DerefMut
    /// extension.increment();
    /// assert_eq!(extension.get_value(), 6);
    /// ```
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
