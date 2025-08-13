// ...existing code...
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, middleware::Logger, Result};
use actix_cors::Cors;
use serde_json::json;
use actix_web_actors::ws;
use actix::prelude::*;
// ...existing code...

// WebSocket connection for real-time chat
#[derive(Debug)]
pub struct ChatWebSocket {
    pub id: String,
    pub user_id: Option<i32>,
    pub role: Option<String>,
    pub manager: ConnectionManager,
}

impl ChatWebSocket {
    pub fn new(id: String, manager: ConnectionManager) -> Self {
        Self {
            id,
            user_id: None,
            role: None,
            manager,
        }
    }
}

impl Actor for ChatWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("💬 Chat WebSocket connected (ID: {})", self.id);
        
        // Add this connection to manager
        self.manager.add_connection(
            self.id.clone(), 
            ctx.address(), 
            self.user_id, 
            self.role.clone()
        );
        
        // Send welcome message
        ctx.text(json!({
            "type": "welcome",
            "message": "Connected to chat service",
            "connection_id": self.id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string());
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("💬 Chat WebSocket disconnected (ID: {})", self.id);
        self.manager.remove_connection(&self.id);
    }
}

// Handle incoming messages from connection manager
impl Handler<WebSocketMessage> for ChatWebSocket {
    type Result = ();

    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let text_str = text.to_string();
                println!("💬 Chat WebSocket received: {}", text_str);
                
                // Parse and handle different message types
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text_str) {
                    if let Some(message_type) = parsed.get("type").and_then(|v| v.as_str()) {
                        match message_type {
                            "auth" => {
                                // Store user info
                                if let Some(user_id) = parsed.get("user_id").and_then(|v| v.as_i64()) {
                                    self.user_id = Some(user_id as i32);
                                }
                                if let Some(role) = parsed.get("role").and_then(|v| v.as_str()) {
                                    self.role = Some(role.to_string());
                                }
                                
                                // Update connection in manager with auth info
                                self.manager.add_connection(
                                    self.id.clone(),
                                    ctx.address(),
                                    self.user_id,
                                    self.role.clone()
                                );
                                
                                println!("💬 WebSocket authenticated: user_id={:?}, role={:?}", self.user_id, self.role);
                                
                                ctx.text(json!({
                                    "type": "auth_success",
                                    "user_id": self.user_id,
                                    "role": self.role,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }).to_string());
                            },
                            "user_message" => {
                                // Handle user message - broadcast to admins
                                let room_id = parsed.get("room_id").and_then(|v| v.as_i64()).unwrap_or(1);
                                let message = parsed.get("message").and_then(|v| v.as_str()).unwrap_or("");
                                
                                println!("💬 User message in room {}: {}", room_id, message);
                                
                                let message_json = json!({
                                    "type": "user_message",
                                    "room_id": room_id,
                                    "message": message,
                                    "sender_id": self.user_id.unwrap_or(1),
                                    "sender_name": "Customer",
                                    "sender_role": "user",
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }).to_string();
                                
                                // Broadcast to all admin connections
                                self.manager.broadcast_to_role("admin", message_json);
                                
                                // Send confirmation back to user
                                ctx.text(json!({
                                    "type": "message_sent",
                                    "room_id": room_id,
                                    "message": message,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }).to_string());
                            },
                            "admin_message" => {
                                // Handle admin message - broadcast to specific user or all users
                                let room_id = parsed.get("room_id").and_then(|v| v.as_i64()).unwrap_or(1);
                                let message = parsed.get("message").and_then(|v| v.as_str()).unwrap_or("");
                                let target_user_id = parsed.get("target_user_id").and_then(|v| v.as_i64());
                                
                                println!("💬 Admin message in room {}: {}", room_id, message);
                                
                                let message_json = json!({
                                    "type": "admin_message",
                                    "room_id": room_id,
                                    "message": message,
                                    "sender_id": 999,
                                    "sender_name": "Customer Support",
                                    "sender_role": "admin",
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                    "is_read": false
                                }).to_string();
                                
                                // Broadcast to specific user or all user connections
                                if let Some(target_user_id) = target_user_id {
                                    self.manager.broadcast_to_user(target_user_id as i32, message_json);
                                } else {
                                    self.manager.broadcast_to_role("customer", message_json.clone());
                                    self.manager.broadcast_to_role("user", message_json);
                                }
                                
                                // Send confirmation back to admin
                                ctx.text(json!({
                                    "type": "message_sent",
                                    "room_id": room_id,
                                    "message": message,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }).to_string());
                            },
                            "ping" => {
                                ctx.text(json!({
                                    "type": "pong",
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }).to_string());
                            },
                            _ => {
                                // Echo other messages
                                ctx.text(json!({
                                    "type": "echo",
                                    "original_type": message_type,
                                    "message": text_str,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }).to_string());
                            }
                        }
                    }
                } else {
                    // Invalid JSON
                    ctx.text(json!({
                        "type": "error",
                        "message": "Invalid JSON format",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }).to_string());
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn notification_websocket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    let resp = ws::start(NotificationWebSocket, &req, stream);
    println!("🚀 Starting Notification WebSocket connection");
    resp
}

// ...existing code...

// BatikKita Backend Server - incrementally adding features
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
// ...existing code...
    println!("🔌 WebSocket endpoints:");
    println!("   GET /ws/notifications - Real-time notifications");
// ...existing code...

    // Create shared connection manager
// ...existing code...

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(connection_manager.clone())
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
            .route("/ws/notifications", web::get().to(notification_websocket))
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
            "features": ["authentication", "user_management", "admin", "product_management", "reviews", "favorites", "cart", "checkout", "orders", "notifications", "websockets", "real_time"]
        },
        "message": "BatikKita Backend Server is running"
    })))
}
