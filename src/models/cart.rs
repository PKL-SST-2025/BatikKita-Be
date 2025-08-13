use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Cart {
    pub id: i32,
    pub user_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CartItem {
    pub id: i32,
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub size: Option<String>,
    pub color: Option<String>,
    pub price_at_time: i64, // harga saat ditambahkan ke cart
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CartItemWithProduct {
    pub id: i32,
    pub cart_id: i32,
    pub product_id: i32,
    pub product_name: String,
    pub product_image: String,
    pub quantity: i32,
    pub size: Option<String>,
    pub color: Option<String>,
    pub price_at_time: i64,
    pub current_price: i64,
    pub stock_available: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToCartRequest {
    pub product_id: i32,
    pub quantity: i32,
    pub size: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCartItemRequest {
    pub quantity: i32,
    pub size: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartSummary {
    pub total_items: i32,
    pub total_price: i64,
    pub items: Vec<CartItemWithProduct>,
}