// routes/user_routes.rs
use ree::{Engine, RequestCtx};
use serde_json::json;

/// Register user-related routes
pub fn register_routes(app: &mut Engine) {
    // Create user group to share common path prefix and middleware
    let mut users = app.group("/users");
    
    // Apply user-specific middleware
    users.use_middleware(crate::middleware::auth::require_auth());
    
    // Define user routes
    users.get("/", get_users);
    users.get("/:id", get_user_by_id);
    users.post("/", create_user);
    users.put("/:id", update_user);
    users.delete("/:id", delete_user);
}

// Handler functions
async fn get_users(_ctx: RequestCtx) -> impl ree::IntoResponse {
    json!({
        "users": [
            {"id": 1, "name": "User 1"},
            {"id": 2, "name": "User 2"}
        ]
    })
}

async fn get_user_by_id(ctx: RequestCtx) -> impl ree::IntoResponse {
    let id = ctx.get_param("id").unwrap_or("0");
    json!({
        "id": id,
        "name": format!("User {}", id),
        "email": format!("user{}@example.com", id)
    })
}

async fn create_user(_ctx: RequestCtx) -> impl ree::IntoResponse {
    json!({
        "status": "success",
        "message": "User created",
        "id": 3
    })
}

async fn update_user(ctx: RequestCtx) -> impl ree::IntoResponse {
    let id = ctx.get_param("id").unwrap_or("0");
    json!({
        "status": "success",
        "message": format!("User {} updated", id)
    })
}

async fn delete_user(ctx: RequestCtx) -> impl ree::IntoResponse {
    let id = ctx.get_param("id").unwrap_or("0");
    json!({
        "status": "success",
        "message": format!("User {} deleted", id)
    })
}
