use serde::{Deserialize, Serialize};

use crate::models::{Comment, Post};

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub author: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct QuickPublishRequest {
    pub title: String,
    pub content: String,
    pub first_comment_author: String,
    pub first_comment_content: String,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub is_published: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: i64,
    pub post_id: i64,
    pub author: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct BlogStatsResponse {
    pub total_posts: i64,
    pub published_posts: i64,
    pub total_comments: i64,
}

impl From<Post> for PostResponse {
    fn from(value: Post) -> Self {
        Self {
            id: value.id,
            title: value.title,
            content: value.content,
            is_published: value.is_published == 1,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Comment> for CommentResponse {
    fn from(value: Comment) -> Self {
        Self {
            id: value.id,
            post_id: value.post_id,
            author: value.author,
            content: value.content,
            created_at: value.created_at,
        }
    }
}
