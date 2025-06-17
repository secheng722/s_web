use ree::{Engine, RequestCtx, Next, Response, ResponseBuilder};
use serde_json::json;

/// 测试中间件 - 添加参数到上下文
async fn test_middleware(ctx: RequestCtx, next: Next) -> Response {
    println!("🔵 测试中间件开始执行");
    println!("🔵 中间件收到的params: {:?}", ctx.params);
    
    // 修改上下文
    let mut ctx = ctx;
    ctx.params.insert("test_param".to_string(), "middleware_value".to_string());
    ctx.params.insert("user_id".to_string(), "12345".to_string());
    
    println!("🔵 中间件设置参数后: {:?}", ctx.params);
    println!("🔵 中间件验证user_id: {:?}", ctx.get_param("user_id"));
    
    // 传递修改后的上下文
    let response = next(ctx).await;
    println!("🔵 中间件处理完成，响应状态: {}", response.status());
    
    response
}

/// 第二个测试中间件 - 验证参数是否能在中间件链中传递
async fn second_middleware(ctx: RequestCtx, next: Next) -> Response {
    println!("🟡 第二个中间件开始执行");
    println!("🟡 第二个中间件收到的params: {:?}", ctx.params);
    println!("🟡 第二个中间件验证user_id: {:?}", ctx.get_param("user_id"));
    
    // 添加更多参数
    let mut ctx = ctx;
    ctx.params.insert("second_param".to_string(), "second_value".to_string());
    
    println!("🟡 第二个中间件设置参数后: {:?}", ctx.params);
    
    next(ctx).await
}

/// 测试处理器 - 检查参数是否正确传递
async fn test_handler(ctx: RequestCtx) -> Response {
    println!("🟢 Handler开始执行");
    println!("🟢 Handler收到的所有params: {:?}", ctx.params);
    
    let mut result = json!({
        "message": "参数传递测试",
        "all_params": ctx.params
    });
    
    // 检查特定参数
    if let Some(user_id) = ctx.get_param("user_id") {
        println!("✅ Handler成功获取user_id: {}", user_id);
        result["user_id_found"] = json!(true);
        result["user_id_value"] = json!(user_id);
    } else {
        println!("❌ Handler未能获取user_id");
        result["user_id_found"] = json!(false);
    }
    
    if let Some(test_param) = ctx.get_param("test_param") {
        println!("✅ Handler成功获取test_param: {}", test_param);
        result["test_param_found"] = json!(true);
        result["test_param_value"] = json!(test_param);
    } else {
        println!("❌ Handler未能获取test_param");
        result["test_param_found"] = json!(false);
    }
    
    if let Some(second_param) = ctx.get_param("second_param") {
        println!("✅ Handler成功获取second_param: {}", second_param);
        result["second_param_found"] = json!(true);
        result["second_param_value"] = json!(second_param);
    } else {
        println!("❌ Handler未能获取second_param");
        result["second_param_found"] = json!(false);
    }
    
    ResponseBuilder::new()
        .status(hyper::StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap())
}

/// 简单测试处理器 - 不使用任何中间件
async fn simple_handler(_ctx: RequestCtx) -> Response {
    println!("🔴 简单Handler执行 - 无中间件");
    
    ResponseBuilder::new()
        .status(hyper::StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&json!({
            "message": "简单处理器，无中间件处理"
        })).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    println!("🚀 启动中间件参数传递测试服务器");
    
    // 添加全局中间件
    app.use_middleware(test_middleware);
    app.use_middleware(second_middleware);
    
    // 测试路由 - 会经过中间件
    app.get("/test", test_handler);
    
    // 简单路由 - 会经过中间件但不使用参数
    app.get("/simple", simple_handler);
    
    // 主页路由
    app.get("/", |_ctx: RequestCtx| async move {
        ResponseBuilder::new()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(r#"
                <h1>中间件参数传递测试</h1>
                <p><a href="/test">测试中间件参数传递</a></p>
                <p><a href="/simple">简单处理器测试</a></p>
            "#)
    });
    
    println!("📍 服务器地址: http://127.0.0.1:3000");
    println!("🔗 测试链接:");
    println!("   - http://127.0.0.1:3000/test    (测试中间件参数传递)");
    println!("   - http://127.0.0.1:3000/simple  (简单处理器)");
    println!("   - http://127.0.0.1:3000/        (主页)");
    
    app.run("127.0.0.1:3000").await?;
    Ok(())
}
