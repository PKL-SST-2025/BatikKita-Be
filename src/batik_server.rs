use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, middleware::Logger, Result};
use actix_cors::Cors;
use serde_json::json;
use actix_web_actors::ws;
use actix::prelude::*;
use sqlx::PgPool;

pub async fn run_batik_server() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Setup database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/batik_kita".to_string());
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("🚀 Starting BatikKita Backend Server");
    println!("📍 Server running at: http://localhost:8080");
    println!("🔐 Authentication endpoints:");
    println!("   POST /api/auth/register - User registration");
    println!("   POST /api/auth/login - User login");
    println!("   GET /api/auth/user/profile - Get user profile");
    println!("   PUT /api/auth/user/profile - Update user profile");
    println!("   GET /api/admin/dashboard - Admin dashboard");
    println!("📦 Product endpoints:");
    println!("   GET /api/products - Get all products");
    println!("   GET /api/products/{{id}} - Get product by ID");
    println!("   POST /api/products/{{id}}/reviews - Create product review");
    println!("   GET /api/products/{{id}}/reviews - Get product reviews");
    println!("   POST /api/admin/products - Create product (Admin)");
    println!("   PUT /api/admin/products/{{id}} - Update product (Admin)");
    println!("   DELETE /api/admin/products/{{id}} - Delete product (Admin)");
    println!("❤️ Favorite endpoints:");
    println!("   GET /api/auth/favorites - Get user favorites");
    println!("   POST /api/auth/favorites/{{id}} - Add to favorites");
    println!("   DELETE /api/auth/favorites/{{id}} - Remove from favorites");
    println!("   GET /api/auth/favorites/check/{{id}} - Check favorite status");
    println!("   DELETE /api/auth/favorites/clear - Clear all favorites");
    println!("🛒 Cart endpoints:");
    println!("   GET /api/auth/cart - Get user cart");
    println!("   POST /api/auth/cart/items - Add item to cart");
    println!("   PUT /api/auth/cart/items/{{id}} - Update cart item");
    println!("   DELETE /api/auth/cart/items/{{id}} - Remove from cart");
    println!("   DELETE /api/auth/cart/clear - Clear cart");
    println!("🛍️ Checkout & Order endpoints:");
    println!("   POST /api/auth/checkout - Create order from cart");
    println!("   GET /api/auth/orders - Get user orders");
    println!("   GET /api/auth/orders/{{id}} - Get order details");
    println!("   GET /api/admin/orders - Get all orders (Admin)");
    println!("   PUT /api/admin/orders/{{id}}/status - Update order status (Admin)");
    println!("🔔 Notification endpoints:");
    println!("   GET /api/auth/notifications - Get user notifications");
    println!("   GET /api/auth/notifications/stats - Get notification stats");
    println!("   POST /api/auth/notifications - Create notification");
    println!("   PUT /api/auth/notifications/{{id}} - Update notification");
    println!("   PUT /api/auth/notifications/bulk - Mark multiple notifications");
    println!("   DELETE /api/auth/notifications/{{id}} - Delete notification");
    println!("   PUT /api/auth/notifications/mark-all-read - Mark all as read");
    println!("   GET /api/auth/notifications/preferences - Get preferences");
    println!("   PUT /api/auth/notifications/preferences - Update preferences");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", web::get().to(health))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api")
                    // Public auth routes (login, register)
                    .service(
                        web::scope("/auth")
                            .service(crate::routes::auth::register)
                            .service(crate::routes::auth::login)
                    )
                    // Protected user routes  
                    .service(
                        web::scope("/auth")
                            .wrap(crate::middleware::AuthMiddleware)
                            .configure(crate::routes::user::configure)
                            .service(crate::routes::cart::get_cart)
                            .service(crate::routes::cart::add_to_cart)
                            .service(crate::routes::cart::update_cart_item)
                            .service(crate::routes::cart::remove_from_cart)
                            .service(crate::routes::cart::clear_cart)
                            .service(crate::routes::cart::get_guest_cart)
                            .configure(crate::routes::checkout::init)
                    )
                    // Admin routes
                    .service(
                        web::scope("/admin")
                            .wrap(crate::middleware::AdminAuth)
                            .configure(crate::routes::admin::admin_scope)
                            .wrap(crate::middleware::AuthMiddleware)
                    )
                    // Public product routes
                    .configure(crate::routes::product::init)
                    // Protected routes that frontend calls without /auth prefix
                    .service(
                        web::scope("")
                            .wrap(crate::middleware::AuthMiddleware)
                            .configure(crate::routes::favorite::init)
                            .configure(crate::routes::notification::init)
                    )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "status": "ok",
            "version": "1.0.0",
            "features": ["authentication", "user_management", "admin", "product_management", "reviews", "favorites", "cart", "checkout", "orders", "notifications", "real_time"]
        },
        "message": "BatikKita Backend Server is running"
    })))
}
