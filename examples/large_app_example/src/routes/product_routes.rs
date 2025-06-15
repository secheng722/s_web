// routes/product_routes.rs
use ree::{Engine, RequestCtx};
use serde_json::json;

/// Register product-related routes
pub fn register_routes(app: &mut Engine) {
    // Create product group
    let mut products = app.group("/products");
    
    // Apply product-specific middleware
    products.use_middleware(crate::middleware::cache::cache_response(300)); // Cache for 5 minutes
    
    // Define product routes
    products.get("/", get_products);
    products.get("/:id", get_product_by_id);
    products.get("/category/:category", get_products_by_category);
    products.post("/", create_product);
    products.put("/:id", update_product);
    products.delete("/:id", delete_product);
}

// Handler functions
async fn get_products(_ctx: RequestCtx) -> impl ree::IntoResponse {
    json!({
        "products": [
            {"id": 1, "name": "Product 1", "price": 29.99},
            {"id": 2, "name": "Product 2", "price": 49.99}
        ]
    })
}

async fn get_product_by_id(ctx: RequestCtx) -> impl ree::IntoResponse {
    let id = ctx.get_param("id").unwrap_or("0");
    json!({
        "id": id,
        "name": format!("Product {}", id),
        "price": 29.99,
        "description": "Product description"
    })
}

async fn get_products_by_category(ctx: RequestCtx) -> impl ree::IntoResponse {
    let category = ctx.get_param("category").unwrap_or("unknown");
    json!({
        "category": category,
        "products": [
            {"id": 1, "name": format!("{} Product 1", category), "price": 29.99},
            {"id": 2, "name": format!("{} Product 2", category), "price": 49.99}
        ]
    })
}

async fn create_product(_ctx: RequestCtx) -> impl ree::IntoResponse {
    json!({
        "status": "success",
        "message": "Product created",
        "id": 3
    })
}

async fn update_product(ctx: RequestCtx) -> impl ree::IntoResponse {
    let id = ctx.get_param("id").unwrap_or("0");
    json!({
        "status": "success",
        "message": format!("Product {} updated", id)
    })
}

async fn delete_product(ctx: RequestCtx) -> impl ree::IntoResponse {
    let id = ctx.get_param("id").unwrap_or("0");
    json!({
        "status": "success",
        "message": format!("Product {} deleted", id)
    })
}
