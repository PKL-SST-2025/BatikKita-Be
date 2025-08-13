use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse, Result,
    body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::utils::jwt::validate_token;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Get the Authorization header
            let auth_header = req.headers().get("Authorization");
            
            if let Some(auth_value) = auth_header {
                if let Ok(auth_str) = auth_value.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        
                        match validate_token(token) {
                            Ok(claims) => {
                                // Insert claims object into request extensions
                                req.extensions_mut().insert(claims);
                                
                                // Continue with the request
                                let res = service.call(req).await?;
                                Ok(res.map_into_left_body())
                            }
                            Err(_) => {
                                // Invalid token
                                let (request, _payload) = req.into_parts();
                                let response = HttpResponse::Unauthorized()
                                    .json(serde_json::json!({"error": "Invalid token"}));
                                
                                Ok(ServiceResponse::new(request, response).map_into_right_body())
                            }
                        }
                    } else {
                        // Invalid authorization format
                        let (request, _payload) = req.into_parts();
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({"error": "Invalid authorization format"}));
                        
                        Ok(ServiceResponse::new(request, response).map_into_right_body())
                    }
                } else {
                    // Invalid header value
                    let (request, _payload) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({"error": "Invalid authorization header"}));
                    
                    Ok(ServiceResponse::new(request, response).map_into_right_body())
                }
            } else {
                // No authorization header
                let (request, _payload) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "Missing authorization header"}));
                
                Ok(ServiceResponse::new(request, response).map_into_right_body())
            }
        })
    }
}

// Helper function to extract user_id from request extensions
pub fn get_user_id(req: &ServiceRequest) -> Option<i32> {
    req.extensions().get::<i32>().copied()
}
