use actix_web::{get, post, put, delete, web, HttpResponse, Responder, Result};
use sqlx::PgPool;
use crate::models::cart::*;
use crate::models::user::Claims;
use crate::middleware::AuthMiddleware;
use crate::utils::error_helpers::handle_db_error;

#[get("/cart")]
async fn get_cart(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    // Get or create cart for user
    let cart = handle_db_error(
        sqlx::query_as::<_, Cart>(
            "INSERT INTO carts (user_id, created_at, updated_at) 
             VALUES ($1, NOW(), NOW()) 
             ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
             RETURNING *"
        )
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await,
        "Failed to get cart"
    )?;

    // Get cart items with product details
    let items = handle_db_error(
        sqlx::query_as::<_, CartItemWithProduct>(
            "SELECT ci.id, ci.cart_id, ci.product_id, p.name as product_name, p.image_url as product_image,
                    ci.quantity, ci.size, ci.color, ci.price_at_time, p.price as current_price, 
                    p.stock as stock_available, ci.created_at
             FROM cart_items ci
             JOIN products p ON ci.product_id = p.id
             WHERE ci.cart_id = $1 AND p.is_active = true
             ORDER BY ci.created_at DESC"
        )
        .bind(cart.id)
        .fetch_all(pool.get_ref())
        .await,
        "Failed to get cart items"
    )?;

    let total_items = items.iter().map(|item| item.quantity).sum();
    let total_price = items.iter().map(|item| item.price_at_time * item.quantity as i64).sum();

    let cart_summary = CartSummary {
        total_items,
        total_price,
        items,
    };

    Ok(HttpResponse::Ok().json(cart_summary))
}

#[post("/cart/items")]
async fn add_to_cart(
    pool: web::Data<PgPool>,
    item_data: web::Json<AddToCartRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    // Get or create cart
    let cart = handle_db_error(
        sqlx::query_as::<_, Cart>(
            "INSERT INTO carts (user_id, created_at, updated_at) 
             VALUES ($1, NOW(), NOW()) 
             ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
             RETURNING *"
        )
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await,
        "Failed to get cart"
    )?;

    // Get product details and check stock
    let product = handle_db_error(
        sqlx::query!(
            "SELECT price, stock FROM products WHERE id = $1 AND is_active = true",
            item_data.product_id
        )
        .fetch_optional(pool.get_ref())
        .await,
        "Database error"
    )?;

    let product = match product {
        Some(p) => p,
        None => return Ok(HttpResponse::NotFound().json("Product not found")),
    };

    if product.stock.unwrap_or(0) < item_data.quantity {
        return Ok(HttpResponse::BadRequest().json("Insufficient stock"));
    }

    // Check if item already exists in cart
    let existing_item = handle_db_error(
        sqlx::query!(
            "SELECT id, quantity FROM cart_items 
             WHERE cart_id = $1 AND product_id = $2 AND size = $3 AND color = $4",
            cart.id,
            item_data.product_id,
            item_data.size,
            item_data.color
        )
        .fetch_optional(pool.get_ref())
        .await,
        "Database error"
    )?;

    if let Some(existing) = existing_item {
        // Update existing item
        let new_quantity = existing.quantity + item_data.quantity;
        if product.stock.unwrap_or(0) < new_quantity {
            return Ok(HttpResponse::BadRequest().json("Insufficient stock"));
        }

        handle_db_error(
            sqlx::query!(
                "UPDATE cart_items SET quantity = $1, updated_at = NOW() WHERE id = $2",
                new_quantity,
                existing.id
            )
            .execute(pool.get_ref())
            .await,
            "Failed to update cart item"
        )?;
    } else {
        // Create new cart item
        handle_db_error(
            sqlx::query!(
                "INSERT INTO cart_items (cart_id, product_id, quantity, size, color, price_at_time, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())",
                cart.id,
                item_data.product_id,
                item_data.quantity,
                item_data.size,
                item_data.color,
                product.price
            )
            .execute(pool.get_ref())
            .await,
            "Failed to add item to cart"
        )?;
    }

    Ok(HttpResponse::Created().json("Item added to cart successfully"))
}

#[put("/cart/items/{id}")]
async fn update_cart_item(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    item_data: web::Json<UpdateCartItemRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let item_id = path.into_inner();

    // Verify item belongs to user's cart
    let cart_item = handle_db_error(
        sqlx::query!(
            "SELECT ci.id, ci.product_id FROM cart_items ci
             JOIN carts c ON ci.cart_id = c.id
             WHERE ci.id = $1 AND c.user_id = $2",
            item_id,
            user_id
        )
        .fetch_optional(pool.get_ref())
        .await,
        "Database error"
    )?;

    if cart_item.is_none() {
        return Ok(HttpResponse::NotFound().json("Cart item not found"));
    }

    let cart_item = cart_item.unwrap();

    // Check stock
    let product = handle_db_error(
        sqlx::query!(
            "SELECT stock FROM products WHERE id = $1",
            cart_item.product_id
        )
        .fetch_one(pool.get_ref())
        .await,
        "Database error"
    )?;

    if product.stock.unwrap_or(0) < item_data.quantity {
        return Ok(HttpResponse::BadRequest().json("Insufficient stock"));
    }

    // Update cart item
    handle_db_error(
        sqlx::query!(
            "UPDATE cart_items SET quantity = $1, size = $2, color = $3, updated_at = NOW() 
             WHERE id = $4",
            item_data.quantity,
            item_data.size,
            item_data.color,
            item_id
        )
        .execute(pool.get_ref())
        .await,
        "Failed to update cart item"
    )?;

    Ok(HttpResponse::Ok().json("Cart item updated successfully"))
}

#[delete("/cart/items/{id}")]
async fn remove_from_cart(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let item_id = path.into_inner();

    let result = handle_db_error(
        sqlx::query!(
            "DELETE FROM cart_items ci
             USING carts c
             WHERE ci.cart_id = c.id AND ci.id = $1 AND c.user_id = $2",
            item_id,
            user_id
        )
        .execute(pool.get_ref())
        .await,
        "Failed to remove item from cart"
    )?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json("Item removed from cart successfully"))
    } else {
        Ok(HttpResponse::NotFound().json("Cart item not found"))
    }
}

#[delete("/cart/clear")]
async fn clear_cart(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    handle_db_error(
        sqlx::query!(
            "DELETE FROM cart_items ci
             USING carts c
             WHERE ci.cart_id = c.id AND c.user_id = $1",
            user_id
        )
        .execute(pool.get_ref())
        .await,
        "Failed to clear cart"
    )?;

    Ok(HttpResponse::Ok().json("Cart cleared successfully"))
}

// Guest cart endpoint (no authentication required)
#[get("/cart")]
async fn get_guest_cart() -> Result<impl Responder> {
    // Return empty cart for guest users
    let cart_summary = CartSummary {
        total_items: 0,
        total_price: 0,
        items: vec![],
    };

    Ok(HttpResponse::Ok().json(cart_summary))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_cart)
        .service(add_to_cart)
        .service(update_cart_item)
        .service(remove_from_cart)
        .service(clear_cart)
        .service(get_guest_cart); // Add guest cart endpoint without auth
}