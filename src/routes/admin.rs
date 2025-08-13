use actix_web::{get, HttpResponse, Responder, web};
use serde_json::json;
use crate::middleware::AdminAuth;

#[get("/dashboard")]
pub async fn dashboard() -> impl Responder {
    HttpResponse::Ok().json(json!({ "message": "Welcome, Admin" }))
}

pub fn admin_scope(cfg: &mut web::ServiceConfig) {
    cfg.service(dashboard);
}