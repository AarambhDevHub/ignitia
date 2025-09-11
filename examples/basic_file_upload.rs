use ignitia::{Method, Multipart, Response, Router, Server, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {
    success: bool,
    message: String,
    files: Vec<FileInfo>,
    form_data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    field_name: String,
    file_name: Option<String>,
    content_type: Option<String>,
    size: u64,
    saved_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    // Create upload directory
    fs::create_dir_all("uploads").await?;

    let router = Router::new()
        .get("/", serve_upload_form)
        .post("/upload", handle_file_upload)
        .get("/uploads/:filename", serve_uploaded_file);

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let server = Server::new(router, addr);

    println!("🔥 Server running on http://127.0.0.1:3000");

    server.ignitia().await?;
    Ok(())
}

// ✅ FIXED: Now using Multipart extractor directly
async fn handle_file_upload(mut multipart: Multipart) -> ignitia::Result<Response> {
    println!("📤 Processing file upload...");

    let mut files = Vec::new();
    let mut form_data = HashMap::new();

    // Process each field
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ignitia::Error::BadRequest(format!("Failed to parse multipart field: {}", e))
    })? {
        let field_name = field.name().to_string();

        if field.is_file() {
            // Extract all info BEFORE calling save_to_file()
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().map(|s| s.to_string());
            // Save file to uploads directory
            let timestamp = chrono::Utc::now().timestamp();
            let safe_filename = format!("{}_{}", timestamp, sanitize_filename(&file_name));
            let file_path = format!("uploads/{}", safe_filename);

            // Now call save_to_file (this consumes the field)
            let file_field = field
                .save_to_file(&file_path)
                .await
                .map_err(|e| ignitia::Error::Internal(format!("Failed to save file: {}", e)))?;

            files.push(FileInfo {
                field_name: field_name.clone(),
                file_name: Some(file_name),
                content_type,
                size: file_field.size,
                saved_path: file_path,
            });

            println!(
                "📁 Saved file: {} ({} bytes)",
                safe_filename, file_field.size
            );
        } else {
            // Handle form field
            let value = field.text().await.map_err(|e| {
                ignitia::Error::BadRequest(format!("Failed to read text field: {}", e))
            })?;

            form_data.insert(field_name.clone(), value.clone());
            println!("📝 Form field: {} = {}", field_name, value);
        }
    }

    let response = UploadResponse {
        success: true,
        message: format!("Successfully uploaded {} files", files.len()),
        files,
        form_data,
    };

    Response::json(response)
}

// Serve the HTML upload form
async fn serve_upload_form() -> ignitia::Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ignitia File Upload Demo</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        .container {
            background: white;
            padding: 2rem;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            text-align: center;
            margin-bottom: 2rem;
        }
        .form-group {
            margin-bottom: 1.5rem;
        }
        label {
            display: block;
            margin-bottom: 0.5rem;
            font-weight: 600;
            color: #555;
        }
        input[type="text"], input[type="email"], textarea, input[type="file"], select {
            width: 100%;
            padding: 0.75rem;
            border: 2px solid #e1e5e9;
            border-radius: 5px;
            font-size: 1rem;
            transition: border-color 0.3s;
            box-sizing: border-box;
        }
        input:focus, textarea:focus, select:focus {
            outline: none;
            border-color: #667eea;
        }
        textarea {
            resize: vertical;
            height: 100px;
        }
        .file-upload {
            border: 2px dashed #ccc;
            border-radius: 10px;
            padding: 2rem;
            text-align: center;
            transition: border-color 0.3s;
        }
        .file-upload.dragover {
            border-color: #667eea;
            background-color: #f8f9ff;
        }
        button {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 1rem 2rem;
            border: none;
            border-radius: 5px;
            font-size: 1rem;
            cursor: pointer;
            width: 100%;
            transition: transform 0.2s;
        }
        button:hover {
            transform: translateY(-2px);
        }
        .result {
            margin-top: 2rem;
            padding: 1rem;
            border-radius: 5px;
            display: none;
        }
        .result.success {
            background-color: #d4edda;
            border: 1px solid #c3e6cb;
            color: #155724;
        }
        .result.error {
            background-color: #f8d7da;
            border: 1px solid #f5c6cb;
            color: #721c24;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔥 Ignitia File Upload Demo</h1>

        <form id="uploadForm" enctype="multipart/form-data">
            <div class="form-group">
                <label for="name">Your Name:</label>
                <input type="text" id="name" name="name" required>
            </div>

            <div class="form-group">
                <label for="email">Email:</label>
                <input type="email" id="email" name="email" required>
            </div>

            <div class="form-group">
                <label for="description">Description:</label>
                <textarea id="description" name="description" placeholder="Tell us about your files..."></textarea>
            </div>

            <div class="form-group">
                <label for="category">Category:</label>
                <select id="category" name="category">
                    <option value="image">Image</option>
                    <option value="document">Document</option>
                    <option value="video">Video</option>
                    <option value="other">Other</option>
                </select>
            </div>

            <div class="form-group">
                <label>Upload Files:</label>
                <div class="file-upload" id="fileUpload">
                    <input type="file" id="files" name="files" multiple accept="*/*">
                    <div>📁 Click to select files or drag and drop here</div>
                    <div>Maximum file size: 10MB per file</div>
                </div>
            </div>

            <button type="submit">🚀 Upload Files</button>
        </form>

        <div id="result" class="result"></div>
    </div>

    <script>
        const form = document.getElementById('uploadForm');
        const fileUpload = document.getElementById('fileUpload');
        const fileInput = document.getElementById('files');
        const result = document.getElementById('result');

        // Drag and drop functionality
        fileUpload.addEventListener('dragover', (e) => {
            e.preventDefault();
            fileUpload.classList.add('dragover');
        });

        fileUpload.addEventListener('dragleave', () => {
            fileUpload.classList.remove('dragover');
        });

        fileUpload.addEventListener('drop', (e) => {
            e.preventDefault();
            fileUpload.classList.remove('dragover');
            fileInput.files = e.dataTransfer.files;
        });

        form.addEventListener('submit', async (e) => {
            e.preventDefault();

            const formData = new FormData(form);

            result.style.display = 'none';

            try {
                const response = await fetch('/upload', {
                    method: 'POST',
                    body: formData
                });

                const text = await response.text();
                console.log('Response text:', text); // For debugging

                let data;
                try {
                    data = JSON.parse(text);
                } catch (parseError) {
                    throw new Error(`Server returned invalid JSON: ${text}`);
                }

                if (data.success) {
                    result.className = 'result success';
                    result.innerHTML = `
                        <h3>✅ Upload Successful!</h3>
                        <p>${data.message}</p>
                        <h4>Uploaded Files:</h4>
                        <ul>
                            ${data.files.map(file => `
                                <li>
                                    <strong>${file.file_name || 'Unknown'}</strong>
                                    (${file.size} bytes, ${file.content_type || 'unknown type'})
                                    <br><a href="/uploads/${file.file_name}" target="_blank">View File</a>
                                </li>
                            `).join('')}
                        </ul>
                        <h4>Form Data:</h4>
                        <ul>
                            ${Object.entries(data.form_data).map(([key, value]) =>
                                `<li><strong>${key}:</strong> ${value}</li>`
                            ).join('')}
                        </ul>
                    `;
                } else {
                    result.className = 'result error';
                    result.innerHTML = `<h3>❌ Upload Failed</h3><p>${data.message}</p>`;
                }
            } catch (error) {
                console.error('Upload error:', error);
                result.className = 'result error';
                result.innerHTML = `<h3>❌ Upload Error</h3><p>${error.message}</p>`;
            }

            result.style.display = 'block';
        });
    </script>
</body>
</html>
    "#;

    Ok(Response::html(html))
}

// Serve uploaded files
async fn serve_uploaded_file(
    ignitia::Path(params): ignitia::Path<HashMap<String, String>>,
) -> ignitia::Result<Response> {
    let filename = params
        .get("filename")
        .ok_or_else(|| ignitia::Error::BadRequest("Missing filename".into()))?;

    let file_path = format!("uploads/{}", sanitize_filename(filename));

    match fs::read(&file_path).await {
        Ok(contents) => {
            let content_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            let mut response = Response::new(StatusCode::OK);
            response
                .headers
                .insert("content-type", content_type.parse().unwrap());
            response.body = contents.into();
            Ok(response)
        }
        Err(_) => Ok(Response::new(StatusCode::NOT_FOUND)),
    }
}

// Helper function
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}
