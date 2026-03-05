//! Handler trait and implementations for request processing.

use std::{future::Future, pin::Pin};
use crate::{RequestCtx, Response, response::IntoResponse};

/// Trait for handling HTTP requests.
/// Uses explicit `Pin<Box<dyn Future>>` return to keep the trait object-safe
/// (required for `Box<dyn Handler>`) without the `async_trait` proc-macro.
pub trait Handler: Send + Sync + 'static {
    fn handle(&self, ctx: RequestCtx) -> Pin<Box<dyn Future<Output = Response> + Send>>;
}

/// Blanket implementation for any async function or closure that returns an `IntoResponse` type.
impl<F, Fut, R> Handler for F
where
    F: Fn(RequestCtx) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + Send + 'static,
{
    fn handle(&self, ctx: RequestCtx) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let fut = (self)(ctx);
        Box::pin(async move { fut.await.into_response() })
    }
}