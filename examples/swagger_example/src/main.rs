use s_web::{Engine, RequestCtx};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    // 添加一些路由
    app.get("/", |_| async { "Hello, Ree!" })
        .get("/users", |_| async { "List of users" })
        .post("/users", |_| async { "Create user" })
        .get("/users/:id", |ctx: RequestCtx| async move {
            let id = ctx.get_param("id").map_or("default", |v| v);
            format!("User ID: {id}")
        })
        .put("/users/:id", |_| async { "Update user" })
        .delete("/users/:id", |_| async { "Delete user" });


    app.run("127.0.0.1:8080").await?;
    Ok(())
}
