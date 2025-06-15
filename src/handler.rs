//! Handler trait and implementations for request processing.

use async_trait::async_trait;
use crate::{RequestCtx, Response};

/// Trait for handling HTTP requests
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, ctx: RequestCtx) -> Response;
}

/// Implement Handler for async functions
#[async_trait]
impl<F, Fut> Handler for F
where
    F: Fn(RequestCtx) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Response> + Send + 'static,
{
    async fn handle(&self, ctx: RequestCtx) -> Response {
        (self)(ctx).await
    }
}
