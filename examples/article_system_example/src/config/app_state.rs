use sqlx::{Pool, Sqlite};
use std::sync::Arc;

pub struct AppState {
    pub db: Pool<Sqlite>,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db: Pool<Sqlite>, jwt_secret: String) -> Arc<Self> {
        Arc::new(Self { db, jwt_secret })
    }
}
