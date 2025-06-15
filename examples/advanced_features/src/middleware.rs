use std::time::Instant;
use async_trait::async_trait;
use ree::{RequestCtx, Response, ResponseBuilder, Middleware, Next, StatusCode};

/// 自定义日志中间件
pub struct CustomLogger {
    app_name: String,
}

impl CustomLogger {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
        }
    }
}

#[async_trait]
impl Middleware for CustomLogger {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        let start = Instant::now();
        let method = ctx.request.method().to_string();
        let path = ctx.request.uri().path().to_string();
        let user_agent = ctx.request
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Unknown");
        
        println!("[{}] 🔍 {} {} - User-Agent: {}", 
                self.app_name, method, path, user_agent);
        
        let response = next.run(ctx).await;
        
        println!("[{}] ✅ {} {} {} - {}ms", 
                self.app_name,
                method, 
                path, 
                response.status().as_str(),
                start.elapsed().as_millis());
        
        response
    }
}

/// 简单的限流中间件
pub struct RateLimiter {
    max_requests: u32,
}

impl RateLimiter {
    pub fn new(max_requests: u32) -> Self {
        Self { max_requests }
    }
}

#[async_trait]
impl Middleware for RateLimiter {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        // 这里应该实现真正的限流逻辑，为了演示简化处理
        println!("🚦 Rate limit check: max {} requests", self.max_requests);
        
        // 模拟限流检查
        let request_count = 50; // 假设当前请求数
        if request_count > self.max_requests {
            return ResponseBuilder::too_many_requests_json(
                r#"{"error": "Rate limit exceeded", "message": "请求过于频繁，请稍后再试"}"#,
            );
        }
        
        next.run(ctx).await
    }
}

/// 认证中间件
pub struct AuthMiddleware {
    required_token: String,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            required_token: "admin-secret-token".to_string(),
        }
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        // 检查Authorization头
        let auth_header = ctx.request
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        
        match auth_header {
            Some(token) if token == format!("Bearer {}", self.required_token) => {
                println!("🔐 Auth success for admin endpoint");
                next.run(ctx).await
            }
            Some(_) => {
                println!("🚫 Auth failed: invalid token");
                ResponseBuilder::unauthorized_json(
                    r#"{"error": "Unauthorized", "message": "无效的认证令牌"}"#,
                )
            }
            None => {
                println!("🚫 Auth failed: missing token");
                ResponseBuilder::unauthorized_json(
                    r#"{"error": "Unauthorized", "message": "缺少认证令牌"}"#,
                )
            }
        }
    }
}
