use actix_web::{get, post, delete, web, HttpResponse, Responder, Result};
use sqlx::PgPool;
use bigdecimal::BigDecimal;
use crate::models::product::Favorite;
use crate::models::user::Claims;
use crate::middleware::AuthMiddleware;

#[get("")]
async fn get_favorites(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    let favorites = match sqlx::query!(
        "SELECT f.id, f.product_id, f.created_at, p.name, p.image_url, p.price, p.original_price, 
                COALESCE(AVG(r.rating), 0) as rating, COUNT(r.id) as reviews_count
         FROM favorites f
         JOIN products p ON f.product_id = p.id
         LEFT JOIN reviews r ON p.id = r.product_id
         WHERE f.user_id = $1 AND p.is_active = true
         GROUP BY f.id, f.product_id, f.created_at, p.name, p.image_url, p.price, p.original_price
         ORDER BY f.created_at DESC",
        user_id
    )
    .fetch_all(pool.get_ref())
    .await {
        Ok(favorites) => favorites,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch favorites"));
        }
    };

    let favorites_with_products: Vec<serde_json::Value> = favorites
        .into_iter()
        .map(|f| {
            serde_json::json!({
                "id": f.id,
                "product_id": f.product_id,
                "created_at": f.created_at,
                "product": {
                    "id": f.product_id,
                    "name": f.name,
                    "image_url": f.image_url,
                    "price": f.price,
                    "original_price": f.original_price,
                    "rating": f.rating.unwrap_or_else(|| bigdecimal::BigDecimal::from(0)),
                    "reviews_count": f.reviews_count.unwrap_or(0)
                }
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(favorites_with_products))
}

#[post("/{product_id}")]
async fn add_to_favorites(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let product_id = path.into_inner();

    // Check if product exists and is active
    let product_exists = match sqlx::query!(
        "SELECT id FROM products WHERE id = $1 AND is_active = true",
        product_id
    )
    .fetch_optional(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    if product_exists.is_none() {
        return Ok(HttpResponse::NotFound().json("Product not found"));
    }

    // Check if already in favorites
    let existing_favorite = match sqlx::query!(
        "SELECT id FROM favorites WHERE user_id = $1 AND product_id = $2",
        user_id,
        product_id
    )
    .fetch_optional(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    if existing_favorite.is_some() {
        return Ok(HttpResponse::BadRequest().json("Product already in favorites"));
    }

    // Add to favorites
    let favorite = match sqlx::query_as::<_, Favorite>(
        "INSERT INTO favorites (user_id, product_id, created_at) 
         VALUES ($1, $2, NOW()) 
         RETURNING *"
    )
    .bind(user_id)
    .bind(product_id)
    .fetch_one(pool.get_ref())
    .await {
        Ok(favorite) => favorite,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to add to favorites"));
        }
    };

    Ok(HttpResponse::Created().json(favorite))
}

#[delete("/{product_id}")]
async fn remove_from_favorites(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let product_id = path.into_inner();

    let result = match sqlx::query!(
        "DELETE FROM favorites WHERE user_id = $1 AND product_id = $2",
        user_id,
        product_id
    )
    .execute(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to remove from favorites"));
        }
    };

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json("Removed from favorites successfully"))
    } else {
        Ok(HttpResponse::NotFound().json("Product not in favorites"))
    }
}

#[get("/check/{product_id}")]
async fn check_favorite_status(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let product_id = path.into_inner();

    let favorite = match sqlx::query!(
        "SELECT id FROM favorites WHERE user_id = $1 AND product_id = $2",
        user_id,
        product_id
    )
    .fetch_optional(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    let is_favorite = favorite.is_some();
    Ok(HttpResponse::Ok().json(serde_json::json!({ "is_favorite": is_favorite })))
}

#[delete("/clear")]
async fn clear_favorites(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    match sqlx::query!(
        "DELETE FROM favorites WHERE user_id = $1",
        user_id
    )
    .execute(pool.get_ref())
    .await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to clear favorites"));
        }
    };

    Ok(HttpResponse::Ok().json("Favorites cleared successfully"))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/favorites")
            .wrap(AuthMiddleware)
            .service(get_favorites)
            .service(add_to_favorites)
            .service(remove_from_favorites)
            .service(check_favorite_status)
            .service(clear_favorites)
    );
}
