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

/// Rate limiter configuration trait for type-safe middleware
pub trait RateLimitConfig {
    fn quota() -> Quota;
    fn name() -> &'static str;
}

/// API rate limiter - moderate limits for read operations
/// 1000 requests per minute per IP
pub struct ApiRateLimit;

impl RateLimitConfig for ApiRateLimit {
    fn quota() -> Quota {
        Quota::per_minute(NonZeroU32::new(1000).unwrap())
    }

    fn name() -> &'static str {
        "API"
    }
}

/// Auth rate limiter - strict limits for write operations
/// 300 requests per minute per IP
pub struct AuthRateLimit;

impl RateLimitConfig for AuthRateLimit {
    fn quota() -> Quota {
        Quota::per_minute(NonZeroU32::new(300).unwrap())
    }

    fn name() -> &'static str {
        "Auth"
    }
}

/// Strict rate limiter - very strict limits for sensitive operations
/// 50 requests per minute per IP
pub struct StrictRateLimit;

impl RateLimitConfig for StrictRateLimit {
    fn quota() -> Quota {
        Quota::per_minute(NonZeroU32::new(50).unwrap())
    }

    fn name() -> &'static str {
        "Strict"
    }
}

/// Type-safe rate limiting middleware
///
/// Usage:
/// ```rust
/// .layer(axum_middleware::from_fn(rate_limit_middleware::<ApiRateLimit>))
/// .layer(axum_middleware::from_fn(rate_limit_middleware::<AuthRateLimit>))
/// ```
pub async fn rate_limit_middleware<T: RateLimitConfig>(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Create rate limiter lazily (shared across all requests)
    use std::sync::OnceLock;
    static API_LIMITER: OnceLock<IpRateLimiter> = OnceLock::new();
    static AUTH_LIMITER: OnceLock<IpRateLimiter> = OnceLock::new();
    static STRICT_LIMITER: OnceLock<IpRateLimiter> = OnceLock::new();

    let limiter = match T::name() {
        "API" => API_LIMITER.get_or_init(|| Arc::new(RateLimiter::keyed(T::quota()))),
        "Auth" => AUTH_LIMITER.get_or_init(|| Arc::new(RateLimiter::keyed(T::quota()))),
        "Strict" => STRICT_LIMITER.get_or_init(|| Arc::new(RateLimiter::keyed(T::quota()))),
        _ => unreachable!("Unknown rate limit type"),
    };

    // Extract client IP
    let client_ip =
        if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
            addr.ip().to_string()
        } else {
            "unknown".to_string()
        };

    // Check rate limit
    match limiter.check_key(&client_ip) {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => {
            tracing::warn!("{} rate limit exceeded for IP: {}", T::name(), client_ip);
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
