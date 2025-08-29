use crate::error::{Error, Result};
use bytes::Bytes;
use serde::de::DeserializeOwned;

pub struct Body {
    inner: Bytes,
}

impl Body {
    pub fn new(bytes: Bytes) -> Self {
        Self { inner: bytes }
    }

    pub fn bytes(&self) -> &Bytes {
        &self.inner
    }

    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.inner.to_vec())
            .map_err(|_| Error::BadRequest("Invalid UTF-8 in body".into()))
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.inner).map_err(Into::into)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl From<Bytes> for Body {
    fn from(bytes: Bytes) -> Self {
        Self::new(bytes)
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self::new(Bytes::from(s))
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        Self::new(Bytes::from(s.to_string()))
    }
}
