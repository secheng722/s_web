# Ree Framework - 链式调用示例

这个示例展示了 Ree 框架支持链式调用后的优雅语法。

## 🎯 主要改进

### ✅ 之前的写法
```rust
let mut app = Engine::new();
app.use_middleware(|ctx, next| logger("Global", ctx, next));
app.use_middleware(|ctx, next| cors(ctx, next));
app.get("/", |_| async { "Hello" });
app.get("/health", |_| async { "OK" });

let api = app.group("/api");
api.use_middleware(|ctx, next| auth("token", ctx, next));
api.get("/users", |_| async { "users" });
api.post("/users", |_| async { "created" });
```

### 🚀 现在的写法 (支持链式调用)
```rust
let mut app = Engine::new();

// 全局中间件链式调用
app.use_middleware(|ctx, next| logger("Global", ctx, next))
    .use_middleware(cors)
    // 路由链式调用
    .get("/", |_| async { "Welcome to Ree!" })
    .get("/health", |_| async { json!({"status": "ok"}) });

// API 路由组，支持链式调用
{
    let api = app.group("/api");
    api.use_middleware(|ctx, next| logger("API", ctx, next))
        .use_middleware(|ctx, next| auth("api-token", ctx, next))
        .get("/users", |_| async { json!({"users": ["alice", "bob"]}) })
        .post("/users", |_| async { json!({"message": "User created"}) })
        .get("/profile", |_| async { json!({"name": "Current User"}) });
}

// 管理员路由组
{
    let admin = app.group("/admin");
    admin
        .use_middleware(|ctx, next| logger("Admin", ctx, next))
        .use_middleware(|ctx, next| auth("admin-token", ctx, next))
        .get("/dashboard", |_| async { "Admin Dashboard" })
        .delete("/users/:id", |ctx: RequestCtx| async move {
            if let Some(id) = ctx.get_param("id") {
                format!("Deleted user {id}")
            } else {
                "User ID not found".to_string()
            }
        });
}
```

## 🏃‍♂️ 运行示例

```bash
cd examples/chain_example
cargo run
```

## 🧪 测试端点

服务器运行在 `http://127.0.0.1:8080`

### 公开端点
```bash
# 基本端点
curl http://127.0.0.1:8080/
# 返回: "Welcome to Ree!"

curl http://127.0.0.1:8080/health
# 返回: {"status": "ok"}
```

### 需要 API 认证的端点
```bash
# 正确的认证
curl -H "Authorization: Bearer api-token" http://127.0.0.1:8080/api/users
# 返回: {"users": ["alice", "bob"]}

curl -X POST -H "Authorization: Bearer api-token" http://127.0.0.1:8080/api/users
# 返回: {"message": "User created"}

curl -H "Authorization: Bearer api-token" http://127.0.0.1:8080/api/profile
# 返回: {"name": "Current User"}

# 错误的认证 (会返回 401)
curl http://127.0.0.1:8080/api/users
# 返回: {"error": "Unauthorized"}
```

### 需要管理员认证的端点
```bash
# 正确的认证
curl -H "Authorization: Bearer admin-token" http://127.0.0.1:8080/admin/dashboard
# 返回: "Admin Dashboard"

curl -X DELETE -H "Authorization: Bearer admin-token" http://127.0.0.1:8080/admin/users/123
# 返回: "Deleted user 123"

# 错误的认证 (会返回 401)
curl http://127.0.0.1:8080/admin/dashboard
# 返回: {"error": "Unauthorized"}
```

## 💡 设计理念

1. **保持现有 API 兼容性** - 旧的写法依然完全支持
2. **增加链式调用便利性** - 新的写法更加流畅自然
3. **零成本抽象** - 链式调用不会带来任何性能开销
4. **函数式风格** - 中间件依然保持简洁的函数式设计
