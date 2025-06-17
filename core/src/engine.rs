//! Main HTTP engine and router group implementations.

use std::{
    collections::HashMap, convert::Infallible, future::Future, net::SocketAddr, pin::Pin, sync::Arc,
};

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown};

use crate::{
    execute_chain, response::IntoResponse, Handler, Middleware, Next, RequestCtx, Response, Router, middleware::IntoNext,
};

/// A group of routes with shared prefix and middleware
pub struct RouterGroup {
    prefix: String,
    router: Router,
    middlewares: Vec<Middleware>,
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
    pub fn use_middleware<F, Fut>(&mut self, middleware: F)
    where
        F: Fn(RequestCtx, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let wrapped = move |ctx, next| {
            let fut = middleware(ctx, next);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Response> + Send>>
        };
        self.middlewares.push(Arc::new(wrapped));
    }

    /// Handle a request using this group's router
    pub async fn handle_request(&self, ctx: RequestCtx) -> Response {
        self.router.handle_request(ctx).await
    }
}

/// Main HTTP engine for building web applications
#[derive(Default)]
pub struct Engine {
    router: Router,
    groups: HashMap<String, RouterGroup>,
    middlewares: Vec<Middleware>,
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
    pub fn use_middleware<F, Fut>(&mut self, middleware: F)
    where
        F: Fn(RequestCtx, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let wrapped = move |ctx, next| {
            let fut = middleware(ctx, next);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Response> + Send>>
        };
        self.middlewares.push(Arc::new(wrapped));
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

        // Pre-process groups for optimal matching
        let mut group_data: Vec<(String, Arc<RouterGroup>)> = self
            .groups
            .into_iter()
            .map(|(prefix, group)| (prefix, Arc::new(group)))
            .collect();

        // Sort by prefix length (longest first) for better matching
        group_data.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let router = Arc::new(self.router);
        let global_middlewares = Arc::new(self.middlewares);
        let groups = Arc::new(group_data);

        // Pre-calculate if we have any middleware for optimization
        let has_global_middleware = !global_middlewares.is_empty();
        // hyper graceful shutdown
        let graceful = GracefulShutdown::new();

        println!("🚀 Server running on http://{}", addr);

        loop {
            tokio::select! {
                Ok((stream, remote_addr)) = listener.accept() => {
                    let io = TokioIo::new(stream);
                    let router = router.clone();
                    let global_middlewares = global_middlewares.clone();
                    let groups = groups.clone();

                    tokio::task::spawn(async move {
                        let service = service_fn(move |req| {
                            let router = router.clone();
                            let global_middlewares = global_middlewares.clone();
                            let groups = groups.clone();

                            async move {
                                let path = req.uri().path();

                                // Fast path matching for groups
                                let matched_group = groups
                                    .iter()
                                    .find(|(prefix, _)| path.starts_with(prefix))
                                    .map(|(_, group)| group.clone());

                                let ctx = match RequestCtx::new(req).await {
                                    Ok(ctx) => ctx,
                                    Err(e) => {
                                        eprintln!("Request context error: {:?}", e);
                                        return Ok("Bad Request".into_response());
                                    }
                                };

                                let response = if let Some(group) = matched_group {
                                    // Group request handling
                                    let has_group_middleware = !group.middlewares.is_empty();

                                    if !has_global_middleware && !has_group_middleware {
                                        // Fast path: no middleware at all
                                        group.handle_request(ctx).await
                                    } else {
                                        // Middleware path
                                        let mut combined_middlewares = Vec::with_capacity(
                                            global_middlewares.len() + group.middlewares.len()
                                        );
                                        combined_middlewares.extend(global_middlewares.iter().cloned());
                                        combined_middlewares.extend(group.middlewares.iter().cloned());

                                        let endpoint = (move |ctx| {
                                            let group = group.clone();
                                            async move { group.handle_request(ctx).await }
                                        }).into_next();

                                        execute_chain(&combined_middlewares, endpoint, ctx).await
                                    }
                                } else {
                                    // Main router handling
                                    if !has_global_middleware {
                                        // Fast path: no middleware
                                        router.handle_request(ctx).await
                                    } else {
                                        // Middleware path
                                        let endpoint = (move |ctx| {
                                            let router = router.clone();
                                            async move { router.handle_request(ctx).await }
                                        }).into_next();

                                        execute_chain(&global_middlewares, endpoint, ctx).await
                                    }
                                };

                                Ok::<_, Infallible>(response)
                            }
                        });

                        if let Err(err) = http1::Builder::new()
                            .serve_connection(io, service)
                            .await
                        {
                            eprintln!("Connection error {}: {:?}", remote_addr, err);
                        }
                    });
                }

                _ = tokio::signal::ctrl_c() => {
                    drop(listener);
                    eprintln!("\n🛑 Graceful shutdown signal received");
                    break;
                }
            }
        }
        tokio::select! {
            _ = graceful.shutdown() => {
                eprintln!("✅ All connections gracefully closed");
            },
            _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                eprintln!("⏰ Timed out waiting for all connections to close");
            }
        }

        Ok(())
    }
}
