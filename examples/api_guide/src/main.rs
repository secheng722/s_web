use ree::{Engine,  ResponseBuilder};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    println!("🎯 Ree HTTP Framework - API Usage Guide");
    println!("═══════════════════════════════════════════");
    println!("✨ Unified API Design - Automatic Type Conversion!");
    println!("   🎉 All handler functions support direct return of various types");
    println!("   🚀 Framework automatically converts to HTTP responses, no manual wrapping");
    println!();
    
    // ========== 统一API: 直接返回各种类型，自动转换 ==========
    println!("🚀 Unified API: Various return types with automatic type conversion");
    
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
    println!("🔧 Advanced usage: Direct Response return - Precise HTTP response control");
    
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
    
    // ========== POST Request Body Examples ==========
    println!("📮 POST Request Examples - Body Reading");
    
    // JSON body parsing
    app.post("/post/json", |ctx: ree::RequestCtx| async move {
        match ctx.body_json::<serde_json::Value>() {
            Ok(Some(json)) => format!("Received JSON: {}", json),
            Ok(None) => "No body provided".to_string(),
            Err(e) => format!("Failed to parse JSON: {}", e),
        }
    });
    
    // Text body reading
    app.post("/post/text", |ctx: ree::RequestCtx| async move {
        match ctx.body_string() {
            Ok(Some(text)) => format!("Received text: {}", text),
            Ok(None) => "No body provided".to_string(),
            Err(e) => format!("Failed to read text: {}", e),
        }
    });
    
    // Raw bytes body reading
    app.post("/post/bytes", |ctx: ree::RequestCtx| async move {
        match ctx.body_bytes() {
            Some(bytes) => format!("Received {} bytes", bytes.len()),
            None => "No body provided".to_string(),
        }
    });
    
    // Form data example (simple parsing)
    app.post("/post/form", |ctx: ree::RequestCtx| async move {
        match ctx.body_string() {
            Ok(Some(body)) => {
                // Simple form parsing (in real app, use a proper form parser)
                let params: std::collections::HashMap<&str, &str> = body
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.split('=');
                        Some((parts.next()?, parts.next()?))
                    })
                    .collect();
                
                format!("Form data: {:?}", params)
            },
            Ok(None) => "No form data provided".to_string(),
            Err(e) => format!("Failed to read form: {}", e),
        }
    });

    println!("✅ Server starting on http://127.0.0.1:8080");
    println!("\n📋 Test endpoints list:");
    println!("   ┌─────────────────────────────────────────────────────────┐");
    println!("   │ 🚀 Unified API - Automatic Type Conversion             │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ GET    /simple/text       - &str → text/plain          │");
    println!("   │ GET    /simple/string     - String → text/plain        │");
    println!("   │ GET    /simple/json       - JSON → application/json    │");
    println!("   │ GET    /simple/greet/:name - Path parameter handling    │");
    println!("   │ GET    /simple/result/:action - Result<T,E> handling    │");
    println!("   │ GET    /simple/find/:id   - Option<T> handling          │");
    println!("   │ POST   /simple/create     - (StatusCode, T) tuple       │");
    println!("   │ DELETE /simple/delete/:id - () → 204 No Content        │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ � POST Request Body Examples                          │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ POST   /post/json         - JSON body parsing          │");
    println!("   │ POST   /post/text         - Text body reading          │");
    println!("   │ POST   /post/bytes        - Raw bytes reading          │");
    println!("   │ POST   /post/form         - Form data parsing          │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ 🔧 Advanced Control - Direct Response Return           │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ GET    /advanced/headers  - Custom response headers    │");
    println!("   │ GET    /advanced/custom   - Custom status/content type │");
    println!("   │ GET    /advanced/page     - HTML page                  │");
    println!("   │ GET    /advanced/error    - Custom error response      │");
    println!("   └─────────────────────────────────────────────────────────┘");
    println!("\n💡 New API Design Advantages:");
    println!("   • ✨ Unified & Simple: No need to distinguish usage styles");
    println!("   • 🚀 Auto Conversion: Supports &str, String, JSON, Result, Option etc");
    println!("   • 📮 Body Reading: Easy POST/PUT request body access");
    println!("   • 🔧 Flexible Control: Direct Response return when needed");
    
    println!("\n🧪 Test POST requests with curl:");
    println!("   # JSON body");
    println!("   curl -X POST http://127.0.0.1:8080/post/json \\");
    println!("        -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"name\": \"Alice\", \"age\": 30}}'");
    println!("   ");
    println!("   # Raw bytes body");
    println!("   curl -X POST http://127.0.0.1:8080/post/bytes \\");
    println!("        -H 'Content-Type: application/octet-stream' \\");
    println!("        --data-binary 'Hello, raw bytes!'");
    println!("   ");
    println!("   # Text body");
    println!("   curl -X POST http://127.0.0.1:8080/post/text \\");
    println!("        -H 'Content-Type: text/plain' \\");
    println!("        -d 'Hello from curl!'");
    println!("   ");
    println!("   # Form data");
    println!("   curl -X POST http://127.0.0.1:8080/post/form \\");
    println!("        -H 'Content-Type: application/x-www-form-urlencoded' \\");
    println!("        -d 'name=Alice&email=alice@example.com&age=30'");
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
