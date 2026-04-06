//! Main HTTP engine and router group implementations.

use std::{
    collections::HashMap, convert::Infallible, future::Future, net::SocketAddr, pin::Pin, sync::Arc,
};

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown};

use crate::{
    Handler, Middleware, Next, RequestCtx, Response, Router, execute_chain, middleware::IntoNext,
    swagger::SwaggerInfo,
};

/// Type alias for lifecycle hooks
type LifecycleHook = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Pre-processed server data ready for the accept loop
type PreprocessedGroup = (String, Arc<RouterGroup>, Arc<Vec<Middleware>>);

struct ServerContext {
    router: Arc<Router>,
    groups: Arc<Vec<PreprocessedGroup>>,
    global_middlewares: Arc<Vec<Middleware>>,
    has_global_middleware: bool,
}

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
    pub fn get(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("GET", path, handler);
        self
    }

    /// Add a POST route to this group
    pub fn post(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("POST", path, handler);
        self
    }

    /// Add a PUT route to this group
    pub fn put(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("PUT", path, handler);
        self
    }

    /// Add a DELETE route to this group
    pub fn delete(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("DELETE", path, handler);
        self
    }

    /// Add middleware to this group
    pub fn use_middleware<F, Fut>(&mut self, middleware: F) -> &mut Self
    where
        F: Fn(RequestCtx, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let wrapped = move |ctx, next| {
            let fut = middleware(ctx, next);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Response> + Send>>
        };
        self.middlewares.push(Arc::new(wrapped));
        self
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
    startup_hooks: Vec<LifecycleHook>,
    shutdown_hooks: Vec<LifecycleHook>,
    swagger_info: HashMap<String, SwaggerInfo>,
    /// Whether to expose Swagger UI at /docs/
    swagger_enabled: bool,
}

impl Engine {
    /// Create a new Engine instance
    pub fn new() -> Self {
        Engine {
            router: Router::new(),
            groups: HashMap::new(),
            middlewares: Vec::new(),
            startup_hooks: Vec::new(),
            shutdown_hooks: Vec::new(),
            swagger_info: HashMap::new(),
            swagger_enabled: false,
        }
    }

    /// Enable the built-in Swagger UI at `/docs/` and `/docs/swagger.json`.
    pub fn enable_swagger(&mut self) -> &mut Self {
        self.swagger_enabled = true;
        self
    }

    /// Add global middleware
    pub fn use_middleware<F, Fut>(&mut self, middleware: F) -> &mut Self
    where
        F: Fn(RequestCtx, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let wrapped = move |ctx, next| {
            let fut = middleware(ctx, next);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Response> + Send>>
        };
        self.middlewares.push(Arc::new(wrapped));
        self
    }

    /// Add a startup hook that will be executed when the server starts
    pub fn on_startup<F, Fut>(mut self, f: F) -> Self 
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let wrapped = move || {
            let fut = f();
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send>>
        };
        self.startup_hooks.push(Box::new(wrapped));
        self
    }

    /// Add a shutdown hook that will be executed during graceful shutdown
    pub fn on_shutdown<F, Fut>(mut self, f: F) -> Self 
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let wrapped = move || {
            let fut = f();
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send>>
        };
        self.shutdown_hooks.push(Box::new(wrapped));
        self
    }

    /// Create (or retrieve) a route group with the given prefix.
    /// Calling `group()` with the same prefix twice returns the existing group
    /// rather than silently discarding previously registered routes.
    pub fn group(&mut self, prefix: &str) -> &mut RouterGroup {
        self.groups
            .entry(prefix.to_string())
            .or_insert_with(|| RouterGroup::new(prefix.to_string()))
    }

    /// Add a route to the main router
    pub fn add_route(&mut self, method: &str, pattern: &str, handler: impl Handler) {
        let handler = Box::new(handler);
        self.router.add_route(method, pattern, handler);
    }

    /// Add a GET route
    pub fn get(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("GET", path, handler);
        self
    }

    /// Add a GET route with swagger info
    pub fn get_with_swagger(&mut self, path: &str, handler: impl Handler, swagger_info: SwaggerInfo) -> &mut Self {
        self.add_route("GET", path, handler);
        self.swagger_for_route("GET", path, swagger_info);
        self
    }

    /// Add a POST route
    pub fn post(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("POST", path, handler);
        self
    }

    /// Add a POST route with swagger info
    pub fn post_with_swagger(&mut self, path: &str, handler: impl Handler, swagger_info: SwaggerInfo) -> &mut Self {
        self.add_route("POST", path, handler);
        self.swagger_for_route("POST", path, swagger_info);
        self
    }

    /// Add a PUT route
    pub fn put(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("PUT", path, handler);
        self
    }

    /// Add a PUT route with swagger info
    pub fn put_with_swagger(&mut self, path: &str, handler: impl Handler, swagger_info: SwaggerInfo) -> &mut Self {
        self.add_route("PUT", path, handler);
        self.swagger_for_route("PUT", path, swagger_info);
        self
    }

    /// Add a DELETE route
    pub fn delete(&mut self, path: &str, handler: impl Handler) -> &mut Self {
        self.add_route("DELETE", path, handler);
        self
    }

    /// Add a DELETE route with swagger info
    pub fn delete_with_swagger(&mut self, path: &str, handler: impl Handler, swagger_info: SwaggerInfo) -> &mut Self {
        self.add_route("DELETE", path, handler);
        self.swagger_for_route("DELETE", path, swagger_info);
        self
    }

    /// Set swagger info for a specific route
    pub fn swagger_for_route(&mut self, method: &str, path: &str, swagger_info: SwaggerInfo) -> &mut Self {
        let route_key = format!("{}-{}", method.to_uppercase(), path);
        self.swagger_info.insert(route_key, swagger_info);
        self
    }

    fn add_swagger_endpoints(&mut self) {
        let mut all_routes = Vec::new();
        all_routes.extend(self.router.get_all_routes());

        for group in self.groups.values() {
            all_routes.extend(group.router.get_all_routes());
        }

        if all_routes.is_empty() {
            return;
        }

        let json_path = "/docs/swagger.json";
        let ui_path = "/docs/";
        let swagger_info = self.swagger_info.clone();

        self.get(json_path, move |_ctx: RequestCtx| {
            let routes = all_routes.clone();
            let swagger_info = swagger_info.clone();
            async move {
                use crate::response::ResponseBuilder;
                use crate::swagger::generate_enhanced_swagger_json;

                let json = generate_enhanced_swagger_json(&routes, &swagger_info);
                ResponseBuilder::new()
                    .status(hyper::StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json)
            }
        });

        self.get(ui_path, |_ctx: RequestCtx| async {
            use crate::response::ResponseBuilder;
            use crate::swagger::generate_swagger_ui;

            let html = generate_swagger_ui("/docs/swagger.json");
            ResponseBuilder::new()
                .status(hyper::StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(html)
        });
    }

    /// Start the HTTP server
    pub async fn run(mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        for hook in &self.startup_hooks {
            hook().await;
        }

        let addr = addr.parse::<SocketAddr>()?;
        println!("🚀 Server running on http://{addr}");
        let listener = tokio::net::TcpListener::bind(addr).await?;

        if self.swagger_enabled {
            self.add_swagger_endpoints();
            println!("📖 Swagger UI available at http://{addr}/docs/");
        }

        let shutdown_hooks = std::mem::take(&mut self.shutdown_hooks);
        let server_ctx = self.build_server_context();
        let graceful = GracefulShutdown::new();

        accept_loop(listener, server_ctx, &graceful).await;

        for hook in &shutdown_hooks {
            hook().await;
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

    /// Pre-process groups and middleware for the request handling path
    fn build_server_context(self) -> ServerContext {
        let global_middlewares = Arc::new(self.middlewares);

        let mut group_data: Vec<(String, Arc<RouterGroup>, Arc<Vec<Middleware>>)> = self
            .groups
            .into_iter()
            .map(|(prefix, group)| {
                let mut combined =
                    Vec::with_capacity(global_middlewares.len() + group.middlewares.len());
                combined.extend(global_middlewares.iter().cloned());
                combined.extend(group.middlewares.iter().cloned());
                (prefix, Arc::new(group), Arc::new(combined))
            })
            .collect();

        group_data.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let has_global_middleware = !global_middlewares.is_empty();

        ServerContext {
            router: Arc::new(self.router),
            groups: Arc::new(group_data),
            global_middlewares,
            has_global_middleware,
        }
    }
}

/// Accept and handle incoming connections
async fn accept_loop(
    listener: tokio::net::TcpListener,
    ctx: ServerContext,
    graceful: &GracefulShutdown,
) {
    loop {
        tokio::select! {
            Ok((stream, remote_addr)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let router = ctx.router.clone();
                let groups = ctx.groups.clone();
                let global_middlewares = ctx.global_middlewares.clone();
                let has_global_middleware = ctx.has_global_middleware;

                let conn = http1::Builder::new()
                    .serve_connection(io, service_fn(move |req| {
                        let router = router.clone();
                        let groups = groups.clone();
                        let global_middlewares = global_middlewares.clone();

                        async move {
                            let path = req.uri().path().to_owned();

                            let matched_group = groups
                                .iter()
                                .find(|(prefix, _, _)| {
                                    path.starts_with(prefix.as_str())
                                        && (path.len() == prefix.len()
                                            || path.as_bytes().get(prefix.len()) == Some(&b'/'))
                                })
                                .map(|(_, group, middlewares)| (group.clone(), middlewares.clone()));

                            let ctx = RequestCtx::new(req).with_remote_addr(remote_addr);

                            let response = if let Some((group, combined_middlewares)) = matched_group {
                                if combined_middlewares.is_empty() {
                                    group.handle_request(ctx).await
                                } else {
                                    let endpoint = (move |ctx| {
                                        let group = group.clone();
                                        async move { group.handle_request(ctx).await }
                                    })
                                    .into_next();
                                    execute_chain(combined_middlewares, endpoint, ctx).await
                                }
                            } else if !has_global_middleware {
                                router.handle_request(ctx).await
                            } else {
                                let endpoint = (move |ctx| {
                                    let router = router.clone();
                                    async move { router.handle_request(ctx).await }
                                })
                                .into_next();
                                execute_chain(global_middlewares, endpoint, ctx).await
                            };

                            Ok::<_, Infallible>(response)
                        }
                    }));

                let fut = graceful.watch(conn);
                tokio::spawn(async move {
                    if let Err(err) = fut.await {
                        eprintln!("Connection error {remote_addr}: {err:?}");
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => {
                drop(listener);
                eprintln!("\n🛑 Graceful shutdown signal received");
                return;
            }
        }
    }
}
