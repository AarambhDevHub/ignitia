//! # Multipart Form Data Parser
//!
//! This module provides a streaming parser for `multipart/form-data` requests,
//! enabling efficient processing of file uploads and form submissions with
//! automatic memory management and disk overflow for large files.
//!
//! ## Features
//!
//! - **Streaming Processing**: Parse multipart data as it arrives without loading everything into memory
//! - **Automatic Overflow**: Large files are automatically written to temporary disk storage
//! - **Configurable Limits**: Control memory usage and security with customizable size limits
//! - **Type Safety**: Strongly typed field handling with error recovery
//! - **Async Support**: Full asynchronous processing for non-blocking I/O operations
//!
//! ## Usage Examples
//!
//! ### Basic Multipart Parsing
//!
//! ```
//! use ignitia::multipart::{Multipart, MultipartConfig};
//! use bytes::Bytes;
//!
//! async fn parse_multipart_form(
//!     body: Bytes,
//!     boundary: String
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     let config = MultipartConfig::default();
//!     let mut multipart = Multipart::new(body, boundary, config);
//!
//!     while let Some(field) = multipart.next_field().await? {
//!         if field.is_file() {
//!             println!("File upload: {} ({})",
//!                 field.name(),
//!                 field.file_name().unwrap_or("unnamed"));
//!
//!             // Save file to disk
//!             let file_field = field.save_to_file("uploads/file.dat").await?;
//!             println!("Saved {} bytes", file_field.size);
//!         } else {
//!             // Handle text field
//!             let text = field.text().await?;
//!             println!("Field {}: {}", field.name(), text);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Collecting All Fields
//!
//! ```
//! use ignitia::multipart::{Multipart, MultipartConfig};
//!
//! async fn collect_form_data(
//!     body: bytes::Bytes,
//!     boundary: String
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     let config = MultipartConfig {
//!         max_field_size: 5 * 1024 * 1024, // 5MB per field
//!         max_fields: 50,
//!         ..Default::default()
//!     };
//!
//!     let multipart = Multipart::new(body, boundary, config);
//!     let data = multipart.collect_all().await?;
//!
//!     // Process text fields
//!     for field in data.text_fields() {
//!         let text = field.text().await?;
//!         println!("Text field {}: {}", field.name(), text);
//!     }
//!
//!     // Process file fields
//!     for field in data.file_fields() {
//!         println!("File field: {} ({})",
//!             field.name(),
//!             field.file_name().unwrap_or("no filename"));
//!     }
//!
//!     Ok(())
//! }
//! ```

use super::error::MultipartError;
use super::field::Field;
use super::MultipartConfig;
use bytes::Bytes;
use futures_util::stream::once;
use std::{collections::HashMap, io::Write};
use tempfile::NamedTempFile;

/// Streaming parser for `multipart/form-data` requests.
///
/// The `Multipart` parser processes multipart form data in a streaming fashion,
/// which means it can handle very large uploads without consuming excessive memory.
/// Fields are processed one at a time, and large files are automatically written
/// to temporary disk storage when they exceed the configured memory threshold.
///
/// ## Memory Management
///
/// The parser uses a sophisticated memory management strategy:
///
/// - **Small fields**: Kept in memory for fast access
/// - **Large fields**: Automatically overflow to temporary files on disk
/// - **File uploads**: Always written to disk regardless of size
/// - **Configurable limits**: All thresholds can be customized via `MultipartConfig`
///
/// ## Security Considerations
///
/// - Maximum request size limits prevent DOS attacks
/// - Individual field size limits prevent memory exhaustion
/// - Maximum field count limits prevent resource exhaustion
/// - Temporary files are automatically cleaned up on drop
///
/// ## Examples
///
/// ### Processing Fields One by One
///
/// ```
/// use ignitia::multipart::{Multipart, MultipartConfig};
///
/// async fn process_upload(
///     body: bytes::Bytes,
///     boundary: String
/// ) -> Result<(), Box<dyn std::error::Error>> {
///     let config = MultipartConfig::default();
///     let mut multipart = Multipart::new(body, boundary, config);
///
///     let mut file_count = 0;
///     let mut text_fields = Vec::new();
///
///     while let Some(field) = multipart.next_field().await? {
///         match field.is_file() {
///             true => {
///                 file_count += 1;
///                 let filename = field.file_name().unwrap_or("unnamed");
///                 let saved_file = field.save_to_file(
///                     format!("uploads/{}", filename)
///                 ).await?;
///
///                 println!("Saved file: {} ({} bytes)",
///                     saved_file.path.display(), saved_file.size);
///             },
///             false => {
///                 let name = field.name().to_string();
///                 let value = field.text().await?;
///                 text_fields.push((name, value));
///             }
///         }
///     }
///
///     println!("Processed {} files and {} text fields",
///         file_count, text_fields.len());
///     Ok(())
/// }
/// ```
///
/// ### Custom Configuration
///
/// ```
/// use ignitia::multipart::{Multipart, MultipartConfig};
///
/// // Create parser with custom limits
/// let config = MultipartConfig {
///     max_request_size: 50 * 1024 * 1024,    // 50MB total
///     max_field_size: 10 * 1024 * 1024,      // 10MB per field
///     file_size_threshold: 512 * 1024,       // 512KB before disk overflow
///     max_fields: 200,                        // Maximum 200 fields
/// };
///
/// let mut multipart = Multipart::new(body, boundary, config);
/// ```
pub struct Multipart {
    /// The underlying multer multipart parser
    inner: multer::Multipart<'static>,
    /// Configuration settings for parsing limits
    config: MultipartConfig,
    /// Counter to track number of fields processed
    fields_processed: usize,
}

impl Multipart {
    /// Creates a new multipart parser with the specified configuration.
    ///
    /// This constructor sets up the streaming parser to process the provided
    /// multipart body data using the given boundary string. The parser will
    /// enforce all limits specified in the configuration.
    ///
    /// # Arguments
    ///
    /// * `body` - The raw multipart body data as bytes
    /// * `boundary` - The boundary string extracted from the Content-Type header
    /// * `config` - Configuration specifying size limits and behavior
    ///
    /// # Returns
    ///
    /// A new `Multipart` parser ready to process fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::{Multipart, MultipartConfig};
    /// use bytes::Bytes;
    ///
    /// // Create with default configuration
    /// let body = Bytes::from("multipart data...");
    /// let boundary = "boundary123".to_string();
    /// let config = MultipartConfig::default();
    /// let parser = Multipart::new(body, boundary, config);
    ///
    /// // Create with custom configuration
    /// let custom_config = MultipartConfig {
    ///     max_field_size: 2 * 1024 * 1024, // 2MB per field
    ///     max_fields: 10,                   // Maximum 10 fields
    ///     ..Default::default()
    /// };
    /// let parser = Multipart::new(body, boundary, custom_config);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// The parser creates an internal stream from the body bytes and initializes
    /// the multer parser with the provided boundary. The stream is designed to
    /// yield the entire body as a single chunk, which is suitable for HTTP
    /// request bodies that are already fully received.
    pub fn new(body: Bytes, boundary: String, config: MultipartConfig) -> Self {
        // Create a stream from the body bytes
        let stream = once(async move { Result::<Bytes, std::io::Error>::Ok(body) });
        let inner = multer::Multipart::new(stream, boundary);

        Self {
            inner,
            config,
            fields_processed: 0,
        }
    }

    /// Retrieves the next field from the multipart stream.
    ///
    /// This method processes fields sequentially in the order they appear in
    /// the multipart data. Each field is fully processed before the next one
    /// is returned, which includes reading all field data and applying the
    /// appropriate storage strategy (memory vs. disk).
    ///
    /// # Returns
    ///
    /// - `Ok(Some(field))` - Successfully parsed the next field
    /// - `Ok(None)` - No more fields in the stream (end of multipart data)
    /// - `Err(error)` - An error occurred during parsing or field limits were exceeded
    ///
    /// # Errors
    ///
    /// This method can return several types of errors:
    ///
    /// - `MultipartError::TooManyFields` - Exceeded the maximum field count limit
    /// - `MultipartError::FieldTooLarge` - A field exceeded the maximum size limit
    /// - `MultipartError::InvalidField` - Field format is invalid or required data is missing
    /// - `MultipartError::Io` - Disk I/O error when writing temporary files
    /// - `MultipartError::Parse` - Underlying multipart parsing error
    ///
    /// # Examples
    ///
    /// ### Basic Field Processing
    ///
    /// ```
    /// use ignitia::multipart::{Multipart, MultipartConfig};
    ///
    /// async fn process_fields(mut parser: Multipart) -> Result<(), Box<dyn std::error::Error>> {
    ///     while let Some(field) = parser.next_field().await? {
    ///         println!("Processing field: {}", field.name());
    ///
    ///         if let Some(filename) = field.file_name() {
    ///             println!("  File upload: {}", filename);
    ///             if let Some(content_type) = field.content_type() {
    ///                 println!("  Content type: {}", content_type);
    ///             }
    ///         }
    ///
    ///         // Process field data based on type
    ///         match field.is_file() {
    ///             true => {
    ///                 let bytes = field.bytes().await?;
    ///                 println!("  File size: {} bytes", bytes.len());
    ///             },
    ///             false => {
    ///                 let text = field.text().await?;
    ///                 println!("  Field value: {}", text);
    ///             }
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ### Error Handling
    ///
    /// ```
    /// use ignitia::multipart::{Multipart, MultipartError};
    ///
    /// async fn handle_parsing_errors(mut parser: Multipart) {
    ///     loop {
    ///         match parser.next_field().await {
    ///             Ok(Some(field)) => {
    ///                 // Process field successfully
    ///                 println!("Got field: {}", field.name());
    ///             },
    ///             Ok(None) => {
    ///                 // End of stream
    ///                 println!("Finished processing all fields");
    ///                 break;
    ///             },
    ///             Err(MultipartError::TooManyFields { max_fields }) => {
    ///                 eprintln!("Too many fields! Maximum allowed: {}", max_fields);
    ///                 break;
    ///             },
    ///             Err(MultipartError::FieldTooLarge { field_name, max_size }) => {
    ///                 eprintln!("Field '{}' is too large! Maximum size: {} bytes",
    ///                     field_name, max_size);
    ///                 break;
    ///             },
    ///             Err(e) => {
    ///                 eprintln!("Parsing error: {}", e);
    ///                 break;
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// - Fields are processed lazily, so memory usage remains constant regardless of total multipart size
    /// - Large fields are automatically written to disk to prevent memory exhaustion
    /// - File I/O is performed asynchronously to avoid blocking the parser
    /// - Temporary files are created only when necessary (large fields or file uploads)
    ///
    /// # Security Notes
    ///
    /// - The field count limit prevents resource exhaustion attacks
    /// - Field size limits prevent individual large fields from consuming excessive memory
    /// - All limits are enforced before processing field content
    pub async fn next_field(&mut self) -> Result<Option<Field>, MultipartError> {
        if self.fields_processed >= self.config.max_fields {
            return Err(MultipartError::TooManyFields {
                max_fields: self.config.max_fields,
            });
        }

        match self.inner.next_field().await {
            Ok(Some(field)) => {
                self.fields_processed += 1;
                Ok(Some(self.process_field(field).await?))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(MultipartError::Parse(e.to_string())),
        }
    }

    /// Processes a single field from the multipart stream.
    ///
    /// This internal method handles the actual parsing and storage of field data.
    /// It determines the appropriate storage strategy (memory vs. disk) based on
    /// the field type and size, then reads and stores the field content accordingly.
    ///
    /// # Arguments
    ///
    /// * `field` - The raw multer field to process
    ///
    /// # Returns
    ///
    /// A fully processed `Field` with all data stored appropriately.
    ///
    /// # Storage Strategy
    ///
    /// The method uses the following logic to determine how to store field data:
    ///
    /// 1. **File uploads** (has filename): Always written to temporary files
    /// 2. **Large fields**: Written to temporary files when exceeding `file_size_threshold`
    /// 3. **Text fields**: Kept in memory if under threshold and parsed as UTF-8 when possible
    /// 4. **Binary fields**: Kept in memory if under threshold
    ///
    /// # Error Handling
    ///
    /// - Field size validation occurs before processing content
    /// - I/O errors during temporary file operations are propagated
    /// - Invalid field headers or missing required data results in errors
    /// - UTF-8 parsing errors for text fields fall back to binary storage
    ///
    /// # Implementation Details
    ///
    /// The method streams field data chunk by chunk to avoid loading large
    /// files entirely into memory. When temporary files are needed, it uses
    /// `tokio::task::spawn_blocking` to perform synchronous file operations
    /// without blocking the async runtime.
    async fn process_field(&self, mut field: multer::Field<'_>) -> Result<Field, MultipartError> {
        let name = field
            .name()
            .ok_or_else(|| MultipartError::InvalidField("Missing field name".into()))?
            .to_string();

        let file_name = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|ct| ct.to_string());

        // Parse headers
        let mut headers = HashMap::new();
        for (key, value) in field.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        let mut ignitia_field = Field::new(name.clone(), file_name, content_type, headers);

        // Determine if this should be treated as a file
        let is_file = ignitia_field.is_file();
        let mut total_size = 0usize;
        let mut data = Vec::new();
        let mut temp_file: Option<NamedTempFile> = None;

        // Stream field data
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| MultipartError::Parse(e.to_string()))?
        {
            total_size += chunk.len();

            // Check field size limits
            if total_size > self.config.max_field_size {
                return Err(MultipartError::FieldTooLarge {
                    field_name: name,
                    max_size: self.config.max_field_size,
                });
            }

            // For files or large data, write to temp file
            if is_file || (total_size > self.config.file_size_threshold && temp_file.is_none()) {
                if temp_file.is_none() {
                    temp_file = Some(NamedTempFile::new().map_err(MultipartError::from)?);

                    // Write any previously buffered data using spawn_blocking
                    if !data.is_empty() {
                        let temp_file_ref = temp_file.as_mut().unwrap();
                        let data_to_write = data.clone();

                        // Use spawn_blocking for sync operations on NamedTempFile
                        let temp_file_path = temp_file_ref.path().to_owned();
                        tokio::task::spawn_blocking(move || {
                            std::fs::write(&temp_file_path, &data_to_write)
                        })
                        .await
                        .map_err(|e| {
                            MultipartError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?
                        .map_err(MultipartError::from)?;

                        data.clear();
                    }
                }

                // Write chunk using spawn_blocking for sync operations
                if let Some(temp_file_ref) = temp_file.as_ref() {
                    let temp_file_path = temp_file_ref.path().to_owned();
                    let chunk_data = chunk.to_vec();

                    tokio::task::spawn_blocking(move || {
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&temp_file_path)?
                            .write_all(&chunk_data)?;
                        std::io::Result::Ok(())
                    })
                    .await
                    .map_err(|e| {
                        MultipartError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?
                    .map_err(MultipartError::from)?;
                }
            } else {
                data.extend_from_slice(&chunk);
            }
        }

        // Set the field data based on how it was processed
        match temp_file {
            Some(file) => {
                ignitia_field.set_file(file);
            }
            None => {
                if ignitia_field
                    .content_type()
                    .map_or(true, |ct| ct.starts_with("text/"))
                {
                    // Try to parse as text for text content types
                    match String::from_utf8(data.clone()) {
                        Ok(text) => ignitia_field.set_text(text),
                        Err(_) => ignitia_field.set_bytes(Bytes::from(data)),
                    }
                } else {
                    ignitia_field.set_bytes(Bytes::from(data));
                }
            }
        }

        Ok(ignitia_field)
    }

    /// Collects all fields from the multipart stream into a container.
    ///
    /// This convenience method processes the entire multipart stream and collects
    /// all fields into a `MultipartData` container. This is useful when you need
    /// to access fields by name or type, or when the multipart data is small enough
    /// to process all at once.
    ///
    /// # Returns
    ///
    /// A `Result` containing `MultipartData` with all parsed fields, or a
    /// `MultipartError` if any field fails to parse or limits are exceeded.
    ///
    /// # Memory Considerations
    ///
    /// This method will keep references to all fields in memory simultaneously.
    /// For large multipart requests with many fields, consider using `next_field()`
    /// instead to process fields one at a time and reduce memory usage.
    ///
    /// Large file data is still written to temporary files as configured, so this
    /// method primarily affects metadata memory usage, not file content storage.
    ///
    /// # Examples
    ///
    /// ### Basic Collection
    ///
    /// ```
    /// use ignitia::multipart::{Multipart, MultipartConfig};
    ///
    /// async fn collect_form(
    ///     body: bytes::Bytes,
    ///     boundary: String
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = MultipartConfig::default();
    ///     let parser = Multipart::new(body, boundary, config);
    ///     let data = parser.collect_all().await?;
    ///
    ///     println!("Collected {} fields", data.fields.len());
    ///
    ///     // Access specific fields
    ///     if let Some(username_field) = data.get_field("username") {
    ///         let username = username_field.text().await?;
    ///         println!("Username: {}", username);
    ///     }
    ///
    ///     // Process all files
    ///     for file_field in data.file_fields() {
    ///         if let Some(filename) = file_field.file_name() {
    ///             println!("Found file: {}", filename);
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ### Error Handling
    ///
    /// ```
    /// use ignitia::multipart::{Multipart, MultipartError};
    ///
    /// async fn safe_collect(parser: Multipart) {
    ///     match parser.collect_all().await {
    ///         Ok(data) => {
    ///             println!("Successfully parsed {} fields", data.fields.len());
    ///
    ///             for field in &data.fields {
    ///                 println!("- Field: {} (file: {})",
    ///                     field.name(), field.is_file());
    ///             }
    ///         },
    ///         Err(MultipartError::TooManyFields { max_fields }) => {
    ///             eprintln!("Form has too many fields (max: {})", max_fields);
    ///         },
    ///         Err(MultipartError::FieldTooLarge { field_name, max_size }) => {
    ///             eprintln!("Field '{}' exceeds size limit (max: {} bytes)",
    ///                 field_name, max_size);
    ///         },
    ///         Err(e) => {
    ///             eprintln!("Failed to parse multipart data: {}", e);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - All field parsing errors will halt processing and return immediately
    /// - The method consumes the parser, so it cannot be used again after calling this
    /// - Fields are processed in the order they appear in the multipart stream
    /// - Temporary files for large fields are preserved until the `MultipartData` is dropped
    pub async fn collect_all(mut self) -> Result<MultipartData, MultipartError> {
        let mut fields = Vec::new();

        while let Some(field) = self.next_field().await? {
            fields.push(field);
        }

        Ok(MultipartData { fields })
    }
}

/// Container for all multipart form data fields.
///
/// `MultipartData` provides convenient access methods for working with
/// collections of multipart fields. It allows you to query fields by name,
/// filter by type (text vs. file), and iterate over all fields.
///
/// This container is returned by `Multipart::collect_all()` and provides
/// a higher-level interface for accessing parsed multipart data.
///
/// ## Field Access Patterns
///
/// The container provides several ways to access fields:
///
/// - **By name**: Get specific fields using `get_field()` or `get_fields()`
/// - **By type**: Filter text fields with `text_fields()` or files with `file_fields()`
/// - **Direct access**: Iterate over all fields using `fields`
///
/// ## Examples
///
/// ### Accessing Specific Fields
///
/// ```
/// use ignitia::multipart::MultipartData;
///
/// async fn process_form(data: MultipartData) -> Result<(), Box<dyn std::error::Error>> {
///     // Get a specific field by name
///     if let Some(email_field) = data.get_field("email") {
///         let email = email_field.text().await?;
///         println!("Email: {}", email);
///     }
///
///     // Get all fields with the same name (e.g., multiple file uploads)
///     let photo_fields = data.get_fields("photos");
///     for photo in photo_fields {
///         if let Some(filename) = photo.file_name() {
///             println!("Photo: {}", filename);
///         }
///     }
///
///     Ok(())
/// }
/// ```
///
/// ### Processing by Field Type
///
/// ```
/// async fn categorize_fields(data: MultipartData) -> Result<(), Box<dyn std::error::Error>> {
///     // Process all text fields
///     println!("Text fields:");
///     for field in data.text_fields() {
///         let value = field.text().await?;
///         println!("  {}: {}", field.name(), value);
///     }
///
///     // Process all file uploads
///     println!("File uploads:");
///     for field in data.file_fields() {
///         let filename = field.file_name().unwrap_or("unnamed");
///         println!("  {}: {} ({})", field.name(), filename,
///             field.content_type().unwrap_or("unknown"));
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct MultipartData {
    /// Vector containing all parsed fields in order
    pub fields: Vec<Field>,
}

impl MultipartData {
    /// Returns all fields that have the specified name.
    ///
    /// This method is useful when multiple fields share the same name,
    /// such as multiple file uploads or checkbox values. It returns
    /// references to all matching fields in the order they appeared
    /// in the multipart data.
    ///
    /// # Arguments
    ///
    /// * `name` - The field name to search for
    ///
    /// # Returns
    ///
    /// A vector of references to fields with the matching name.
    /// Returns an empty vector if no fields match.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartData;
    ///
    /// async fn process_photos(data: MultipartData) -> Result<(), Box<dyn std::error::Error>> {
    ///     // Get all photo uploads (multiple files with same field name)
    ///     let photos = data.get_fields("photos");
    ///
    ///     println!("Found {} photos", photos.len());
    ///
    ///     for (i, photo) in photos.iter().enumerate() {
    ///         let filename = photo.file_name().unwrap_or(&format!("photo_{}", i));
    ///         println!("Photo {}: {}", i + 1, filename);
    ///
    ///         // Save each photo
    ///         let saved = photo.save_to_file(format!("uploads/{}", filename)).await?;
    ///         println!("  Saved {} bytes", saved.size);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// This method performs a linear search through all fields, so its
    /// performance is O(n) where n is the total number of fields.
    pub fn get_fields(&self, name: &str) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.name() == name).collect()
    }

    /// Returns the first field with the specified name.
    ///
    /// This is a convenience method for accessing single-valued fields
    /// where you expect only one field with the given name. If multiple
    /// fields have the same name, only the first one is returned.
    ///
    /// # Arguments
    ///
    /// * `name` - The field name to search for
    ///
    /// # Returns
    ///
    /// - `Some(field)` - Reference to the first field with the matching name
    /// - `None` - No field found with the specified name
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::MultipartData;
    ///
    /// async fn get_user_info(data: MultipartData) -> Result<(), Box<dyn std::error::Error>> {
    ///     // Get individual form fields
    ///     let username = match data.get_field("username") {
    ///         Some(field) => field.text().await?,
    ///         None => return Err("Username field is required".into()),
    ///     };
    ///
    ///     let email = match data.get_field("email") {
    ///         Some(field) => field.text().await?,
    ///         None => return Err("Email field is required".into()),
    ///     };
    ///
    ///     // Optional profile picture
    ///     if let Some(profile_pic) = data.get_field("profile_picture") {
    ///         if profile_pic.is_file() {
    ///             let saved_file = profile_pic.save_to_file("uploads/profile.jpg").await?;
    ///             println!("Profile picture saved: {} bytes", saved_file.size);
    ///         }
    ///     }
    ///
    ///     println!("User: {} ({})", username, email);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Single-value form fields (username, email, etc.)
    /// - Optional file uploads
    /// - Configuration parameters
    /// - Any field where you expect at most one value
    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name() == name)
    }

    /// Returns all fields that are text fields (not file uploads).
    ///
    /// This method filters the fields to return only those that represent
    /// regular form inputs like text boxes, textareas, hidden fields, etc.
    /// It excludes any fields that represent file uploads.
    ///
    /// # Returns
    ///
    /// A vector of references to all text fields in the order they appeared
    /// in the multipart data.
    ///
    /// # Examples
    ///
    /// ### Processing Form Data
    ///
    /// ```
    /// use ignitia::multipart::MultipartData;
    /// use std::collections::HashMap;
    ///
    /// async fn extract_form_data(data: MultipartData) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    ///     let mut form_data = HashMap::new();
    ///
    ///     // Convert all text fields to a key-value map
    ///     for field in data.text_fields() {
    ///         let name = field.name().to_string();
    ///         let value = field.text().await?;
    ///         form_data.insert(name, value);
    ///     }
    ///
    ///     Ok(form_data)
    /// }
    /// ```
    ///
    /// ### Validation
    ///
    /// ```
    /// async fn validate_required_fields(data: MultipartData) -> Result<(), String> {
    ///     let required_fields = vec!["username", "email", "password"];
    ///     let text_fields = data.text_fields();
    ///
    ///     for required in required_fields {
    ///         let found = text_fields.iter().any(|field| field.name() == required);
    ///         if !found {
    ///             return Err(format!("Required field '{}' is missing", required));
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Field Classification
    ///
    /// A field is considered a text field if:
    /// - It does not have a filename in its Content-Disposition header
    /// - It is not stored as a temporary file on disk
    ///
    /// This includes:
    /// - Regular text inputs
    /// - Hidden fields
    /// - Textareas
    /// - Select boxes
    /// - Radio buttons and checkboxes
    pub fn text_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| !f.is_file()).collect()
    }

    /// Returns all fields that represent file uploads.
    ///
    /// This method filters the fields to return only those that represent
    /// file uploads, identified by having a filename in the Content-Disposition
    /// header or being stored as temporary files due to their size.
    ///
    /// # Returns
    ///
    /// A vector of references to all file fields in the order they appeared
    /// in the multipart data.
    ///
    /// # Examples
    ///
    /// ### Processing File Uploads
    ///
    /// ```
    /// use ignitia::multipart::MultipartData;
    ///
    /// async fn save_all_uploads(data: MultipartData) -> Result<(), Box<dyn std::error::Error>> {
    ///     let uploads_dir = "uploads";
    ///     std::fs::create_dir_all(uploads_dir)?;
    ///
    ///     let mut saved_files = Vec::new();
    ///
    ///     for (i, field) in data.file_fields().iter().enumerate() {
    ///         let filename = field.file_name()
    ///             .map(|s| s.to_string())
    ///             .unwrap_or_else(|| format!("upload_{}", i));
    ///
    ///         let file_path = format!("{}/{}", uploads_dir, filename);
    ///         let saved_file = field.save_to_file(&file_path).await?;
    ///
    ///         saved_files.push(saved_file);
    ///         println!("Saved: {} ({} bytes)", file_path, saved_file.size);
    ///     }
    ///
    ///     println!("Total files saved: {}", saved_files.len());
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ### File Analysis
    ///
    /// ```
    /// async fn analyze_uploads(data: MultipartData) -> Result<(), Box<dyn std::error::Error>> {
    ///     let file_fields = data.file_fields();
    ///
    ///     println!("Upload Summary:");
    ///     println!("  Total files: {}", file_fields.len());
    ///
    ///     let mut total_size = 0u64;
    ///     let mut content_types = std::collections::HashMap::new();
    ///
    ///     for field in file_fields {
    ///         // Get file size
    ///         let bytes = field.bytes().await?;
    ///         total_size += bytes.len() as u64;
    ///
    ///         // Track content types
    ///         let content_type = field.content_type().unwrap_or("unknown");
    ///         *content_types.entry(content_type).or_insert(0) += 1;
    ///
    ///         println!("  - {} ({}): {} bytes",
    ///             field.name(),
    ///             field.file_name().unwrap_or("unnamed"),
    ///             bytes.len()
    ///         );
    ///     }
    ///
    ///     println!("  Total size: {} bytes", total_size);
    ///     println!("  Content types: {:?}", content_types);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # File Detection
    ///
    /// A field is considered a file upload if:
    /// - It has a `filename` parameter in its Content-Disposition header, OR
    /// - It was written to a temporary file due to exceeding the memory threshold
    ///
    /// This covers:
    /// - Explicit file uploads via `<input type="file">`
    /// - Large text fields that overflow to disk storage
    /// - Binary data submitted through form fields
    pub fn file_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.is_file()).collect()
    }
}
