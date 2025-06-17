//! Handler trait and implementations for request processing.

use async_trait::async_trait;
use std::future::Future;
use crate::{RequestCtx, Response, response::IntoResponse};

/// Trait for handling HTTP requests
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, ctx: RequestCtx) -> Response;
}

/// Implement Handler for async functions that return IntoResponse types (包括 Response)
#[async_trait]
impl<F, Fut, R> Handler for F
where
    F: Fn(RequestCtx) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + Send + 'static,
{
    async fn handle(&self, ctx: RequestCtx) -> Response {
        (self)(ctx).await.into_response()
    }
}