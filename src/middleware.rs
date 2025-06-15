//! Middleware trait and built-in middleware implementations.

use std::{sync::Arc, time::Instant};
use async_trait::async_trait;
use crate::{RequestCtx, Response, Handler};

/// Trait for middleware components
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response;
}

/// Represents the next handler in the middleware chain
pub struct Next<'a> {
    pub endpoint: &'a dyn Handler,
    pub next_middleware: &'a [Arc<dyn Middleware>],
}

impl Next<'_> {
    /// Execute the next middleware or handler in the chain
    pub async fn run(mut self, ctx: RequestCtx) -> Response {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(ctx, self).await
        } else {
            self.endpoint.handle(ctx).await
        }
    }
}

/// Built-in access logging middleware
pub struct AccessLog;

#[async_trait]
impl Middleware for AccessLog {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        let start = Instant::now();
        let method = ctx.request.method().to_string();
        let path = ctx.request.uri().path().to_string();
        
        let response = next.run(ctx).await;
        
        println!(
            "{} {} {} {}ms",
            method,
            path,
            response.status().as_str(),
            start.elapsed().as_millis()
        );
        
        response
    }
}

/// CORS middleware
pub struct Cors {
    allow_origin: String,
    allow_methods: String,
    allow_headers: String,
}

impl Cors {
    pub fn new() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization".to_string(),
        }
    }

    pub fn allow_origin(mut self, origin: &str) -> Self {
        self.allow_origin = origin.to_string();
        self
    }

    pub fn allow_methods(mut self, methods: &str) -> Self {
        self.allow_methods = methods.to_string();
        self
    }

    pub fn allow_headers(mut self, headers: &str) -> Self {
        self.allow_headers = headers.to_string();
        self
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for Cors {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        let mut response = next.run(ctx).await;
        
        let headers = response.headers_mut();
        headers.insert("Access-Control-Allow-Origin", self.allow_origin.parse().unwrap());
        headers.insert("Access-Control-Allow-Methods", self.allow_methods.parse().unwrap());
        headers.insert("Access-Control-Allow-Headers", self.allow_headers.parse().unwrap());
        
        response
    }
}
