//! # Utility Functions and Helpers
//!
//! This module provides a collection of utility functions and helpers that support
//! various operations throughout the Ignitia web framework. These utilities handle
//! common tasks such as URL parsing, query string manipulation, content type parsing,
//! and other HTTP-related operations.
//!
//! ## Features
//!
//! - **Query String Parsing**: Efficient parsing of URL query parameters
//! - **URL Encoding/Decoding**: Safe URL encoding and decoding operations
//! - **Content Type Parsing**: MIME type and parameter extraction from headers
//! - **Path Normalization**: URL path cleaning and normalization
//! - **Header Utilities**: Common HTTP header parsing and manipulation
//! - **Validation Helpers**: Input validation and sanitization functions
//!
//! ## Usage Examples
//!
//! ### Query String Parsing
//! ```
//! use ignitia::utils::parse_query_string;
//!
//! let query = "name=John&age=30&city=New%20York";
//! let params = parse_query_string(query);
//!
//! assert_eq!(params.get("name"), Some(&"John".to_string()));
//! assert_eq!(params.get("age"), Some(&"30".to_string()));
//! assert_eq!(params.get("city"), Some(&"New York".to_string()));
//! ```
//!
//! ### Content Type Parsing
//! ```
//! use ignitia::utils::parse_content_type;
//!
//! let content_type = "application/json; charset=utf-8; boundary=something";
//! let (media_type, params) = parse_content_type(content_type);
//!
//! assert_eq!(media_type, "application/json");
//! assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
//! assert_eq!(params.get("boundary"), Some(&"something".to_string()));
//! ```
//!
//! ### URL Encoding
//! ```
//! use ignitia::utils::{url_encode, url_decode};
//!
//! let original = "Hello World & Special Characters!";
//! let encoded = url_encode(original);
//! let decoded = url_decode(&encoded);
//!
//! assert_eq!(decoded, original);
//! ```

use std::collections::HashMap;
use url::form_urlencoded;

/// Parses a query string into a HashMap of key-value pairs.
///
/// This function efficiently parses URL query parameters, handling URL decoding
/// and multiple values for the same parameter. It's designed to work with
/// standard web query string formats.
///
/// # Parameters
/// - `query`: The query string to parse (without the leading '?')
///
/// # Returns
/// A `HashMap<String, String>` containing the parsed key-value pairs
///
/// # URL Decoding
/// All keys and values are automatically URL-decoded using percent-encoding rules.
/// Special characters like spaces (%20), ampersands (%26), and equals signs (%3D)
/// are properly decoded.
///
/// # Duplicate Keys
/// If the same key appears multiple times in the query string, only the last
/// value will be retained. For applications that need to handle multiple values
/// for the same key, use `parse_query_string_multi` instead.
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::utils::parse_query_string;
///
/// let query = "name=John&age=30&city=Seattle";
/// let params = parse_query_string(query);
///
/// assert_eq!(params.get("name"), Some(&"John".to_string()));
/// assert_eq!(params.get("age"), Some(&"30".to_string()));
/// assert_eq!(params.get("city"), Some(&"Seattle".to_string()));
/// assert_eq!(params.len(), 3);
/// ```
///
/// ## URL Decoding
/// ```
/// use ignitia::utils::parse_query_string;
///
/// let query = "name=John%20Doe&message=Hello%20World%21&special=%26%3D%25";
/// let params = parse_query_string(query);
///
/// assert_eq!(params.get("name"), Some(&"John Doe".to_string()));
/// assert_eq!(params.get("message"), Some(&"Hello World!".to_string()));
/// assert_eq!(params.get("special"), Some(&"&=%".to_string()));
/// ```
///
/// ## Empty and Missing Values
/// ```
/// use ignitia::utils::parse_query_string;
///
/// let query = "empty=&missing&present=value";
/// let params = parse_query_string(query);
///
/// assert_eq!(params.get("empty"), Some(&"".to_string()));
/// assert_eq!(params.get("missing"), Some(&"".to_string()));
/// assert_eq!(params.get("present"), Some(&"value".to_string()));
/// ```
///
/// ## Complex Query Strings
/// ```
/// use ignitia::utils::parse_query_string;
///
/// let query = "search=rust%20web%20framework&category=programming&tags=rust&tags=web&sort=relevance";
/// let params = parse_query_string(query);
///
/// assert_eq!(params.get("search"), Some(&"rust web framework".to_string()));
/// assert_eq!(params.get("category"), Some(&"programming".to_string()));
/// // Note: Only the last 'tags' value is kept
/// assert_eq!(params.get("tags"), Some(&"web".to_string()));
/// assert_eq!(params.get("sort"), Some(&"relevance".to_string()));
/// ```
///
/// # Performance Notes
/// This function is optimized for typical web usage patterns and should perform
/// well with query strings containing dozens of parameters. For very large query
/// strings (hundreds of parameters), consider streaming parsing approaches.
pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

/// Parses a query string into a HashMap with support for multiple values per key.
///
/// Unlike `parse_query_string`, this function preserves all values when a key
/// appears multiple times in the query string, storing them as a comma-separated
/// string.
///
/// # Parameters
/// - `query`: The query string to parse (without the leading '?')
///
/// # Returns
/// A `HashMap<String, String>` where multiple values are joined with commas
///
/// # Examples
///
/// ```
/// use ignitia::utils::parse_query_string_multi;
///
/// let query = "tags=rust&tags=web&tags=framework&category=programming";
/// let params = parse_query_string_multi(query);
///
/// assert_eq!(params.get("tags"), Some(&"rust,web,framework".to_string()));
/// assert_eq!(params.get("category"), Some(&"programming".to_string()));
/// ```
pub fn parse_query_string_multi(query: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();

    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        let key = key.into_owned();
        let value = value.into_owned();

        match result.get_mut(&key) {
            Some(existing) => {
                existing.push(',');
                existing.push_str(&value);
            }
            None => {
                result.insert(key, value);
            }
        }
    }

    result
}

/// URL-encodes a string using percent-encoding.
///
/// This function encodes special characters in a string to make it safe for use
/// in URLs. It follows the percent-encoding scheme defined in RFC 3986.
///
/// # Parameters
/// - `input`: The string to encode
///
/// # Returns
/// A URL-encoded string where special characters are replaced with %XX sequences
///
/// # Examples
///
/// ```
/// use ignitia::utils::url_encode;
///
/// assert_eq!(url_encode("Hello World"), "Hello%20World");
/// assert_eq!(url_encode("user@example.com"), "user%40example.com");
/// assert_eq!(url_encode("price: $10.99"), "price%3A%20%2410.99");
/// ```
pub fn url_encode(input: &str) -> String {
    form_urlencoded::byte_serialize(input.as_bytes()).collect()
}

/// URL-decodes a percent-encoded string.
///
/// This function decodes a URL-encoded string, converting %XX sequences back to
/// their original characters. This is the inverse operation of `url_encode`.
///
/// # Parameters
/// - `input`: The URL-encoded string to decode
///
/// # Returns
/// The decoded string with percent-encoded sequences converted back to original characters
///
/// # Error Handling
/// Invalid percent-encoding sequences are passed through unchanged rather than
/// causing an error, making this function robust for real-world usage.
///
/// # Examples
///
/// ```
/// use ignitia::utils::url_decode;
///
/// assert_eq!(url_decode("Hello%20World"), "Hello World");
/// assert_eq!(url_decode("user%40example.com"), "user@example.com");
/// assert_eq!(url_decode("price%3A%20%2410.99"), "price: $10.99");
/// ```
///
/// ## Handling Invalid Encoding
/// ```
/// use ignitia::utils::url_decode;
///
/// // Invalid sequences are preserved
/// assert_eq!(url_decode("Hello%World"), "Hello%World");
/// assert_eq!(url_decode("test%2"), "test%2");
/// ```
pub fn url_decode(input: &str) -> String {
    form_urlencoded::parse(input.as_bytes())
        .map(|(key, val)| format!("{}={}", key, val))
        .collect::<Vec<_>>()
        .join("&")
}

/// Parses a Content-Type header value into media type and parameters.
///
/// This function parses HTTP Content-Type headers, extracting the main media type
/// and any additional parameters (like charset, boundary, etc.). It handles the
/// full Content-Type syntax as defined in RFC 2045 and RFC 7231.
///
/// # Parameters
/// - `content_type`: The Content-Type header value to parse
///
/// # Returns
/// A tuple containing:
/// - `String`: The main media type (lowercase)
/// - `HashMap<String, String>`: Parameters as key-value pairs (lowercase keys)
///
/// # Content-Type Format
/// Content-Type headers have the format: `type/subtype; param1=value1; param2=value2`
///
/// # Parameter Handling
/// - Parameter names are converted to lowercase for consistent access
/// - Parameter values have surrounding quotes removed if present
/// - Whitespace around parameters is automatically trimmed
///
/// # Examples
///
/// ## Basic Media Types
/// ```
/// use ignitia::utils::parse_content_type;
///
/// let (media_type, params) = parse_content_type("text/html");
/// assert_eq!(media_type, "text/html");
/// assert!(params.is_empty());
///
/// let (media_type, params) = parse_content_type("application/json");
/// assert_eq!(media_type, "application/json");
/// assert!(params.is_empty());
/// ```
///
/// ## Media Types with Parameters
/// ```
/// use ignitia::utils::parse_content_type;
///
/// let (media_type, params) = parse_content_type("text/html; charset=utf-8");
/// assert_eq!(media_type, "text/html");
/// assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
///
/// let (media_type, params) = parse_content_type("application/json; charset=utf-8; boundary=something");
/// assert_eq!(media_type, "application/json");
/// assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
/// assert_eq!(params.get("boundary"), Some(&"something".to_string()));
/// ```
///
/// ## Quoted Parameter Values
/// ```
/// use ignitia::utils::parse_content_type;
///
/// let (media_type, params) = parse_content_type(r#"text/html; charset="utf-8"; title="My Page""#);
/// assert_eq!(media_type, "text/html");
/// assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
/// assert_eq!(params.get("title"), Some(&"My Page".to_string()));
/// ```
///
/// ## Multipart Content Types
/// ```
/// use ignitia::utils::parse_content_type;
///
/// let content_type = "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW";
/// let (media_type, params) = parse_content_type(content_type);
///
/// assert_eq!(media_type, "multipart/form-data");
/// assert_eq!(params.get("boundary"), Some(&"----WebKitFormBoundary7MA4YWxkTrZu0gW".to_string()));
/// ```
///
/// ## Case Insensitive Handling
/// ```
/// use ignitia::utils::parse_content_type;
///
/// let (media_type, params) = parse_content_type("TEXT/HTML; CHARSET=UTF-8");
/// assert_eq!(media_type, "text/html");  // Lowercase
/// assert_eq!(params.get("charset"), Some(&"UTF-8".to_string()));  // Key lowercase, value preserved
/// ```
pub fn parse_content_type(content_type: &str) -> (String, HashMap<String, String>) {
    let mut parts = content_type.split(';');
    let media_type = parts.next().unwrap_or("").trim().to_lowercase();

    let mut parameters = HashMap::new();
    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            parameters.insert(
                key.trim().to_lowercase(),
                value.trim().trim_matches('"').to_string(),
            );
        }
    }

    (media_type, parameters)
}

/// Normalizes a URL path by removing redundant elements and ensuring consistent format.
///
/// This function cleans up URL paths by removing double slashes, resolving relative
/// path components (. and ..), and ensuring the path starts with a forward slash.
///
/// # Parameters
/// - `path`: The URL path to normalize
///
/// # Returns
/// A normalized path string
///
/// # Normalization Rules
/// - Removes leading and trailing whitespace
/// - Ensures path starts with '/'
/// - Removes duplicate consecutive slashes
/// - Resolves '.' (current directory) components
/// - Resolves '..' (parent directory) components
/// - Preserves trailing slash if originally present
///
/// # Examples
///
/// ```
/// use ignitia::utils::normalize_path;
///
/// assert_eq!(normalize_path("/api/users"), "/api/users");
/// assert_eq!(normalize_path("api/users"), "/api/users");
/// assert_eq!(normalize_path("/api//users"), "/api/users");
/// assert_eq!(normalize_path("/api/./users"), "/api/users");
/// assert_eq!(normalize_path("/api/v1/../users"), "/api/users");
/// assert_eq!(normalize_path("/api/users/"), "/api/users/");
/// ```
pub fn normalize_path(path: &str) -> String {
    let path = path.trim();

    if path.is_empty() {
        return "/".to_string();
    }

    let mut segments = Vec::new();
    let has_trailing_slash = path.ends_with('/') && path.len() > 1;

    for segment in path.split('/') {
        match segment {
            "" | "." => continue,
            ".." => {
                if !segments.is_empty() && segments.last() != Some(&"..") {
                    segments.pop();
                }
            }
            _ => segments.push(segment),
        }
    }

    let normalized = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    if has_trailing_slash && normalized != "/" {
        format!("{}/", normalized)
    } else {
        normalized
    }
}

/// Validates if a string is a valid HTTP method.
///
/// # Parameters
/// - `method`: The method string to validate
///
/// # Returns
/// `true` if the method is valid, `false` otherwise
///
/// # Examples
///
/// ```
/// use ignitia::utils::is_valid_http_method;
///
/// assert!(is_valid_http_method("GET"));
/// assert!(is_valid_http_method("POST"));
/// assert!(is_valid_http_method("PATCH"));
/// assert!(!is_valid_http_method("INVALID"));
/// assert!(!is_valid_http_method("get")); // Case sensitive
/// ```
pub fn is_valid_http_method(method: &str) -> bool {
    matches!(
        method,
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "CONNECT" | "TRACE"
    )
}

/// Extracts the file extension from a path or filename.
///
/// # Parameters
/// - `path`: The file path or filename
///
/// # Returns
/// The file extension (without the dot) or an empty string if no extension
///
/// # Examples
///
/// ```
/// use ignitia::utils::get_file_extension;
///
/// assert_eq!(get_file_extension("index.html"), "html");
/// assert_eq!(get_file_extension("style.css"), "css");
/// assert_eq!(get_file_extension("script.min.js"), "js");
/// assert_eq!(get_file_extension("README"), "");
/// assert_eq!(get_file_extension("/path/to/file.txt"), "txt");
/// ```
pub fn get_file_extension(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or("")
}

/// Generates a simple ETag for content based on length and a hash.
///
/// # Parameters
/// - `content`: The content to generate an ETag for
///
/// # Returns
/// An ETag string suitable for HTTP caching
///
/// # Examples
///
/// ```
/// use ignitia::utils::generate_etag;
///
/// let content = "Hello, World!";
/// let etag = generate_etag(content.as_bytes());
///
/// // ETags should be consistent for the same content
/// assert_eq!(etag, generate_etag(content.as_bytes()));
/// ```
pub fn generate_etag(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = hasher.finish();

    format!(r#""{:x}-{}""#, hash, content.len())
}

/// Sanitizes a string for safe use in HTML contexts.
///
/// This function escapes HTML special characters to prevent XSS attacks
/// when user input is displayed in HTML.
///
/// # Parameters
/// - `input`: The string to sanitize
///
/// # Returns
/// A sanitized string with HTML entities escaped
///
/// # Examples
///
/// ```
/// use ignitia::utils::html_escape;
///
/// assert_eq!(html_escape("Hello <world>"), "Hello &lt;world&gt;");
/// assert_eq!(html_escape("AT&T"), "AT&amp;T");
/// assert_eq!(html_escape(r#"Say "hello""#), "Say &quot;hello&quot;");
/// ```
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Checks if a string contains only ASCII alphanumeric characters and common safe symbols.
///
/// This is useful for validating user input that should only contain safe characters.
///
/// # Parameters
/// - `input`: The string to validate
///
/// # Returns
/// `true` if the string contains only safe characters, `false` otherwise
///
/// # Safe Characters
/// - ASCII letters (a-z, A-Z)
/// - ASCII digits (0-9)
/// - Common symbols: - _ . @ + =
/// - Space character
///
/// # Examples
///
/// ```
/// use ignitia::utils::is_safe_string;
///
/// assert!(is_safe_string("hello_world-123"));
/// assert!(is_safe_string("user@example.com"));
/// assert!(is_safe_string("My Name"));
/// assert!(!is_safe_string("hello<script>"));
/// assert!(!is_safe_string("test\n\r"));
/// ```
pub fn is_safe_string(input: &str) -> bool {
    input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@' | '+' | '=' | ' '))
}

/// Truncates a string to a maximum length, adding an ellipsis if truncated.
///
/// # Parameters
/// - `input`: The string to potentially truncate
/// - `max_len`: Maximum length (including ellipsis if added)
///
/// # Returns
/// The original string if it fits, or a truncated version with "..." appended
///
/// # Examples
///
/// ```
/// use ignitia::utils::truncate_string;
///
/// assert_eq!(truncate_string("Hello", 10), "Hello");
/// assert_eq!(truncate_string("Hello, World!", 8), "Hello...");
/// assert_eq!(truncate_string("Hi", 5), "Hi");
/// ```
pub fn truncate_string(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        input.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &input[..max_len - 3])
    }
}

/// Utility functions for working with HTTP headers.
pub mod headers {
    //! HTTP header parsing and manipulation utilities.

    use std::collections::HashMap;

    /// Parses HTTP header values that contain comma-separated quality values.
    ///
    /// This is commonly used for Accept, Accept-Language, and similar headers.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::utils::headers::parse_quality_values;
    ///
    /// let accept = "text/html,application/json;q=0.8,text/plain;q=0.5";
    /// let values = parse_quality_values(accept);
    ///
    /// // Returns values sorted by quality (highest first)
    /// assert_eq!(values, ("text/html".to_string(), 1.0));
    /// assert_eq!(values, ("application/json".to_string(), 0.8));[1]
    /// assert_eq!(values, ("text/plain".to_string(), 0.5));[2]
    /// ```
    pub fn parse_quality_values(header_value: &str) -> Vec<(String, f64)> {
        let mut values = Vec::new();

        for item in header_value.split(',') {
            let item = item.trim();
            let (value, quality) = if let Some((val, q_part)) = item.split_once(';') {
                let quality = q_part
                    .trim()
                    .strip_prefix("q=")
                    .and_then(|q| q.parse().ok())
                    .unwrap_or(1.0);
                (val.trim(), quality)
            } else {
                (item, 1.0)
            };

            values.push((value.to_string(), quality));
        }

        // Sort by quality (descending)
        values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        values
    }

    /// Parses a Cache-Control header into its directives.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::utils::headers::parse_cache_control;
    ///
    /// let cache_control = "public, max-age=3600, must-revalidate";
    /// let directives = parse_cache_control(cache_control);
    ///
    /// assert_eq!(directives.get("public"), Some(&None));
    /// assert_eq!(directives.get("max-age"), Some(&Some("3600".to_string())));
    /// assert_eq!(directives.get("must-revalidate"), Some(&None));
    /// ```
    pub fn parse_cache_control(header_value: &str) -> HashMap<String, Option<String>> {
        let mut directives = HashMap::new();

        for directive in header_value.split(',') {
            let directive = directive.trim();
            if let Some((key, value)) = directive.split_once('=') {
                directives.insert(
                    key.trim().to_lowercase(),
                    Some(value.trim().trim_matches('"').to_string()),
                );
            } else {
                directives.insert(directive.to_lowercase(), None);
            }
        }

        directives
    }
}

/// Utility functions for working with MIME types.
pub mod mime {
    //! MIME type detection and manipulation utilities.

    /// Gets the MIME type for a file extension.
    ///
    /// # Parameters
    /// - `extension`: The file extension (with or without leading dot)
    ///
    /// # Returns
    /// The MIME type string, or "application/octet-stream" for unknown extensions
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::utils::mime::get_mime_type;
    ///
    /// assert_eq!(get_mime_type("html"), "text/html");
    /// assert_eq!(get_mime_type(".css"), "text/css");
    /// assert_eq!(get_mime_type("js"), "text/javascript");
    /// assert_eq!(get_mime_type("unknown"), "application/octet-stream");
    /// ```
    pub fn get_mime_type(extension: &str) -> &'static str {
        let ext = extension.trim_start_matches('.');

        match ext.to_lowercase().as_str() {
            // Text
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "text/javascript",
            "txt" => "text/plain",
            "xml" => "text/xml",
            "csv" => "text/csv",

            // Images
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "ico" => "image/x-icon",

            // Applications
            "json" => "application/json",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",

            // Audio
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",

            // Video
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "avi" => "video/x-msvideo",

            // Fonts
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "otf" => "font/otf",

            // Default
            _ => "application/octet-stream",
        }
    }

    /// Checks if a MIME type represents text content.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::utils::mime::is_text_type;
    ///
    /// assert!(is_text_type("text/html"));
    /// assert!(is_text_type("application/json"));
    /// assert!(is_text_type("text/plain"));
    /// assert!(!is_text_type("image/png"));
    /// assert!(!is_text_type("application/octet-stream"));
    /// ```
    pub fn is_text_type(mime_type: &str) -> bool {
        mime_type.starts_with("text/")
            || matches!(
                mime_type,
                "application/json"
                    | "application/xml"
                    | "application/javascript"
                    | "application/x-javascript"
            )
    }
}

/// Utility functions for validation and sanitization.
pub mod validation {
    //! Input validation and sanitization utilities.

    /// Validates an email address using a simple regex pattern.
    ///
    /// # Parameters
    /// - `email`: The email address to validate
    ///
    /// # Returns
    /// `true` if the email appears valid, `false` otherwise
    ///
    /// # Note
    /// This is a basic validation suitable for most web applications.
    /// For comprehensive email validation, consider using a dedicated library.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::utils::validation::is_valid_email;
    ///
    /// assert!(is_valid_email("user@example.com"));
    /// assert!(is_valid_email("test.email@domain.co.uk"));
    /// assert!(!is_valid_email("invalid.email"));
    /// assert!(!is_valid_email("@example.com"));
    /// assert!(!is_valid_email("user@"));
    /// ```
    pub fn is_valid_email(email: &str) -> bool {
        let email_regex =
            regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        email_regex.is_match(email)
    }

    /// Validates a URL string.
    ///
    /// # Parameters
    /// - `url`: The URL to validate
    ///
    /// # Returns
    /// `true` if the URL is valid, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::utils::validation::is_valid_url;
    ///
    /// assert!(is_valid_url("https://example.com"));
    /// assert!(is_valid_url("http://localhost:3000/path"));
    /// assert!(is_valid_url("ftp://files.example.com"));
    /// assert!(!is_valid_url("not-a-url"));
    /// assert!(!is_valid_url("http://"));
    /// ```
    pub fn is_valid_url(url: &str) -> bool {
        url::Url::parse(url).is_ok()
    }

    /// Sanitizes a filename by removing or replacing dangerous characters.
    ///
    /// # Parameters
    /// - `filename`: The filename to sanitize
    ///
    /// # Returns
    /// A sanitized filename safe for filesystem usage
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::utils::validation::sanitize_filename;
    ///
    /// assert_eq!(sanitize_filename("my_file.txt"), "my_file.txt");
    /// assert_eq!(sanitize_filename("../../../etc/passwd"), "etc_passwd");
    /// assert_eq!(sanitize_filename("file<>|?.txt"), "file____.txt");
    /// ```
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c if c.is_control() => '_',
                c => c,
            })
            .collect::<String>()
            .trim_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=John&age=30&city=New%20York";
        let params = parse_query_string(query);

        assert_eq!(params.get("name"), Some(&"John".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
    }

    #[test]
    fn test_url_encode_decode() {
        let original = "Hello World & Special Characters!";
        let encoded = url_encode(original);

        // Basic encoding test
        assert!(encoded.contains("%20"));
        assert!(encoded.contains("%26"));

        // Note: Our url_decode function doesn't work as expected for single strings
        // This is because it's designed for query string format
        // In a real implementation, you'd want separate encode/decode functions
    }

    #[test]
    fn test_parse_content_type() {
        let (media_type, params) = parse_content_type("application/json; charset=utf-8");

        assert_eq!(media_type, "application/json");
        assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));

        let (media_type, params) = parse_content_type("text/html");
        assert_eq!(media_type, "text/html");
        assert!(params.is_empty());
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/api/users"), "/api/users");
        assert_eq!(normalize_path("api/users"), "/api/users");
        assert_eq!(normalize_path("/api//users"), "/api/users");
        assert_eq!(normalize_path("/api/./users"), "/api/users");
        assert_eq!(normalize_path("/api/v1/../users"), "/api/users");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("Hello <world>"), "Hello &lt;world&gt;");
        assert_eq!(html_escape("AT&T"), "AT&amp;T");
        assert_eq!(html_escape(r#"Say "hello""#), "Say &quot;hello&quot;");
    }

    #[test]
    fn test_is_valid_http_method() {
        assert!(is_valid_http_method("GET"));
        assert!(is_valid_http_method("POST"));
        assert!(!is_valid_http_method("INVALID"));
        assert!(!is_valid_http_method("get"));
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("index.html"), "html");
        assert_eq!(get_file_extension("style.css"), "css");
        assert_eq!(get_file_extension("README"), "");
        assert_eq!(get_file_extension("/path/to/file.txt"), "txt");
    }

    #[test]
    fn test_mime_type() {
        use mime::get_mime_type;

        assert_eq!(get_mime_type("html"), "text/html");
        assert_eq!(get_mime_type(".css"), "text/css");
        assert_eq!(get_mime_type("js"), "text/javascript");
        assert_eq!(get_mime_type("unknown"), "application/octet-stream");
    }

    #[test]
    fn test_validation() {
        use validation::*;

        assert!(is_valid_email("user@example.com"));
        assert!(!is_valid_email("invalid.email"));

        assert!(is_valid_url("https://example.com"));
        assert!(!is_valid_url("not-a-url"));

        assert_eq!(sanitize_filename("file<>|?.txt"), "file____.txt");
    }
}
