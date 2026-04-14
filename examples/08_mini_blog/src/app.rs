use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{
    handlers::{comments, examples, posts},
    middleware,
    repository::BlogRepository,
};

pub fn register_routes(app: &mut s_web::Engine, db: Arc<SqlitePool>) {
    app.use_middleware(middleware::access_log);

    let api = app.group("/api");
    let repo = BlogRepository::new(db);

    {
        let repo = repo.clone();
        api.get("/posts", move |ctx| {
            let repo = repo.clone();
            async move { posts::list_posts(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.post("/posts", move |ctx| {
            let repo = repo.clone();
            async move { posts::create_post(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.get("/posts/:id", move |ctx| {
            let repo = repo.clone();
            async move { posts::get_post(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.put("/posts/:id", move |ctx| {
            let repo = repo.clone();
            async move { posts::update_post(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.add_route("PATCH", "/posts/:id/publish", move |ctx| {
            let repo = repo.clone();
            async move { posts::publish_post(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.delete("/posts/:id", move |ctx| {
            let repo = repo.clone();
            async move { posts::delete_post(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.get("/posts/:id/comments", move |ctx| {
            let repo = repo.clone();
            async move { comments::list_comments(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.post("/posts/:id/comments", move |ctx| {
            let repo = repo.clone();
            async move { comments::create_comment(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.get("/stats", move |ctx| {
            let repo = repo.clone();
            async move { examples::stats(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.post("/examples/reset", move |ctx| {
            let repo = repo.clone();
            async move { examples::reset_demo(ctx, repo).await }
        });
    }

    {
        let repo = repo.clone();
        api.post("/examples/quick-publish", move |ctx| {
            let repo = repo.clone();
            async move { examples::quick_publish(ctx, repo).await }
        });
    }
}
