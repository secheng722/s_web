use std::collections::HashMap;

use crate::{
    context::RequestCtx,
    ree::{Handler, Response, ResponseBuilder},
    tire::Node,
};

type HandlerFunc = Box<dyn Handler>;

#[derive(Default)]
pub struct Router {
    roots: HashMap<String, Node>,
    handlers: HashMap<String, HandlerFunc>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            roots: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    //only one * is allowed
    pub fn parse_pattern(pattern: &str) -> Vec<&str> {
        let vs = pattern.split('/').collect::<Vec<&str>>();
        let mut part = Vec::new();
        for &item in vs.iter() {
            if !item.is_empty() {
                part.push(item);
                if item.starts_with("*") {
                    break;
                }
            }
        }
        part
    }

    pub fn add_route(&mut self, method: &str, pattern: &str, handler: HandlerFunc) {
        let parts = Self::parse_pattern(pattern);
        let key = format!("{}-{}", method, pattern);
        self.roots
            .entry(method.to_string())
            .or_default()
            .insert(pattern, parts, 0);
        self.handlers.insert(key, handler);
    }

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

    pub fn handle(&self, key: &str) -> Option<&HandlerFunc> {
        self.handlers.get(key)
    }

    // 新增方法，处理HTTP请求
    pub async fn handle_request(&self, mut ctx: RequestCtx) -> Response {
        // 提取HTTP方法和路径
        let method = ctx.request.method().to_string();
        let path = ctx.request.uri().path().to_string();
        let (node, params) = self.get_route(&method, &path);
        if node.is_none() {
            // 路由未找到，返回404 Not Found响应
            return ResponseBuilder::with_text("404 Not Found");
        }
        ctx.params = params;
        let node = node.unwrap();
        let key = format!("{}-{}", method, node.pattern);
        // 查找对应的路由处理器
        if let Some(handler) = self.handle(&key) {
            // 创建请求上下文
            // 调用处理函数并等待结果
            handler.handle(ctx).await
        } else {
            // 路由未找到，返回404 Not Found响应
            ResponseBuilder::with_text("404 Not Found")
        }
    }
}

#[async_trait::async_trait]
pub trait Middleware: Send + Sync + 'static {
    fn handle(&self, ctx: RequestCtx) -> Result<Response, hyper::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ree::ResponseBuilder;

    #[test]
    fn test_new_router() {
        let mut router = Router::new();
        router.add_route(
            "GET",
            "/",
            Box::new(|_ctx| async { ResponseBuilder::with_text("Hello, World!") }),
        );
        router.add_route(
            "GET",
            "/hello",
            Box::new(|_ctx| async { ResponseBuilder::with_text("Hello!") }),
        );
        println!("{:?}", router.roots);
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
            Box::new(|_ctx| async { ResponseBuilder::with_text("Hello, World!") }),
        );
        let (node, params) = router.get_route("GET", "/p/rust/doc");
        assert!(node.is_some());
        assert_eq!(params.get("lang").unwrap(), "rust");
    }
}
