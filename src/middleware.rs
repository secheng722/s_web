//! 简化的函数式中间件系统

use std::{sync::Arc, future::Future, pin::Pin};
use crate::{RequestCtx, Response};

/// 中间件函数类型
pub type MiddlewareFn = Arc<dyn Fn(RequestCtx, MiddlewareNext) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// 下一个处理器的类型
pub type MiddlewareNext = Arc<dyn Fn(RequestCtx) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

/// 执行中间件链的函数
pub fn execute_middleware_chain(
    middlewares: &[MiddlewareFn],
    endpoint: MiddlewareNext,
    ctx: RequestCtx,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    if middlewares.is_empty() {
        return endpoint(ctx);
    }
    
    let (first, rest) = middlewares.split_first().unwrap();
    let next = create_next_fn(rest, endpoint);
    first(ctx, next)
}

/// 创建下一个处理器函数
fn create_next_fn(
    remaining_middlewares: &[MiddlewareFn],
    endpoint: MiddlewareNext,
) -> MiddlewareNext {
    let remaining = remaining_middlewares.to_vec();
    Arc::new(move |ctx| {
        execute_middleware_chain(&remaining, endpoint.clone(), ctx)
    })
}

/// 创建中间件的便利函数
pub fn middleware<F, Fut>(f: F) -> MiddlewareFn
where
    F: Fn(RequestCtx, MiddlewareNext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    Arc::new(move |ctx, next| Box::pin(f(ctx, next)))
}
