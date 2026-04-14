use s_web::{IntoResponse, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde_json::json;

use crate::{
    dto::{CommentResponse, CreateCommentRequest},
    error::AppError,
    repository::BlogRepository,
};

fn parse_post_id(ctx: &RequestCtx) -> Result<i64, AppError> {
    ctx.get_param("id")
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|id| *id > 0)
        .ok_or_else(|| AppError::BadRequest("文章 id 必须是正整数".to_string()))
}

pub async fn list_comments(ctx: RequestCtx, repo: BlogRepository) -> Response {
    let post_id = match parse_post_id(&ctx) {
        Ok(id) => id,
        Err(e) => return e.to_response(),
    };

    match repo.list_comments(post_id).await {
        Ok(comments) => {
            let items: Vec<CommentResponse> = comments.into_iter().map(Into::into).collect();
            json!({ "count": items.len(), "comments": items }).into_response()
        }
        Err(e) => e.to_response(),
    }
}

pub async fn create_comment(mut ctx: RequestCtx, repo: BlogRepository) -> Response {
    let post_id = match parse_post_id(&ctx) {
        Ok(id) => id,
        Err(e) => return e.to_response(),
    };

    let payload: CreateCommentRequest = match ctx.json().await {
        Ok(v) => v,
        Err(_) => return AppError::BadRequest("请求体必须是合法 JSON".to_string()).to_response(),
    };

    if payload.author.trim().is_empty() || payload.content.trim().is_empty() {
        return AppError::BadRequest("author 和 content 不能为空".to_string()).to_response();
    }

    match repo.create_comment(post_id, payload).await {
        Ok(comment) => ResponseBuilder::new()
            .status(StatusCode::CREATED)
            .content_type("application/json; charset=utf-8")
            .body(json!(CommentResponse::from(comment)).to_string()),
        Err(e) => e.to_response(),
    }
}
