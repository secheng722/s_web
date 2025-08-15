# Lifecycle Example

这个例子演示了 Ree 框架的异步生命周期钩子功能。

## 功能特性

- **启动钩子**: 在服务器启动时执行异步初始化任务
- **关闭钩子**: 在收到关闭信号时执行异步清理任务
- **并行执行**: 支持多个钩子并行执行
- **顺序保证**: 钩子按注册顺序执行
- **自动包装**: 框架自动处理 `Box::pin` 包装，用户只需传入普通的 async 函数

## 运行示例

```bash
cargo run --example lifecycle_example
```

## 测试生命周期

1. **启动过程**: 观察启动时的初始化日志
   ```bash
   curl http://127.0.0.1:8080/
   curl http://127.0.0.1:8080/health
   curl http://127.0.0.1:8080/status
   ```

2. **优雅关闭**: 按 `Ctrl+C` 观察关闭时的清理日志

## 示例输出

### 启动时
```
🌟 Lifecycle example server starting...
🚀 Starting application initialization...
🔌 Initializing database connection...
🧠 Initializing cache system...
📡 Registering service to discovery...
✅ Database connected successfully
✅ Cache system ready
✅ Service registered successfully
🎉 Application initialization completed!
🔥 Warming up system...
✅ System warmed up
🚀 Server running on http://127.0.0.1:8080
```

### 关闭时
```
^C
🛑 Graceful shutdown signal received
🛑 Starting graceful shutdown...
🔌 Closing database connections...
🧹 Cleaning up cache...
📡 Unregistering service from discovery...
✅ Service unregistered
✅ Cache cleaned up
✅ Database connections closed
✅ Graceful shutdown completed!
🧹 Final cleanup...
✅ Final cleanup completed
✅ All connections gracefully closed
```

## 实际应用场景

- 数据库连接池初始化/关闭
- 缓存系统预热/持久化
- 服务注册/注销（如 Nacos、Consul）
- 后台任务启动/停止
- 监控指标收集器初始化/清理
- 日志系统初始化/刷新
