//! # Multipart Field Handling
//!
//! This module provides types and utilities for working with individual fields
//! within multipart/form-data requests. It supports both text fields and file
//! uploads with automatic memory management and disk streaming for large files.
//!
//! ## Features
//!
//! - **Unified Field Interface**: Handle both text and file fields through a single API
//! - **Memory Management**: Automatic switching between memory and disk storage
//! - **Temporary File Handling**: Safe temporary file creation and cleanup
//! - **Type Safety**: Strong typing to prevent common errors
//! - **Async Support**: Full asynchronous I/O for non-blocking operations
//!
//! ## Usage Examples
//!
//! ### Basic Field Processing
//!
//! ```
//! use ignitia::multipart::Field;
//!
//! async fn process_field(field: Field) -> Result<(), Box<dyn std::error::Error>> {
//!     if field.is_file() {
//!         // Handle file upload
//!         let file_name = field.file_name().unwrap_or("unknown");
//!         let bytes = field.bytes().await?;
//!         println!("File: {} ({} bytes)", file_name, bytes.len());
//!     } else {
//!         // Handle text field
//!         let text = field.text().await?;
//!         println!("Field {}: {}", field.name(), text);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Saving Files to Disk
//!
//! ```
//! use ignitia::multipart::Field;
//!
//! async fn save_upload(field: Field) -> Result<(), Box<dyn std::error::Error>> {
//!     if field.is_file() {
//!         let file_field = field.save_to_file("uploads/file.dat").await?;
//!         println!("Saved {} bytes to {}", file_field.size, file_field.path.display());
//!     }
//!     Ok(())
//! }
//! ```

use super::error::MultipartError;
use bytes::Bytes;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tokio::io::AsyncReadExt;

/// Represents a field in a multipart/form-data request.
///
/// A field can contain either text data or file data, and this type provides
/// a unified interface for working with both. Large files are automatically
/// stored in temporary files to avoid memory exhaustion.
///
/// # Field Types
///
/// - **Text Fields**: Simple form fields containing text data
/// - **File Fields**: File uploads with optional filename and content type
///
/// # Memory Management
///
/// Fields automatically choose the most appropriate storage method:
/// - Small data is kept in memory for fast access
/// - Large data is written to temporary files
/// - Files are automatically cleaned up when the Field is dropped
///
/// # Examples
///
/// ```
/// use ignitia::multipart::Field;
/// use std::collections::HashMap;
///
/// // Create a text field
/// let mut field = Field::new(
///     "username".to_string(),
///     None,
///     Some("text/plain".to_string()),
///     HashMap::new()
/// );
/// field.set_text("john_doe".to_string());
///
/// // Check field properties
/// assert_eq!(field.name(), "username");
/// assert!(!field.is_file());
/// assert_eq!(field.content_type(), Some("text/plain"));
/// ```
#[derive(Debug)]
pub struct Field {
    /// The name of the form field
    name: String,
    /// Optional filename for file uploads
    file_name: Option<String>,
    /// Optional MIME content type
    content_type: Option<String>,
    /// Additional headers from the multipart field
    headers: HashMap<String, String>,
    /// The actual field data
    data: FieldData,
}

/// Internal storage mechanism for field data.
///
/// This enum allows fields to store data in the most appropriate format:
/// - `Text`: UTF-8 text data stored in memory
/// - `Bytes`: Binary data stored in memory
/// - `File`: Large data stored in a temporary file
///
/// The choice of storage is made automatically based on data size and type.
#[derive(Debug)]
enum FieldData {
    /// UTF-8 text data stored in memory
    Text(String),
    /// Binary data stored in memory
    Bytes(Bytes),
    /// Data stored in a temporary file on disk
    File(NamedTempFile),
}

impl Field {
    /// Creates a new field with the specified properties.
    ///
    /// This constructor initializes a field with empty binary data. Use the
    /// `set_*` methods to populate the field with actual content.
    ///
    /// # Arguments
    ///
    /// * `name` - The form field name
    /// * `file_name` - Optional filename for file uploads
    /// * `content_type` - Optional MIME content type
    /// * `headers` - Additional headers from the multipart field
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// let field = Field::new(
    ///     "avatar".to_string(),
    ///     Some("profile.jpg".to_string()),
    ///     Some("image/jpeg".to_string()),
    ///     HashMap::new()
    /// );
    ///
    /// assert_eq!(field.name(), "avatar");
    /// assert_eq!(field.file_name(), Some("profile.jpg"));
    /// assert_eq!(field.content_type(), Some("image/jpeg"));
    /// assert!(field.is_file());
    /// ```
    pub fn new(
        name: String,
        file_name: Option<String>,
        content_type: Option<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            file_name,
            content_type,
            headers,
            data: FieldData::Bytes(Bytes::new()),
        }
    }

    /// Returns the name of the form field.
    ///
    /// This corresponds to the `name` attribute in HTML form fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// let field = Field::new("email".to_string(), None, None, HashMap::new());
    /// assert_eq!(field.name(), "email");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the filename if this field represents a file upload.
    ///
    /// This value comes from the `filename` parameter in the Content-Disposition
    /// header of the multipart field.
    ///
    /// # Returns
    ///
    /// - `Some(filename)` if this is a file upload with a filename
    /// - `None` if this is not a file upload or no filename was provided
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// // File upload with filename
    /// let file_field = Field::new(
    ///     "document".to_string(),
    ///     Some("report.pdf".to_string()),
    ///     Some("application/pdf".to_string()),
    ///     HashMap::new()
    /// );
    /// assert_eq!(file_field.file_name(), Some("report.pdf"));
    ///
    /// // Text field without filename
    /// let text_field = Field::new("message".to_string(), None, None, HashMap::new());
    /// assert_eq!(text_field.file_name(), None);
    /// ```
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    /// Returns the MIME content type of the field data.
    ///
    /// This value comes from the Content-Type header of the multipart field.
    /// For file uploads, this typically indicates the file type (e.g., "image/jpeg").
    /// For text fields, this might be "text/plain" or omitted entirely.
    ///
    /// # Returns
    ///
    /// - `Some(content_type)` if a content type was specified
    /// - `None` if no content type was provided
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// let field = Field::new(
    ///     "photo".to_string(),
    ///     Some("vacation.jpg".to_string()),
    ///     Some("image/jpeg".to_string()),
    ///     HashMap::new()
    /// );
    /// assert_eq!(field.content_type(), Some("image/jpeg"));
    /// ```
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Returns all headers associated with this field.
    ///
    /// This includes any additional headers that were present in the multipart
    /// field beyond the standard Content-Disposition and Content-Type headers.
    ///
    /// # Returns
    ///
    /// A reference to a HashMap containing all field headers as key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("x-custom-header".to_string(), "custom-value".to_string());
    ///
    /// let field = Field::new("data".to_string(), None, None, headers);
    /// assert_eq!(field.headers().get("x-custom-header"), Some(&"custom-value".to_string()));
    /// ```
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Determines whether this field represents a file upload.
    ///
    /// A field is considered a file upload if:
    /// - It has a filename, OR
    /// - Its data is stored in a temporary file
    ///
    /// This is useful for distinguishing between regular form fields and file uploads.
    ///
    /// # Returns
    ///
    /// `true` if this field represents a file upload, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// // File upload (has filename)
    /// let file_field = Field::new(
    ///     "upload".to_string(),
    ///     Some("data.txt".to_string()),
    ///     None,
    ///     HashMap::new()
    /// );
    /// assert!(file_field.is_file());
    ///
    /// // Text field (no filename)
    /// let text_field = Field::new("name".to_string(), None, None, HashMap::new());
    /// assert!(!text_field.is_file());
    /// ```
    pub fn is_file(&self) -> bool {
        self.file_name.is_some() || matches!(self.data, FieldData::File(_))
    }

    /// Consumes the field and returns its content as bytes.
    ///
    /// This method works with all field types:
    /// - Text fields are converted to UTF-8 bytes
    /// - Binary fields return their raw bytes
    /// - File fields are read from disk and returned as bytes
    ///
    /// # Returns
    ///
    /// A `Result` containing the field data as `Bytes`, or a `MultipartError`
    /// if the data cannot be read (e.g., file I/O error).
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// async fn process_field_data(field: Field) -> Result<(), Box<dyn std::error::Error>> {
    ///     let bytes = field.bytes().await?;
    ///     println!("Field contains {} bytes", bytes.len());
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MultipartError::Io` if there's an error reading from a temporary file.
    pub async fn bytes(self) -> Result<Bytes, MultipartError> {
        match self.data {
            FieldData::Bytes(bytes) => Ok(bytes),
            FieldData::Text(text) => Ok(Bytes::from(text)),
            FieldData::File(file) => {
                let mut bytes = Vec::new();
                let mut file_handle = tokio::fs::File::from_std(file.reopen()?);
                file_handle.read_to_end(&mut bytes).await?;
                Ok(Bytes::from(bytes))
            }
        }
    }

    /// Consumes the field and returns its content as a UTF-8 string.
    ///
    /// This method is most appropriate for text fields, but can be used with
    /// binary fields if they contain valid UTF-8 data.
    ///
    /// # Returns
    ///
    /// A `Result` containing the field data as a `String`, or a `MultipartError`
    /// if the data cannot be read or contains invalid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// async fn get_field_text(field: Field) -> Result<String, Box<dyn std::error::Error>> {
    ///     let text = field.text().await?;
    ///     Ok(text)
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - `MultipartError::Io` if there's an error reading from a temporary file
    /// - `MultipartError::InvalidUtf8` if the field data is not valid UTF-8
    pub async fn text(self) -> Result<String, MultipartError> {
        match self.data {
            FieldData::Text(text) => Ok(text),
            FieldData::Bytes(bytes) => std::str::from_utf8(&bytes)
                .map(|s| s.to_string())
                .map_err(MultipartError::from),
            FieldData::File(file) => {
                let mut content = String::new();
                let mut file_handle = tokio::fs::File::from_std(file.reopen()?);
                file_handle.read_to_string(&mut content).await?;
                Ok(content)
            }
        }
    }

    /// Consumes the field and saves its content to a file at the specified path.
    ///
    /// This method creates a `FileField` that represents the saved file with
    /// metadata about its location and size. The original field data is moved
    /// to the specified location.
    ///
    /// # Arguments
    ///
    /// * `path` - The destination path where the file should be saved
    ///
    /// # Returns
    ///
    /// A `Result` containing a `FileField` with information about the saved file,
    /// or a `MultipartError` if the save operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// async fn save_upload(field: Field) -> Result<(), Box<dyn std::error::Error>> {
    ///     let file_field = field.save_to_file("uploads/user_file.dat").await?;
    ///
    ///     println!("Saved file: {}", file_field.path.display());
    ///     println!("File size: {} bytes", file_field.size);
    ///
    ///     if let Some(original_name) = file_field.file_name {
    ///         println!("Original filename: {}", original_name);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Behavior by Field Type
    ///
    /// - **Temporary File**: The temporary file is moved to the destination
    /// - **Memory Data**: The data is written to a new file at the destination
    /// - **Text Data**: The text is written as UTF-8 bytes to the destination
    ///
    /// # Errors
    ///
    /// - `MultipartError::Io` if there's an error during file operations
    /// - File system errors (permissions, disk space, etc.)
    pub async fn save_to_file<P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<FileField, crate::multipart::MultipartError> {
        let file_path = path.as_ref().to_path_buf();

        match self.data {
            FieldData::File(temp_file) => {
                let path_for_metadata = file_path.clone();

                // Try to persist first (atomic move if on same filesystem)
                match temp_file.persist(&file_path) {
                    Ok(_) => {
                        // Success - file moved atomically
                        let metadata = tokio::fs::metadata(&path_for_metadata).await?;
                        Ok(FileField {
                            name: self.name,
                            file_name: self.file_name,
                            content_type: self.content_type,
                            path: path_for_metadata,
                            size: metadata.len(),
                        })
                    }
                    Err(persist_error) => {
                        // If persist fails (cross-device link), fall back to copy + delete
                        tracing::debug!(
                            "Atomic persist failed, falling back to copy: {}",
                            persist_error.error
                        );

                        // Get the temp file back from the error
                        let temp_file = persist_error.file;

                        // Copy the file contents instead
                        tokio::fs::copy(temp_file.path(), &file_path)
                            .await
                            .map_err(MultipartError::from)?;

                        let metadata = tokio::fs::metadata(&path_for_metadata).await?;

                        // The temp file will be automatically deleted when it goes out of scope
                        Ok(FileField {
                            name: self.name,
                            file_name: self.file_name,
                            content_type: self.content_type,
                            path: path_for_metadata,
                            size: metadata.len(),
                        })
                    }
                }
            }
            FieldData::Bytes(bytes) => {
                tokio::fs::write(&file_path, &bytes).await?;
                let size = bytes.len() as u64;

                Ok(FileField {
                    name: self.name,
                    file_name: self.file_name,
                    content_type: self.content_type,
                    size,
                    path: file_path,
                })
            }
            FieldData::Text(text) => {
                let bytes = text.as_bytes();
                tokio::fs::write(&file_path, bytes).await?;
                let size = bytes.len() as u64;

                Ok(FileField {
                    name: self.name,
                    file_name: self.file_name,
                    content_type: self.content_type,
                    size,
                    path: file_path,
                })
            }
        }
    }

    /// Sets the field data to the provided bytes.
    ///
    /// This method is used internally during multipart parsing to populate
    /// field data. It replaces any existing data in the field.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The binary data to store in this field
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use bytes::Bytes;
    /// use std::collections::HashMap;
    ///
    /// let mut field = Field::new("data".to_string(), None, None, HashMap::new());
    /// field.set_bytes(Bytes::from("Hello, world!"));
    /// ```
    pub fn set_bytes(&mut self, bytes: Bytes) {
        self.data = FieldData::Bytes(bytes);
    }

    /// Sets the field data to the provided text string.
    ///
    /// This method is used internally during multipart parsing to populate
    /// text field data. It replaces any existing data in the field.
    ///
    /// # Arguments
    ///
    /// * `text` - The text data to store in this field
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use std::collections::HashMap;
    ///
    /// let mut field = Field::new("message".to_string(), None, None, HashMap::new());
    /// field.set_text("Hello, world!".to_string());
    /// ```
    pub fn set_text(&mut self, text: String) {
        self.data = FieldData::Text(text);
    }

    /// Sets the field data to be stored in the provided temporary file.
    ///
    /// This method is used internally during multipart parsing when field
    /// data is large enough to warrant disk storage. It replaces any
    /// existing data in the field.
    ///
    /// # Arguments
    ///
    /// * `file` - The temporary file containing the field data
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::Field;
    /// use tempfile::NamedTempFile;
    /// use std::collections::HashMap;
    ///
    /// let mut field = Field::new("upload".to_string(), None, None, HashMap::new());
    /// let temp_file = NamedTempFile::new().unwrap();
    /// field.set_file(temp_file);
    /// ```
    pub fn set_file(&mut self, file: NamedTempFile) {
        self.data = FieldData::File(file);
    }
}

/// Represents a field that has been permanently saved to disk.
///
/// A `FileField` is created when a multipart field is saved to a specific
/// location using `Field::save_to_file()`. It provides metadata about the
/// saved file and methods for accessing its contents.
///
/// Unlike `Field`, a `FileField` always represents data stored on disk,
/// making it suitable for long-term storage and processing of uploaded files.
///
/// # Examples
///
/// ```
/// use ignitia::multipart::FileField;
/// use std::path::PathBuf;
///
/// let file_field = FileField {
///     name: "document".to_string(),
///     file_name: Some("report.pdf".to_string()),
///     content_type: Some("application/pdf".to_string()),
///     path: PathBuf::from("/uploads/report.pdf"),
///     size: 1024 * 1024, // 1MB
/// };
///
/// println!("Saved {} ({} bytes)", file_field.name, file_field.size);
/// ```
#[derive(Debug, Clone)]
pub struct FileField {
    /// Original field name from the form
    pub name: String,
    /// Original filename if provided in the upload
    pub file_name: Option<String>,
    /// MIME content type if provided
    pub content_type: Option<String>,
    /// Current path where the file is stored
    pub path: PathBuf,
    /// Size of the file in bytes
    pub size: u64,
}

impl FileField {
    /// Reads the entire file content into memory as bytes.
    ///
    /// This method loads the complete file into memory. For large files,
    /// consider using streaming I/O instead to avoid memory pressure.
    ///
    /// # Returns
    ///
    /// A `Result` containing the file contents as `Bytes`, or a `MultipartError`
    /// if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::FileField;
    ///
    /// async fn process_file(file_field: &FileField) -> Result<(), Box<dyn std::error::Error>> {
    ///     let contents = file_field.bytes().await?;
    ///     println!("File contains {} bytes", contents.len());
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// This method reads the entire file into memory. For large files, consider:
    /// - Using streaming APIs for processing
    /// - Reading the file in chunks
    /// - Using `tokio::fs::File` directly for more control
    ///
    /// # Errors
    ///
    /// Returns `MultipartError::Io` if the file cannot be read due to:
    /// - File not found
    /// - Permission denied
    /// - I/O errors
    /// - Out of memory (for very large files)
    pub async fn bytes(&self) -> Result<Bytes, MultipartError> {
        let bytes = tokio::fs::read(&self.path).await?;
        Ok(Bytes::from(bytes))
    }

    /// Reads the entire file content as a UTF-8 string.
    ///
    /// This method is appropriate for text files. It will return an error
    /// if the file contains invalid UTF-8 sequences.
    ///
    /// # Returns
    ///
    /// A `Result` containing the file contents as a `String`, or a `MultipartError`
    /// if the file cannot be read or contains invalid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::FileField;
    ///
    /// async fn read_text_file(file_field: &FileField) -> Result<String, Box<dyn std::error::Error>> {
    ///     let content = file_field.text().await?;
    ///     println!("File content: {}", content);
    ///     Ok(content)
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - `MultipartError::Io` if the file cannot be read
    /// - `MultipartError::InvalidUtf8` if the file contains invalid UTF-8
    pub async fn text(&self) -> Result<String, MultipartError> {
        let text = tokio::fs::read_to_string(&self.path).await?;
        Ok(text)
    }

    /// Moves the file to a new location and returns an updated FileField.
    ///
    /// This method performs a file system move operation, updating the
    /// FileField's path to reflect the new location. The original file
    /// is moved (not copied), so it will no longer exist at the old location.
    ///
    /// # Arguments
    ///
    /// * `new_path` - The destination path for the file
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `FileField` with the updated path,
    /// or a `MultipartError` if the move operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::FileField;
    ///
    /// async fn organize_file(file_field: FileField) -> Result<(), Box<dyn std::error::Error>> {
    ///     let moved_file = file_field.persist("permanent/location.dat").await?;
    ///     println!("File moved to: {}", moved_file.path.display());
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Behavior
    ///
    /// - The file is moved atomically if the source and destination are on the same filesystem
    /// - If cross-filesystem, the file is copied then the original is deleted
    /// - Parent directories are NOT created automatically
    /// - The operation will fail if the destination already exists
    ///
    /// # Errors
    ///
    /// - `MultipartError::Io` if the move operation fails due to:
    ///   - Permission denied
    ///   - Destination already exists
    ///   - Cross-device operation not supported
    ///   - Insufficient disk space
    pub async fn persist<P: AsRef<Path>>(self, new_path: P) -> Result<FileField, MultipartError> {
        tokio::fs::rename(&self.path, new_path.as_ref()).await?;
        Ok(FileField {
            name: self.name,
            file_name: self.file_name,
            content_type: self.content_type,
            path: new_path.as_ref().to_path_buf(),
            size: self.size,
        })
    }
}

/// Represents a simple text field from a multipart form.
///
/// A `TextField` is a simplified representation specifically for text-based
/// form fields. It's useful when you know a field contains only text data
/// and want a more convenient interface than the generic `Field` type.
///
/// # Examples
///
/// ```
/// use ignitia::multipart::TextField;
///
/// let field = TextField::new("username".to_string(), "john_doe".to_string())
///     .with_content_type("text/plain".to_string());
///
/// assert_eq!(field.name, "username");
/// assert_eq!(field.value, "john_doe");
/// assert_eq!(field.content_type, Some("text/plain".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct TextField {
    /// The name of the form field
    pub name: String,
    /// The text value of the field
    pub value: String,
    /// Optional MIME content type
    pub content_type: Option<String>,
}

impl TextField {
    /// Creates a new text field with the specified name and value.
    ///
    /// # Arguments
    ///
    /// * `name` - The form field name
    /// * `value` - The text value of the field
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::TextField;
    ///
    /// let field = TextField::new("email".to_string(), "user@example.com".to_string());
    /// assert_eq!(field.name, "email");
    /// assert_eq!(field.value, "user@example.com");
    /// assert_eq!(field.content_type, None);
    /// ```
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            content_type: None,
        }
    }

    /// Sets the content type for this text field and returns the updated field.
    ///
    /// This method uses the builder pattern, allowing for method chaining.
    ///
    /// # Arguments
    ///
    /// * `content_type` - The MIME content type for this field
    ///
    /// # Returns
    ///
    /// The updated `TextField` with the content type set.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::multipart::TextField;
    ///
    /// let field = TextField::new("description".to_string(), "Long text...".to_string())
    ///     .with_content_type("text/plain; charset=utf-8".to_string());
    ///
    /// assert_eq!(field.content_type, Some("text/plain; charset=utf-8".to_string()));
    /// ```
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }
}
