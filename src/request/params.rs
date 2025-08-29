use crate::error::{Error, Result};
use std::collections::HashMap;
use std::str::FromStr;

pub struct Params {
    inner: HashMap<String, String>,
}

impl Params {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.inner.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    pub fn get_parsed<T: FromStr>(&self, key: &str) -> Result<T> {
        self.inner
            .get(key)
            .ok_or_else(|| Error::BadRequest(format!("Missing parameter: {}", key)))?
            .parse()
            .map_err(|_| Error::BadRequest(format!("Invalid parameter format: {}", key)))
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.inner.iter()
    }
}

impl From<HashMap<String, String>> for Params {
    fn from(map: HashMap<String, String>) -> Self {
        Self { inner: map }
    }
}
