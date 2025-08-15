use ree::Engine;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Engine::new()
        // 简洁的启动钩子 - 无需 Box::pin!
        .on_startup(|| async {
            println!("🔌 Connecting to database...");
            sleep(Duration::from_millis(100)).await;
            println!("✅ Database connected");
        })
        
        .on_startup(|| async {
            println!("🧠 Initializing cache...");
            sleep(Duration::from_millis(50)).await;
            println!("✅ Cache ready");
        })
        
        // 简洁的关闭钩子
        .on_shutdown(|| async {
            println!("🔌 Closing database...");
            sleep(Duration::from_millis(50)).await;
            println!("✅ Database closed");
        })
        
        .on_shutdown(|| async {
            println!("🧹 Final cleanup...");
            sleep(Duration::from_millis(30)).await;
            println!("✅ Cleanup done");
        });

    // 添加一些路由
    let mut app = app;
    app.get("/", |_| async { "Hello from Ree with lifecycle hooks!" });
    app.get("/health", |_| async { "OK" });

    println!("🌟 Simple lifecycle example");
    println!("📋 Try: curl http://127.0.0.1:8080/");
    println!("💡 Press Ctrl+C to see shutdown hooks");

    app.run("127.0.0.1:8080").await
}
