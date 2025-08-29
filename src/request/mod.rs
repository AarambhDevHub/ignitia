pub mod body;
pub mod params;

use crate::error::Result;
use bytes::Bytes;
use http::{HeaderMap, Method, Uri, Version};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl Request {
    pub fn new(
        method: Method,
        uri: Uri,
        version: Version,
        headers: HeaderMap,
        body: Bytes,
    ) -> Self {
        let query_params = Self::parse_query_params(&uri);

        Self {
            method,
            uri,
            version,
            headers,
            body,
            params: HashMap::new(),
            query_params,
        }
    }

    fn parse_query_params(uri: &Uri) -> HashMap<String, String> {
        uri.query()
            .map(|query| {
                query
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.split('=');
                        match (parts.next(), parts.next()) {
                            (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                            _ => None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(Into::into)
    }

    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }
}
