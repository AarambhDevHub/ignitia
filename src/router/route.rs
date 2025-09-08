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
    // Cache the total parameter count and segment count for faster matching
    total_params: usize,
    segment_count: usize,
}

impl Route {
    pub fn new(path: &str, method: Method, handler: HandlerFn) -> Self {
        let (regex_pattern, param_names, wildcard_names) = Self::build_regex(path);
        let regex = Self::compile_regex(&regex_pattern);
        let total_params = param_names.len() + wildcard_names.len();
        let segment_count = path.matches('/').count();

        Self {
            path: path.to_string(),
            method,
            handler,
            regex,
            param_names,
            wildcard_names,
            total_params,
            segment_count,
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
        let path_with_params =
            PARAM_REGEX.replace_all(&path_with_wildcards, |caps: &regex::Captures| {
                param_names.push(caps[1].to_string());
                "([^/]+)"
            });

        // Escape only the parts that aren't our regex groups
        let escaped_pattern = escape_regex_selective(&path_with_params);

        (
            format!("^{}$", escaped_pattern),
            param_names,
            wildcard_names,
        )
    }

    pub fn matches(&self, req: &Request) -> Option<HashMap<String, String>> {
        // Fast path: check method first
        if self.method != req.method {
            return None;
        }

        let path = req.uri.path();

        // Quick length and segment checks for early rejection
        let min_length = self.path.len().saturating_sub(self.total_params * 3);
        if path.len() < min_length {
            return None;
        }

        // Check segment count for early rejection
        let request_segments = path.matches('/').count();
        if request_segments < self.segment_count.saturating_sub(self.total_params) {
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

    // Helper method for testing and debugging
    pub fn get_param_names(&self) -> Vec<String> {
        let mut names = self.param_names.clone();
        names.extend(self.wildcard_names.clone());
        names
    }
}

fn escape_regex_selective(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Don't escape if we're in a regex group pattern
            '(' => {
                result.push(c);
                // Copy everything until the matching ')'
                let mut paren_count = 1;
                while let Some(inner_c) = chars.next() {
                    result.push(inner_c);
                    match inner_c {
                        '(' => paren_count += 1,
                        ')' => {
                            paren_count -= 1;
                            if paren_count == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Escape other regex special characters
            '\\' | '.' | '+' | '*' | '?' | '^' | '$' | '[' | ']' | '{' | '}' | '|' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result
}
