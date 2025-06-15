use ree::{Engine, ResponseBuilder};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 测试新的统一API - 不需要 handler() 包装
    app.get("/text", |_| async { "Hello World!" });
    app.get("/string", |_| async { "Hello".to_string() });
    app.get("/json", |_| async { json!({"message": "hello"}) });
    app.get("/response", |_| async { ResponseBuilder::with_text("Direct response") });
    app.get("/result", |_| async { Ok::<_, &str>("Success") });
    app.get("/option", |_| async { Some("Found") });
    app.get("/unit", |_| async { () }); // 空响应
    
    println!("✅ 统一Handler API测试成功！所有类型都可以直接使用，无需包装。");
    println!("🚀 启动服务器：http://127.0.0.1:8080");
    
    app.run("127.0.0.1:8080").await
}
