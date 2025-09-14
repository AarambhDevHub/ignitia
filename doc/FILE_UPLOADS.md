# File Uploads

Learn how to handle file uploads in Ignitia using multipart form data processing.

## Overview

Ignitia provides robust support for handling file uploads through multipart/form-data requests. The framework includes a streaming multipart parser that can handle large files efficiently, with configurable limits and automatic memory management.

## Key Features

- **Streaming Processing**: Handle large files without loading everything into memory
- **Configurable Limits**: Set maximum file sizes, request sizes, and field counts
- **Automatic File Management**: Smart handling of small vs large files
- **Type Safety**: Strong typing with extractors and error handling
- **Async Support**: Non-blocking file operations throughout

## Basic Usage

### Simple File Upload Handler

```rust
use ignitia::{Router, Response, Result, multipart::Multipart};

async fn upload_handler(mut multipart: Multipart) -> Result<Response> {
    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            let file_name = field.file_name().unwrap_or("unknown");
            let content_type = field.content_type().unwrap_or("application/octet-stream");

            println!("Uploading file: {} ({})", file_name, content_type);

            // Save file to disk
            let file_path = format!("./uploads/{}", file_name);
            let saved_file = field.save_to_file(&file_path).await?;

            println!("File saved to: {:?} ({} bytes)", saved_file.path, saved_file.size);
        } else {
            // Handle text fields
            let field_name = field.name();
            let text_value = field.text().await?;
            println!("Text field '{}': {}", field_name, text_value);
        }
    }

    Ok(Response::text("Upload successful!"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .post("/upload", upload_handler);

    ignitia::Server::new(app, "127.0.0.1:3000".parse()?)
        .ignitia()
        .await?;

    Ok(())
}
```

## Configuration

### MultipartConfig

Customize upload behavior with `MultipartConfig`:

```rust
use ignitia::multipart::{Multipart, MultipartConfig};

// Create custom configuration
let config = MultipartConfig {
    max_request_size: 50 * 1024 * 1024,  // 50MB total request
    max_field_size: 10 * 1024 * 1024,    // 10MB per field
    file_size_threshold: 1024 * 1024,    // 1MB before writing to disk
    max_fields: 50,                      // Maximum 50 fields
};

// Use in handler
async fn upload_with_config(request: Request) -> Result<Response> {
    // Extract boundary manually for custom config
    let content_type = request.header("content-type")
        .ok_or_else(|| Error::BadRequest("Missing Content-Type".into()))?;

    let boundary = extract_boundary(content_type)
        .ok_or_else(|| Error::BadRequest("Missing boundary".into()))?;

    let mut multipart = Multipart::new(request.body, boundary, config);

    // Process fields...
    Ok(Response::text("Upload complete"))
}
```

### Default Limits

```rust
// Default configuration values:
MultipartConfig {
    max_request_size: 10 * 1024 * 1024,  // 10MB
    max_field_size: 1 * 1024 * 1024,     // 1MB
    file_size_threshold: 256 * 1024,     // 256KB
    max_fields: 100,                     // 100 fields
}
```

## Working with Fields

### Field Types

```rust
use ignitia::multipart::{Field, FileField, TextField};

async fn process_field(field: Field) -> Result<()> {
    println!("Field name: {}", field.name());

    if let Some(filename) = field.file_name() {
        println!("Filename: {}", filename);
    }

    if let Some(content_type) = field.content_type() {
        println!("Content-Type: {}", content_type);
    }

    // Check field type
    if field.is_file() {
        // Handle as file
        let bytes = field.bytes().await?;
        println!("File size: {} bytes", bytes.len());
    } else {
        // Handle as text
        let text = field.text().await?;
        println!("Text value: {}", text);
    }

    Ok(())
}
```

### Saving Files

```rust
async fn save_upload(field: Field) -> Result<FileField> {
    let filename = field.file_name()
        .unwrap_or("unknown")
        .to_string();

    // Create safe filename
    let safe_filename = sanitize_filename(&filename);
    let file_path = format!("./uploads/{}", safe_filename);

    // Save to disk
    let saved_file = field.save_to_file(&file_path).await?;

    println!("Saved {} ({} bytes) to {:?}",
        filename, saved_file.size, saved_file.path);

    Ok(saved_file)
}

fn sanitize_filename(filename: &str) -> String {
    filename.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect()
}
```

## Advanced Examples

### Image Upload with Validation

```rust
use ignitia::{Response, Result, multipart::Multipart};
use std::path::Path;

async fn upload_image(mut multipart: Multipart) -> Result<Response> {
    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            // Validate image type
            let content_type = field.content_type().unwrap_or("");
            if !is_image_type(content_type) {
                return Ok(Response::new(400)
                    .with_body("Only image files are allowed"));
            }

            let filename = field.file_name().unwrap_or("image");
            let extension = get_extension(content_type);
            let safe_name = format!("{}_{}.{}",
                chrono::Utc::now().timestamp(),
                sanitize_filename(filename),
                extension
            );

            let file_path = format!("./images/{}", safe_name);
            let saved_file = field.save_to_file(&file_path).await?;

            return Ok(Response::json(serde_json::json!({
                "success": true,
                "filename": safe_name,
                "size": saved_file.size,
                "url": format!("/images/{}", safe_name)
            }))?);
        }
    }

    Ok(Response::new(400).with_body("No image file found"))
}

fn is_image_type(content_type: &str) -> bool {
    matches!(content_type,
        "image/jpeg" | "image/jpg" | "image/png" |
        "image/gif" | "image/webp"
    )
}

fn get_extension(content_type: &str) -> &str {
    match content_type {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin"
    }
}
```

### Multiple File Upload

```rust
async fn upload_multiple(mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();
    let mut text_fields = std::collections::HashMap::new();

    while let Some(field) = multipart.next_field().await? {
        if field.is_file() {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let file_path = format!("./uploads/{}", sanitize_filename(&filename));

            let saved_file = field.save_to_file(&file_path).await?;

            uploaded_files.push(serde_json::json!({
                "original_name": filename,
                "saved_path": saved_file.path,
                "size": saved_file.size,
                "content_type": saved_file.content_type
            }));
        } else {
            let name = field.name().to_string();
            let value = field.text().await?;
            text_fields.insert(name, value);
        }
    }

    Ok(Response::json(serde_json::json!({
        "files_uploaded": uploaded_files.len(),
        "files": uploaded_files,
        "form_data": text_fields
    }))?)
}
```

### Streaming Large Files

```rust
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn stream_large_file(mut multipart: Multipart) -> Result<Response> {
    while let Some(mut field) = multipart.next_field().await? {
        if field.is_file() {
            let filename = field.file_name().unwrap_or("large_file");
            let file_path = format!("./large_uploads/{}", sanitize_filename(filename));

            // Create file for streaming
            let mut file = File::create(&file_path).await
                .map_err(|e| Error::Internal(format!("Failed to create file: {}", e)))?;

            let mut total_bytes = 0u64;

            // Stream file chunks directly to disk
            while let Some(chunk) = field.chunk().await
                .map_err(|e| Error::Internal(format!("Stream error: {}", e)))?
            {
                file.write_all(&chunk).await
                    .map_err(|e| Error::Internal(format!("Write error: {}", e)))?;
                total_bytes += chunk.len() as u64;
            }

            file.sync_all().await
                .map_err(|e| Error::Internal(format!("Sync error: {}", e)))?;

            return Ok(Response::json(serde_json::json!({
                "success": true,
                "filename": filename,
                "bytes_written": total_bytes,
                "path": file_path
            }))?);
        }
    }

    Ok(Response::new(400).with_body("No file found"))
}
```

## Error Handling

### Common Upload Errors

```rust
use ignitia::multipart::MultipartError;

async fn upload_with_errors(mut multipart: Multipart) -> Result<Response> {
    match process_upload(&mut multipart).await {
        Ok(result) => Ok(Response::json(result)?),
        Err(e) => {
            let (status, message) = match e {
                Error::Custom(custom_err) => {
                    if let Some(multipart_err) = custom_err.downcast_ref::<MultipartError>() {
                        match multipart_err {
                            MultipartError::FieldTooLarge { field_name, max_size } => {
                                (413, format!("Field '{}' exceeds {}MB limit", field_name, max_size / 1024 / 1024))
                            },
                            MultipartError::RequestTooLarge { max_size } => {
                                (413, format!("Request exceeds {}MB limit", max_size / 1024 / 1024))
                            },
                            MultipartError::TooManyFields { max_fields } => {
                                (400, format!("Too many fields, maximum {}", max_fields))
                            },
                            MultipartError::InvalidBoundary => {
                                (400, "Invalid multipart boundary".to_string())
                            },
                            _ => (400, "Upload processing failed".to_string())
                        }
                    } else {
                        (500, "Internal server error".to_string())
                    }
                },
                _ => (500, "Unexpected error".to_string())
            };

            Ok(Response::new(status).with_body(message))
        }
    }
}
```

## HTML Form Example

### Client-Side Form

```html
<!DOCTYPE html>
<html>
<head>
    <title>File Upload</title>
</head>
<body>
    <form action="/upload" method="post" enctype="multipart/form-data">
        <div>
            <label for="title">Title:</label>
            <input type="text" id="title" name="title" required>
        </div>

        <div>
            <label for="description">Description:</label>
            <textarea id="description" name="description"></textarea>
        </div>

        <div>
            <label for="file">Choose file:</label>
            <input type="file" id="file" name="file" required>
        </div>

        <div>
            <label for="multiple">Multiple files:</label>
            <input type="file" id="multiple" name="multiple[]" multiple>
        </div>

        <button type="submit">Upload</button>
    </form>
</body>
</html>
```

## Best Practices

### Security Considerations

```rust
use std::path::Path;

fn validate_upload(field: &Field) -> Result<()> {
    // 1. Check file extension
    if let Some(filename) = field.file_name() {
        let allowed_extensions = ["jpg", "jpeg", "png", "gif", "pdf", "txt"];
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !allowed_extensions.contains(&extension.as_str()) {
            return Err(Error::BadRequest("File type not allowed".into()));
        }
    }

    // 2. Check content type
    if let Some(content_type) = field.content_type() {
        let allowed_types = [
            "image/jpeg", "image/png", "image/gif",
            "application/pdf", "text/plain"
        ];

        if !allowed_types.contains(&content_type) {
            return Err(Error::BadRequest("Content type not allowed".into()));
        }
    }

    // 3. Validate file name
    if let Some(filename) = field.file_name() {
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(Error::BadRequest("Invalid filename".into()));
        }
    }

    Ok(())
}
```

### Performance Tips

1. **Use streaming for large files** to avoid memory issues
2. **Configure appropriate thresholds** for memory vs disk usage
3. **Validate early** to reject invalid uploads quickly
4. **Use async file operations** to avoid blocking
5. **Implement cleanup** for failed uploads

### Directory Structure

```rust
use tokio::fs;

async fn ensure_upload_dirs() -> Result<()> {
    fs::create_dir_all("./uploads/images").await
        .map_err(|e| Error::Internal(format!("Failed to create upload dirs: {}", e)))?;
    fs::create_dir_all("./uploads/documents").await
        .map_err(|e| Error::Internal(format!("Failed to create upload dirs: {}", e)))?;
    Ok(())
}
```

This comprehensive guide covers file upload handling in Ignitia, from basic usage to advanced streaming scenarios with proper error handling and security considerations.
