use s_web::{Engine, RequestCtx};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserPreferences {
    theme: String,
    notifications: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            notifications: true,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    println!("🎯 s_web - 结构体转换示例");
    println!("═══════════════════════");
    
    // 方式1: 使用原有的 body_json 方法（支持可选body）
    app.post("/api/user/create", |ctx: RequestCtx| async move {
        match ctx.body_json::<User>() {
            Ok(Some(user)) => {
                println!("创建用户: {:?}", user);
                format!("用户 {} 创建成功！", user.name)
            },
            Ok(None) => "错误：请提供用户数据".to_string(),
            Err(e) => format!("JSON解析错误: {}", e),
        }
    });
    
    // 方式2: 使用新的 json 方法（要求必须有body）
    app.post("/api/user/update", |ctx: RequestCtx| async move {
        match ctx.json::<User>() {
            Ok(user) => {
                println!("更新用户: {:?}", user);
                format!("用户 {} 更新成功！", user.name)
            },
            Err(e) => format!("错误: {}", e),
        }
    });
    
    // 方式3: 使用 json_or_default 方法（有默认值）
    app.post("/api/user/preferences", |ctx: RequestCtx| async move {
        match ctx.json_or_default::<UserPreferences>() {
            Ok(prefs) => {
                println!("用户偏好设置: {:?}", prefs);
                format!("偏好设置已保存: 主题={}, 通知={}", prefs.theme, prefs.notifications)
            },
            Err(e) => format!("错误: {}", e),
        }
    });
    
    // 复杂示例：组合使用
    app.post("/api/user/profile", |ctx: RequestCtx| async move {
        #[derive(Deserialize)]
        struct ProfileRequest {
            user: User,
            preferences: Option<UserPreferences>,
        }
        
        match ctx.json::<ProfileRequest>() {
            Ok(profile) => {
                let prefs = profile.preferences.unwrap_or_default();
                println!("完整资料: 用户={:?}, 偏好={:?}", profile.user, prefs);
                "用户资料保存成功！"
            },
            Err(e) => format!("错误: {}", e).leak(),
        }
    });
    
    println!("\n🚀 服务器启动中...");
    println!("测试示例:");
    println!("curl -X POST http://127.0.0.1:3000/api/user/create \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"name\":\"张三\",\"email\":\"zhangsan@example.com\",\"age\":25}}'");
    println!();
    println!("curl -X POST http://127.0.0.1:3000/api/user/preferences \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"theme\":\"dark\",\"notifications\":false}}'");
    
    app.run("127.0.0.1:3000").await
}
