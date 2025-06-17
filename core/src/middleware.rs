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

/// Execute a chain of middlewares
pub async fn execute_chain(middlewares: &[Middleware], endpoint: Next, ctx: RequestCtx) -> Response {
    if middlewares.is_empty() {
        return endpoint(ctx).await;
    }
    
    let (first, rest) = middlewares.split_first().unwrap();
    let next = create_next(rest, endpoint);
    first(ctx, next).await
}

/// Create the next middleware function
fn create_next(remaining: &[Middleware], endpoint: Next) -> Next {
    let middlewares = remaining.to_vec();
    let endpoint = endpoint.clone();
    
    Arc::new(move |ctx| {
        let middlewares = middlewares.clone();
        let endpoint = endpoint.clone();
        
        Box::pin(async move {
            execute_chain(&middlewares, endpoint, ctx).await
        })
    })
}
