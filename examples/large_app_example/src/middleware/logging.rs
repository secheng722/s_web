// middleware/logging.rs
use ree::{middleware, MiddlewareFn};
use std::time::Instant;

/// Request logger middleware
pub fn request_logger() -> MiddlewareFn {
    middleware(|ctx, next| async move {
        let method = ctx.request.method().to_string();
        let path = ctx.request.uri().path().to_string();
        
        println!("Request: {} {}", method, path);
        
        let response = next(ctx).await;
        
        println!(
            "Response: {} {} {}",
            method,
            path,
            response.status().as_str()
        );
        
        response
    })
}

/// Timer middleware
pub fn timer() -> MiddlewareFn {
    middleware(|ctx, next| async move {
        let start = Instant::now();
        let method = ctx.request.method().to_string();
        let path = ctx.request.uri().path().to_string();
        
        let response = next(ctx).await;
        
        let elapsed = start.elapsed().as_millis();
        println!("Time: {} {} - {}ms", method, path, elapsed);
        
        response
    })
}
