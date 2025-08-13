use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger, Result};
use actix_cors::Cors;
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Global cart storage (in-memory for simplicity)
type CartStorage = Arc<Mutex<HashMap<String, Vec<CartItem>>>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CartItem {
    id: i32,
    product_id: i32,
    quantity: i32,
    product: CartProduct,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CartProduct {
    id: i32,
    name: String,
    price: i64,
    discount_price: Option<i64>,
    image_url: String,
    stock_quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddToCartRequest {
    product_id: i32,
    quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateCartRequest {
    quantity: i32,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
    first_name: String,
    last_name: String,
    phone: Option<String>,
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "status": "ok",
            "version": "1.0.0"
        },
        "message": "Batik Shop API is running"
    })))
}

async fn get_products() -> Result<HttpResponse> {
    let products_data = vec![
        json!({
            "id": 1,
            "name": "Batik Mega Mendung",
            "description": "Traditional Indonesian batik with cloud pattern from Cirebon. This beautiful piece represents the rich cultural heritage of Cirebon with its distinctive cloud motifs.",
            "short_description": "Traditional cloud pattern batik from Cirebon",
            "price": 150000,
            "discount_price": 120000,
            "sku": "BTK-MM-001",
            "stock_quantity": 10,
            "category": "Traditional",
            "brand": "Batik Kita",
            "weight": 200,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 1,
                    "product_id": 1,
                    "image_url": "/images/batik-1.jpg",
                    "alt_text": "Batik Mega Mendung",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 1,
                    "product_id": 1,
                    "feature_name": "Material",
                    "feature_value": "100% Cotton"
                },
                {
                    "id": 2,
                    "product_id": 1,
                    "feature_name": "Origin",
                    "feature_value": "Cirebon, West Java"
                }
            ],
            "reviews": [],
            "average_rating": 4.5,
            "review_count": 12
        }),
        json!({
            "id": 2,
            "name": "Batik Parang",
            "description": "Classic diagonal pattern batik from Central Java. The Parang pattern is one of the oldest and most revered batik designs in Indonesian culture.",
            "short_description": "Classic diagonal pattern batik from Central Java",
            "price": 200000,
            "discount_price": 160000,
            "sku": "BTK-PR-001",
            "stock_quantity": 5,
            "category": "Classic",
            "brand": "Batik Kita",
            "weight": 200,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 2,
                    "product_id": 2,
                    "image_url": "/images/batik-2.jpg",
                    "alt_text": "Batik Parang",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 3,
                    "product_id": 2,
                    "feature_name": "Material",
                    "feature_value": "100% Silk"
                },
                {
                    "id": 4,
                    "product_id": 2,
                    "feature_name": "Origin",
                    "feature_value": "Solo, Central Java"
                }
            ],
            "reviews": [],
            "average_rating": 4.8,
            "review_count": 8
        }),
        json!({
            "id": 3,
            "name": "Batik Kawung",
            "description": "Kawung is one of the oldest batik patterns, traditionally worn by the royal family of Yogyakarta. The circular pattern represents hope and prosperity.",
            "short_description": "Royal pattern from Yogyakarta Sultan Palace",
            "price": 175000,
            "discount_price": 140000,
            "sku": "BTK-KW-001",
            "stock_quantity": 8,
            "category": "Traditional",
            "brand": "Batik Kita",
            "weight": 180,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 3,
                    "product_id": 3,
                    "image_url": "/images/batik-3.jpg",
                    "alt_text": "Batik Kawung",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 5,
                    "product_id": 3,
                    "feature_name": "Material",
                    "feature_value": "Premium Cotton"
                },
                {
                    "id": 6,
                    "product_id": 3,
                    "feature_name": "Origin",
                    "feature_value": "Yogyakarta"
                }
            ],
            "reviews": [],
            "average_rating": 4.7,
            "review_count": 15
        }),
        json!({
            "id": 4,
            "name": "Batik Sekar Jagad",
            "description": "Sekar Jagad means 'flower of the world', representing the beauty and diversity of nature. This sophisticated pattern showcases the finest batik craftsmanship.",
            "short_description": "Flower of the world pattern with intricate details",
            "price": 225000,
            "discount_price": 180000,
            "sku": "BTK-SJ-001",
            "stock_quantity": 6,
            "category": "Premium",
            "brand": "Batik Kita",
            "weight": 250,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 4,
                    "product_id": 4,
                    "image_url": "/images/batik-4.jpg",
                    "alt_text": "Batik Sekar Jagad",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 7,
                    "product_id": 4,
                    "feature_name": "Material",
                    "feature_value": "Pure Silk"
                },
                {
                    "id": 8,
                    "product_id": 4,
                    "feature_name": "Technique",
                    "feature_value": "Hand-drawn Tulis"
                }
            ],
            "reviews": [],
            "average_rating": 4.9,
            "review_count": 20
        }),
        json!({
            "id": 5,
            "name": "Batik Truntum",
            "description": "Truntum pattern symbolizes blossoming love and eternal affection. This modern interpretation maintains traditional values with contemporary aesthetics.",
            "short_description": "Modern floral pattern with contemporary style",
            "price": 135000,
            "discount_price": 108000,
            "sku": "BTK-TR-001",
            "stock_quantity": 15,
            "category": "Modern",
            "brand": "Batik Kita",
            "weight": 170,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": false,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 5,
                    "product_id": 5,
                    "image_url": "/images/batik-5.jpg",
                    "alt_text": "Batik Truntum",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 9,
                    "product_id": 5,
                    "feature_name": "Material",
                    "feature_value": "Cotton Blend"
                },
                {
                    "id": 10,
                    "product_id": 5,
                    "feature_name": "Care",
                    "feature_value": "Machine Washable"
                }
            ],
            "reviews": [],
            "average_rating": 4.4,
            "review_count": 18
        }),
        json!({
            "id": 6,
            "name": "Batik Sogan Klasik",
            "description": "Sogan is the classic brown color of traditional Solo batik. This timeless piece represents the authentic heritage of Central Javanese batik artistry.",
            "short_description": "Traditional brown batik with authentic Solo heritage",
            "price": 190000,
            "discount_price": 152000,
            "sku": "BTK-SG-001",
            "stock_quantity": 7,
            "category": "Classic",
            "brand": "Batik Kita",
            "weight": 200,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 6,
                    "product_id": 6,
                    "image_url": "/images/batik-6.jpg",
                    "alt_text": "Batik Sogan Klasik",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 11,
                    "product_id": 6,
                    "feature_name": "Color",
                    "feature_value": "Natural Sogan"
                },
                {
                    "id": 12,
                    "product_id": 6,
                    "feature_name": "Origin",
                    "feature_value": "Solo, Central Java"
                }
            ],
            "reviews": [],
            "average_rating": 4.6,
            "review_count": 11
        }),
        json!({
            "id": 7,
            "name": "Batik Pekalongan Pesisir",
            "description": "Pekalongan batik is famous for its bright colors and coastal influences. This piece captures the spirit of Indonesia's maritime culture with beautiful ocean-inspired patterns.",
            "short_description": "Coastal batik with vibrant colors and maritime motifs",
            "price": 165000,
            "discount_price": 132000,
            "sku": "BTK-PK-001",
            "stock_quantity": 12,
            "category": "Traditional",
            "brand": "Batik Kita",
            "weight": 190,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": false,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 7,
                    "product_id": 7,
                    "image_url": "/images/batik-7.jpg",
                    "alt_text": "Batik Pekalongan Pesisir",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 13,
                    "product_id": 7,
                    "feature_name": "Style",
                    "feature_value": "Coastal Pesisir"
                },
                {
                    "id": 14,
                    "product_id": 7,
                    "feature_name": "Origin",
                    "feature_value": "Pekalongan, Central Java"
                }
            ],
            "reviews": [],
            "average_rating": 4.3,
            "review_count": 9
        }),
        json!({
            "id": 8,
            "name": "Batik Indigo Modern",
            "description": "Modern interpretation of traditional indigo dyeing techniques. This contemporary piece features geometric patterns that blend traditional craftsmanship with modern design sensibilities.",
            "short_description": "Contemporary indigo batik with geometric patterns",
            "price": 145000,
            "discount_price": 116000,
            "sku": "BTK-IM-001",
            "stock_quantity": 20,
            "category": "Modern",
            "brand": "Batik Kita",
            "weight": 160,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 8,
                    "product_id": 8,
                    "image_url": "/images/batik-8.jpg",
                    "alt_text": "Batik Indigo Modern",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 15,
                    "product_id": 8,
                    "feature_name": "Dye",
                    "feature_value": "Natural Indigo"
                },
                {
                    "id": 16,
                    "product_id": 8,
                    "feature_name": "Pattern",
                    "feature_value": "Geometric Modern"
                }
            ],
            "reviews": [],
            "average_rating": 4.2,
            "review_count": 14
        })
    ];

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": products_data,
        "pagination": {
            "page": 1,
            "per_page": 10,
            "total": 2,
            "total_pages": 1,
            "has_next": false,
            "has_prev": false
        },
        "message": "Products retrieved successfully"
    })))
}

async fn login(login_data: web::Json<LoginRequest>) -> Result<HttpResponse> {
    // Simple mock authentication
    if login_data.email == "admin@batik.com" && login_data.password == "admin123" {
        Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "token": "mock_jwt_token_12345",
                "user": {
                    "id": 1,
                    "username": "admin",
                    "email": "admin@batik.com",
                    "first_name": "Admin",
                    "last_name": "Batik",
                    "phone": "081234567890",
                    "role": "admin",
                    "is_active": true,
                    "email_verified": true,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            },
            "message": "Login successful"
        })))
    } else if login_data.email == "user@batik.com" && login_data.password == "user123" {
        Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "token": "mock_jwt_token_67890",
                "user": {
                    "id": 2,
                    "username": "user",
                    "email": "user@batik.com",
                    "first_name": "User",
                    "last_name": "Batik",
                    "phone": "081234567891",
                    "role": "customer",
                    "is_active": true,
                    "email_verified": true,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            },
            "message": "Login successful"
        })))
    } else {
        Ok(HttpResponse::Unauthorized().json(json!({
            "success": false,
            "message": "Invalid email or password",
            "error_code": "INVALID_CREDENTIALS"
        })))
    }
}

async fn register(register_data: web::Json<RegisterRequest>) -> Result<HttpResponse> {
    // Simple mock registration
    Ok(HttpResponse::Created().json(json!({
        "success": true,
        "data": {
            "token": "mock_jwt_token_new_user",
            "user": {
                "id": 3,
                "username": register_data.username,
                "email": register_data.email,
                "first_name": register_data.first_name,
                "last_name": register_data.last_name,
                "phone": register_data.phone,
                "role": "customer",
                "is_active": true,
                "email_verified": false,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        },
        "message": "Registration successful"
    })))
}

// Cart functions
async fn get_cart(cart_storage: web::Data<CartStorage>) -> Result<HttpResponse> {
    // For simplicity, use a fixed user ID (in real app, get from JWT token)
    let user_id = "user_1".to_string();
    
    let carts = cart_storage.lock().unwrap();
    let user_cart = carts.get(&user_id).cloned().unwrap_or_default();
    
    let total_items: i32 = user_cart.iter().map(|item| item.quantity).sum();
    let subtotal: i64 = user_cart.iter().map(|item| {
        let price = item.product.discount_price.unwrap_or(item.product.price);
        price * item.quantity as i64
    }).sum();
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "items": user_cart,
            "total_items": total_items,
            "subtotal": subtotal,
            "total_discount": 0,
            "total_amount": subtotal
        },
        "message": "Cart retrieved successfully"
    })))
}

async fn add_to_cart(
    cart_storage: web::Data<CartStorage>,
    add_request: web::Json<AddToCartRequest>
) -> Result<HttpResponse> {
    let user_id = "user_1".to_string();
    
    // Mock product data lookup
    let products_map = get_products_map();
    let product = match products_map.get(&add_request.product_id) {
        Some(p) => p,
        None => return Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "message": "Product not found"
        })))
    };
    
    let mut carts = cart_storage.lock().unwrap();
    let user_cart = carts.entry(user_id).or_insert_with(Vec::new);
    
    // Check if item already exists
    if let Some(existing_item) = user_cart.iter_mut().find(|item| item.product_id == add_request.product_id) {
        existing_item.quantity += add_request.quantity;
    } else {
        let new_cart_item = CartItem {
            id: user_cart.len() as i32 + 1,
            product_id: add_request.product_id,
            quantity: add_request.quantity,
            product: product.clone(),
        };
        user_cart.push(new_cart_item);
    }
    
    Ok(HttpResponse::Created().json(json!({
        "success": true,
        "data": {
            "message": "Item added to cart successfully"
        },
        "message": "Item added to cart"
    })))
}

async fn update_cart_item(
    cart_storage: web::Data<CartStorage>,
    path: web::Path<i32>,
    update_request: web::Json<UpdateCartRequest>
) -> Result<HttpResponse> {
    let item_id = path.into_inner();
    let user_id = "user_1".to_string();
    
    let mut carts = cart_storage.lock().unwrap();
    let user_cart = carts.entry(user_id).or_insert_with(Vec::new);
    
    if let Some(item) = user_cart.iter_mut().find(|item| item.id == item_id) {
        if update_request.quantity <= 0 {
            user_cart.retain(|item| item.id != item_id);
        } else {
            item.quantity = update_request.quantity;
        }
        Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Cart item updated successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "message": "Cart item not found"
        })))
    }
}

async fn remove_from_cart(
    cart_storage: web::Data<CartStorage>,
    path: web::Path<i32>
) -> Result<HttpResponse> {
    let item_id = path.into_inner();
    let user_id = "user_1".to_string();
    
    let mut carts = cart_storage.lock().unwrap();
    let user_cart = carts.entry(user_id).or_insert_with(Vec::new);
    
    let initial_len = user_cart.len();
    user_cart.retain(|item| item.id != item_id);
    
    if user_cart.len() < initial_len {
        Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Item removed from cart"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "success": false,
            "message": "Cart item not found"
        })))
    }
}

async fn clear_cart(cart_storage: web::Data<CartStorage>) -> Result<HttpResponse> {
    let user_id = "user_1".to_string();
    
    let mut carts = cart_storage.lock().unwrap();
    carts.insert(user_id, Vec::new());
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Cart cleared successfully"
    })))
}

// Helper function to get products as a map for cart operations
fn get_products_map() -> HashMap<i32, CartProduct> {
    let mut products = HashMap::new();
    
    products.insert(1, CartProduct {
        id: 1,
        name: "Batik Mega Mendung".to_string(),
        price: 150000,
        discount_price: Some(120000),
        image_url: "/images/batik-1.jpg".to_string(),
        stock_quantity: 10,
    });
    
    products.insert(2, CartProduct {
        id: 2,
        name: "Batik Parang".to_string(),
        price: 200000,
        discount_price: Some(160000),
        image_url: "/images/batik-2.jpg".to_string(),
        stock_quantity: 5,
    });
    
    products.insert(3, CartProduct {
        id: 3,
        name: "Batik Kawung".to_string(),
        price: 175000,
        discount_price: Some(140000),
        image_url: "/images/batik-3.jpg".to_string(),
        stock_quantity: 8,
    });
    
    products.insert(4, CartProduct {
        id: 4,
        name: "Batik Sekar Jagad".to_string(),
        price: 225000,
        discount_price: Some(180000),
        image_url: "/images/batik-4.jpg".to_string(),
        stock_quantity: 6,
    });
    
    products.insert(5, CartProduct {
        id: 5,
        name: "Batik Truntum".to_string(),
        price: 135000,
        discount_price: Some(108000),
        image_url: "/images/batik-5.jpg".to_string(),
        stock_quantity: 15,
    });
    
    products.insert(6, CartProduct {
        id: 6,
        name: "Batik Sogan Klasik".to_string(),
        price: 190000,
        discount_price: Some(152000),
        image_url: "/images/batik-6.jpg".to_string(),
        stock_quantity: 7,
    });
    
    products.insert(7, CartProduct {
        id: 7,
        name: "Batik Pekalongan Pesisir".to_string(),
        price: 165000,
        discount_price: Some(132000),
        image_url: "/images/batik-7.jpg".to_string(),
        stock_quantity: 12,
    });
    
    products.insert(8, CartProduct {
        id: 8,
        name: "Batik Indigo Modern".to_string(),
        price: 145000,
        discount_price: Some(116000),
        image_url: "/images/batik-8.jpg".to_string(),
        stock_quantity: 20,
    });
    
    products
}

async fn get_product_by_id(path: web::Path<i32>) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    
    // Return first product as example
    let product = json!({
        "success": true,
        "data": {
            "id": product_id,
            "name": "Batik Mega Mendung",
            "description": "Traditional Indonesian batik with cloud pattern from Cirebon. This beautiful piece represents the rich cultural heritage of Cirebon with its distinctive cloud motifs.",
            "short_description": "Traditional cloud pattern batik from Cirebon",
            "price": 150000,
            "discount_price": 120000,
            "sku": "BTK-MM-001",
            "stock_quantity": 10,
            "category": "Traditional",
            "brand": "Batik Kita",
            "weight": 200,
            "dimensions": "200cm x 110cm",
            "is_active": true,
            "is_featured": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "images": [
                {
                    "id": 1,
                    "product_id": product_id,
                    "image_url": "/images/batik-1.jpg",
                    "alt_text": "Batik Mega Mendung",
                    "is_primary": true,
                    "sort_order": 1
                }
            ],
            "features": [
                {
                    "id": 1,
                    "product_id": product_id,
                    "feature_name": "Material",
                    "feature_value": "100% Cotton"
                },
                {
                    "id": 2,
                    "product_id": product_id,
                    "feature_name": "Origin",
                    "feature_value": "Cirebon, West Java"
                }
            ],
            "reviews": [],
            "average_rating": 4.5,
            "review_count": 12
        },
        "message": "Product retrieved successfully"
    });

    Ok(HttpResponse::Ok().json(product))
}

pub async fn run_api_server() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Batik Shop API Server");
    println!("📍 Server running at: http://127.0.0.1:8080");
    println!("📱 Health check: http://127.0.0.1:8080/health");
    println!("🛍️  Products API: http://127.0.0.1:8080/api/products");
    println!("🔐 Auth API: http://127.0.0.1:8080/api/auth/login");
    println!("🛒 Cart API: http://127.0.0.1:8080/api/cart");

    // Initialize cart storage
    let cart_storage: CartStorage = Arc::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        // Get environment variables for CORS
        let frontend_url = std::env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(cart_storage.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", web::get().to(health))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health))
                    .route("/products", web::get().to(get_products))
                    .route("/products/{id}", web::get().to(get_product_by_id))
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(login))
                            .route("/register", web::post().to(register))
                    )
                    .service(
                        web::scope("/cart")
                            .route("", web::get().to(get_cart))
                            .route("", web::post().to(add_to_cart))
                            .route("", web::delete().to(clear_cart))
                            .route("/{id}", web::put().to(update_cart_item))
                            .route("/{id}", web::delete().to(remove_from_cart))
                    )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
