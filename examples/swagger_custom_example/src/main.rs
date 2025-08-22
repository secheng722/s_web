//! Custom Swagger configuration example for Ree framework
//! This example demonstrates the new enhanced swagger system with custom configurations

use ree::{Engine, RequestCtx, SwaggerInfo};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct CreateUserRequest {
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
    let user_id = ctx.param("id").unwrap_or("0");
    
    let user = User {
        id: user_id.parse().unwrap_or(0),
        name: format!("User {}", user_id),
        email: format!("user{}@example.com", user_id),
    };
    
    serde_json::to_string(&user).unwrap()
}

async fn create_user(ctx: RequestCtx) -> String {
    // In real app, you'd parse the body
    let new_user = User {
        id: 3,
        name: "New User".to_string(),
        email: "newuser@example.com".to_string(),
    };
    
    serde_json::to_string(&new_user).unwrap()
}

async fn update_user(ctx: RequestCtx) -> String {
    let user_id = ctx.param("id").unwrap_or("0");
    
    let updated_user = User {
        id: user_id.parse().unwrap_or(0),
        name: "Updated User".to_string(),
        email: "updated@example.com".to_string(),
    };
    
    serde_json::to_string(&updated_user).unwrap()
}

async fn delete_user(ctx: RequestCtx) -> String {
    let user_id = ctx.param("id").unwrap_or("0");
    json!({
        "message": format!("User {} deleted successfully", user_id)
    }).to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Custom Swagger Example Server...");
    
    let mut app = Engine::new();

    // Configure detailed swagger documentation for each endpoint

    // GET /users - List all users
    app.get_with_swagger(
        "/users", 
        get_users,
        SwaggerInfo::new()
            .summary("Get all users")
            .description("Retrieve a list of all users in the system")
            .tag("Users")
            .json_response("200", "List of users", Some(json!([
                {
                    "id": 1,
                    "name": "Alice",
                    "email": "alice@example.com"
                }
            ])))
            .bearer_auth()
    );

    // GET /users/:id - Get user by ID
    app.get_with_swagger(
        "/users/:id",
        get_user_by_id,
        SwaggerInfo::new()
            .summary("Get user by ID")
            .description("Retrieve a specific user by their unique identifier")
            .tag("Users")
            .parameter("id", "path", Some("User ID".to_string()), true)
            .json_response("200", "User details", Some(json!({
                "id": 1,
                "name": "Alice",
                "email": "alice@example.com"
            })))
            .response("404", "User not found")
            .bearer_auth()
    );

    // POST /users - Create new user
    app.post_with_swagger(
        "/users",
        create_user,
        SwaggerInfo::new()
            .summary("Create a new user")
            .description("Create a new user account in the system")
            .tag("Users")
            .request_body(json!({
                "name": "John Doe",
                "email": "john@example.com"
            }))
            .json_response("201", "User created successfully", Some(json!({
                "id": 3,
                "name": "John Doe",
                "email": "john@example.com"
            })))
            .response("400", "Invalid input data")
            .bearer_auth()
    );

    // PUT /users/:id - Update user
    app.put_with_swagger(
        "/users/:id",
        update_user,
        SwaggerInfo::new()
            .summary("Update user")
            .description("Update an existing user's information")
            .tag("Users")
            .parameter("id", "path", Some("User ID to update".to_string()), true)
            .request_body(json!({
                "name": "Updated Name",
                "email": "updated@example.com"
            }))
            .json_response("200", "User updated successfully", Some(json!({
                "id": 1,
                "name": "Updated Name",
                "email": "updated@example.com"
            })))
            .response("404", "User not found")
            .response("400", "Invalid input data")
            .bearer_auth()
    );

    // DELETE /users/:id - Delete user
    app.delete_with_swagger(
        "/users/:id",
        delete_user,
        SwaggerInfo::new()
            .summary("Delete user")
            .description("Remove a user from the system")
            .tag("Users")
            .parameter("id", "path", Some("User ID to delete".to_string()), true)
            .json_response("200", "User deleted successfully", Some(json!({
                "message": "User deleted successfully"
            })))
            .response("404", "User not found")
            .bearer_auth()
    );

    // Add some routes without custom swagger (will use auto-generation)
    app.get("/health", |_ctx: RequestCtx| async {
        json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string()
    });

    app.get("/version", |_ctx: RequestCtx| async {
        json!({
            "version": "1.0.0",
            "name": "Swagger Custom Example"
        }).to_string()
    });

    println!("📖 Custom Swagger documentation will be available at:");
    println!("   http://127.0.0.1:3000/docs/");
    println!("📄 Raw OpenAPI JSON at:");
    println!("   http://127.0.0.1:3000/docs/swagger.json");
    println!("\n🎯 Available endpoints:");
    println!("   GET    /users           - List all users (with custom swagger)");
    println!("   GET    /users/:id       - Get user by ID (with custom swagger)");
    println!("   POST   /users           - Create user (with custom swagger)");
    println!("   PUT    /users/:id       - Update user (with custom swagger)");
    println!("   DELETE /users/:id       - Delete user (with custom swagger)");
    println!("   GET    /health          - Health check (auto swagger)");
    println!("   GET    /version         - Version info (auto swagger)");

    app.run("127.0.0.1:3000").await
}
