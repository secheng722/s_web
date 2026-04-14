# mini_blog 示例

位置：`examples/08_mini_blog`

这是一个多文件结构的博客示例，演示如何在不修改 core 源码的前提下，通过传递数据库对象（Arc<SqlitePool>）完成 SQLite CRUD。

## 目录结构

- src/main.rs: 启动入口
- src/app.rs: 路由装配
- src/db.rs: 建表和种子数据
- src/models.rs: 数据模型
- src/dto.rs: 请求/响应 DTO
- src/repository.rs: 仓储层（数据库访问）
- src/error.rs: 统一错误映射
- src/middleware.rs: 日志中间件
- src/handlers/posts.rs: 文章接口
- src/handlers/comments.rs: 评论接口
- src/handlers/examples.rs: 统计与事务示例接口

## 运行

cargo run -p mini_blog

服务地址: http://127.0.0.1:3008

## 接口示例

1) 列出文章（支持搜索和发布过滤）

GET /api/posts
GET /api/posts?q=Rust
GET /api/posts?published=true

2) 创建草稿文章

POST /api/posts
{
  "title": "我的第一篇文章",
  "content": "先写草稿，再发布"
}

3) 发布文章

PATCH /api/posts/1/publish

4) 新增评论

POST /api/posts/1/comments
{
  "author": "Tom",
  "content": "写得很好"
}

5) 事务示例：一次请求内创建已发布文章 + 首条评论

POST /api/examples/quick-publish
{
  "title": "事务创建文章",
  "content": "这个接口演示一个事务内完成两次写入",
  "first_comment_author": "Jerry",
  "first_comment_content": "首评到此一游"
}

6) 统计与重置

GET /api/stats
POST /api/examples/reset
