use ree::{Engine, IntoNext, RequestCtx};
use std::sync::Arc;

use crate::config::AppState;
use crate::handlers;
use crate::middleware::auth_middleware;

/// 注册 Auth 相关路由
pub fn register_routes(app: &mut Engine, state: Arc<AppState>) {
    // 公开路由 - 直接使用 handler
    app.post("/api/auth/register", {
        let register_state = state.clone();
        move |ctx| {
            let register_state = register_state.clone();
            async move { handlers::register(register_state.clone(), ctx).await }
        }
    });

    let login_state = state.clone();
    app.post("/api/auth/login", move |ctx| {
        handlers::login(login_state.clone(), ctx)
    });

    let me_handler = {
        let me_state = state.clone();
        move |ctx| {
            let me_state = me_state.clone();
            async move { handlers::me(me_state, ctx).await }
        }
    };

    let profile_state = state.clone();
    app.get("/api/auth/profile", {
        move |ctx: RequestCtx| {
            auth_middleware(profile_state.clone(), ctx, me_handler.clone().into_next())
        }
    });
}
