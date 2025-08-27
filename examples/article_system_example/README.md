# s_web Framework - 博客API系统示例

基于 Rust 和 s_web 框架构建的完整博客API系统，提供用户认证、文章管理等功能。

## 📋 目录

- [功能特性](#功能特性)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [快速开始](#快速开始)
- [API文档](#api文档)
- [配置说明](#配置说明)
- [开发指南](#开发指南)

## ✨ 功能特性

- 🔐 **用户认证系统**
  - 用户注册与登录
  - JWT令牌认证
  - 密码加密存储

- 📝 **文章管理系统**
  - 创建、编辑、删除文章
  - 文章列表查看
  - 用户权限控制

- 🎨 **前端界面**
  - 响应式设计
  - 现代化UI界面
  - AJAX异步交互

- 🛡️ **安全特性**
  - JWT认证中间件
  - 请求参数验证
  - SQL注入防护

## 🛠️ 技术栈

### 后端
- **框架**: [s_web](https://github.com/lsc/ree) - 轻量级异步Web框架
- **数据库**: SQLite + SQLx
- **认证**: JWT (jsonwebtoken)
- **密码加密**: bcrypt
- **序列化**: serde + serde_json
- **异步运行时**: tokio
- **异步运行时**: tokio

### 前端
- **HTML5 + CSS3 + JavaScript**
- **响应式设计**
- **现代化UI组件**

### 开发工具
- **语言**: Rust 2021 Edition
- **包管理**: Cargo
- **数据库**: SQLite

## 📁 项目结构

```
article_system_example/
├── src/
│   ├── main.rs              # 应用入口
│   ├── config/              # 配置模块
│   │   ├── mod.rs
│   │   └── app_state.rs     # 应用状态管理
│   ├── db/                  # 数据库模块
│   │   ├── mod.rs
│   │   └── sqlite.rs        # SQLite数据库初始化
│   ├── handlers/            # 请求处理器
│   │   ├── mod.rs
│   │   ├── auth.rs          # 认证相关处理器
│   │   └── article.rs       # 文章相关处理器
│   ├── middleware/          # 中间件
│   │   ├── mod.rs
│   │   └── auth.rs          # 认证中间件
│   ├── models/              # 数据模型
│   │   ├── mod.rs
│   │   ├── user.rs          # 用户模型
│   │   └── article.rs       # 文章模型
│   └── routes/              # 路由配置
│       ├── mod.rs
│       ├── auth_routes.rs   # 认证路由
│       ├── article_routes.rs # 文章路由
│       └── base_routes.rs   # 基础路由
├── frontend/                # 前端文件
│   ├── *.html              # HTML页面
│   ├── css/                # 样式文件
│   │   └── style.css
│   └── js/                 # JavaScript文件
│       ├── app.js          # 应用核心
│       ├── auth.js         # 认证相关
│       ├── articles.js     # 文章列表
│       └── create-article.js # 文章创建
├── data/                   # 数据目录
│   └── app.db             # SQLite数据库文件
├── Cargo.toml             # 项目配置
└── README.md              # 项目说明
```

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 安装步骤

1. **进入项目目录**
   ```bash
   cd examples/article_system_example
   ```

2. **安装依赖**
   ```bash
   cargo build
   ```

3. **启动服务**
   ```bash
   cargo run
   ```

4. **访问应用**
   打开浏览器访问: http://127.0.0.1:3000

### 数据库初始化

应用首次启动时会自动创建SQLite数据库和必要的表结构：

- `users` 表：存储用户信息
- `articles` 表：存储文章信息

## 📚 API文档

### 认证相关

#### 用户注册
```http
POST /api/auth/register
Content-Type: application/json

{
  "username": "用户名",
  "email": "邮箱地址",
  "password": "密码"
}
```

#### 用户登录
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "用户名",
  "password": "密码"
}
```

#### 获取用户信息
```http
GET /api/auth/me
Authorization: Bearer <JWT_TOKEN>
```

### 文章相关

#### 获取文章列表
```http
GET /api/articles
```

#### 获取单篇文章
```http
GET /api/articles/:id
```

#### 创建文章（需要认证）
```http
POST /api/articles/protected
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "title": "文章标题",
  "content": "文章内容"
}
```

#### 更新文章（需要认证）
```http
PUT /api/articles/protected/:id
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json

{
  "title": "新标题",
  "content": "新内容"
}
```

#### 删除文章（需要认证）
```http
DELETE /api/articles/protected/:id
Authorization: Bearer <JWT_TOKEN>
```

## ⚙️ 配置说明

### 应用配置

在 `src/main.rs` 中可以修改以下配置：

```rust
// JWT密钥（生产环境建议从环境变量读取）
let jwt_secret = "your_jwt_secret_key_here".to_string();

// 服务器监听地址
app.run("127.0.0.1:3000").await?;
```

### 数据库配置

数据库文件位置：`data/app.db`

如需修改数据库配置，编辑 `src/db/sqlite.rs` 文件。

## 🔧 开发指南

### 添加新的API端点

1. 在 `src/models/` 中定义数据模型
2. 在 `src/handlers/` 中实现处理逻辑
3. 在 `src/routes/` 中注册路由
4. 在前端添加相应的JavaScript代码

### 中间件开发

参考 `src/middleware/auth.rs` 实现自定义中间件：

```rust
pub async fn your_middleware(
    state: Arc<AppState>, 
    ctx: RequestCtx, 
    next: Next
) -> Response {
    // 中间件逻辑
    next(ctx).await
}
```

### 数据库迁移

修改 `src/db/sqlite.rs` 中的建表语句来更新数据库结构。

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 🐛 问题反馈

如果您发现任何问题或有改进建议，请通过以下方式联系：

- 提交 [Issue](../../issues)
- 发送邮件至：your-email@example.com

## 📈 更新日志

### v0.1.0 (2025-06-27)
- ✨ 初始版本发布
- 🔐 用户认证系统
- 📝 文章管理功能
- 🎨 前端界面
- 🛡️ JWT认证中间件

---

**使用愉快！如果这个项目对您有帮助，请给个 ⭐️**
