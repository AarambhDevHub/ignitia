use super::Response;
use crate::error::Result;
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use serde::Serialize;

pub struct ResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
    body: Option<Bytes>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: None,
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
        K::Error: std::fmt::Debug,
        V::Error: std::fmt::Debug,
    {
        if let (Ok(name), Ok(val)) = (key.try_into(), value.try_into()) {
            self.headers.insert(name, val);
        }
        self
    }

    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self> {
        let body = serde_json::to_vec(data)?;
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        self.body = Some(Bytes::from(body));
        Ok(self)
    }

    pub fn text<T: Into<String>>(mut self, text: T) -> Self {
        let text = text.into();
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        self.body = Some(Bytes::from(text));
        self
    }

    pub fn html<T: Into<String>>(mut self, html: T) -> Self {
        let html = html.into();
        self.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        self.body = Some(Bytes::from(html));
        self
    }

    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn build(self) -> Response {
        Response {
            status: self.status,
            headers: self.headers,
            body: self.body.unwrap_or_else(|| Bytes::new()),
        }
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}
