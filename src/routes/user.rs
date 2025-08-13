use actix_web::{get, post, put, delete, web, HttpResponse, Responder, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use crate::models::user::{User, UserPublic, Claims, UserAddress, CreateAddressRequest, UpdateAddressRequest, ChangePasswordRequest, UpdateProfileRequest};
use crate::middleware::AuthMiddleware;
use crate::utils::error_helpers::handle_db_error;
use bcrypt::{hash, verify, DEFAULT_COST};

#[get("/profile")]
async fn get_profile(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    let user = handle_db_error(
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await,
        "Failed to fetch user"
    )?;

    match user {
        Some(user) => {
            let user_public = UserPublic {
                id: user.id,
                name: user.name.unwrap_or_else(|| "".to_string()),
                email: user.email,
                role: user.role,
                phone: user.phone,
                email_verified: user.email_verified,
                created_at: user.created_at.unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            };
            Ok(HttpResponse::Ok().json(user_public))
        }
        None => Ok(HttpResponse::NotFound().json("User not found")),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .wrap(AuthMiddleware)
            .service(get_profile)
    );
}
