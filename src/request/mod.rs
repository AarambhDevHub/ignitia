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
        uri.query()
            .map(|query| {
                query
                    .split('&') // ✅ Fixed: was &amp;
                    .filter_map(|pair| {
                        let mut parts = pair.split('=');
                        match (parts.next(), parts.next()) {
                            (Some(key), Some(value)) => {
                                // ✅ Added URL decoding
                                let decoded_key = urlencoding::decode(key)
                                    .unwrap_or_else(|_| key.into())
                                    .into_owned();
                                let decoded_value = urlencoding::decode(value)
                                    .unwrap_or_else(|_| value.into())
                                    .into_owned();
                                Some((decoded_key, decoded_value))
                            }
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
