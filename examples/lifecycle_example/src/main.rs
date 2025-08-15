use ree::Engine;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};

// 模拟应用状态
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

// 模拟数据库连接
async fn init_database() {
    println!("🔌 Initializing database connection...");
    sleep(Duration::from_millis(500)).await; // 模拟连接时间
    println!("✅ Database connected successfully");
}

// 模拟缓存初始化
async fn init_cache() {
    println!("🧠 Initializing cache system...");
    sleep(Duration::from_millis(300)).await;
    println!("✅ Cache system ready");
}

// 模拟服务注册
async fn register_service() {
    println!("📡 Registering service to discovery...");
    sleep(Duration::from_millis(200)).await;
    println!("✅ Service registered successfully");
}

// 模拟数据库关闭
async fn close_database() {
    println!("🔌 Closing database connections...");
    sleep(Duration::from_millis(300)).await;
    println!("✅ Database connections closed");
}

// 模拟缓存清理
async fn cleanup_cache() {
    println!("🧹 Cleaning up cache...");
    sleep(Duration::from_millis(200)).await;
    println!("✅ Cache cleaned up");
}

// 模拟服务注销
async fn unregister_service() {
    println!("📡 Unregistering service from discovery...");
    sleep(Duration::from_millis(150)).await;
    println!("✅ Service unregistered");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Engine::new()
        // 启动钩子：初始化所有必要的服务
        .on_startup(|| async {
            println!("🚀 Starting application initialization...");
            
            // 并行初始化服务
            let (_db_result, _cache_result, _register_result) = tokio::join!(
                init_database(),
                init_cache(),
                register_service()
            );
            
            // 标记初始化完成
            IS_INITIALIZED.store(true, Ordering::SeqCst);
            println!("🎉 Application initialization completed!");
        })
        
        // 另一个启动钩子：预热系统
        .on_startup(|| async {
            println!("🔥 Warming up system...");
            sleep(Duration::from_millis(100)).await;
            println!("✅ System warmed up");
        })
        
        // 关闭钩子：清理资源
        .on_shutdown(|| async {
            println!("🛑 Starting graceful shutdown...");
            
            // 并行清理资源
            let (_db_cleanup, _cache_cleanup, _unregister_result) = tokio::join!(
                close_database(),
                cleanup_cache(),
                unregister_service()
            );
            
            println!("✅ Graceful shutdown completed!");
        })
        
        // 另一个关闭钩子：最终清理
        .on_shutdown(|| async {
            println!("🧹 Final cleanup...");
            IS_INITIALIZED.store(false, Ordering::SeqCst);
            sleep(Duration::from_millis(50)).await;
            println!("✅ Final cleanup completed");
        });

    // 添加路由
    let mut app = app;
    
    app.get("/", |_| async {
        if IS_INITIALIZED.load(Ordering::SeqCst) {
            serde_json::json!({
                "message": "Hello from Ree!",
                "status": "initialized",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        } else {
            serde_json::json!({
                "message": "Application is starting...",
                "status": "initializing"
            })
        }
    });

    app.get("/health", |_| async {
        serde_json::json!({
            "status": "healthy",
            "initialized": IS_INITIALIZED.load(Ordering::SeqCst),
            "uptime": "running"
        })
    });

    app.get("/status", |_| async {
        serde_json::json!({
            "application": "lifecycle_example",
            "version": "0.1.0",
            "ready": IS_INITIALIZED.load(Ordering::SeqCst)
        })
    });

    println!("🌟 Lifecycle example server starting...");
    println!("   📋 Available endpoints:");
    println!("   GET  /         - Main endpoint");
    println!("   GET  /health   - Health check");
    println!("   GET  /status   - Application status");
    println!("   📖 Swagger UI: http://127.0.0.1:8080/docs/");
    println!("   💡 Press Ctrl+C to see graceful shutdown in action");

    app.run("127.0.0.1:8080").await
}
