use crate::extension::Extension;
use crate::{Error, Request, Result};
use serde::de::DeserializeOwned;
use std::collections::HashMap;

// Helper function to convert HashMap<String, String> to serde_json::Value
// with intelligent type conversion
fn convert_string_map_to_json_value(map: &HashMap<String, String>) -> serde_json::Value {
    let mut json_map = serde_json::Map::new();

    for (key, value) in map {
        // Try to parse as number first, fall back to string
        let json_value = if let Ok(num) = value.parse::<i64>() {
            serde_json::Value::Number(serde_json::Number::from(num))
        } else if let Ok(num) = value.parse::<f64>() {
            serde_json::Value::Number(
                serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0)),
            )
        } else if value == "true" {
            serde_json::Value::Bool(true)
        } else if value == "false" {
            serde_json::Value::Bool(false)
        } else {
            serde_json::Value::String(value.clone())
        };
        json_map.insert(key.clone(), json_value);
    }

    serde_json::Value::Object(json_map)
}

pub trait FromRequest: Sized {
    fn from_request(req: &Request) -> Result<Self>;
}

// Extension extractor
impl<T> FromRequest for Extension<T>
where
    T: Send + Sync + Clone + 'static,
{
    fn from_request(req: &Request) -> Result<Self> {
        req.get_extension::<T>()
            .map(|arc_value| Extension((*arc_value).clone()))
            .ok_or_else(|| {
                Error::Internal(format!(
                    "Extension of type {} not found",
                    std::any::type_name::<T>()
                ))
            })
    }
}

// Path parameter extractor
#[derive(Debug)]
pub struct Path<T>(pub T);

impl<T> Path<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Path<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for Path<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        if req.params.is_empty() {
            return Err(Error::BadRequest(
                "No path parameters found in request".into(),
            ));
        }

        let params_value = convert_string_map_to_json_value(&req.params);

        let extracted = T::deserialize(params_value).map_err(|e| {
            Error::BadRequest(format!(
                "Failed to extract path parameters: {} (from params: {:?})",
                e, req.params
            ))
        })?;

        Ok(Path(extracted))
    }
}

// Query parameter extractor
#[derive(Debug)]
pub struct Query<T>(pub T);

impl<T> Query<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Query<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned,
{
    fn from_request(req: &Request) -> Result<Self> {
        let query_value = convert_string_map_to_json_value(&req.query_params);

        let extracted = T::deserialize(query_value).map_err(|e| {
            Error::BadRequest(format!(
                "Failed to extract query parameters: {} (from query_params: {:?})",
                e, req.query_params
            ))
        })?;

        Ok(Query(extracted))
    }
}

// JSON body extractor
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
#[derive(Debug)]
pub struct Headers(pub HashMap<String, String>);

impl std::ops::Deref for Headers {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

// Cookie extractor
#[derive(Debug)]
pub struct Cookies(pub crate::CookieJar);

impl std::ops::Deref for Cookies {
    type Target = crate::CookieJar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Cookies {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Cookies(req.cookies()))
    }
}

// Raw body extractor
#[derive(Debug)]
pub struct Body(pub bytes::Bytes);

impl std::ops::Deref for Body {
    type Target = bytes::Bytes;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Body {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Body(req.body.clone()))
    }
}

// Method extractor
#[derive(Debug)]
pub struct Method(pub http::Method);

impl std::ops::Deref for Method {
    type Target = http::Method;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Method {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Method(req.method.clone()))
    }
}

// Uri extractor
#[derive(Debug)]
pub struct Uri(pub http::Uri);

impl std::ops::Deref for Uri {
    type Target = http::Uri;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Uri {
    fn from_request(req: &Request) -> Result<Self> {
        Ok(Uri(req.uri.clone()))
    }
}

// // Request extractor (for cases where you still need the full request)
// impl FromRequest for Request {
//     fn from_request(req: &Request) -> Result<Self> {
//         // We can't move out of a reference, so we'll need to clone
//         // This is a limitation - in a real implementation, you'd want to avoid this
//         Ok(Request {
//             method: req.method.clone(),
//             uri: req.uri.clone(),
//             version: req.version,
//             headers: req.headers.clone(),
//             body: req.body.clone(),
//             params: req.params.clone(),
//             query_params: req.query_params.clone(),
//             extensions: req.extensions.clone(),
//         })
//     }
// }
