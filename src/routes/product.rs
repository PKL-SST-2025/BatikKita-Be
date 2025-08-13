use actix_web::{get, post, put, delete, web, HttpResponse, Responder, Result};
use sqlx::PgPool;
use crate::models::product::*;
use crate::models::user::Claims;
use crate::middleware::auth;
use crate::utils::error;

#[get("/products")]
async fn get_products(
    pool: web::Data<PgPool>,
    query: web::Query<ProductFilter>,
) -> Result<impl Responder> {
    let mut sql = "SELECT * FROM products WHERE is_active = true".to_string();
    let mut params = Vec::new();
    let mut param_count = 1;

    // Apply filters
    if let Some(category) = &query.category {
        sql.push_str(&format!(" AND category = ${}", param_count));
        params.push(category.clone());
        param_count += 1;
    }

    if let Some(search) = &query.search {
        sql.push_str(&format!(" AND (name ILIKE ${})", param_count));
        params.push(format!("%{}%", search));
        param_count += 1;
    }

    if let Some(min_price) = &query.min_price {
        sql.push_str(&format!(" AND price >= ${}", param_count));
        params.push(min_price.to_string());
        param_count += 1;
    }

    if let Some(max_price) = &query.max_price {
        sql.push_str(&format!(" AND price <= ${}", param_count));
        params.push(max_price.to_string());
        param_count += 1;
    }

    if query.in_stock_only.unwrap_or(false) {
        sql.push_str(" AND stock_quantity > 0");
    }

    sql.push_str(" ORDER BY created_at DESC");

    let products = match sqlx::query_as::<_, Product>(&sql)
        .fetch_all(pool.get_ref())
        .await 
    {
        Ok(products) => products,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return error::internal_error("Failed to fetch products");
        }
    };

    use crate::utils::response::ApiResponse;
    Ok(HttpResponse::Ok().json(ApiResponse::success(products, "Products retrieved successfully")))
}

#[get("/products/{id}")]
async fn get_product_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<impl Responder> {
    let product_id = path.into_inner();

    let product = match sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = $1 AND is_active = true"
    )
    .bind(product_id)
    .fetch_optional(pool.get_ref())
    .await {
        Ok(product) => product,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch product"));
        }
    };
    
    match product {
        Some(product) => Ok(HttpResponse::Ok().json(product)),
        None => Ok(HttpResponse::NotFound().json("Product not found")),
    }
}

#[get("/products/{id}/reviews")]
async fn get_product_reviews(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<impl Responder> {
    let product_id = path.into_inner();

    let reviews = match sqlx::query_as::<_, Review>(
        "SELECT r.*, u.name as user_name 
         FROM reviews r 
         JOIN users u ON r.user_id = u.id 
         WHERE r.product_id = $1 
         ORDER BY r.created_at DESC"
    )
    .bind(product_id)
    .fetch_all(pool.get_ref())
    .await {
        Ok(reviews) => reviews,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch reviews"));
        }
    };

    Ok(HttpResponse::Ok().json(reviews))
}

#[post("/products/{id}/reviews")]
async fn create_review(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    review_data: web::Json<CreateReviewRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let product_id = path.into_inner();
    let user_id: i32 = claims.sub.parse().unwrap();

    // Check if user has already reviewed this product
    let existing_review = match sqlx::query!(
        "SELECT id FROM reviews WHERE product_id = $1 AND user_id = $2",
        product_id,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    if existing_review.is_some() {
        return Ok(HttpResponse::BadRequest().json("You have already reviewed this product"));
    }

    // Create new review
    let review = match sqlx::query_as::<_, Review>(
        "INSERT INTO reviews (product_id, user_id, rating, comment, created_at) 
         VALUES ($1, $2, $3, $4, NOW()) 
         RETURNING id, product_id, user_id, '' as user_name, rating, comment, false as is_verified, created_at"
    )
    .bind(product_id)
    .bind(user_id)
    .bind(review_data.rating)
    .bind(&review_data.comment)
    .fetch_one(pool.get_ref())
    .await {
        Ok(review) => review,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to create review"));
        }
    };

    Ok(HttpResponse::Created().json(review))
}

// Admin routes
#[post("/admin/products")]
async fn create_product(
    pool: web::Data<PgPool>,
    product_data: web::Json<CreateProductRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }

    let product = match sqlx::query_as::<_, Product>(
        "INSERT INTO products (name, description, short_description, price, discount_price, sku, stock_quantity, category, brand, weight, dimensions, is_active, is_featured, stock, image_url, additional_images, original_price, size_options, color_options, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, NOW(), NOW())
         RETURNING *"
    )
    .bind(&product_data.name)
    .bind(&product_data.description)
    .bind(&product_data.short_description)
    .bind(&product_data.price)
    .bind(&product_data.discount_price)
    .bind(&product_data.sku)
    .bind(product_data.stock_quantity)
    .bind(&product_data.category)
    .bind(&product_data.brand)
    .bind(&product_data.weight)
    .bind(&product_data.dimensions)
    .bind(product_data.is_active.unwrap_or(true))
    .bind(product_data.is_featured.unwrap_or(false))
    .bind(product_data.stock)
    .bind(&product_data.image_url)
    .bind(&product_data.additional_images)
    .bind(&product_data.original_price)
    .bind(&product_data.size_options)
    .bind(&product_data.color_options)
    .fetch_one(pool.get_ref())
    .await {
        Ok(product) => product,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to create product"));
        }
    };

    Ok(HttpResponse::Created().json(product))
}

#[put("/admin/products/{id}")]
async fn update_product(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    product_data: web::Json<UpdateProductRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }

    let product_id = path.into_inner();

    let product = match sqlx::query_as::<_, Product>(
        "UPDATE products SET 
         name = COALESCE($1, name),
         description = COALESCE($2, description),
         short_description = COALESCE($3, short_description),
         price = COALESCE($4, price),
         discount_price = COALESCE($5, discount_price),
         sku = COALESCE($6, sku),
         stock_quantity = COALESCE($7, stock_quantity),
         category = COALESCE($8, category),
         brand = COALESCE($9, brand),
         weight = COALESCE($10, weight),
         dimensions = COALESCE($11, dimensions),
         is_active = COALESCE($12, is_active),
         is_featured = COALESCE($13, is_featured),
         stock = COALESCE($14, stock),
         image_url = COALESCE($15, image_url),
         additional_images = COALESCE($16, additional_images),
         original_price = COALESCE($17, original_price),
         size_options = COALESCE($18, size_options),
         color_options = COALESCE($19, color_options),
         updated_at = NOW()
         WHERE id = $20
         RETURNING *"
    )
    .bind(&product_data.name)
    .bind(&product_data.description)
    .bind(&product_data.short_description)
    .bind(&product_data.price)
    .bind(&product_data.discount_price)
    .bind(&product_data.sku)
    .bind(product_data.stock_quantity)
    .bind(&product_data.category)
    .bind(&product_data.brand)
    .bind(&product_data.weight)
    .bind(&product_data.dimensions)
    .bind(product_data.is_active)
    .bind(product_data.is_featured)
    .bind(product_data.stock)
    .bind(&product_data.image_url)
    .bind(&product_data.additional_images)
    .bind(&product_data.original_price)
    .bind(&product_data.size_options)
    .bind(&product_data.color_options)
    .bind(product_id)
    .fetch_optional(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to update product"));
        }
    };

    match product {
        Some(product) => Ok(HttpResponse::Ok().json(product)),
        None => Ok(HttpResponse::NotFound().json("Product not found")),
    }
}

#[delete("/admin/products/{id}")]
async fn delete_product(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }

    let product_id = path.into_inner();

    let result = match sqlx::query!(
        "UPDATE products SET is_active = false, updated_at = NOW() WHERE id = $1",
        product_id
    )
    .execute(pool.get_ref())
    .await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to delete product"));
        }
    };

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json("Product deleted successfully"))
    } else {
        Ok(HttpResponse::NotFound().json("Product not found"))
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_products)
        .service(get_product_by_id)
        .service(get_product_reviews)
        .service(
            web::scope("/auth")
                .wrap(auth::AuthMiddleware)
                .service(create_review)
        )
        .service(
            web::scope("/admin")
                .wrap(auth::AuthMiddleware)
                .service(create_product)
                .service(update_product)
                .service(delete_product)
        );
}