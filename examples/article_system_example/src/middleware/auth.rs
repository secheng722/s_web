use std::sync::Arc;

use chrono::Utc;
use ree::{IntoResponse, Next, RequestCtx, Response, StatusCode};

use crate::config::AppState;

// 日志中间件
pub async fn logging_middleware(
    prefix: &'static str,
    ctx: RequestCtx,
    next: ree::Next,
) -> Response {
    let start = Utc::now();
    println!(
        "{} - [{}] Request: {} {}",
        prefix,
        start.format("%Y-%m-%d %H:%M:%S"),
        ctx.request.method(),
        ctx.request.uri()
    );

    let response = next(ctx).await;

    let duration = Utc::now() - start;
    println!(
        "{} - [{}] Response: {} (took {}ms)",
        prefix,
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        response.status(),
        duration.num_milliseconds()
    );

    response
}

// 认证中间件
pub async fn auth_middleware(state: Arc<AppState>, ctx: RequestCtx, next: Next) -> Response {
    if ctx.request.method() == "GET" {
        // 如果是 GET 请求，直接放行
        return next(ctx).await;
    }

    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String, // 用户ID
        exp: usize,  // 过期时间
    }

    // 从请求头中获取令牌
    let auth_header = ctx.request.headers().get("Authorization");
    if auth_header.is_none() {
        return (StatusCode::UNAUTHORIZED, "Missing authorization header").into_response();
    }
    let auth_header = auth_header.unwrap();

    // 解析令牌
    let auth_str = auth_header.to_str();
    if auth_str.is_err() {
        return (StatusCode::UNAUTHORIZED, "Invalid authorization header").into_response();
    }
    let auth_str = auth_str.unwrap();

    // 检查令牌格式
    if !auth_str.starts_with("Bearer ") {
        return (StatusCode::UNAUTHORIZED, "Invalid authorization format").into_response();
    }

    // 提取令牌
    let token = &auth_str[7..];

    // 验证令牌
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    );
    if token_data.is_err() {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    }
    let claims = token_data.unwrap().claims;

    // 将用户ID添加到请求中
    let mut ctx = ctx;
    ctx.params.insert("user_id".to_string(), claims.sub.clone());
    // 继续处理请求
    next(ctx).await
}
