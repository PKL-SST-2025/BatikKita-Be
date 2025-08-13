use actix_web::{get, post, put, web, HttpResponse, Responder, Result};
use sqlx::PgPool;
use uuid::Uuid;
use bigdecimal::{BigDecimal, Zero, ToPrimitive};
use crate::models::order::*;
use crate::models::user::Claims;
use crate::middleware::AuthMiddleware;

#[post("/checkout")]
async fn checkout(
    pool: web::Data<PgPool>,
    order_data: web::Json<CreateOrderRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    // Start transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to start transaction"));
        }
    };

    // Calculate total amount
    let mut total_amount = BigDecimal::zero();
    let mut order_items = Vec::new();

    for item in &order_data.items {
        // Get product details and check stock
        let product = match sqlx::query!(
            "SELECT price, stock, name, image_url FROM products WHERE id = $1 AND is_active = true",
            item.product_id
        )
        .fetch_optional(&mut *tx)
        .await {
            Ok(Some(p)) => p,
            Ok(None) => return Ok(HttpResponse::BadRequest().json(format!("Product {} not found", item.product_id))),
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Database error"));
            }
        };

        if product.stock.unwrap_or(0) < item.quantity {
            return Ok(HttpResponse::BadRequest().json(format!("Insufficient stock for product {}", item.product_id)));
        }

        let item_total = &product.price * BigDecimal::from(item.quantity);
        total_amount += &item_total;

        order_items.push((item, product, item_total));
    }

    // Apply coupon if provided
    let mut discount_amount = BigDecimal::zero();
    if let Some(coupon_code) = &order_data.coupon_code {
        let coupon = match sqlx::query_as::<_, Coupon>(
            "SELECT * FROM coupons WHERE code = $1 AND is_active = true 
             AND valid_from <= NOW() AND valid_until >= NOW()"
        )
        .bind(coupon_code)
        .fetch_optional(&mut *tx)
        .await {
            Ok(coupon) => coupon,
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Database error"));
            }
        };

        if let Some(coupon) = coupon {
            // Check usage limit
            if let Some(limit) = coupon.usage_limit {
                if coupon.used_count >= limit {
                    return Ok(HttpResponse::BadRequest().json("Coupon usage limit exceeded"));
                }
            }

            // Check minimum order amount
            if let Some(min_amount) = coupon.min_order_amount {
                if total_amount < BigDecimal::from(min_amount) {
                    return Ok(HttpResponse::BadRequest().json("Order amount below minimum for coupon"));
                }
            }

            // Calculate discount
            if coupon.discount_type == "percentage" {
                discount_amount = (&total_amount * BigDecimal::from(coupon.discount_value)) / BigDecimal::from(100);
            } else {
                discount_amount = BigDecimal::from(coupon.discount_value);
            }

            // Apply maximum discount limit
            if let Some(max_discount) = coupon.max_discount_amount {
                discount_amount = discount_amount.min(BigDecimal::from(max_discount));
            }

            // Update coupon usage
            if let Err(e) = sqlx::query!(
                "UPDATE coupons SET used_count = used_count + 1 WHERE id = $1",
                coupon.id
            )
            .execute(&mut *tx)
            .await {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Database error"));
            }
        } else {
            return Ok(HttpResponse::BadRequest().json("Invalid coupon code"));
        }
    }

    // Calculate shipping cost (simplified - you can implement more complex logic)
    let shipping_cost = if total_amount >= BigDecimal::from(500000) { BigDecimal::zero() } else { BigDecimal::from(15000) };
    let final_amount = &total_amount + &shipping_cost - &discount_amount;

    // Generate order number
    let order_number = format!("BK-{}", Uuid::new_v4().to_string().split('-').next().unwrap().to_uppercase());

    // Create order
    let order = match sqlx::query_as::<_, Order>(
        "INSERT INTO orders (user_id, order_number, status, total_amount, shipping_cost, discount_amount, 
         final_amount, payment_method, payment_status, shipping_address, billing_address, notes, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
         RETURNING *"
    )
    .bind(user_id)
    .bind(&order_number)
    .bind(OrderStatus::Pending)
    .bind(total_amount)
    .bind(shipping_cost)
    .bind(if discount_amount > BigDecimal::zero() { Some(&discount_amount) } else { None })
    .bind(final_amount)
    .bind(&order_data.payment_method)
    .bind(PaymentStatus::Pending)
    .bind(serde_json::to_value(&order_data.shipping_address).unwrap())
    .bind(serde_json::to_value(&order_data.billing_address).unwrap())
    .bind(&order_data.notes)
    .fetch_one(&mut *tx)
    .await {
        Ok(order) => order,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to create order"));
        }
    };

    // Create order items and update stock
    let mut created_items = Vec::new();
    for (item_data, product, item_total) in order_items {
        // Create order item
        let order_item = match sqlx::query_as::<_, OrderItem>(
            "INSERT INTO order_items (order_id, product_id, product_name, product_image, quantity, 
             size, color, price_at_time, total_price, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
             RETURNING *"
        )
        .bind(order.id)
        .bind(item_data.product_id)
        .bind(&product.name)
        .bind(&product.image_url)
        .bind(item_data.quantity)
        .bind(&item_data.size)
        .bind(&item_data.color)
        .bind(product.price)
        .bind(item_total)
        .fetch_one(&mut *tx)
        .await {
            Ok(item) => item,
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Failed to create order item"));
            }
        };

        // Update product stock
        if let Err(e) = sqlx::query!(
            "UPDATE products SET stock = stock - $1, sold_count = sold_count + $2 WHERE id = $3",
            item_data.quantity,
            item_data.quantity,
            item_data.product_id
        )
        .execute(&mut *tx)
        .await {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to update stock"));
        }

        created_items.push(order_item);
    }

    // Clear user's cart
    if let Err(e) = sqlx::query!(
        "DELETE FROM cart_items ci USING carts c WHERE ci.cart_id = c.id AND c.user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await {
        eprintln!("Database error: {}", e);
        return Ok(HttpResponse::InternalServerError().json("Failed to clear cart"));
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        eprintln!("Database error: {}", e);
        return Ok(HttpResponse::InternalServerError().json("Failed to commit transaction"));
    }

    let order_with_items = OrderWithItems {
        order,
        items: created_items,
    };

    Ok(HttpResponse::Created().json(order_with_items))
}

#[get("/orders")]
async fn get_user_orders(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();

    let orders = match sqlx::query_as::<_, OrderSummary>(
        "SELECT o.id, o.order_number, o.status, o.final_amount as total_amount, 
                COUNT(oi.id) as item_count, o.created_at
         FROM orders o
         LEFT JOIN order_items oi ON o.id = oi.order_id
         WHERE o.user_id = $1
         GROUP BY o.id, o.order_number, o.status, o.final_amount, o.created_at
         ORDER BY o.created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await {
        Ok(orders) => orders,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch orders"));
        }
    };

    Ok(HttpResponse::Ok().json(orders))
}

#[get("/orders/{id}")]
async fn get_order_details(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    let user_id: i32 = claims.sub.parse().unwrap();
    let order_id = path.into_inner();

    // Get order
    let order = match sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE id = $1 AND user_id = $2"
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await {
        Ok(Some(o)) => o,
        Ok(None) => return Ok(HttpResponse::NotFound().json("Order not found")),
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    // Get order items
    let items = match sqlx::query_as::<_, OrderItem>(
        "SELECT * FROM order_items WHERE order_id = $1 ORDER BY created_at"
    )
    .bind(order_id)
    .fetch_all(pool.get_ref())
    .await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch order items"));
        }
    };

    let order_with_items = OrderWithItems { order, items };

    Ok(HttpResponse::Ok().json(order_with_items))
}

// Admin routes
#[get("/admin/orders")]
async fn get_all_orders(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }

    let orders = match sqlx::query!(
        "SELECT o.id, o.order_number, o.status, o.final_amount, o.created_at,
                u.name as user_name, u.email as user_email,
                COUNT(oi.id) as item_count
         FROM orders o
         JOIN users u ON o.user_id = u.id
         LEFT JOIN order_items oi ON o.id = oi.order_id
         GROUP BY o.id, o.order_number, o.status, o.final_amount, o.created_at, u.name, u.email
         ORDER BY o.created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await {
        Ok(orders) => orders,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Failed to fetch orders"));
        }
    };

    // Convert to serializable format
    let serializable_orders: Vec<_> = orders.into_iter().map(|order| {
        serde_json::json!({
            "id": order.id,
            "order_number": order.order_number,
            "status": order.status,
            "final_amount": order.final_amount,
            "created_at": order.created_at,
            "user_name": order.user_name,
            "user_email": order.user_email,
            "item_count": order.item_count.unwrap_or(0)
        })
    }).collect();

    Ok(HttpResponse::Ok().json(serializable_orders))
}

#[put("/admin/orders/{id}/status")]
async fn update_order_status(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    status_data: web::Json<UpdateOrderStatusRequest>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder> {
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }

    let order_id = path.into_inner();

    let mut update_query = "UPDATE orders SET status = $1, updated_at = NOW()".to_string();
    let mut bind_count = 2;

    // Add shipped_at or delivered_at based on status
    match status_data.status {
        OrderStatus::Shipped => {
            update_query.push_str(", shipped_at = NOW()");
        }
        OrderStatus::Delivered => {
            update_query.push_str(", delivered_at = NOW()");
        }
        _ => {}
    }

    if status_data.notes.is_some() {
        update_query.push_str(&format!(", notes = ${}", bind_count));
        bind_count += 1;
    }

    update_query.push_str(&format!(" WHERE id = ${} RETURNING *", bind_count));

    let mut query = sqlx::query_as::<_, Order>(&update_query)
        .bind(&status_data.status);

    if let Some(notes) = &status_data.notes {
        query = query.bind(notes);
    }

    let order = match query
        .bind(order_id)
        .fetch_optional(pool.get_ref())
        .await {
            Ok(Some(order)) => order,
            Ok(None) => return Ok(HttpResponse::NotFound().json("Order not found")),
            Err(e) => {
                eprintln!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json("Failed to update order status"));
            }
        };

    Ok(HttpResponse::Ok().json(order))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .wrap(AuthMiddleware)
            .service(checkout)
            .service(get_user_orders)
            .service(get_order_details)
    )
    .service(
        web::scope("/admin")
            .wrap(AuthMiddleware)
            .service(get_all_orders)
            .service(update_order_status)
    );
}