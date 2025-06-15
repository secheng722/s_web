// middleware/auth.rs
use ree::{middleware, MiddlewareFn, RequestCtx, ResponseBuilder};
use std::sync::Arc;

/// Authentication middleware
pub fn require_auth() -> MiddlewareFn {
    middleware(|ctx, next| async move {
        // Check for Authorization header
        if let Some(auth) = ctx.request.headers().get("Authorization") {
            if let Ok(auth_str) = auth.to_str() {
                // Validate token (simplified for demonstration)
                if auth_str.starts_with("Bearer ") {
                    // In a real application, you would validate the token
                    return next(ctx).await;
                }
            }
        }
        
        // Unauthorized
        ResponseBuilder::unauthorized_json(r#"{"error":"Authentication required"}"#)
    })
}

/// Role-based authorization middleware
pub fn require_role(role: &'static str) -> MiddlewareFn {
    middleware(move |ctx, next| async move {
        // In a real application, you would extract the role from the JWT token
        // This is simplified for demonstration
        if let Some(auth) = ctx.request.headers().get("Authorization") {
            if let Ok(auth_str) = auth.to_str() {
                if auth_str.contains(&format!("role={}", role)) {
                    return next(ctx).await;
                }
            }
        }
        
        // Forbidden
        ResponseBuilder::forbidden_json(r#"{"error":"Insufficient permissions"}"#)
    })
}
