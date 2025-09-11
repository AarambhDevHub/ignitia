use ignitia::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {
    message: String,
    files: Vec<FileInfo>,
    form_data: FormData,
    upload_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    field_name: String,
    original_name: Option<String>,
    content_type: Option<String>,
    size: u64,
    saved_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    is_public: bool,
}

// Custom multipart configuration for different upload types
struct MultipartConfigs;

impl MultipartConfigs {
    // Small files (avatars, thumbnails) - keep in memory
    pub fn small_files() -> MultipartConfig {
        MultipartConfig {
            max_request_size: 5 * 1024 * 1024, // 5MB total
            max_field_size: 2 * 1024 * 1024,   // 2MB per field
            file_size_threshold: 1024 * 1024,  // 1MB before writing to disk
            max_fields: 10,                    // Max 10 fields
        }
    }

    // Large files (documents, videos) - immediate disk storage
    pub fn large_files() -> MultipartConfig {
        MultipartConfig {
            max_request_size: 100 * 1024 * 1024, // 100MB total
            max_field_size: 50 * 1024 * 1024,    // 50MB per field
            file_size_threshold: 64 * 1024,      // 64KB before writing to disk
            max_fields: 5,                       // Max 5 fields
        }
    }

    // Batch uploads - many small files
    pub fn batch_upload() -> MultipartConfig {
        MultipartConfig {
            max_request_size: 50 * 1024 * 1024, // 50MB total
            max_field_size: 5 * 1024 * 1024,    // 5MB per field
            file_size_threshold: 256 * 1024,    // 256KB before writing to disk
            max_fields: 50,                     // Max 50 fields
        }
    }
}

// Advanced file upload handler with custom multipart processing
async fn advanced_upload_handler(req: Request) -> Result<Response> {
    println!("🚀 Processing advanced multipart upload...");

    // Extract content type and determine config
    let content_type = req
        .header("content-type")
        .ok_or_else(|| Error::BadRequest("Missing Content-Type header".into()))?;

    if !content_type.starts_with("multipart/form-data") {
        return Err(Error::BadRequest(
            "Request is not multipart/form-data".into(),
        ));
    }

    // Extract boundary
    let boundary = extract_boundary(content_type)
        .ok_or_else(|| Error::BadRequest("Missing boundary in Content-Type".into()))?;

    // Determine upload type from headers or query params
    let upload_type = req.query("type").map(|s| s.as_str()).unwrap_or("standard");

    let config = match upload_type {
        "avatar" | "thumbnail" => MultipartConfigs::small_files(),
        "document" | "video" => MultipartConfigs::large_files(),
        "batch" => MultipartConfigs::batch_upload(),
        _ => MultipartConfig::default(),
    };

    println!("📋 Using config for '{}' uploads", upload_type);
    println!(
        "   Max request size: {} MB",
        config.max_request_size / 1024 / 1024
    );
    println!(
        "   Max field size: {} MB",
        config.max_field_size / 1024 / 1024
    );
    println!(
        "   File threshold: {} KB",
        config.file_size_threshold / 1024
    );
    println!("   Max fields: {}", config.max_fields);

    // Create multipart parser with custom config
    let mut multipart = Multipart::new(req.body.clone(), boundary, config);

    let mut files = Vec::new();
    let mut form_data = FormData {
        title: None,
        description: None,
        category: None,
        tags: Vec::new(),
        is_public: false,
    };

    // Create upload directory
    let upload_id = uuid::Uuid::new_v4().to_string();
    let upload_dir = format!("uploads/{}", upload_id);
    fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| Error::Internal(format!("Failed to create upload directory: {}", e)))?;

    println!("📁 Created upload directory: {}", upload_dir);

    // Process each field
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::Internal(format!("Multipart parsing error: {}", e)))?
    {
        let field_name = field.name().to_string();
        println!("🔍 Processing field: {}", field_name);

        if field.is_file() {
            // Handle file fields
            let file_info = process_file_field(field, &upload_dir).await?;
            println!(
                "✅ Saved file: {} ({} bytes)",
                file_info.saved_path, file_info.size
            );
            files.push(file_info);
        } else {
            // Handle form data fields
            process_form_field(field, &mut form_data).await?;
        }
    }

    // Validate uploaded data
    validate_upload(&files, &form_data)?;

    // Generate response
    let response = UploadResponse {
        message: format!("Successfully uploaded {} files", files.len()),
        files,
        form_data,
        upload_id,
    };

    println!("✨ Upload completed successfully!");
    Response::json(response)
}

// Process file fields with different strategies based on size
async fn process_file_field(field: Field, upload_dir: &str) -> Result<FileInfo> {
    // Extract all data BEFORE moving the field
    let field_name = field.name().to_string();
    let original_name = field.file_name().map(|s| s.to_string());
    let content_type = field.content_type().map(|s| s.to_string());

    // Generate unique filename
    let extension = original_name
        .as_ref()
        .and_then(|name| Path::new(name).extension())
        .and_then(|ext| ext.to_str())
        .unwrap_or("bin");

    let unique_name = format!(
        "{}_{}.{}",
        field_name,
        uuid::Uuid::new_v4().simple(),
        extension
    );

    let file_path = format!("{}/{}", upload_dir, unique_name);

    // NOW move the field to save it
    let file_field = field
        .save_to_file(&file_path)
        .await
        .map_err(|e| Error::Internal(format!("Failed to save file: {}", e)))?;

    Ok(FileInfo {
        field_name,
        original_name,
        content_type,
        size: file_field.size,
        saved_path: file_path,
    })
}

// Process form data fields with type conversion
// Process form data fields with type conversion
async fn process_form_field(field: Field, form_data: &mut FormData) -> Result<()> {
    // Get the field name BEFORE consuming the field
    let field_name = field.name().to_string(); // Clone the name

    // Now we can consume the field safely
    let text_value = field
        .text()
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to read field '{}': {}", field_name, e)))?;

    println!("📝 Form field: {} = {}", field_name, text_value);

    match field_name.as_str() {
        "title" => form_data.title = Some(text_value),
        "description" => form_data.description = Some(text_value),
        "category" => form_data.category = Some(text_value),
        "tags" => {
            // Parse comma-separated tags
            form_data.tags = text_value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "is_public" => {
            form_data.is_public = text_value.parse::<bool>().unwrap_or(false);
        }
        _ => {
            println!("⚠️ Unknown form field: {}", field_name);
        }
    }

    Ok(())
}

// Validate the uploaded data
fn validate_upload(files: &[FileInfo], form_data: &FormData) -> Result<()> {
    if files.is_empty() {
        return Err(Error::BadRequest("No files uploaded".into()));
    }

    // Validate file types
    for file in files {
        if let Some(content_type) = &file.content_type {
            if !is_allowed_content_type(content_type) {
                return Err(Error::BadRequest(format!(
                    "File type not allowed: {}",
                    content_type
                )));
            }
        }

        // Check file size limits based on content type
        validate_file_size(file)?;
    }

    // Validate form data
    if let Some(title) = &form_data.title {
        if title.trim().is_empty() {
            return Err(Error::BadRequest("Title cannot be empty".into()));
        }
        if title.len() > 100 {
            return Err(Error::BadRequest(
                "Title too long (max 100 characters)".into(),
            ));
        }
    }

    if form_data.tags.len() > 10 {
        return Err(Error::BadRequest("Too many tags (max 10)".into()));
    }

    Ok(())
}

// Check allowed content types
fn is_allowed_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "image/jpeg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "application/pdf"
            | "text/plain"
            | "text/csv"
            | "application/json"
            | "video/mp4"
            | "video/webm"
            | "audio/mpeg"
            | "audio/wav"
    )
}

// Validate file sizes based on content type
fn validate_file_size(file: &FileInfo) -> Result<()> {
    let max_size = match file.content_type.as_deref() {
        Some("image/jpeg") | Some("image/png") | Some("image/gif") | Some("image/webp") => {
            10 * 1024 * 1024 // 10MB for images
        }
        Some("video/mp4") | Some("video/webm") => {
            100 * 1024 * 1024 // 100MB for videos
        }
        Some("application/pdf") => {
            50 * 1024 * 1024 // 50MB for PDFs
        }
        _ => {
            5 * 1024 * 1024 // 5MB for other files
        }
    };

    if file.size > max_size {
        return Err(Error::BadRequest(format!(
            "File '{}' too large: {} bytes (max: {} bytes)",
            file.field_name, file.size, max_size
        )));
    }

    Ok(())
}

// Batch file processing handler
async fn batch_upload_handler(req: Request) -> Result<Response> {
    println!("📦 Processing batch upload...");

    let content_type = req
        .header("content-type")
        .ok_or_else(|| Error::BadRequest("Missing Content-Type header".into()))?;

    let boundary = extract_boundary(content_type)
        .ok_or_else(|| Error::BadRequest("Missing boundary in Content-Type".into()))?;

    // Use batch upload configuration
    let config = MultipartConfigs::batch_upload();
    let mut multipart = Multipart::new(req.body.clone(), boundary, config);

    let mut processed_files = Vec::new();
    let mut errors = Vec::new();

    let batch_id = uuid::Uuid::new_v4().to_string();
    let batch_dir = format!("uploads/batch_{}", batch_id);
    fs::create_dir_all(&batch_dir)
        .await
        .map_err(|e| Error::Internal(format!("Failed to create batch directory: {}", e)))?;

    // Process files concurrently in small batches
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::Internal(format!("Multipart parsing error: {}", e)))?
    {
        if field.is_file() {
            match process_file_field(field, &batch_dir).await {
                Ok(file_info) => {
                    println!(
                        "✅ Processed: {}",
                        file_info.original_name.as_deref().unwrap_or("unknown")
                    );
                    processed_files.push(file_info);
                }
                Err(e) => {
                    println!("❌ Failed to process file: {}", e);
                    errors.push(e.to_string());
                }
            }
        }
    }

    let response = serde_json::json!({
        "batch_id": batch_id,
        "processed": processed_files.len(),
        "errors": errors.len(),
        "files": processed_files,
        "errors_detail": errors
    });

    Response::json(response)
}

// Helper function to extract boundary from content-type
fn extract_boundary(content_type: &str) -> Option<String> {
    content_type.split(';').find_map(|part| {
        let part = part.trim();
        if part.starts_with("boundary=") {
            Some(part[9..].trim_matches('"').to_string())
        } else {
            None
        }
    })
}

// File management endpoints
async fn list_uploads_handler(_req: Request) -> Result<Response> {
    let uploads_dir = "uploads";
    let mut upload_dirs = Vec::new();

    if let Ok(mut entries) = fs::read_dir(uploads_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        upload_dirs.push(name.to_string());
                    }
                }
            }
        }
    }

    let response = serde_json::json!({
        "upload_directories": upload_dirs,
        "count": upload_dirs.len()
    });

    Response::json(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    // Create uploads directory
    fs::create_dir_all("uploads").await?;

    let router = Router::new()
        // Advanced multipart upload with custom configuration
        .post("/upload/advanced", advanced_upload_handler)

        // Batch upload handler
        .post("/upload/batch", batch_upload_handler)

        // Different upload types with different configs
        .post("/upload/avatar", |req: Request| async move {
            println!("🎭 Processing avatar upload (small file config)");
            advanced_upload_handler(req).await
        })

        .post("/upload/document", |req: Request| async move {
            println!("📄 Processing document upload (large file config)");
            advanced_upload_handler(req).await
        })

        // File management
        .get("/uploads", list_uploads_handler)

        // Health check
        .get("/", || async {
            Ok(Response::html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Ignitia Multipart Upload Example</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .upload-form { margin: 20px 0; padding: 20px; border: 1px solid #ddd; }
        input, textarea, select { margin: 5px 0; padding: 8px; width: 100%; }
        button { padding: 10px 20px; background: #007cba; color: white; border: none; cursor: pointer; }
    </style>
</head>
<body>
    <h1>🔥 Ignitia Advanced Multipart Upload</h1>

    <div class="upload-form">
        <h3>Advanced Upload</h3>
        <form action="/upload/advanced" method="post" enctype="multipart/form-data">
            <input type="text" name="title" placeholder="Title" required>
            <textarea name="description" placeholder="Description"></textarea>
            <select name="category">
                <option value="image">Image</option>
                <option value="document">Document</option>
                <option value="video">Video</option>
                <option value="other">Other</option>
            </select>
            <input type="text" name="tags" placeholder="Tags (comma-separated)">
            <label><input type="checkbox" name="is_public"> Make public</label>
            <input type="file" name="file1" multiple>
            <input type="file" name="file2">
            <button type="submit">Upload</button>
        </form>
    </div>

    <div class="upload-form">
        <h3>Batch Upload</h3>
        <form action="/upload/batch" method="post" enctype="multipart/form-data">
            <input type="file" name="files" multiple required>
            <button type="submit">Batch Upload</button>
        </form>
    </div>

    <div class="upload-form">
        <h3>Avatar Upload (Small Files)</h3>
        <form action="/upload/avatar?type=avatar" method="post" enctype="multipart/form-data">
            <input type="text" name="title" placeholder="Avatar name">
            <input type="file" name="avatar" accept="image/*" required>
            <button type="submit">Upload Avatar</button>
        </form>
    </div>

    <p><a href="/uploads">View Uploads</a></p>
</body>
</html>
            "#))
        })

        // Error handling
        .not_found(|| async {
            Ok(Response::json(serde_json::json!({
                "error": "Not Found",
                "message": "The requested endpoint was not found"
            })).unwrap().with_status(StatusCode::NOT_FOUND))
        });

    let server = Server::new(router, "127.0.0.1:3000".parse().unwrap());

    println!("🔥 Ignitia Advanced Multipart Server running on http://127.0.0.1:3000");
    println!("📋 Available endpoints:");
    println!("   POST /upload/advanced - Advanced multipart upload");
    println!("   POST /upload/batch - Batch file upload");
    println!("   POST /upload/avatar - Avatar upload (small files)");
    println!("   POST /upload/document - Document upload (large files)");
    println!("   GET  /uploads - List uploaded directories");
    println!("   GET  / - Upload form");

    server.ignitia().await.unwrap();

    Ok(())
}
