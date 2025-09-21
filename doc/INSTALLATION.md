# Installation Guide

This guide will help you install and set up Ignitia, a blazing fast, lightweight web framework for Rust that ignites your development journey.

## Prerequisites

Before installing Ignitia, ensure you have the following prerequisites:

### Rust Installation
- **Rust**: Version 1.70.0 or later
- **Cargo**: Comes bundled with Rust

Install Rust using rustup (recommended):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify your Rust installation:
```bash
rustc --version
cargo --version
```

### System Requirements
- **Operating System**: Linux, macOS, or Windows
- **Memory**: At least 512MB available RAM
- **Network**: Internet connection for downloading dependencies

## Installation Methods

### Method 1: Using Cargo (Recommended)

Add Ignitia to your existing Rust project:

```bash
cargo add ignitia
```

Or manually add to your `Cargo.toml`:

```toml
[dependencies]
ignitia = "0.2.1"
```

### Method 2: From Source

Clone and build from the repository:

```bash
git clone https://github.com/AarambhDevHub/ignitia.git
cd ignitia
cargo build --release
```

## Feature Flags

Ignitia supports several optional features that you can enable based on your needs:

### Core Features
```toml
[dependencies]
ignitia = { version = "0.2.1", features = ["websocket", "tls"] }
```

### Available Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `websocket` | WebSocket support for real-time communication | `tokio-tungstenite`, `tungstenite` |
| `tls` | HTTPS/TLS support for secure connections | `tokio-rustls`, `rustls` |
| `self-signed` | Self-signed certificate generation (dev only) | `rcgen` |

### Feature Combinations

**For Web APIs:**
```toml
ignitia = { version = "0.2.1", features = ["tls"] }
```

**For Real-time Applications:**
```toml
ignitia = { version = "0.2.1", features = ["websocket", "tls"] }
```

**For Development:**
```toml
ignitia = { version = "0.2.1", features = ["websocket", "tls", "self-signed"] }
```

## Creating a New Project

### Step 1: Initialize Project
```bash
cargo new my-ignitia-app
cd my-ignitia-app
```

### Step 2: Configure Dependencies
Edit `Cargo.toml`:
```toml
[package]
name = "my-ignitia-app"
version = "0.1.0"
edition = "2021"

[dependencies]
ignitia = { version = "0.2.1", features = ["websocket"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Step 3: Create Basic Application
Create `src/main.rs`:
```rust
use ignitia::{Router, Server, Response};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();

    // Create router
    let router = Router::new()
        .get("/", || async {
            Ok(Response::text("Hello, Ignitia! 🔥"))
        })
        .get("/health", || async {
            Ok(Response::json(&serde_json::json!({
                "status": "healthy",
                "framework": "ignitia"
            }))?)
        });

    // Configure server
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let server = Server::new(router, addr);

    println!("🔥 Ignitia server blazing on http://{}", addr);

    // Start server
    server.ignitia().await?;
    Ok(())
}
```

### Step 4: Build and Run
```bash
cargo build
cargo run
```

## Verification

### Test Basic Functionality
1. **Start your application:**
   ```bash
   cargo run
   ```

2. **Test endpoints:**
   ```bash
   # Test basic endpoint
   curl http://127.0.0.1:3000/

   # Test JSON endpoint
   curl http://127.0.0.1:3000/health
   ```

3. **Expected outputs:**
   ```bash
   # From /
   Hello, Ignitia! 🔥

   # From /health
   {"status":"healthy","framework":"ignitia"}
   ```

### Verify Features

**WebSocket Support (if enabled):**
```rust
let router = Router::new()
    .websocket("/ws", |ws| async move {
        // WebSocket handler
        Ok(())
    });
```

**TLS Support (if enabled):**
```rust
let server = Server::new(router, addr)
    .enable_https("cert.pem", "key.pem")?;
```

## Advanced Installation Options

### Docker Installation

Create `Dockerfile`:
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my-ignitia-app /usr/local/bin/app

EXPOSE 3000
CMD ["app"]
```

Build and run:
```bash
docker build -t my-ignitia-app .
docker run -p 3000:3000 my-ignitia-app
```

### Production Optimizations

**Cargo.toml optimizations:**
```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

**Compile with optimizations:**
```bash
cargo build --release
```

## Environment-Specific Setup

### Development Environment
```toml
[dependencies]
ignitia = { version = "0.2.1", features = ["websocket", "self-signed"] }
tracing-subscriber = "0.3"
```

### Production Environment
```toml
[dependencies]
ignitia = { version = "0.2.1", features = ["tls"] }
```

## Troubleshooting

### Common Issues

**1. Compilation Errors**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build
```

**2. Feature Conflicts**
- Ensure compatible feature combinations
- Check dependency versions in `Cargo.lock`

**3. TLS Certificate Issues**
```bash
# For development, use self-signed certificates
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

**4. Port Already in Use**
```bash
# Find process using port
lsof -i :3000

# Kill process
kill -9 <PID>
```

**5. WebSocket Connection Issues**
- Ensure `websocket` feature is enabled
- Check firewall settings
- Verify client WebSocket implementation

### Platform-Specific Issues

**Windows:**
- Install Visual Studio Build Tools
- Use PowerShell or CMD for commands

**macOS:**
- Install Xcode Command Line Tools: `xcode-select --install`

**Linux:**
- Install build essentials: `sudo apt-get install build-essential`

## Next Steps

After successful installation:

1. **📖 Read the [Quick Start Guide](QUICK_START.md)** - Learn basic concepts
2. **🛣️ Explore [Routing Guide](ROUTING_GUIDE.md)** - Set up routes and handlers
3. **🔧 Check [Middleware Guide](MIDDLEWARE_GUIDE.md)** - Add middleware functionality
4. **🌐 See [Examples](EXAMPLES.md)** - View real-world examples

## Getting Help

If you encounter issues:

- **📚 Documentation**: Read the full documentation
- **💬 Community**: Join our Discord server
- **🐛 Issues**: Report bugs on GitHub
- **📧 Support**: Contact support@aarambhdevhub.com

## Version Compatibility

| Ignitia Version | Rust Version | Status |
|----------------|--------------|--------|
| 0.2.x | 1.70+ | Current |
| 0.1.x | 1.65+ | Maintenance |

***

**🔥 Ready to ignite your web development journey with Ignitia!**
