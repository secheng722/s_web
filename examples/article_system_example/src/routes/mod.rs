mod auth_routes;
mod article_routes;
mod base_routes;

use s_web::Engine;
use std::sync::Arc;

use crate::config::AppState;

/// 注册所有路由到应用
pub fn register_all_routes(app: &mut Engine, state: Arc<AppState>) {
    // 注册基础路由
    base_routes::register_routes(app);
    
    // 注册 Auth 路由
    auth_routes::register_routes(app, state.clone());
    
    // 注册文章路由
    article_routes::register_routes(app, state);
}
