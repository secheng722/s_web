use ree::Engine;
use std::sync::Arc;

use crate::config::AppState;
use crate::handlers;
use crate::middleware;

/// 注册文章相关路由
pub fn register_routes(app: &mut Engine, state: Arc<AppState>) {
    let articels_state = state.clone();
    // 公开路由
    app.get("/api/articles", move |ctx| {
        handlers::get_all_articles(articels_state.clone(), ctx)
    });
    let id_articles_state = state.clone();
    app.get("/api/articles/:id", move |ctx| {
        handlers::get_article(id_articles_state.clone(), ctx)
    });

    // 受保护的路由（需要认证）
    let protected_routes = app.group("/api/articles/protected");
    let auth_state = state.clone();
    protected_routes.use_middleware(move |ctx, next| {
        middleware::auth_middleware(auth_state.clone(), ctx, next)
    });

    // 注册路由处理函数，使用闭包传递 state
    let create_state = state.clone();
    let update_state = state.clone();
    let delete_state = state.clone();

    protected_routes.post("/", move |ctx| {
        handlers::create_article(create_state.clone(), ctx)
    });
    protected_routes.put("/:id", move |ctx| {
        handlers::update_article(update_state.clone(), ctx)
    });
    protected_routes.delete("/:id", move |ctx| {
        handlers::delete_article(delete_state.clone(), ctx)
    });
}
