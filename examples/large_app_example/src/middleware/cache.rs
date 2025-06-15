// middleware/cache.rs
use ree::{middleware, MiddlewareFn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Simple in-memory cache middleware
pub fn cache_response(seconds: u64) -> MiddlewareFn {
    // Shared cache store
    let cache = Arc::new(Mutex::new(HashMap::new()));
    
    middleware(move |ctx, next| {
        let cache = cache.clone();
        let ttl = Duration::from_secs(seconds);
        
        async move {
            // Generate cache key from request path and query
            let cache_key = format!(
                "{}{}",
                ctx.request.uri().path(),
                ctx.request.uri().query().map_or("", |q| q)
            );
            
            // Try to get from cache
            {
                let cache_ref = cache.lock().unwrap();
                if let Some((timestamp, cached_response)) = cache_ref.get(&cache_key) {
                    if timestamp.elapsed() < ttl {
                        println!("Cache hit: {}", cache_key);
                        return cached_response.clone();
                    }
                }
            }
            
            // Cache miss - execute the handler
            println!("Cache miss: {}", cache_key);
            let response = next(ctx).await;
            
            // Store in cache
            let mut cache_ref = cache.lock().unwrap();
            cache_ref.insert(cache_key, (Instant::now(), response.clone()));
            
            response
        }
    })
}
