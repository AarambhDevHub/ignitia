// use crate::{HandlerFn, Request};
// use http::Method;
// use once_cell::sync::Lazy;
// use regex::Regex;
// use std::collections::HashMap;

// static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

// pub struct Route {
//     pub path: String,
//     pub method: Method,
//     pub handler: HandlerFn,
//     pub regex: Regex,
//     pub param_names: Vec<String>,
// }

// impl Route {
//     pub fn new(path: &str, method: Method, handler: HandlerFn) -> Self {
//         let (regex_pattern, param_names) = Self::build_regex(path);
//         let regex = Regex::new(&regex_pattern).unwrap();

//         Self {
//             path: path.to_string(),
//             method,
//             handler,
//             regex,
//             param_names,
//         }
//     }

//     fn build_regex(path: &str) -> (String, Vec<String>) {
//         let mut param_names = Vec::new();
//         let regex_pattern = PARAM_REGEX.replace_all(path, |caps: &regex::Captures| {
//             param_names.push(caps[1].to_string());
//             "([^/]+)"
//         });

//         (format!("^{}$", regex_pattern), param_names)
//     }

//     pub fn matches(&self, req: &Request) -> Option<HashMap<String, String>> {
//         if self.method != req.method {
//             return None;
//         }

//         let path = req.uri.path();
//         if let Some(captures) = self.regex.captures(path) {
//             let mut params = HashMap::new();
//             for (i, name) in self.param_names.iter().enumerate() {
//                 if let Some(value) = captures.get(i + 1) {
//                     params.insert(name.clone(), value.as_str().to_string());
//                 }
//             }
//             Some(params)
//         } else {
//             None
//         }
//     }
// }

use crate::{HandlerFn, Request};
use http::Method;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

// Add wildcard regex
static WILDCARD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

pub struct Route {
    pub path: String,
    pub method: Method,
    pub handler: HandlerFn,
    pub regex: Regex,
    pub param_names: Vec<String>,
    pub wildcard_names: Vec<String>,
}

impl Route {
    pub fn new(path: &str, method: Method, handler: HandlerFn) -> Self {
        let (regex_pattern, param_names, wildcard_names) = Self::build_regex(path);
        let regex = Regex::new(&regex_pattern).unwrap();

        Self {
            path: path.to_string(),
            method,
            handler,
            regex,
            param_names,
            wildcard_names,
        }
    }

    fn build_regex(path: &str) -> (String, Vec<String>, Vec<String>) {
        let mut param_names = Vec::new();
        let mut wildcard_names = Vec::new();

        // First handle wildcards
        let path_with_wildcards = WILDCARD_REGEX.replace_all(path, |caps: &regex::Captures| {
            wildcard_names.push(caps[1].to_string());
            "(.+)" // Match everything including slashes
        });

        // Then handle regular parameters
        let regex_pattern =
            PARAM_REGEX.replace_all(&path_with_wildcards, |caps: &regex::Captures| {
                param_names.push(caps[1].to_string());
                "([^/]+)"
            });

        (format!("^{}$", regex_pattern), param_names, wildcard_names)
    }

    pub fn matches(&self, req: &Request) -> Option<HashMap<String, String>> {
        if self.method != req.method {
            return None;
        }

        let path = req.uri.path();
        if let Some(captures) = self.regex.captures(path) {
            let mut params = HashMap::new();

            // Handle regular parameters
            for (i, name) in self.param_names.iter().enumerate() {
                if let Some(value) = captures.get(i + 1) {
                    params.insert(name.clone(), value.as_str().to_string());
                }
            }

            // Handle wildcard parameters
            let wildcard_offset = self.param_names.len();
            for (i, name) in self.wildcard_names.iter().enumerate() {
                if let Some(value) = captures.get(wildcard_offset + i + 1) {
                    params.insert(name.clone(), value.as_str().to_string());
                }
            }

            Some(params)
        } else {
            None
        }
    }
}
