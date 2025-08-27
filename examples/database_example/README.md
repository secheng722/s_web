# 📊 s_web Framework - Database Example

这个示例展示如何使用 s_web 框架构建一个带有数据库的完整 CRUD API。

## 🌟 特性

- **🗄️ SQLite 数据库**: 轻量级数据库，无需额外配置
- **🔄 完整 CRUD**: 创建、读取、更新、删除操作
- **🛡️ 错误处理**: 优雅的错误处理和用户友好的错误消息
- **📝 数据验证**: 请求体验证和数据类型安全
- **🌐 CORS 支持**: 跨域资源共享支持
- **📊 JSON API**: RESTful JSON API 设计
- **🎯 路由分组**: 使用 `/api/v1` 前缀组织路由
- **💾 状态管理**: 共享数据库连接池

## 🚀 快速开始

### 运行示例

```bash
cd examples/database_example
cargo run
```

### 访问 API

服务器启动后，访问 http://127.0.0.1:3000 查看 API 文档。

## 📋 API 端点

| 方法   | 端点                | 描述         |
|--------|---------------------|--------------|
| GET    | `/api/v1/users`     | 获取所有用户 |
| POST   | `/api/v1/users`     | 创建新用户   |
| GET    | `/api/v1/users/:id` | 获取特定用户 |
| PUT    | `/api/v1/users/:id` | 更新用户     |
| DELETE | `/api/v1/users/:id` | 删除用户     |
| GET    | `/health`           | 健康检查     |

## 📊 数据模型

### User

```json
{
  "id": "uuid",
  "name": "string",
  "email": "string", 
  "created_at": "ISO 8601 timestamp"
}
```

### CreateUserRequest

```json
{
  "name": "string",
  "email": "string"
}
```

### UpdateUserRequest

```json
{
  "name": "string (optional)",
  "email": "string (optional)"
}
```

## 🧪 测试 API

服务器运行在 `http://127.0.0.1:3000`

### 查看 API 文档

```bash
curl http://127.0.0.1:3000/
# 返回: HTML 页面，包含完整的 API 文档
```

### 健康检查

```bash
curl http://127.0.0.1:3000/health
# 返回: {"status": "healthy", "timestamp": "...", "service": "s-web-database-example"}
```

### 创建用户

```bash
curl -X POST http://127.0.0.1:3000/api/v1/users \
  -H 'Content-Type: application/json' \
  -d '{"name":"张三","email":"zhangsan@example.com"}'
# 返回: 创建的用户信息，包含生成的 UUID
```

### 获取所有用户

```bash
curl http://127.0.0.1:3000/api/v1/users
# 返回: 所有用户的数组
```

### 获取特定用户

```bash
# 使用创建时返回的 UUID
curl http://127.0.0.1:3000/api/v1/users/{user_id}
# 返回: 用户详细信息
```

### 更新用户

```bash
curl -X PUT http://127.0.0.1:3000/api/v1/users/{user_id} \
  -H 'Content-Type: application/json' \
  -d '{"name":"李四","email":"lisi@example.com"}'
# 返回: 更新后的用户信息
```

### 删除用户

```bash
curl -X DELETE http://127.0.0.1:3000/api/v1/users/{user_id}
# 返回: 删除确认消息
```

### 错误处理示例

```bash
# 用户不存在 (404)
curl http://127.0.0.1:3000/api/v1/users/nonexistent-id
# 返回: {"error": "用户未找到"}

# 无效的 JSON (400)
curl -X POST http://127.0.0.1:3000/api/v1/users \
  -H 'Content-Type: application/json' \
  -d '{"invalid": json}'
# 返回: {"error": "Invalid request body"}

# 缺少必填字段 (400)
curl -X POST http://127.0.0.1:3000/api/v1/users \
  -H 'Content-Type: application/json' \
  -d '{}'
# 返回: 反序列化错误信息
```

## 🔧 技术栈

- **s_web Framework**: 轻量级 HTTP 框架
- **SQLx**: 异步 SQL 工具包
- **SQLite**: 嵌入式数据库
- **Serde**: 序列化/反序列化
- **UUID**: 唯一标识符生成
- **Chrono**: 日期时间处理

## 📁 项目结构

```
database_example/
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs
└── templates/
    └── index.html
```

## 💡 学习要点

1. **状态管理**: 如何在 s_web 应用中共享数据库连接池
2. **错误处理**: 数据库操作的错误处理模式
3. **中间件**: CORS 中间件的实现
4. **路由分组**: API 版本化的最佳实践
5. **数据验证**: 请求体解析和验证
6. **异步编程**: 异步数据库操作的处理

## 🔄 扩展建议

- 添加用户认证和授权
- 实现分页查询
- 添加数据库迁移系统
- 实现缓存层
- 添加日志记录
- 实现 API 限流
- 添加测试用例

这个示例为构建真实世界的 web 应用提供了良好的基础！
