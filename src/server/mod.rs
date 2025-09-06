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

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("Server running on http://{}", self.addr);

        let service = move |router: Arc<Router>| {
            service_fn(move |req| {
                let router = router.clone();
                async move { handle_request(router, req).await }
            })
        };

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router_clone = self.router.clone();
            let service_clone = service(router_clone);

            tokio::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_clone)
                    .await
                {
                    eprintln!("Connection error: {err}");
                }
            });
        }
    }

    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("🔥 ignitia server blazing on http://{}", self.addr);

        let service = move |router: Arc<Router>| {
            service_fn(move |req| {
                let router = router.clone();
                async move { handle_request(router, req).await }
            })
        };

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router_clone = self.router.clone();
            let service_clone = service(router_clone);

            tokio::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_clone)
                    .await
                {
                    eprintln!("Connection error: {err}");
                }
            });
        }
    }
}

// Optimized request handling
async fn handle_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let (parts, body) = req.into_parts();

    // Stream body collection with size limit
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    // Limit body size to prevent DoS
    if body_bytes.len() > 10 * 1024 * 1024 {
        // 10MB limit
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
