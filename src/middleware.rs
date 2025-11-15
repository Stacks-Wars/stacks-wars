use crate::models::redis::keys::RedisKey;
use crate::state::AppState;
use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::{net::SocketAddr, time::Duration};
use tower_http::cors::CorsLayer;

/// Type-safe rate limiting middleware (Redis-backed)
///
/// Policies are selected by the generic type name passed earlier. Behavior:
/// - `ApiRateLimit`: unauthenticated => 60/min per IP, authenticated => 300/min per user
/// - `AuthRateLimit` / `StrictRateLimit`: authenticated strict routes => 30/min per user
/// When Redis is unavailable, the middleware falls back to allowing the request.
pub trait RateLimitConfig {
    fn name() -> &'static str;
}

pub struct ApiRateLimit;
impl RateLimitConfig for ApiRateLimit {
    fn name() -> &'static str {
        "API"
    }
}

pub struct AuthRateLimit;
impl RateLimitConfig for AuthRateLimit {
    fn name() -> &'static str {
        "Auth"
    }
}

pub struct StrictRateLimit;
impl RateLimitConfig for StrictRateLimit {
    fn name() -> &'static str {
        "Strict"
    }
}

/// Redis-backed middleware. It reads AppState from request extensions if present.
pub async fn rate_limit_middleware<T: RateLimitConfig>(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // extract client IP
    let client_ip =
        if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
            addr.ip().to_string()
        } else {
            "unknown".to_string()
        };

    // try to obtain AppState from extensions (set by Router::with_state)
    let app_state_opt: Option<AppState> =
        if let Some(s) = request.extensions().get::<axum::extract::State<AppState>>() {
            Some(s.0.clone())
        }
        // else if let Some(s) = request.extensions().get::<AppState>() {
        //    Some(s.clone())
        //}
        else {
            None
        };

    // determine key and limit
    // prefer an `AuthClaims` instance placed in request extensions by an upstream auth extractor/middleware
    let user_id_opt = request
        .extensions()
        .get::<crate::auth::extractors::AuthClaims>()
        .and_then(|claims| claims.user_id().ok());

    let (key, limit) = match T::name() {
        "API" => {
            if let Some(user_id) = user_id_opt {
                (RedisKey::rate_user_auth(user_id), 300)
            } else {
                (RedisKey::rate_user_ip(&client_ip), 60)
            }
        }
        "Auth" | "Strict" => {
            if let Some(user_id) = user_id_opt {
                (RedisKey::rate_user_strict(user_id), 30)
            } else {
                (RedisKey::rate_user_ip(&client_ip), 30)
            }
        }
        _ => (RedisKey::rate_user_ip(&client_ip), 60),
    };

    // If we have AppState, use Redis for counting. Capture count and ttl to append headers later.
    let mut maybe_count: Option<i64> = None;
    let mut maybe_ttl: Option<i64> = None;

    if let Some(state) = app_state_opt {
        if let Ok(mut conn) = state.redis.get().await {
            // INCR and set EXPIRE to 60s when count == 1
            let count_res: redis::RedisResult<i64> = conn.incr(&key, 1).await;
            match count_res {
                Ok(count) => {
                    maybe_count = Some(count);
                    if count == 1 {
                        let _: redis::RedisResult<bool> = conn.expire(&key, 60).await;
                        // fall through to read TTL below
                    }
                    // attempt to read TTL for reset header
                    if let Ok(ttl) = conn.ttl(&key).await {
                        maybe_ttl = Some(ttl);
                    }

                    if count as usize > limit {
                        tracing::warn!("rate limit exceeded key={} ip={}", key, client_ip);
                        return Err(StatusCode::TOO_MANY_REQUESTS);
                    }
                }
                Err(e) => {
                    tracing::error!("redis incr error: {}", e);
                    // allow request on redis error
                }
            }
        } else {
            tracing::warn!("could not get redis connection for rate limiter");
        }
    }

    // Run request and attach headers with rate info when available.
    let mut response = next.run(request).await;

    if let Some(count) = maybe_count {
        use axum::http::header::{HeaderName, HeaderValue};

        let remaining = if count as usize >= limit {
            0
        } else {
            limit - count as usize
        };
        let reset_secs = maybe_ttl.unwrap_or(60).max(0) as i64;

        response.headers_mut().insert(
            HeaderName::from_static("x-ratelimit-limit"),
            HeaderValue::from_str(&limit.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("")),
        );

        response.headers_mut().insert(
            HeaderName::from_static("x-ratelimit-remaining"),
            HeaderValue::from_str(&remaining.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        );

        response.headers_mut().insert(
            HeaderName::from_static("x-ratelimit-reset"),
            HeaderValue::from_str(&reset_secs.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("60")),
        );
    }

    Ok(response)
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
