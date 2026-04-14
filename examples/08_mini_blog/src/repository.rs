use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{
    dto::{CreateCommentRequest, CreatePostRequest, QuickPublishRequest, UpdatePostRequest},
    error::{AppError, AppResult},
    models::{Comment, Post},
};

#[derive(Clone)]
pub struct BlogRepository {
    pub pool: Arc<SqlitePool>,
}

impl BlogRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn list_posts(&self, keyword: Option<String>, published_only: bool) -> AppResult<Vec<Post>> {
        let posts = match (keyword, published_only) {
            (Some(k), true) if !k.trim().is_empty() => {
                let pattern = format!("%{}%", k.trim());
                sqlx::query_as::<_, Post>(
                    "SELECT id, title, content, is_published, created_at, updated_at FROM posts WHERE is_published = 1 AND (title LIKE ? OR content LIKE ?) ORDER BY id DESC",
                )
                .bind(pattern.clone())
                .bind(pattern)
                .fetch_all(self.pool.as_ref())
                .await?
            }
            (Some(k), false) if !k.trim().is_empty() => {
                let pattern = format!("%{}%", k.trim());
                sqlx::query_as::<_, Post>(
                    "SELECT id, title, content, is_published, created_at, updated_at FROM posts WHERE title LIKE ? OR content LIKE ? ORDER BY id DESC",
                )
                .bind(pattern.clone())
                .bind(pattern)
                .fetch_all(self.pool.as_ref())
                .await?
            }
            (_, true) => {
                sqlx::query_as::<_, Post>(
                    "SELECT id, title, content, is_published, created_at, updated_at FROM posts WHERE is_published = 1 ORDER BY id DESC",
                )
                .fetch_all(self.pool.as_ref())
                .await?
            }
            _ => {
                sqlx::query_as::<_, Post>(
                    "SELECT id, title, content, is_published, created_at, updated_at FROM posts ORDER BY id DESC",
                )
                .fetch_all(self.pool.as_ref())
                .await?
            }
        };

        Ok(posts)
    }

    pub async fn get_post(&self, id: i64) -> AppResult<Post> {
        let post = sqlx::query_as::<_, Post>(
            "SELECT id, title, content, is_published, created_at, updated_at FROM posts WHERE id = ?",
        )
        .bind(id)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(post)
    }

    pub async fn create_post(&self, payload: CreatePostRequest) -> AppResult<Post> {
        let result = sqlx::query(
            "INSERT INTO posts (title, content, is_published) VALUES (?, ?, 0)",
        )
        .bind(payload.title)
        .bind(payload.content)
        .execute(self.pool.as_ref())
        .await?;

        self.get_post(result.last_insert_rowid()).await
    }

    pub async fn update_post(&self, id: i64, payload: UpdatePostRequest) -> AppResult<Post> {
        let result = sqlx::query(
            "UPDATE posts SET title = ?, content = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
        )
        .bind(payload.title)
        .bind(payload.content)
        .bind(id)
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("文章不存在".to_string()));
        }

        self.get_post(id).await
    }

    pub async fn publish_post(&self, id: i64) -> AppResult<Post> {
        let result = sqlx::query(
            "UPDATE posts SET is_published = 1, updated_at = datetime('now', 'localtime') WHERE id = ?",
        )
        .bind(id)
        .execute(self.pool.as_ref())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("文章不存在".to_string()));
        }

        self.get_post(id).await
    }

    pub async fn delete_post(&self, id: i64) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM posts WHERE id = ?")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_comments(&self, post_id: i64) -> AppResult<Vec<Comment>> {
        let comments = sqlx::query_as::<_, Comment>(
            "SELECT id, post_id, author, content, created_at FROM comments WHERE post_id = ? ORDER BY id DESC",
        )
        .bind(post_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(comments)
    }

    pub async fn create_comment(&self, post_id: i64, payload: CreateCommentRequest) -> AppResult<Comment> {
        let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM posts WHERE id = ?")
            .bind(post_id)
            .fetch_optional(self.pool.as_ref())
            .await?;

        if exists.is_none() {
            return Err(AppError::NotFound("文章不存在".to_string()));
        }

        let result = sqlx::query("INSERT INTO comments (post_id, author, content) VALUES (?, ?, ?)")
            .bind(post_id)
            .bind(payload.author)
            .bind(payload.content)
            .execute(self.pool.as_ref())
            .await?;

        let comment = sqlx::query_as::<_, Comment>(
            "SELECT id, post_id, author, content, created_at FROM comments WHERE id = ?",
        )
        .bind(result.last_insert_rowid())
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(comment)
    }

    pub async fn stats(&self) -> AppResult<(i64, i64, i64)> {
        let (total_posts,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
            .fetch_one(self.pool.as_ref())
            .await?;

        let (published_posts,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE is_published = 1")
            .fetch_one(self.pool.as_ref())
            .await?;

        let (total_comments,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok((total_posts, published_posts, total_comments))
    }

    pub async fn reset_demo_data(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM comments").execute(self.pool.as_ref()).await?;
        sqlx::query("DELETE FROM posts").execute(self.pool.as_ref()).await?;

        sqlx::query("INSERT INTO posts (title, content, is_published) VALUES (?, ?, 1)")
            .bind("重置后的示例文章")
            .bind("通过 /api/examples/reset 重新初始化的数据")
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn quick_publish_with_comment(
        &self,
        payload: QuickPublishRequest,
    ) -> AppResult<(Post, Comment)> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            "INSERT INTO posts (title, content, is_published) VALUES (?, ?, 1)",
        )
        .bind(payload.title)
        .bind(payload.content)
        .execute(&mut *tx)
        .await?;

        let post_id = result.last_insert_rowid();

        let comment_result = sqlx::query(
            "INSERT INTO comments (post_id, author, content) VALUES (?, ?, ?)",
        )
        .bind(post_id)
        .bind(payload.first_comment_author)
        .bind(payload.first_comment_content)
        .execute(&mut *tx)
        .await?;

        let post = sqlx::query_as::<_, Post>(
            "SELECT id, title, content, is_published, created_at, updated_at FROM posts WHERE id = ?",
        )
        .bind(post_id)
        .fetch_one(&mut *tx)
        .await?;

        let comment = sqlx::query_as::<_, Comment>(
            "SELECT id, post_id, author, content, created_at FROM comments WHERE id = ?",
        )
        .bind(comment_result.last_insert_rowid())
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((post, comment))
    }
}
