//! HTTP router with trie-based pattern matching.

use std::collections::HashMap;
use crate::{
    RequestCtx,
    Response,
    ResponseBuilder,
    Handler,
    trie::Node,
};

type HandlerFunc = Box<dyn Handler>;

/// HTTP router for matching requests to handlers
#[derive(Default)]
pub struct Router {
    roots: HashMap<String, Node>,
    handlers: HashMap<String, HandlerFunc>,
}

impl Router {
    pub fn new() -> Self {
        Router::default()
    }

    /// Parse a route pattern into parts (only one * is allowed)
    pub fn parse_pattern(pattern: &str) -> Vec<&str> {
        let vs = pattern.split('/').collect::<Vec<&str>>();
        let mut part = Vec::new();
        for &item in vs.iter() {
            if !item.is_empty() {
                part.push(item);
                if item.starts_with('*') {
                    break;
                }
            }
        }
        part
    }

    /// Add a route with the specified method, pattern, and handler
    pub fn add_route(&mut self, method: &str, pattern: &str, handler: HandlerFunc) {
        let parts = Self::parse_pattern(pattern);
        let key = format!("{}-{}", method, pattern);
        self.roots
            .entry(method.to_string())
            .or_default()
            .insert(pattern, parts, 0);
        self.handlers.insert(key, handler);
    }

    /// Get a route handler for the given method and path
    pub fn get_route(&self, method: &str, path: &str) -> (Option<&Node>, HashMap<String, String>) {
        let search_parts = Self::parse_pattern(path);
        let mut params = HashMap::new();
        let root = self.roots.get(method);
        if root.is_none() {
            return (None, HashMap::new());
        }
        if let Some(node) = root.unwrap().search(&search_parts, 0) {
            let parts = Self::parse_pattern(&node.pattern);
            for (index, ele) in parts.iter().enumerate() {
                if let Some(param_name) = ele.strip_prefix(':') {
                    params.insert(param_name.to_string(), search_parts[index].to_string());
                } else if let Some(param_name) = ele.strip_prefix('*') {
                    params.insert(param_name.to_string(), search_parts[index..].join("/"));
                    break;
                }
            }
            return (Some(node), params);
        }
        (None, HashMap::new())
    }

    /// Get a handler by key
    pub fn handle(&self, key: &str) -> Option<&HandlerFunc> {
        self.handlers.get(key)
    }

    /// Handle an HTTP request
    pub async fn handle_request(&self, mut ctx: RequestCtx) -> Response {
        let method = ctx.request.method().as_str();
        let path = ctx.request.uri().path();
        let (node, params) = self.get_route(method, path);
        
        if node.is_none() {
            return ResponseBuilder::not_found();
        }
        
        ctx.params = params;
        let node = node.unwrap();
        let key = format!("{}-{}", method, node.pattern);
        
        if let Some(handler) = self.handle(&key) {
            handler.handle(ctx).await
        } else {
            ResponseBuilder::not_found()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_router() {
        let mut router = Router::new();
        router.add_route(
            "GET",
            "/",
            Box::new(|_ctx| async { "Hello, World!" }),
        );
        router.add_route(
            "GET",
            "/hello",
            Box::new(|_ctx| async { "Hello!" }),
        );
        assert_eq!(router.roots.len(), 2);
    }

    #[test]
    fn test_parse_pattern() {
        let pattern = "/p/:lang/doc";
        let parts = Router::parse_pattern(pattern);
        assert_eq!(parts, vec!["p", ":lang", "doc"]);
    }

    #[test]
    fn test_get_route() {
        let mut router = Router::new();
        router.add_route(
            "GET",
            "/p/:lang/doc",
            Box::new(|_ctx| async { "Hello, World!" }),
        );
        let (node, params) = router.get_route("GET", "/p/rust/doc");
        assert!(node.is_some());
        assert_eq!(params.get("lang").unwrap(), "rust");
    }
}