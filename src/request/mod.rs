pub mod body;
pub mod params;

use crate::{error::Result, Extensions};
use bytes::Bytes;
use http::{HeaderMap, Method, Uri, Version};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub extensions: Extensions,
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
            extensions: Extensions::new(),
        }
    }

    fn parse_query_params(uri: &Uri) -> HashMap<String, String> {
        let query = match uri.query() {
            Some(q) => q,
            None => return HashMap::new(),
        };

        let mut params = HashMap::new();
        let mut key = String::with_capacity(32);
        let mut value = String::with_capacity(64);
        let mut parsing_key = true;

        for c in query.chars() {
            match c {
                '&' => {
                    if !key.is_empty() {
                        params.insert(std::mem::take(&mut key), std::mem::take(&mut value));
                    }
                    parsing_key = true;
                }
                '=' if parsing_key => {
                    parsing_key = false;
                }
                _ if parsing_key => {
                    key.push(c);
                }
                _ => {
                    value.push(c);
                }
            }
        }

        if !key.is_empty() {
            params.insert(key, value);
        }

        params
    }

    // Optimized JSON parsing with pre-check
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        if self.body.is_empty() {
            return Err(crate::Error::BadRequest("Empty body".into()));
        }

        // Quick check for JSON content type
        if let Some(content_type) = self.header("content-type") {
            if !content_type.starts_with("application/json") {
                return Err(crate::Error::BadRequest(
                    "Expected JSON content type".into(),
                ));
            }
        }

        serde_json::from_slice(&self.body).map_err(Into::into)
    }

    // Inline these methods for better performance
    #[inline]
    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    #[inline]
    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    #[inline]
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    // Extension methods
    /// Insert an extension value
    pub fn insert_extension<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.insert(value);
    }

    /// Get an extension value (returns Arc<T> for shared ownership)
    pub fn get_extension<T: Send + Sync + Clone + 'static>(&self) -> Option<Arc<T>> {
        self.extensions.get()
    }

    /// Remove an extension value
    pub fn remove_extension<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.extensions.remove()
    }

    /// Check if an extension exists
    pub fn has_extension<T: Send + Sync + 'static>(&self) -> bool {
        self.extensions.contains::<T>()
    }
}
