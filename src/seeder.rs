use sqlx::PgPool;
use bcrypt::{hash, DEFAULT_COST};

pub async fn create_admin(pool: &PgPool) {
    let email = "admin@batik.com";
    let password = "admin123";
    let role = "admin";
    let name = "Admin";

    // Cek apakah admin sudah ada
    let existing: Option<(i32,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .expect("Failed to check admin");

    if existing.is_none() {
        let hashed_pw = hash(password, DEFAULT_COST).unwrap();
        sqlx::query("INSERT INTO users (name, email, password, role) VALUES ($1, $2, $3, $4)")
            .bind(name)
            .bind(email)
            .bind(&hashed_pw)
            .bind(role)
            .execute(pool)
            .await
            .expect("Failed to insert admin");
        println!("✅ Admin account created: admin@batik.com / admin123");
    } else {
        println!("ℹ️ Admin account already exists.");
    }
}