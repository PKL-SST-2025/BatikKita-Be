use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::models::user::User;
use crate::utils::jwt::create_jwt;
use crate::utils::response::ApiResponse;
use sqlx::PgPool;
use bcrypt::{verify, hash, DEFAULT_COST};
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub role: Option<String>,
}

#[post("/register")]
pub async fn register(
    data: web::Json<RegisterRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    // Cek apakah email sudah terdaftar
    let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", data.email)
        .fetch_optional(pool.get_ref())
        .await;
    if let Ok(Some(_)) = existing {
        return HttpResponse::BadRequest().body("Email sudah terdaftar");
    }
    // Hash password
    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Gagal hash password"),
    };
    // Insert user baru
    let role = data.role.clone().unwrap_or_else(|| "customer".to_string());
    let full_name = format!("{} {}", data.first_name, data.last_name);
    let inserted = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, first_name, last_name, phone, role, name, password) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id, username, email, first_name, last_name, role",
        data.username, data.email, hashed, data.first_name, data.last_name, data.phone, role, full_name, hashed
    )
    .fetch_one(pool.get_ref())
    .await;
    match inserted {
        Ok(u) => HttpResponse::Ok().json(super::auth::UserResponse {
            id: u.id,
            name: format!("{} {}", u.first_name, u.last_name),
            email: u.email,
            role: u.role,
        }),
        Err(e) => {
            println!("Gagal insert user: {}", e);
            HttpResponse::InternalServerError().body("Gagal register user")
        }
    }
}


#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
}

#[post("/login")]
pub async fn login(
    data: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user = sqlx::query!(
        "SELECT id, username, first_name, last_name, email, password_hash, role FROM users WHERE email = $1", 
        data.email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(u)) => {
            if verify(&data.password, &u.password_hash).unwrap_or(false) {
                let token = create_jwt(u.id, u.role.clone()).unwrap();
                let response_data = LoginResponse {
                    user: UserResponse {
                        id: u.id,
                        name: format!("{} {}", u.first_name, u.last_name),
                        email: u.email,
                        role: u.role,
                    },
                    token,
                };
                HttpResponse::Ok().json(ApiResponse::success(response_data, "Login successful"))
            } else {
                HttpResponse::Unauthorized().json("Invalid credentials")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json("User not found"),
        Err(_) => HttpResponse::InternalServerError().json("Database error"),
    }
}