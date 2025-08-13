use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDateTime};
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub price: BigDecimal,
    pub discount_price: Option<BigDecimal>,
    pub sku: String,
    pub stock_quantity: i32,
    pub category: String,
    pub brand: Option<String>,
    pub weight: Option<BigDecimal>,
    pub dimensions: Option<String>,
    pub is_active: bool,
    pub is_featured: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub stock: Option<i32>,
    pub image_url: Option<String>,
    pub additional_images: Option<Vec<String>>,
    pub original_price: Option<BigDecimal>,
    pub sold_count: Option<i32>,
    pub size_options: Option<Vec<String>>,
    pub color_options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub price: BigDecimal,
    pub discount_price: Option<BigDecimal>,
    pub sku: String,
    pub stock_quantity: i32,
    pub category: String,
    pub brand: Option<String>,
    pub weight: Option<BigDecimal>,
    pub dimensions: Option<String>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
    pub stock: Option<i32>,
    pub image_url: Option<String>,
    pub additional_images: Option<Vec<String>>,
    pub original_price: Option<BigDecimal>,
    pub size_options: Option<Vec<String>>,
    pub color_options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub price: Option<BigDecimal>,
    pub discount_price: Option<BigDecimal>,
    pub sku: Option<String>,
    pub stock_quantity: Option<i32>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub weight: Option<BigDecimal>,
    pub dimensions: Option<String>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
    pub stock: Option<i32>,
    pub image_url: Option<String>,
    pub additional_images: Option<Vec<String>>,
    pub original_price: Option<BigDecimal>,
    pub size_options: Option<Vec<String>>,
    pub color_options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductFilter {
    pub category: Option<String>,
    pub min_price: Option<BigDecimal>,
    pub max_price: Option<BigDecimal>,
    pub search: Option<String>,
    pub size_options: Option<Vec<String>>,
    pub color_options: Option<Vec<String>>,
    pub in_stock_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSort {
    pub field: String, // price, rating, created_at, sold_count
    pub direction: String, // asc, desc
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: i32,
    pub product_id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReviewRequest {
    pub product_id: i32,
    pub rating: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Favorite {
    pub id: i32,
    pub user_id: i32,
    pub product_id: i32,
    pub created_at: NaiveDateTime,
}