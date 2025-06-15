# Hello World Example

这是一个使用 Ree HTTP 框架的简单示例项目。

## 运行示例

```bash
cd examples/hello_world
cargo run
```

或者从项目根目录运行：

```bash
cargo run --example hello_world
```

## 功能展示

### 基本路由
- `GET /` - 简单的问候页面
- `GET /hello/:name` - 带参数的个性化问候

### API 路由组
- `GET /api/users` - 获取用户列表
- `GET /api/users/:id` - 根据ID获取特定用户

### 中间件
- 访问日志中间件 - 记录每个请求的方法、路径、状态码和响应时间

## 测试端点

启动服务器后，你可以访问以下URL：

- http://127.0.0.1:8080/
- http://127.0.0.1:8080/hello/世界
- http://127.0.0.1:8080/api/users
- http://127.0.0.1:8080/api/users/1

## 代码结构

- `src/main.rs` - 主入口文件，配置路由和中间件
- `src/handlers.rs` - 请求处理函数
- `Cargo.toml` - 项目依赖配置
