use s_web::Engine;
use std::sync::Arc;

use crate::config::AppState;
use crate::handlers;
use crate::middleware;

/// 注册文章相关路由
pub fn register_routes(app: &mut Engine, state: Arc<AppState>) {
    let article_routes = app.group("/api/articles");

    let auth_state = state.clone();
    article_routes.use_middleware(move |ctx, next| {
        middleware::auth_middleware(auth_state.clone(), ctx, next)
    });

    let articels_state = state.clone();
    article_routes.get("/", move |ctx| {
        handlers::get_all_articles(articels_state.clone(), ctx)
    });

    let id_articles_state = state.clone();
    article_routes.get("/:id", move |ctx| {
        handlers::get_article(id_articles_state.clone(), ctx)
    });

    let create_state = state.clone();
    article_routes.post("/", move |ctx| {
        handlers::create_article(create_state.clone(), ctx)
    });

    let update_state = state.clone();
    article_routes.put("/:id", move |ctx| {
        handlers::update_article(update_state.clone(), ctx)
    });

    let delete_state = state.clone();
    article_routes.delete("/:id", move |ctx| {
        handlers::delete_article(delete_state.clone(), ctx)
    });
}
