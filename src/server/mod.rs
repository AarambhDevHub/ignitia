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

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = self.router.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let router = router.clone();
                    async move { handle_request(router, req).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Error serving connection: {err}");
                }
            });
        }
    }

    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("🔥 ignitia server blazing on http://{}", self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = self.router.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let router = router.clone();
                    async move { handle_request(router, req).await }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Error serving connection: {err}");
                }
            });
        }
    }
}

async fn handle_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let (parts, body) = req.into_parts();

    let body_bytes = body
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .unwrap_or_else(|_| Bytes::new());

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
