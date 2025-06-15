//! Main HTTP engine and router group implementations.

use std::{
    collections::HashMap, 
    convert::Infallible, 
    net::SocketAddr, 
    sync::Arc,
};

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;

use crate::{
    RequestCtx, 
    Response, 
    Handler, 
    Router,
    MiddlewareFn,
    MiddlewareNext,
    execute_middleware_chain,
};

/// A group of routes with shared prefix and middleware
pub struct RouterGroup {
    prefix: String,
    router: Router,
    middlewares: Vec<MiddlewareFn>,
}

impl RouterGroup {
    fn new(prefix: String) -> Self {
        Self {
            prefix,
            router: Router::new(),
            middlewares: Vec::new(),
        }
    }

    /// Add a route to this group
    pub fn add_route(&mut self, method: &str, pattern: &str, handler: impl Handler) {
        let handler = Box::new(handler);
        let full_pattern = format!("{}{}", self.prefix, pattern);
        self.router.add_route(method, &full_pattern, handler);
    }

    /// Add a GET route to this group
    pub fn get(&mut self, path: &str, handler: impl Handler) {
        self.add_route("GET", path, handler);
    }

    /// Add a POST route to this group
    pub fn post(&mut self, path: &str, handler: impl Handler) {
        self.add_route("POST", path, handler);
    }

    /// Add a PUT route to this group
    pub fn put(&mut self, path: &str, handler: impl Handler) {
        self.add_route("PUT", path, handler);
    }

    /// Add a DELETE route to this group
    pub fn delete(&mut self, path: &str, handler: impl Handler) {
        self.add_route("DELETE", path, handler);
    }

    /// Add middleware to this group
    pub fn use_middleware(&mut self, middleware: MiddlewareFn) {
        self.middlewares.push(middleware);
    }

    /// Handle a request using this group's router
    pub async fn handle_request(&self, ctx: RequestCtx) -> Response {
        self.router.handle_request(ctx).await
    }
}

/// Main HTTP engine for building web applications
pub struct Engine {
    router: Router,
    groups: HashMap<String, RouterGroup>,
    middlewares: Vec<MiddlewareFn>,
}

impl Engine {
    /// Create a new Engine instance
    pub fn new() -> Self {
        Engine {
            router: Router::new(),
            groups: HashMap::new(),
            middlewares: Vec::new(),
        }
    }

    /// Add global middleware
    pub fn use_middleware(&mut self, middleware: MiddlewareFn) {
        self.middlewares.push(middleware);
    }

    /// Create a route group with the given prefix
    pub fn group(&mut self, prefix: &str) -> &mut RouterGroup {
        let group = RouterGroup::new(prefix.to_string());
        self.groups.insert(prefix.to_string(), group);
        self.groups.get_mut(prefix).unwrap()
    }

    /// Add a route to the main router
    pub fn add_route(&mut self, method: &str, pattern: &str, handler: impl Handler) {
        let handler = Box::new(handler);
        self.router.add_route(method, pattern, handler);
    }

    /// Add a GET route
    pub fn get(&mut self, path: &str, handler: impl Handler) {
        self.add_route("GET", path, handler);
    }

    /// Add a POST route
    pub fn post(&mut self, path: &str, handler: impl Handler) {
        self.add_route("POST", path, handler);
    }

    /// Add a PUT route
    pub fn put(&mut self, path: &str, handler: impl Handler) {
        self.add_route("PUT", path, handler);
    }

    /// Add a DELETE route
    pub fn delete(&mut self, path: &str, handler: impl Handler) {
        self.add_route("DELETE", path, handler);
    }

    /// Start the HTTP server
    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.parse::<SocketAddr>()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let router = Arc::new(self.router);
        let middlewares = Arc::new(self.middlewares);
        let groups = Arc::new(
            self.groups
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect::<HashMap<_, _>>(),
        );

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = Arc::clone(&router);
            let middlewares = Arc::clone(&middlewares);
            let groups = Arc::clone(&groups);
            
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let router = Arc::clone(&router);
                            let middlewares = Arc::clone(&middlewares);
                            let groups = Arc::clone(&groups);
                            
                            async move {
                                // Check if request matches any group
                                let group = groups
                                    .iter()
                                    .find(|(_, g)| req.uri().path().starts_with(&g.prefix))
                                    .map(|(_, g)| g.clone());

                                let ctx = RequestCtx {
                                    request: req,
                                    params: HashMap::new(),
                                };

                                if let Some(group) = group {
                                    // Use group-specific handler with combined middleware
                                    let mut all_middlewares = Vec::new();
                                    // 首先应用全局中间件
                                    all_middlewares.extend(middlewares.iter().cloned());
                                    // 然后应用组中间件
                                    all_middlewares.extend(group.middlewares.iter().cloned());

                                    // 创建组的端点处理器
                                    let endpoint: MiddlewareNext = Arc::new(move |ctx| {
                                        let group = Arc::clone(&group);
                                        Box::pin(async move { group.handle_request(ctx).await })
                                    });

                                    let resp = execute_middleware_chain(&all_middlewares, endpoint, ctx).await;
                                    return Ok::<_, Infallible>(resp);
                                }

                                // Use main router with global middleware
                                let endpoint: MiddlewareNext = Arc::new(move |ctx| {
                                    let router = Arc::clone(&router);
                                    Box::pin(async move { router.handle_request(ctx).await })
                                });
                                
                                let resp = execute_middleware_chain(&middlewares, endpoint, ctx).await;
                                Ok::<_, Infallible>(resp)
                            }
                        }),
                    )
                    .await
                {
                    eprintln!("Error handling connection: {:?}", err);
                }
            });
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
