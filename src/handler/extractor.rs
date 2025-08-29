use crate::{Error, Request, Result};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

pub trait FromRequest: Sized {
    fn from_request(req: &Request) -> Result<Self>;
}

// Path parameter extractor
pub struct Path<T>(pub T);

impl<T> FromRequest for Path<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        let params_value = serde_json::to_value(&req.params)
            .map_err(|_| Error::BadRequest("Failed to serialize params".into()))?;

        let extracted = T::deserialize(params_value)
            .map_err(|_| Error::BadRequest("Failed to extract path parameters".into()))?;

        Ok(Path(extracted))
    }
}

// Query parameter extractor
pub struct Query<T>(pub T);

impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        let query_value = serde_json::to_value(&req.query_params)
            .map_err(|_| Error::BadRequest("Failed to serialize query params".into()))?;

        let extracted = T::deserialize(query_value)
            .map_err(|_| Error::BadRequest("Failed to extract query parameters".into()))?;

        Ok(Query(extracted))
    }
}

// JSON body extractor
pub struct Json<T>(pub T);

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        let extracted = req.json()?;
        Ok(Json(extracted))
    }
}

// Header extractor
pub struct Headers(pub HashMap<String, String>);

impl FromRequest for Headers {
    fn from_request(req: &Request) -> Result<Self> {
        let mut headers = HashMap::new();
        for (key, value) in req.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }
        Ok(Headers(headers))
    }
}
