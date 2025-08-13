use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use chrono::{Utc, Duration};
use crate::models::user::Claims;
use std::env;

pub fn create_jwt(user_id: i32, role: String) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    let key = EncodingKey::from_secret(secret.as_bytes());
    
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        role,
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &key)
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
}

pub fn extract_role_from_token(token: &str) -> Option<String> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    let decoded = decode::<Claims>(token, &key, &validation).ok()?;
    Some(decoded.claims.role)
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}