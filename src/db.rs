use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL harus diset di .env");
    PgPoolOptions::new().max_connections(5).connect(&database_url).await
}