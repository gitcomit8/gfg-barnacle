use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, http::header, body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

pub struct AuthGuard;

impl<S, B> Transform<S, ServiceRequest> for AuthGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthGuardMiddleware { service }))
    }
}

pub struct AuthGuardMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        
        // BUG: No route whitelisting! 
        // The /login route is also checked for authentication
        // This causes infinite redirects!
        
        let auth_cookie = req.cookie("auth_token");
        
        // Check if user is authenticated by looking for auth cookie
        // BUG: We redirect to /login even if we're already on /login!
        let is_authenticated = auth_cookie.is_some();
        
        if !is_authenticated {
            // BUG: Redirect to /login even if we're ALREADY on /login
            // This creates an infinite redirect loop!
            println!("❌ Not authenticated! Redirecting {} to /login", path);
            
            let response = HttpResponse::Found()
                .insert_header((header::LOCATION, "/login"))
                .finish();
            
            return Box::pin(async move {
                Ok(req.into_response(response).map_into_right_body())
            });
        }
        
        println!("✅ Authenticated! Allowing access to {}", path);
        
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
