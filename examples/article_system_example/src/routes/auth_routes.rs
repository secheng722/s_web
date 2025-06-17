use ree::Engine;
use std::sync::Arc;

use crate::config::AppState;
use crate::handlers;
use crate::middleware;

/// 注册 Auth 相关路由
pub fn register_routes(app: &mut Engine, state: Arc<AppState>) {
    let register_state = state.clone();
    // 公开路由
    app.post("/api/auth/register", move |ctx| {
        handlers::register(register_state.clone(), ctx)
    });
    let login_state = state.clone();
    app.post("/api/auth/login", move |ctx| {
        handlers::login(login_state.clone(), ctx)
    });

    // 受保护的路由（需要认证）
    let protected_routes = app.group("/api/auth/protected");
    let auth_state = state.clone();
    protected_routes.use_middleware(move |ctx, next| {
        middleware::auth_middleware(auth_state.clone(), ctx, next)
    });
    let me_state = state.clone();
    protected_routes.get("/me", move |ctx| handlers::me(me_state.clone(), ctx));
}
