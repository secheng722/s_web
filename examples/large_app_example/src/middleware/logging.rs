// middleware/logging.rs
use ree::{RequestCtx, Next, Response};
use std::time::Instant;

/// Request logger middleware
pub async fn request_logger(ctx: RequestCtx, next: Next) -> Response {
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
}

/// Timer middleware
pub async fn timer(ctx: RequestCtx, next: Next) -> Response {
    let start = Instant::now();
    let method = ctx.request.method().to_string();
    let path = ctx.request.uri().path().to_string();
    
    let response = next(ctx).await;
    
    let elapsed = start.elapsed().as_millis();
    println!("Time: {} {} - {}ms", method, path, elapsed);
    
    response
}
