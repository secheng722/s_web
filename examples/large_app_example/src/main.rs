// main.rs
mod middleware;
mod routes;
mod config;

use ree::Engine;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting Large Application Example");
    
    // Initialize app
    let mut app = Engine::new();
    
    // Register global middleware
    middleware::register_global_middleware(&mut app);
    
    // Register all routes
    routes::register_all_routes(&mut app);
    
    // Start the server
    println!("Server listening on http://127.0.0.1:3000");
    app.run("127.0.0.1:3000").await?;
    
    Ok(())
}
