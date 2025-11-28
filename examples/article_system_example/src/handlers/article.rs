use chrono::Utc;
use s_web::{IntoResponse, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::AppState;
use crate::models::{Article, CreateArticleDto, UpdateArticleDto};

// 创建文章
pub async fn create_article(state: Arc<AppState>, mut ctx: RequestCtx) -> Response {
    // 从参数中获取用户ID
    let user_id = match ctx.get_param("user_id") {
        Some(id) => {
            println!("create_article: user_id found = {id}");
            id.to_string()
        }
        None => {
            println!("create_article: user_id not found in params");
            return ResponseBuilder::new()
                .status(StatusCode::UNAUTHORIZED)
                .body("Not authenticated");
        }
    };

    // 解析请求体
    let article_dto = match ctx.json::<CreateArticleDto>().await {
        Ok(article) => article,
        Err(e) => {
            return ResponseBuilder::new()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Invalid request body: {e}"));
        }
    };

    let now = Utc::now();
    let article_id = Uuid::new_v4();

    // 插入文章
    let result = sqlx::query(
        r#"
        INSERT INTO articles (id, title, content, author_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(article_id.to_string())
    .bind(&article_dto.title)
    .bind(&article_dto.content)
    .bind(user_id)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            // 获取刚插入的文章
            let article_result = sqlx::query_as::<_, Article>(
                r#"
                SELECT * FROM articles WHERE id = ?
                "#,
            )
            .bind(article_id.to_string())
            .fetch_one(&state.db)
            .await;

            match article_result {
                Ok(article) => json!(article).into_response(),
                Err(e) => ResponseBuilder::new()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Database error: {e}")),
            }
        }
        Err(e) => ResponseBuilder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Database error: {e}")),
    }
}

// 获取所有文章
pub async fn get_all_articles(state: Arc<AppState>, _ctx: RequestCtx) -> Response {
    // 查询所有文章
    let articles_result = sqlx::query_as::<_, Article>(
        r#"
        SELECT * FROM articles ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await;

    match articles_result {
        Ok(articles) => json!(articles).into_response(),
        Err(e) => ResponseBuilder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Database error: {e}")),
    }
}

// 获取单个文章
pub async fn get_article(state: Arc<AppState>, ctx: RequestCtx) -> Response {
    // 从路径参数获取文章ID
    let article_id = match ctx.get_param("id") {
        Some(id) => id,
        None => {
            return ResponseBuilder::new()
                .status(StatusCode::BAD_REQUEST)
                .body("Missing article ID");
        }
    };

    // 查询文章
    let article_result = sqlx::query_as::<_, Article>(
        r#"
        SELECT * FROM articles WHERE id = ?
        "#,
    )
    .bind(article_id)
    .fetch_optional(&state.db)
    .await;

    match article_result {
        Ok(Some(article)) => json!(article).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Article not found").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
            .into_response(),
    }
}

// 更新文章
pub async fn update_article(state: Arc<AppState>, mut ctx: RequestCtx) -> Response {
    // 从参数中获取用户ID
    let user_id = match ctx.get_param("user_id") {
        Some(id) => id.to_string(),
        None => {
            return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response();
        }
    };

    // 从路径参数获取文章ID
    let article_id = match ctx.get_param("id") {
        Some(id) => id.to_string(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing article ID").into_response();
        }
    };

    // 检查文章是否存在并且属于当前用户
    let article_check = sqlx::query_as::<_, Article>(
        r#"
        SELECT * FROM articles WHERE id = ? AND author_id = ?
        "#,
    )
    .bind(&article_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await;

    let article = match article_check {
        Ok(Some(article)) => article,
        Ok(None) => {
            return ResponseBuilder::new()
                .status(StatusCode::FORBIDDEN)
                .body("Article not found or you don't have permission");
        }
        Err(e) => {
            return ResponseBuilder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Database error: {e}"));
        }
    };

    // 解析请求体
    let update_dto = match ctx.json::<UpdateArticleDto>().await {
        Ok(update) => update,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Invalid request body: {e}"),
            )
                .into_response();
        }
    };

    // 更新文章
    let now = Utc::now();
    let title = update_dto.title.unwrap_or_else(|| article.title.clone());
    let content = update_dto
        .content
        .unwrap_or_else(|| article.content.clone());

    let update_result = sqlx::query(
        r#"
        UPDATE articles SET title = ?, content = ?, updated_at = ? WHERE id = ?
        "#,
    )
    .bind(&title)
    .bind(&content)
    .bind(now)
    .bind(&article_id)
    .execute(&state.db)
    .await;

    match update_result {
        Ok(_) => {
            // 获取更新后的文章
            let updated_article = sqlx::query_as::<_, Article>(
                r#"
                SELECT * FROM articles WHERE id = ?
                "#,
            )
            .bind(&article_id)
            .fetch_one(&state.db)
            .await;

            match updated_article {
                Ok(article) => json!(article).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {e}"),
                )
                    .into_response(),
            }
        }
        Err(e) => ResponseBuilder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to update article: {e}")),
    }
}

// 删除文章
pub async fn delete_article(state: Arc<AppState>, ctx: RequestCtx) -> Response {
    // 从扩展中获取用户ID
    let user_id = match ctx.get_param("user_id") {
        Some(id) => id,
        None => {
            return ResponseBuilder::new()
                .status(StatusCode::UNAUTHORIZED)
                .body("Not authenticated");
        }
    };

    // 从路径参数获取文章ID
    let article_id = match ctx.get_param("id") {
        Some(id) => id,
        None => {
            return ResponseBuilder::new()
                .status(StatusCode::BAD_REQUEST)
                .body("Missing article ID");
        }
    };

    // 检查文章是否存在并且属于当前用户
    let article_check = sqlx::query(
        r#"
        SELECT id FROM articles WHERE id = ? AND author_id = ?
        "#,
    )
    .bind(article_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await;

    match article_check {
        Ok(Some(_)) => {
            // 删除文章
            let delete_result = sqlx::query(
                r#"
                DELETE FROM articles WHERE id = ?
                "#,
            )
            .bind(article_id)
            .execute(&state.db)
            .await;

            match delete_result {
                Ok(_) => ResponseBuilder::new()
                    .status(StatusCode::NO_CONTENT)
                    .body(""),
                Err(e) => ResponseBuilder::new()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Failed to delete article: {e}")),
            }
        }
        Ok(None) => ResponseBuilder::new()
            .status(StatusCode::FORBIDDEN)
            .body("Article not found or you don't have permission"),
        Err(e) => ResponseBuilder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Database error: {e}")),
    }
}
