use crate::{HandlerFn, Request};
use http::Method;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
static WILDCARD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub method: Method,
    pub handler: HandlerFn,
    pub regex: Regex,
    pub param_names: Vec<String>,
    pub wildcard_names: Vec<String>,
    // Cache the total parameter count
    total_params: usize,
}

impl Route {
    pub fn new(path: &str, method: Method, handler: HandlerFn) -> Self {
        let (regex_pattern, param_names, wildcard_names) = Self::build_regex(path);
        let regex = Self::compile_regex(&regex_pattern);
        let total_params = param_names.len() + wildcard_names.len();

        Self {
            path: path.to_string(),
            method,
            handler,
            regex,
            param_names,
            wildcard_names,
            total_params,
        }
    }

    pub fn compile_regex(pattern: &str) -> Regex {
        RegexBuilder::new(pattern)
            .size_limit(5 * 1024)
            .dfa_size_limit(5 * 1024)
            .build()
            .expect("Invalid regex pattern")
    }

    fn build_regex(path: &str) -> (String, Vec<String>, Vec<String>) {
        let mut param_names = Vec::new();
        let mut wildcard_names = Vec::new();

        // Handle wildcards first
        let path_with_wildcards = WILDCARD_REGEX.replace_all(path, |caps: &regex::Captures| {
            wildcard_names.push(caps[1].to_string());
            "(.+)"
        });

        // Then handle regular parameters
        let regex_pattern =
            PARAM_REGEX.replace_all(&path_with_wildcards, |caps: &regex::Captures| {
                param_names.push(caps[1].to_string());
                "([^/]+)"
            });

        let escaped_pattern = escape_regex(&regex_pattern);
        (
            format!("^{}$", escaped_pattern),
            param_names,
            wildcard_names,
        )
    }

    pub fn matches(&self, req: &Request) -> Option<HashMap<String, String>> {
        if self.method != req.method {
            return None;
        }

        let path = req.uri.path();

        // Quick length check - if path is shorter than pattern minus params, skip
        let min_length = self.path.len().saturating_sub(self.total_params * 3);
        if path.len() < min_length {
            return None;
        }

        let captures = match self.regex.captures(path) {
            Some(caps) => caps,
            None => return None,
        };

        // Pre-allocate HashMap with expected size
        let mut params = HashMap::with_capacity(self.total_params);

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
    }
}

fn escape_regex(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);

    for c in s.chars() {
        match c {
            '\\' | '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result
}
