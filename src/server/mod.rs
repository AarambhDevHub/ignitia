pub mod connection;

use crate::{Request, Response, Router};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub struct Server {
    router: Arc<Router>,
    addr: SocketAddr,
}

impl Server {
    pub fn new(router: Router, addr: SocketAddr) -> Self {
        Self {
            router: Arc::new(router),
            addr,
        }
    }

    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("🔥 ignitia server blazing on http://{}", self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = Arc::clone(&self.router);

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let router = Arc::clone(&router);
                    async move { handle_request_with_websocket_support(router, req).await }
                });

                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    eprintln!("Connection error: {err}");
                }
            });
        }
    }
}
async fn handle_request_with_websocket_support(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();

    // Check if this is a WebSocket upgrade request
    #[cfg(feature = "websocket")]
    {
        if is_websocket_upgrade(&req) {
            return handle_websocket_upgrade(router, req, path).await;
        }
    }

    // Handle regular HTTP request
    handle_regular_http_request(router, req).await
}

#[cfg(feature = "websocket")]
fn is_websocket_upgrade(req: &hyper::Request<hyper::body::Incoming>) -> bool {
    use hyper::header::{CONNECTION, UPGRADE};

    req.headers()
        .get(CONNECTION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_lowercase().contains("upgrade"))
        .unwrap_or(false)
        && req
            .headers()
            .get(UPGRADE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase().contains("websocket"))
            .unwrap_or(false)
        && req.headers().get("sec-websocket-key").is_some()
}

#[cfg(feature = "websocket")]
async fn handle_websocket_upgrade(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    path: String,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    // Get the WebSocket handler for this path
    let websocket_handlers = router.get_websocket_handlers();
    let handler = match websocket_handlers.get(&path) {
        Some(handler) => Arc::clone(handler),
        None => {
            info!("⚠️  No WebSocket handler found for path: {}", path);
            return Ok(hyper::Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("WebSocket endpoint not found")))
                .unwrap());
        }
    };

    // Generate WebSocket accept key
    let websocket_key = req
        .headers()
        .get("sec-websocket-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let accept_key = generate_websocket_accept_key(websocket_key);

    // Create upgrade response
    let mut response = hyper::Response::builder()
        .status(101) // Switching Protocols
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-accept", accept_key);

    // Handle protocols if specified
    if let Some(protocols) = req.headers().get("sec-websocket-protocol") {
        if let Ok(protocols_str) = protocols.to_str() {
            if let Some(protocol) = protocols_str.split(',').next() {
                response = response.header("sec-websocket-protocol", protocol.trim());
            }
        }
    }

    let response = response.body(Full::new(Bytes::new())).unwrap();

    // Spawn task to handle the WebSocket connection after upgrade
    tokio::spawn(async move {
        // Wait for the HTTP upgrade to complete
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                info!(
                    "🔌 WebSocket connection upgraded successfully for path: {}",
                    path
                );
                if let Err(e) = crate::websocket::handle_websocket_upgrade(upgraded, handler).await
                {
                    eprintln!("❌ WebSocket handler error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("❌ WebSocket upgrade failed: {}", e);
            }
        }
    });

    Ok(response)
}

#[cfg(feature = "websocket")]
fn generate_websocket_accept_key(key: &str) -> String {
    use sha1::{Digest, Sha1};

    const WEBSOCKET_MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WEBSOCKET_MAGIC.as_bytes());
    let hash = hasher.finalize();
    base64::encode(hash)
}

async fn handle_regular_http_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let (parts, body) = req.into_parts();

    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    if body_bytes.len() > 10 * 1024 * 1024 {
        let mut response = hyper::Response::new(Full::new(Bytes::from("Request too large")));
        *response.status_mut() = http::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(response);
    }

    let request = Request::new(
        parts.method,
        parts.uri,
        parts.version,
        parts.headers,
        body_bytes,
    );

    let response = match router.handle(request).await {
        Ok(res) => res,
        Err(err) => {
            let status = err.status_code();
            let mut res = Response::new(status);
            res.body = Bytes::from(err.to_string());
            res
        }
    };

    let mut builder = hyper::Response::builder().status(response.status);
    for (key, value) in response.headers.iter() {
        builder = builder.header(key, value);
    }

    Ok(builder.body(Full::new(response.body)).unwrap())
}
