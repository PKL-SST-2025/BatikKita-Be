pub mod auth;

pub use auth::AuthMiddleware;

use actix_web::{
    dev::{ServiceRequest, ServiceResponse, Transform, forward_ready},
    Error, HttpResponse, body::{EitherBody, BoxBody}
};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use std::rc::Rc;

/// Middleware entry point
pub struct AdminAuth;

impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    S: actix_service::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = AdminAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

/// Middleware struct
pub struct AdminAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> actix_service::Service<ServiceRequest> for AdminAuthMiddleware<S>
where
    S: actix_service::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check for Authorization header with Bearer token
        let is_admin = if let Some(auth_header) = req.headers().get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..]; // Remove "Bearer " prefix
                    // Decode JWT token to check role
                    match crate::utils::jwt::verify_token(token) {
                        Ok(claims) => claims.role == "admin",
                        Err(_) => false,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            // Fallback to x-role header for backward compatibility
            req.headers()
                .get("x-role")
                .and_then(|v| v.to_str().ok())
                .map(|role| role == "admin")
                .unwrap_or(false)
        };

        let srv = self.service.clone();

        Box::pin(async move {
            if is_admin {
                let res = srv.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                let response = HttpResponse::Unauthorized()
                    .finish()
                    .map_into_right_body();
                let service_response = req.into_response(response);
                Ok(service_response)
            }
        })
    }
}