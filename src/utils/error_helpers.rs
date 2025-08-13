use actix_web::{HttpResponse, Result as ActixResult};
use sqlx;

// Enhanced error handling helper
pub fn handle_db_error<T>(result: Result<T, sqlx::Error>, error_msg: &'static str) -> ActixResult<T> {
    result.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError(error_msg)
    })
}

// Helper for optional results
pub fn handle_db_error_optional<T>(result: Result<Option<T>, sqlx::Error>, error_msg: &'static str) -> ActixResult<Option<T>> {
    result.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError(error_msg)
    })
}

// Helper for database execution results
pub fn handle_db_execute(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>, error_msg: &'static str) -> ActixResult<sqlx::postgres::PgQueryResult> {
    result.map_err(|e| {
        eprintln!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError(error_msg)
    })
}
