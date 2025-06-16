// middleware/mod.rs
pub mod auth;
pub mod cache;
pub mod logging;

use ree::Engine;

/// Register global middlewares
pub fn register_global_middleware(app: &mut Engine) {
    // Add global middleware
    app.use_middleware(logging::request_logger);
    app.use_middleware(logging::timer);
}
