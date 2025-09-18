use crate::HandlerFn;
use http::Method;
use std::collections::HashMap;
use std::fmt;

/// A compressed trie (radix tree) node for efficient path routing
#[derive(Clone)]
struct RadixNode {
    /// The path segment for this node (compressed)
    path: String,
    /// Handler for this exact path (if any)
    handlers: HashMap<Method, HandlerFn>,
    /// Child nodes
    children: Vec<RadixNode>,
    /// Parameter name if this is a parameter node
    param_name: Option<String>,
    /// Whether this node accepts wildcard matches
    is_wildcard: bool,
    /// Indices for quick child lookup
    indices: String,
}

// Custom Debug implementation to handle non-Debug HandlerFn
impl fmt::Debug for RadixNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RadixNode")
            .field("path", &self.path)
            .field(
                "handlers",
                &format!(
                    "HashMap<Method, HandlerFn> with {} entries",
                    self.handlers.len()
                ),
            )
            .field("children", &self.children)
            .field("param_name", &self.param_name)
            .field("is_wildcard", &self.is_wildcard)
            .field("indices", &self.indices)
            .finish()
    }
}

impl RadixNode {
    fn new() -> Self {
        Self {
            path: String::new(),
            handlers: HashMap::new(),
            children: Vec::new(),
            param_name: None,
            is_wildcard: false,
            indices: String::new(),
        }
    }

    fn with_path(path: String) -> Self {
        Self {
            path,
            handlers: HashMap::new(),
            children: Vec::new(),
            param_name: None,
            is_wildcard: false,
            indices: String::new(),
        }
    }
}

/// High-performance radix tree router
#[derive(Clone)]
pub struct RadixRouter {
    root: RadixNode,
}

// Custom Debug implementation for RadixRouter
impl fmt::Debug for RadixRouter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RadixRouter")
            .field("root", &self.root)
            .finish()
    }
}

impl RadixRouter {
    pub fn new() -> Self {
        Self {
            root: RadixNode::new(),
        }
    }

    /// Insert a route into the radix tree
    pub fn insert(&mut self, path: &str, method: Method, handler: HandlerFn) {
        let normalized_path = normalize_radix_path(path);
        tracing::debug!(
            "Inserting route: {} {} into radix tree",
            method,
            normalized_path
        );

        // Split path into segments for proper processing
        let segments: Vec<&str> = normalized_path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        Self::insert_segments(&mut self.root, &segments, method, handler);
    }

    fn insert_segments(
        node: &mut RadixNode,
        segments: &[&str],
        method: Method,
        handler: HandlerFn,
    ) {
        // If no more segments, store the handler here
        if segments.is_empty() {
            tracing::debug!("Storing handler for method {} at node", method);
            node.handlers.insert(method, handler);
            return;
        }

        let segment = segments[0];
        let remaining_segments = &segments[1..];

        tracing::trace!("Processing segment: '{}'", segment);

        // Check if this segment is a parameter or wildcard
        if segment.starts_with(':') {
            // Parameter segment
            let param_name = segment[1..].to_string();

            // Look for existing parameter node
            for child in &mut node.children {
                if let Some(existing_param) = &child.param_name {
                    if existing_param == &param_name && !child.is_wildcard {
                        Self::insert_segments(child, remaining_segments, method, handler);
                        return;
                    }
                }
            }

            // Create new parameter node
            let mut param_node = RadixNode::new();
            param_node.param_name = Some(param_name.clone());
            param_node.path = segment.to_string(); // Keep the original :param format for debugging

            Self::insert_segments(&mut param_node, remaining_segments, method, handler);

            // Insert parameter node before wildcards but after static routes
            let insert_pos = node
                .children
                .iter()
                .position(|child| child.is_wildcard)
                .unwrap_or(node.children.len());
            node.children.insert(insert_pos, param_node);
        } else if segment.starts_with('*') {
            // Wildcard segment
            let wildcard_name = segment[1..].to_string();

            // Look for existing wildcard node
            for child in &mut node.children {
                if let Some(existing_param) = &child.param_name {
                    if existing_param == &wildcard_name && child.is_wildcard {
                        Self::insert_segments(child, remaining_segments, method, handler);
                        return;
                    }
                }
            }

            // Create new wildcard node
            let mut wildcard_node = RadixNode::new();
            wildcard_node.param_name = Some(wildcard_name);
            wildcard_node.is_wildcard = true;
            wildcard_node.path = segment.to_string();

            Self::insert_segments(&mut wildcard_node, remaining_segments, method, handler);

            // Wildcards go at the end
            node.children.push(wildcard_node);
        } else {
            // Static segment
            // Look for existing static child with exact match
            for child in &mut node.children {
                if child.param_name.is_none() && !child.is_wildcard && child.path == segment {
                    Self::insert_segments(child, remaining_segments, method, handler);
                    return;
                }
            }

            // Create new static child
            let mut static_node = RadixNode::with_path(segment.to_string());
            Self::insert_segments(&mut static_node, remaining_segments, method, handler);

            // Insert static routes before parameters
            let insert_pos = node
                .children
                .iter()
                .position(|child| child.param_name.is_some())
                .unwrap_or(node.children.len());
            node.children.insert(insert_pos, static_node);

            // Update indices for fast lookup
            Self::update_indices(node);
        }
    }

    fn update_indices(node: &mut RadixNode) {
        let mut indices = String::new();
        for child in &node.children {
            if child.param_name.is_none() && !child.is_wildcard {
                if let Some(first_char) = child.path.chars().next() {
                    if !indices.contains(first_char) {
                        indices.push(first_char);
                    }
                }
            }
        }
        node.indices = indices;
    }

    /// Lookup a route in the radix tree
    pub fn lookup(
        &self,
        method: &Method,
        path: &str,
    ) -> Option<(HandlerFn, HashMap<String, String>)> {
        let normalized_path = normalize_radix_path(path);
        tracing::debug!("Looking up route: {} {}", method, normalized_path);

        let segments: Vec<&str> = if normalized_path == "/" {
            vec![]
        } else {
            normalized_path
                .split('/')
                .filter(|s| !s.is_empty())
                .collect()
        };

        let mut params = HashMap::new();
        if let Some(handler) = self.lookup_segments(&self.root, &segments, method, &mut params) {
            tracing::debug!("Route found with params: {:?}", params);
            Some((handler, params))
        } else {
            tracing::debug!("Route not found: {} {}", method, normalized_path);
            None
        }
    }

    fn lookup_segments(
        &self,
        node: &RadixNode,
        segments: &[&str],
        method: &Method,
        params: &mut HashMap<String, String>,
    ) -> Option<HandlerFn> {
        tracing::trace!(
            "lookup_segments: segments={:?}, node has {} children",
            segments,
            node.children.len()
        );

        // If no more segments, check for handler at this node
        if segments.is_empty() {
            if let Some(handler) = node.handlers.get(method) {
                tracing::trace!("Found handler for method {} at leaf node", method);
                return Some(handler.clone());
            } else {
                tracing::trace!("No handler for method {} at leaf node", method);
                return None;
            }
        }

        let segment = segments[0];
        let remaining_segments = &segments[1..];

        tracing::trace!(
            "Processing segment: '{}', remaining: {:?}",
            segment,
            remaining_segments
        );

        // First, try static routes (most specific)
        for child in &node.children {
            if child.param_name.is_none() && !child.is_wildcard && child.path == segment {
                tracing::trace!("Matched static route: '{}'", child.path);
                if let Some(handler) =
                    self.lookup_segments(child, remaining_segments, method, params)
                {
                    return Some(handler);
                }
            }
        }

        // Then, try parameter routes
        for child in &node.children {
            if let Some(param_name) = &child.param_name {
                if !child.is_wildcard {
                    tracing::trace!(
                        "Trying parameter route: param '{}' for segment '{}'",
                        param_name,
                        segment
                    );

                    // Store the parameter value
                    let old_value = params.insert(param_name.clone(), segment.to_string());

                    if let Some(handler) =
                        self.lookup_segments(child, remaining_segments, method, params)
                    {
                        tracing::trace!("Parameter route matched!");
                        return Some(handler);
                    }

                    // Backtrack if no match found
                    if let Some(old) = old_value {
                        params.insert(param_name.clone(), old);
                    } else {
                        params.remove(param_name);
                    }
                    tracing::trace!("Parameter route backtracked");
                }
            }
        }

        // Finally, try wildcard routes (least specific)
        for child in &node.children {
            if let Some(param_name) = &child.param_name {
                if child.is_wildcard {
                    tracing::trace!("Trying wildcard route: param '{}'", param_name);

                    // Wildcard captures everything remaining
                    let wildcard_value = segments.join("/");
                    params.insert(param_name.clone(), wildcard_value);

                    if let Some(handler) = child.handlers.get(method) {
                        tracing::trace!("Wildcard route matched!");
                        return Some(handler.clone());
                    }

                    // Remove if no handler found
                    params.remove(param_name);
                }
            }
        }

        tracing::trace!("No route matched for segment: '{}'", segment);
        None
    }

    /// Get statistics about the radix tree
    pub fn stats(&self) -> RadixStats {
        let mut stats = RadixStats::default();
        self.collect_stats(&self.root, &mut stats, 0);
        stats
    }

    fn collect_stats(&self, node: &RadixNode, stats: &mut RadixStats, depth: usize) {
        stats.node_count += 1;
        stats.handler_count += node.handlers.len();
        stats.max_depth = stats.max_depth.max(depth);

        if node.param_name.is_some() {
            stats.param_node_count += 1;
        }

        if node.is_wildcard {
            stats.wildcard_node_count += 1;
        }

        for child in &node.children {
            self.collect_stats(child, stats, depth + 1);
        }
    }

    /// Debug method to print the tree structure
    pub fn print_tree(&self) {
        self.print_node(&self.root, 0);
    }

    fn print_node(&self, node: &RadixNode, depth: usize) {
        let indent = "  ".repeat(depth);
        println!(
            "{}Node: path='{}', param={:?}, wildcard={}, handlers={}",
            indent,
            node.path,
            node.param_name,
            node.is_wildcard,
            node.handlers.len()
        );

        for child in &node.children {
            self.print_node(child, depth + 1);
        }
    }
}

#[derive(Debug, Default)]
pub struct RadixStats {
    pub node_count: usize,
    pub handler_count: usize,
    pub param_node_count: usize,
    pub wildcard_node_count: usize,
    pub max_depth: usize,
}

fn normalize_radix_path(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }

    let mut normalized = String::with_capacity(path.len());

    // Ensure path starts with '/'
    if !path.starts_with('/') {
        normalized.push('/');
    }

    // Add the path, removing multiple consecutive slashes
    let mut prev_char = '\0';
    for ch in path.chars() {
        if ch == '/' && prev_char == '/' {
            continue; // Skip multiple consecutive slashes
        }
        normalized.push(ch);
        prev_char = ch;
    }

    // Remove trailing slash unless it's the root path
    if normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }

    normalized
}

impl Default for RadixRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;
    use http::Method;
    use std::sync::Arc;

    fn dummy_handler() -> HandlerFn {
        Arc::new(|_req| Box::pin(async { Ok(Response::ok()) }))
    }

    #[test]
    fn test_static_routes() {
        let mut router = RadixRouter::new();

        router.insert("/", Method::GET, dummy_handler());
        router.insert("/users", Method::GET, dummy_handler());
        router.insert("/users/profile", Method::GET, dummy_handler());
        router.insert("/admin", Method::GET, dummy_handler());

        assert!(router.lookup(&Method::GET, "/").is_some());
        assert!(router.lookup(&Method::GET, "/users").is_some());
        assert!(router.lookup(&Method::GET, "/users/profile").is_some());
        assert!(router.lookup(&Method::GET, "/admin").is_some());
        assert!(router.lookup(&Method::GET, "/nonexistent").is_none());
    }

    #[test]
    fn test_param_routes() {
        let mut router = RadixRouter::new();

        router.insert("/users/:id", Method::GET, dummy_handler());
        router.insert("/users/:id/posts/:post_id", Method::GET, dummy_handler());

        // Test string parameter
        if let Some((_, params)) = router.lookup(&Method::GET, "/users/123") {
            assert_eq!(params.get("id"), Some(&"123".to_string()));
        } else {
            panic!("Route /users/123 not found");
        }

        // Test numeric parameter (should work the same way)
        if let Some((_, params)) = router.lookup(&Method::GET, "/users/456") {
            assert_eq!(params.get("id"), Some(&"456".to_string()));
        } else {
            panic!("Route /users/456 not found");
        }

        if let Some((_, params)) = router.lookup(&Method::GET, "/users/456/posts/789") {
            assert_eq!(params.get("id"), Some(&"456".to_string()));
            assert_eq!(params.get("post_id"), Some(&"789".to_string()));
        } else {
            panic!("Route not found");
        }
    }

    #[test]
    fn test_wildcard_routes() {
        let mut router = RadixRouter::new();

        router.insert("/static/*filepath", Method::GET, dummy_handler());

        if let Some((_, params)) = router.lookup(&Method::GET, "/static/css/main.css") {
            assert_eq!(params.get("filepath"), Some(&"css/main.css".to_string()));
        } else {
            panic!("Wildcard route not found");
        }
    }

    #[test]
    fn test_route_priority() {
        let mut router = RadixRouter::new();

        // Static routes should have priority over param routes
        router.insert("/users/new", Method::GET, dummy_handler());
        router.insert("/users/:id", Method::GET, dummy_handler());

        // Should match static route, not param route
        assert!(router.lookup(&Method::GET, "/users/new").is_some());

        // Should match param route
        if let Some((_, params)) = router.lookup(&Method::GET, "/users/123") {
            assert_eq!(params.get("id"), Some(&"123".to_string()));
        } else {
            panic!("Param route not found");
        }
    }

    #[test]
    fn test_numeric_params() {
        let mut router = RadixRouter::new();

        router.insert("/users/:id", Method::GET, dummy_handler());
        router.insert(
            "/posts/:id/comments/:comment_id",
            Method::GET,
            dummy_handler(),
        );

        // Test single digit
        if let Some((_, params)) = router.lookup(&Method::GET, "/users/1") {
            assert_eq!(params.get("id"), Some(&"1".to_string()));
        } else {
            panic!("Route /users/1 not found");
        }

        // Test multi digit
        if let Some((_, params)) = router.lookup(&Method::GET, "/users/123456") {
            assert_eq!(params.get("id"), Some(&"123456".to_string()));
        } else {
            panic!("Route /users/123456 not found");
        }

        // Test nested numeric params
        if let Some((_, params)) = router.lookup(&Method::GET, "/posts/42/comments/7") {
            assert_eq!(params.get("id"), Some(&"42".to_string()));
            assert_eq!(params.get("comment_id"), Some(&"7".to_string()));
        } else {
            panic!("Route /posts/42/comments/7 not found");
        }
    }
}
