//! Ultra-simplified middleware system
//! 
//! This middleware system allows using async functions directly as middleware,
//! providing a clean and intuitive API without boilerplate.

use std::{sync::Arc, future::Future, pin::Pin};
use crate::{RequestCtx, Response};

/// A middleware function that processes a request and passes it to the next handler
pub type Middleware = Arc<dyn Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// The next handler in the middleware chain
pub type Next = Arc<dyn Fn(RequestCtx) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// Trait for converting types into Next
pub trait IntoNext {
    fn into_next(self) -> Next;
}

/// Implement IntoNext for async functions that return Response
impl<F, Fut> IntoNext for F
where
    F: Fn(RequestCtx) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn into_next(self) -> Next {
        Arc::new(move |ctx| {
            Box::pin((self)(ctx))
        })
    }
}

/// Execute a middleware chain.
///
/// Takes an `Arc<Vec<Middleware>>` so inner `Next` closures can hold a cheap `Arc` clone
/// (reference-count bump) instead of a deep `Vec` clone per middleware layer.
pub async fn execute_chain(middlewares: Arc<Vec<Middleware>>, endpoint: Next, ctx: RequestCtx) -> Response {
    execute_at(middlewares, 0, endpoint, ctx).await
}

/// Recursive index-based executor — no Vec allocation per layer.
fn execute_at(
    middlewares: Arc<Vec<Middleware>>,
    index: usize,
    endpoint: Next,
    ctx: RequestCtx,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    Box::pin(async move {
        if index >= middlewares.len() {
            return endpoint(ctx).await;
        }
        let mw = middlewares[index].clone();
        let next: Next = Arc::new(move |ctx| {
            let mws = middlewares.clone();   // cheap Arc clone, not Vec clone
            let ep = endpoint.clone();
            execute_at(mws, index + 1, ep, ctx)
        });
        mw(ctx, next).await
    })
}
