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
let mut app = Engine::new()
    .use_middleware(|ctx, next| logger("Global", ctx, next))
    .use_middleware(|ctx, next| cors(ctx, next))
    .get("/", |_| async { "Hello" })
    .get("/health", |_| async { "OK" });

let api = app.group("/api");
api.use_middleware(|ctx, next| auth("token", ctx, next))
   .get("/users", |_| async { "users" })
   .post("/users", |_| async { "created" });
```

## 🏃‍♂️ 运行示例

```bash
cd examples/chain_example
cargo run
```

## 🧪 测试端点

### 公开端点
```bash
# 基本端点
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/health
```

### 需要 API 认证的端点
```bash
# 正确的认证
curl -H "Authorization: Bearer api-token" http://127.0.0.1:8080/api/users
curl -X POST -H "Authorization: Bearer api-token" http://127.0.0.1:8080/api/users

# 错误的认证 (会返回 401)
curl http://127.0.0.1:8080/api/users
```

### 需要管理员认证的端点
```bash
# 正确的认证
curl -H "Authorization: Bearer admin-token" http://127.0.0.1:8080/admin/dashboard
curl -X DELETE -H "Authorization: Bearer admin-token" http://127.0.0.1:8080/admin/users/123

# 错误的认证 (会返回 401)
curl http://127.0.0.1:8080/admin/dashboard
```

## 💡 设计理念

1. **保持现有 API 兼容性** - 旧的写法依然完全支持
2. **增加链式调用便利性** - 新的写法更加流畅自然
3. **零成本抽象** - 链式调用不会带来任何性能开销
4. **函数式风格** - 中间件依然保持简洁的函数式设计
