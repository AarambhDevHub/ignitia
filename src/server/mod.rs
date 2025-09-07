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
            let (stream, _) = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(e) => {
                    tracing::warn!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            let router = Arc::clone(&self.router);

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let router = Arc::clone(&router);
                    async move { handle_request(router, req).await }
                });

                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    tracing::debug!("Connection error: {}", err);
                }
            });
        }
    }
}

async fn handle_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();

    // Check for WebSocket upgrade first (fast path)
    #[cfg(feature = "websocket")]
    {
        if is_websocket_upgrade(&req) {
            return handle_websocket_upgrade(router, req, &path).await;
        }
    }

    // Handle regular HTTP request
    handle_regular_http_request(router, req).await
}

#[cfg(feature = "websocket")]
fn is_websocket_upgrade(req: &hyper::Request<hyper::body::Incoming>) -> bool {
    use hyper::header::{CONNECTION, UPGRADE};

    // Fast path checks with early returns
    let connection_header = match req.headers().get(CONNECTION) {
        Some(h) => h,
        None => return false,
    };

    let upgrade_header = match req.headers().get(UPGRADE) {
        Some(h) => h,
        None => return false,
    };

    let connection = match connection_header.to_str() {
        Ok(c) => c.to_lowercase(),
        Err(_) => return false,
    };

    let upgrade = match upgrade_header.to_str() {
        Ok(u) => u.to_lowercase(),
        Err(_) => return false,
    };

    connection.contains("upgrade")
        && upgrade.contains("websocket")
        && req.headers().get("sec-websocket-key").is_some()
        && req
            .headers()
            .get("sec-websocket-version")
            .map(|v| v == "13")
            .unwrap_or(false)
}

#[cfg(feature = "websocket")]
async fn handle_websocket_upgrade(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    path: &str,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    use hyper::header::SEC_WEBSOCKET_KEY;

    let websocket_handlers = router.get_websocket_handlers();
    let handler = match websocket_handlers.get(path) {
        Some(handler) => Arc::clone(handler),
        None => {
            tracing::debug!("No WebSocket handler found for path: {}", path);
            return Ok(hyper::Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("WebSocket endpoint not found")))
                .unwrap());
        }
    };

    let websocket_key = match req.headers().get(SEC_WEBSOCKET_KEY) {
        Some(key) => match key.to_str() {
            Ok(k) => k,
            Err(_) => {
                return Ok(hyper::Response::builder()
                    .status(400)
                    .body(Full::new(Bytes::from("Invalid Sec-WebSocket-Key")))
                    .unwrap())
            }
        },
        None => {
            return Ok(hyper::Response::builder()
                .status(400)
                .body(Full::new(Bytes::from("Missing Sec-WebSocket-Key")))
                .unwrap())
        }
    };

    let accept_key = generate_websocket_accept_key(websocket_key);

    let mut response = hyper::Response::builder()
        .status(101)
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-accept", accept_key);

    if let Some(protocols) = req.headers().get("sec-websocket-protocol") {
        if let Ok(protocols_str) = protocols.to_str() {
            if let Some(protocol) = protocols_str.split(',').find(|p| !p.trim().is_empty()) {
                response = response.header("sec-websocket-protocol", protocol.trim());
            }
        }
    }

    let response = response.body(Full::new(Bytes::new())).unwrap();

    // Spawn WebSocket handling task
    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                if let Err(e) = crate::websocket::handle_websocket_upgrade(upgraded, handler).await
                {
                    tracing::debug!("WebSocket handler error: {}", e);
                }
            }
            Err(e) => {
                tracing::debug!("WebSocket upgrade failed: {}", e);
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

    // Limit request body size (10MB max)
    const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    let body_bytes = match body.collect().await {
        Ok(collected) => {
            let bytes = collected.to_bytes();
            // Check body size using len() instead of size()
            if bytes.len() > MAX_BODY_SIZE {
                let mut response =
                    hyper::Response::new(Full::new(Bytes::from("Request too large")));
                *response.status_mut() = http::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(response);
            }
            bytes
        }
        Err(_) => Bytes::new(),
    };

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

    // Pre-allocate headers vector for better performance
    for (key, value) in response.headers.iter() {
        builder = builder.header(key, value);
    }

    Ok(builder.body(Full::new(response.body)).unwrap())
}
