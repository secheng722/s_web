// routes/mod.rs
pub mod user_routes;
pub mod product_routes;
pub mod auth_routes;
pub mod admin_routes;

use ree::Engine;

/// Register all routes to the application
pub fn register_all_routes(app: &mut Engine) {
    user_routes::register_routes(app);
    product_routes::register_routes(app);
    auth_routes::register_routes(app);
    admin_routes::register_routes(app);
}
