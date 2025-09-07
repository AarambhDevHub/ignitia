pub mod builder;
pub mod status;

use crate::error::Result;
use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use serde::Serialize;

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Bytes::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    pub fn internal_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn json<T: Serialize>(data: T) -> Result<Self> {
        let body = serde_json::to_vec(&data)?;
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        response.body = Bytes::from(body);
        Ok(response)
    }

    pub fn text(text: impl Into<String>) -> Self {
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.body = Bytes::from(text.into());
        response
    }

    pub fn html(html: impl Into<String>) -> Self {
        let mut response = Self::new(StatusCode::OK);
        response.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        response.body = Bytes::from(html.into());
        response
    }
}

pub use builder::ResponseBuilder;

// pub struct ResponseBuilder {
//     response: Response,
// }

// impl ResponseBuilder {
//     pub fn new() -> Self {
//         Self {
//             response: Response::new(StatusCode::OK),
//         }
//     }

//     pub fn status(mut self, status: StatusCode) -> Self {
//         self.response.status = status;
//         self
//     }

//     pub fn header<K, V>(mut self, key: K, value: V) -> Self
//     where
//         K: TryInto<HeaderName>,
//         V: TryInto<HeaderValue>,
//         K::Error: std::fmt::Debug,
//         V::Error: std::fmt::Debug,
//     {
//         if let (Ok(name), Ok(val)) = (key.try_into(), value.try_into()) {
//             self.response.headers.insert(name, val);
//         }
//         self
//     }

//     pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self> {
//         let body = serde_json::to_vec(data)?;
//         self.response.headers.insert(
//             HeaderName::from_static("content-type"),
//             HeaderValue::from_static("application/json"),
//         );
//         self.response.body = Bytes::from(body);
//         Ok(self)
//     }

//     pub fn text<T: Into<String>>(mut self, text: T) -> Self {
//         let text = text.into();
//         self.response.headers.insert(
//             HeaderName::from_static("content-type"),
//             HeaderValue::from_static("text/plain; charset=utf-8"),
//         );
//         self.response.body = Bytes::from(text);
//         self
//     }

//     pub fn html<T: Into<String>>(mut self, html: T) -> Self {
//         let html = html.into();
//         self.response.headers.insert(
//             HeaderName::from_static("content-type"),
//             HeaderValue::from_static("text/html; charset=utf-8"),
//         );
//         self.response.body = Bytes::from(html);
//         self
//     }

//     pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
//         self.response.body = body.into();
//         self
//     }

//     pub fn build(self) -> Response {
//         self.response
//     }
// }

// impl Default for ResponseBuilder {
//     fn default() -> Self {
//         Self::new()
//     }
// }
