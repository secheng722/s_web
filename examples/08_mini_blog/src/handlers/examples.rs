use s_web::{IntoResponse, RequestCtx, Response};
use serde_json::json;

use crate::{
    dto::{BlogStatsResponse, CommentResponse, PostResponse, QuickPublishRequest},
    error::AppError,
    repository::BlogRepository,
};

pub async fn stats(_ctx: RequestCtx, repo: BlogRepository) -> Response {
    match repo.stats().await {
        Ok((total_posts, published_posts, total_comments)) => {
            let payload = BlogStatsResponse {
                total_posts,
                published_posts,
                total_comments,
            };
            json!(payload).into_response()
        }
        Err(e) => e.to_response(),
    }
}

pub async fn reset_demo(_ctx: RequestCtx, repo: BlogRepository) -> Response {
    match repo.reset_demo_data().await {
        Ok(()) => json!({ "ok": true, "message": "示例数据已重置" }).into_response(),
        Err(e) => e.to_response(),
    }
}

pub async fn quick_publish(mut ctx: RequestCtx, repo: BlogRepository) -> Response {
    let payload: QuickPublishRequest = match ctx.json().await {
        Ok(v) => v,
        Err(_) => return AppError::BadRequest("请求体必须是合法 JSON".to_string()).to_response(),
    };

    if payload.title.trim().is_empty()
        || payload.content.trim().is_empty()
        || payload.first_comment_author.trim().is_empty()
        || payload.first_comment_content.trim().is_empty()
    {
        return AppError::BadRequest("所有字段都不能为空".to_string()).to_response();
    }

    match repo.quick_publish_with_comment(payload).await {
        Ok((post, comment)) => json!({
            "post": PostResponse::from(post),
            "first_comment": CommentResponse::from(comment),
            "message": "事务示例执行成功：已创建文章并写入首条评论"
        })
        .into_response(),
        Err(e) => e.to_response(),
    }
}
