use s_web::{IntoResponse, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde_json::json;

use crate::{
    dto::{CreatePostRequest, PostResponse, UpdatePostRequest},
    error::AppError,
    repository::BlogRepository,
};

fn parse_id(ctx: &RequestCtx) -> Result<i64, AppError> {
    ctx.get_param("id")
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|id| *id > 0)
        .ok_or_else(|| AppError::BadRequest("id 必须是正整数".to_string()))
}

pub async fn list_posts(ctx: RequestCtx, repo: BlogRepository) -> Response {
    let keyword = ctx.query_param("q");
    let published_only = matches!(
        ctx.query_param("published").as_deref(),
        Some("1") | Some("true") | Some("yes")
    );

    match repo.list_posts(keyword, published_only).await {
        Ok(posts) => {
            let list: Vec<PostResponse> = posts.into_iter().map(Into::into).collect();
            json!({ "count": list.len(), "posts": list }).into_response()
        }
        Err(e) => e.to_response(),
    }
}

pub async fn get_post(ctx: RequestCtx, repo: BlogRepository) -> Response {
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(e) => return e.to_response(),
    };

    match repo.get_post(id).await {
        Ok(post) => json!(PostResponse::from(post)).into_response(),
        Err(e) => e.to_response(),
    }
}

pub async fn create_post(mut ctx: RequestCtx, repo: BlogRepository) -> Response {
    let payload: CreatePostRequest = match ctx.json().await {
        Ok(v) => v,
        Err(_) => return AppError::BadRequest("请求体必须是合法 JSON".to_string()).to_response(),
    };

    if payload.title.trim().is_empty() || payload.content.trim().is_empty() {
        return AppError::BadRequest("title 和 content 不能为空".to_string()).to_response();
    }

    match repo.create_post(payload).await {
        Ok(post) => ResponseBuilder::new()
            .status(StatusCode::CREATED)
            .content_type("application/json; charset=utf-8")
            .body(json!(PostResponse::from(post)).to_string()),
        Err(e) => e.to_response(),
    }
}

pub async fn update_post(mut ctx: RequestCtx, repo: BlogRepository) -> Response {
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(e) => return e.to_response(),
    };

    let payload: UpdatePostRequest = match ctx.json().await {
        Ok(v) => v,
        Err(_) => return AppError::BadRequest("请求体必须是合法 JSON".to_string()).to_response(),
    };

    if payload.title.trim().is_empty() || payload.content.trim().is_empty() {
        return AppError::BadRequest("title 和 content 不能为空".to_string()).to_response();
    }

    match repo.update_post(id, payload).await {
        Ok(post) => json!(PostResponse::from(post)).into_response(),
        Err(e) => e.to_response(),
    }
}

pub async fn publish_post(ctx: RequestCtx, repo: BlogRepository) -> Response {
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(e) => return e.to_response(),
    };

    match repo.publish_post(id).await {
        Ok(post) => json!(PostResponse::from(post)).into_response(),
        Err(e) => e.to_response(),
    }
}

pub async fn delete_post(ctx: RequestCtx, repo: BlogRepository) -> Response {
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(e) => return e.to_response(),
    };

    match repo.delete_post(id).await {
        Ok(true) => ResponseBuilder::new().status(StatusCode::NO_CONTENT).body(String::new()),
        Ok(false) => AppError::NotFound("文章不存在".to_string()).to_response(),
        Err(e) => e.to_response(),
    }
}
