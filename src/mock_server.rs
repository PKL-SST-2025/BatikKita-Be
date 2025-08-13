use actix_web::{web, HttpResponse, Result, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use serde_json::json;

// Offline mock handlers for testing compilation
pub async fn mock_get_products() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "products": [],
        "message": "Mock data - database not connected"
    })))
}

pub async fn mock_get_product(path: web::Path<i32>) -> Result<HttpResponse> {
    let id = path.into_inner();
    Ok(HttpResponse::Ok().json(json!({
        "id": id,
        "name": "Mock Product",
        "message": "Mock data - database not connected"
    })))
}

pub async fn mock_auth_login() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "token": "mock_token",
        "message": "Mock auth - database not connected"
    })))
}

pub async fn mock_auth_register() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Mock registration - database not connected"
    })))
}

pub async fn mock_get_cart() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "items": [],
        "message": "Mock cart - database not connected"
    })))
}

pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "message": "Server is running in offline mode"
    })))
}

#[actix_web::main]
#[actix_web::main]
#[actix_web::main]
pub async fn run_mock_server() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Mock Batik Shop Server on http://127.0.0.1:8080");
    println!("📝 Note: Running in offline mode without database connection");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/api/products", web::get().to(mock_get_products))
            .route("/api/products/{id}", web::get().to(mock_get_product))
            .route("/api/auth/login", web::post().to(mock_auth_login))
            .route("/api/auth/register", web::post().to(mock_auth_register))
            .route("/api/cart", web::get().to(mock_get_cart))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
