use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{Quota, RateLimiter, clock::DefaultClock, state::keyed::DefaultKeyedStateStore};
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;

pub type IpRateLimiter = Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>;

// Create different IP-based rate limiters for different endpoints
pub fn create_global_rate_limiter() -> IpRateLimiter {
    // Allow 1000 requests per minute per IP globally (generous for normal usage)
    let quota = Quota::per_minute(NonZeroU32::new(1000).unwrap());
    Arc::new(RateLimiter::keyed(quota))
}

pub fn create_api_rate_limiter() -> IpRateLimiter {
    // Allow 300 requests per minute per IP for API endpoints
    let quota = Quota::per_minute(NonZeroU32::new(1000).unwrap());
    Arc::new(RateLimiter::keyed(quota))
}

pub fn create_auth_rate_limiter() -> IpRateLimiter {
    // Allow 50 requests per minute per IP for auth endpoints (stricter)
    let quota = Quota::per_minute(NonZeroU32::new(300).unwrap());
    Arc::new(RateLimiter::keyed(quota))
}

// IP-based rate limiting middleware function
pub async fn rate_limit_middleware(
    rate_limiter: IpRateLimiter,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the client IP address
    let client_ip =
        if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
            addr.ip().to_string()
        } else {
            // Fallback to a default IP if we can't extract it
            "unknown".to_string()
        };

    // Check rate limit for this specific IP
    match rate_limiter.check_key(&client_ip) {
        Ok(_) => {
            // Request is within rate limit for this IP, proceed
            let response = next.run(request).await;
            Ok(response)
        }
        Err(_) => {
            // Rate limit exceeded for this IP
            tracing::warn!("Rate limit exceeded for IP: {}", client_ip);
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

// CORS configuration using multiple allowed origins from env
pub fn cors_layer() -> CorsLayer {
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect::<Vec<_>>();

    tracing::info!("CORS allowed origins: {:?}", allowed_origins);

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}
