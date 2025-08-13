use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: i32,
    pub user_id: i32,
    pub order_number: String,
    pub status: OrderStatus,
    pub total_amount: i64,
    pub shipping_cost: i64,
    pub discount_amount: Option<i64>,
    pub final_amount: i64,
    pub payment_method: String,
    pub payment_status: PaymentStatus,
    pub shipping_address: serde_json::Value,
    pub billing_address: serde_json::Value,
    pub notes: Option<String>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub product_name: String,
    pub product_image: String,
    pub quantity: i32,
    pub size: Option<String>,
    pub color: Option<String>,
    pub price_at_time: i64,
    pub total_price: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Refunded,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub items: Vec<OrderItemRequest>,
    pub shipping_address: AddressRequest,
    pub billing_address: Option<AddressRequest>,
    pub payment_method: String,
    pub notes: Option<String>,
    pub coupon_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemRequest {
    pub product_id: i32,
    pub quantity: i32,
    pub size: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressRequest {
    pub full_name: String,
    pub phone: String,
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderSummary {
    pub id: i32,
    pub order_number: String,
    pub status: OrderStatus,
    pub total_amount: i64,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Coupon {
    pub id: i32,
    pub code: String,
    pub discount_type: String, // percentage, fixed
    pub discount_value: i64,
    pub min_order_amount: Option<i64>,
    pub max_discount_amount: Option<i64>,
    pub usage_limit: Option<i32>,
    pub used_count: i32,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}