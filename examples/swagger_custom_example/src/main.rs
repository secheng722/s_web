//! Custom Swagger configuration example for s_web framework
//! This example demonstrates how to use SwaggerInfo for route documentation

use s_web::{swagger, Engine, RequestCtx};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}
async fn get_users(_ctx: RequestCtx) -> String {
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    serde_json::to_string(&users).unwrap()
}

async fn get_user_by_id(ctx: RequestCtx) -> String {
    let user_id = ctx.get_param("id").map_or("0", |v| v);

    let user = User {
        id: user_id.parse().unwrap_or(0),
        name: format!("User {}", user_id),
        email: format!("user{}@example.com", user_id),
    };

    serde_json::to_string(&user).unwrap()
}

async fn create_user(_ctx: RequestCtx) -> String {
    // In real app, you'd parse the body
    let new_user = User {
        id: 3,
        name: "New User".to_string(),
        email: "newuser@example.com".to_string(),
    };

    serde_json::to_string(&new_user).unwrap()
}

async fn update_user(ctx: RequestCtx) -> String {
    let user_id = ctx.get_param("id").map_or("0", |v| v);

    let updated_user = User {
        id: user_id.parse().unwrap_or(0),
        name: "Updated User".to_string(),
        email: "updated@example.com".to_string(),
    };

    serde_json::to_string(&updated_user).unwrap()
}

async fn delete_user(ctx: RequestCtx) -> String {
    let user_id = ctx.get_param("id").map_or("0", |v| v);
    json!({
        "message": format!("User {} deleted successfully", user_id)
    })
    .to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Custom Swagger Example Server...");

    let mut app = Engine::new();

    // Configure detailed swagger documentation for each endpoint

    // GET /users - List all users
    app.get("/users", get_users);

    // GET /users/:id - Get user by ID
    app.get("/users/:id", get_user_by_id);

    // POST /users - Create new user
    // app.post("/users", create_user);
    app.post_with_swagger(
        "/users",
        create_user,
        swagger()
            .summary("Create a new user")
            .description("This endpoint creates a new user.")
            .tag("User")
            .request_body(json!({
                "name": "New User",
                "email": "newuser@example.com"
            }))
            .response("201", "User created successfully")
            .json_response("201", "User created successfully", Some(json!(
                User {
                    id: 3,
                    name: "New User".to_string(),
                    email: "newuser@example.com".to_string(),
                }
            )))
            .build()
    );

    // PUT /users/:id - Update user
    app.put("/users/:id", update_user);

    // DELETE /users/:id - Delete user
    app.delete("/users/:id", delete_user);

    // Add some routes without custom swagger (will use auto-generation)
    app.get("/health", |_ctx: RequestCtx| async {
        json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string()
    });

    app.get("/version", |_ctx: RequestCtx| async {
        json!({
            "version": "1.0.0",
            "name": "Swagger Custom Example"
        })
        .to_string()
    });

    println!("📖 Swagger documentation will be available at:");
    println!("   http://127.0.0.1:3000/swagger-ui");
    println!("📄 Raw OpenAPI JSON at:");
    println!("   http://127.0.0.1:3000/api-docs");
    println!("\n🎯 Available endpoints:");
    println!("   GET    /users           - List all users (auto swagger)");
    println!("   GET    /users/:id       - Get user by ID (auto swagger)");
    println!("   POST   /users           - Create user (auto swagger)");
    println!("   PUT    /users/:id       - Update user (auto swagger)");
    println!("   DELETE /users/:id       - Delete user (auto swagger)");
    println!("   GET    /health          - Health check (auto swagger)");
    println!("   GET    /version         - Version info (auto swagger)");

    app.run("127.0.0.1:3000").await
}
