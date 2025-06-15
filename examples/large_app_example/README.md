# Large Application Example

This example demonstrates how to structure a larger application using the Ree framework.

## Project Structure

```
src/
├── main.rs              # Application entry point
├── routes/              # Route definitions (organized by feature)
│   ├── mod.rs           # Route module exports
│   ├── user_routes.rs   # User-related routes
│   ├── product_routes.rs # Product-related routes
│   └── ...
├── middleware/          # Custom middleware
│   ├── mod.rs           # Middleware module exports
│   ├── auth.rs          # Authentication middleware
│   ├── cache.rs         # Cache middleware
│   └── ...
├── handlers/            # Business logic for request handling
│   ├── mod.rs
│   ├── user.rs
│   └── ...
├── models/              # Data models
│   ├── mod.rs
│   ├── user.rs
│   └── ...
└── config/              # Application configuration
    └── mod.rs

```

## Key Concepts

1. **Modular Organization**: Routes, middleware, handlers and models are organized into separate modules.
2. **Feature-based Structure**: Routes and handlers are grouped by feature (users, products, etc.).
3. **Centralized Route Registration**: All routes are registered in a central place for clarity.
4. **Reusable Middleware**: Middleware is defined separately and applied globally or per route group.

## Getting Started

Run the example:

```bash
cargo run --example large_app_example
```

## API Endpoints

The example implements these endpoints:

- **User API**:
  - `GET /users` - List all users
  - `GET /users/:id` - Get user by ID
  - `POST /users` - Create a new user
  - `PUT /users/:id` - Update a user
  - `DELETE /users/:id` - Delete a user

- **Product API**:
  - `GET /products` - List all products
  - `GET /products/:id` - Get product by ID
  - `GET /products/category/:category` - Get products by category
  - `POST /products` - Create a new product
  - `PUT /products/:id` - Update a product
  - `DELETE /products/:id` - Delete a product

## Authentication

For protected endpoints, include an Authorization header:

```
Authorization: Bearer your-token
```
