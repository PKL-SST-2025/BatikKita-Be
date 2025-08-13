use actix_web::{HttpResponse, Result};

pub fn internal_error(msg: &str) -> Result<HttpResponse> {
    eprintln!("Internal error: {}", msg);
    Ok(HttpResponse::InternalServerError().json(msg))
}

pub fn bad_request(msg: &str) -> Result<HttpResponse> {
    eprintln!("Bad request: {}", msg);
    Ok(HttpResponse::BadRequest().json(msg))
}

pub fn not_found(msg: &str) -> Result<HttpResponse> {
    eprintln!("Not found: {}", msg);
    Ok(HttpResponse::NotFound().json(msg))
}

pub fn unauthorized(msg: &str) -> Result<HttpResponse> {
    eprintln!("Unauthorized: {}", msg);
    Ok(HttpResponse::Unauthorized().json(msg))
}
