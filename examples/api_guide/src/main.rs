use ree::{Engine,  ResponseBuilder,RequestCtx};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    println!("🎯 Ree HTTP Framework - API使用指南");
    println!("═══════════════════════════════════════");
    println!("✨ 统一的API设计 - 自动类型转换！");
    println!("   🎉 所有处理函数都支持直接返回各种类型");
    println!("   🚀 框架自动转换为HTTP响应，无需手动包装");
    println!();
    
    // ========== 统一API: 直接返回各种类型，自动转换 ==========
    println!("🚀 统一API: 支持自动类型转换的各种返回类型");
    
    // 返回 &str -> text/plain
    app.get("/simple/text", |_| async { 
        "Hello! This is converted to text/plain automatically." 
    });
    
    // 返回 String -> text/plain
    app.get("/simple/string", |_| async { 
        format!("Dynamic content: {}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs())
    });
    
    // 返回 JSON -> application/json
    app.get("/simple/json", |_| async { 
        json!({
            "message": "Automatic JSON conversion",
            "framework": "Ree",
            "easy": true,
            "unified_api": true
        })
    });
    
    // 路径参数
    app.get("/simple/greet/:name", |ctx: ree::RequestCtx| async move {
        let name = ctx.get_param("name").map_or("Guest", |v| v);
        format!("Hello, {}! 👋", name)
    });
    
    // Result处理 - 自动转换错误
    app.get("/simple/result/:action", |ctx: ree::RequestCtx| async move {
        match ctx.get_param("action").map_or("", |v| v) {
            "success" => Ok("Operation completed! ✅"),
            "fail" => Err("Something went wrong! ❌"),
            _ => Err("Unknown action! ❓")
        }
    });
    
    // Option处理 - None自动变404
    app.get("/simple/find/:id", |ctx:ree::RequestCtx| async move {
        let id = ctx.get_param("id").map_or("", |v| v);
        if id == "123" {
            Some(json!({
                "id": id,
                "name": "Sample Item",
                "found": true
            }))
        } else {
            None  // 自动返回 404
        }
    });
    
    // 状态码控制
    app.post("/simple/create", |_| async {
        (ree::StatusCode::CREATED, json!({
            "message": "Resource created",
            "id": 456
        }))
    });
    
    // 空响应 - 204 No Content
    app.delete("/simple/delete/:id", |_| async { () });
    
    // ========== 高级用法: 当需要精确控制时直接返回 Response ==========
    println!("🔧 高级用法: 直接返回 Response - 精确控制HTTP响应");
    
    // 自定义响应头
    app.get("/advanced/headers", |_| async {
        let mut response = ResponseBuilder::with_json(r#"{"message": "With custom headers"}"#);
        response.headers_mut().insert("X-Framework", "Ree".parse().unwrap());
        response.headers_mut().insert("X-Version", "0.1.0".parse().unwrap());
        response.headers_mut().insert("X-Custom", "Advanced-Control".parse().unwrap());
        response
    });
    
    // 自定义状态码和内容类型
    app.get("/advanced/custom", |_| async {
        ResponseBuilder::with_status_and_content_type(
            ree::StatusCode::IM_A_TEAPOT,
            "text/plain; charset=utf-8",
            "I'm a teapot! This is advanced response control. ☕"
        )
    });
    
    // HTML响应
    app.get("/advanced/page", |_| async {
        ResponseBuilder::with_html(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Ree Framework Demo</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 40px; }
                    .highlight { background: #f0f8ff; padding: 20px; border-radius: 8px; }
                </style>
            </head>
            <body>
                <h1>🎯 Ree HTTP Framework</h1>
                <div class="highlight">
                    <h2>统一API设计</h2>
                    <p><strong>自动转换</strong>：所有处理函数都支持直接返回各种类型</p>
                    <p><strong>灵活可控</strong>：需要精确控制时仍可直接返回 <code>Response</code></p>
                    <p><strong>向后兼容</strong>：<code>handler()</code> 函数仍然可用但不再必需</p>
                </div>
                <h3>🔗 测试链接</h3>
                <ul>
                    <li><a href="/simple/text">简单文本</a></li>
                    <li><a href="/simple/json">JSON响应</a></li>
                    <li><a href="/simple/greet/Alice">问候 Alice</a></li>
                    <li><a href="/simple/result/success">成功结果</a></li>
                    <li><a href="/simple/find/123">查找存在的项目</a></li>
                    <li><a href="/advanced/headers">自定义响应头</a></li>
                </ul>
            </body>
            </html>
        "#)
    });
    
    // 错误处理
    app.get("/advanced/error", |_| async {
        ResponseBuilder::with_status_and_content_type(
            ree::StatusCode::BAD_REQUEST,
            "application/json; charset=utf-8",
            r#"{"error": "Bad Request", "message": "This is a custom error response"}"#
        )
    });

    app.get("/compat/without-handler", |_| async { 
        "This doesn't use handler() wrapper - same result!" 
    });
    
    println!("✅ Server starting on http://127.0.0.1:8080");
    println!("\n📋 测试端点列表:");
    println!("   ┌─────────────────────────────────────────────────────────┐");
    println!("   │ 🚀 统一API - 自动类型转换                              │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ GET    /simple/text       - &str → text/plain          │");
    println!("   │ GET    /simple/string     - String → text/plain        │");
    println!("   │ GET    /simple/json       - JSON → application/json    │");
    println!("   │ GET    /simple/greet/:name - 路径参数处理               │");
    println!("   │ GET    /simple/result/:action - Result<T,E> 处理       │");
    println!("   │ GET    /simple/find/:id   - Option<T> 处理             │");
    println!("   │ POST   /simple/create     - (StatusCode, T) 元组       │");
    println!("   │ DELETE /simple/delete/:id - () → 204 No Content        │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ 🔧 高级控制 - 直接返回 Response                        │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ GET    /advanced/headers  - 自定义响应头               │");
    println!("   │ GET    /advanced/custom   - 自定义状态码/内容类型      │");
    println!("   │ GET    /advanced/page     - HTML 页面                  │");
    println!("   │ GET    /advanced/error    - 自定义错误响应             │");
    println!("   └─────────────────────────────────────────────────────────┘");
    println!("\n💡 新的API设计优势:");
    println!("   • ✨ 统一简洁：无需区分两种使用方式");
    println!("   • 🚀 自动转换：支持 &str, String, JSON, Result, Option 等");
    println!("   • 🔧 精确控制：需要时仍可直接返回 Response");
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
