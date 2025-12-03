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
use uuid::Uuid;

/// Redis-backed, type-safe rate limiting middleware.
///
/// Marker types select the policy. Behavior summary:
/// - ApiRateLimit: unauthenticated => 60/min by IP; authenticated => 300/min by user
/// - AuthRateLimit / StrictRateLimit: strict write routes => 30/min per user
///
/// Adds X-RateLimit-Limit, X-RateLimit-Remaining and X-RateLimit-Reset headers.
/// On Redis errors the middleware fails open (allows the request).
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

    // Keep only concise diagnostics to avoid noisy logs in high-volume tests.
    // Lower-frequency builds can enable `trace` to see these internal details.
    tracing::trace!("rate_limit: extensions.len={}", request.extensions().len());

    // try to obtain AppState from extensions (set by Router::with_state)
    // Try to read the shared AppState from request extensions. Depending on how the
    // router/service was constructed the state may be stored as `axum::extract::State<AppState>`
    // or left as the bare `AppState` in extensions — try both.
    // Try a few different ways the state might be stored in extensions. Different
    // versions/compositions of axum/tower can end up storing the state as
    // `axum::extract::State<T>`, `axum::Extension<T>`, or the bare `T`.
    let app_state_opt: Option<AppState> =
        if let Some(s) = request.extensions().get::<axum::extract::State<AppState>>() {
            Some(s.0.clone())
        } else if let Some(s) = request.extensions().get::<axum::Extension<AppState>>() {
            Some(s.0.clone())
        } else if let Some(s) = request.extensions().get::<AppState>() {
            Some(s.clone())
        } else {
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

    tracing::debug!(
        "rate_limit: policy={} selected key={} limit={} client_ip={}",
        T::name(),
        key,
        limit,
        client_ip
    );

    if let Some(state) = app_state_opt {
        tracing::trace!("rate_limit: AppState found, using redis for counting");
        match state.redis.get().await {
            Ok(mut conn) => {
                // INCR and set EXPIRE to 60s when count == 1
                let count_res: redis::RedisResult<i64> = conn.incr(&key, 1).await;
                match count_res {
                    Ok(count) => {
                        maybe_count = Some(count);
                        if count == 1 {
                            // best-effort: set expire, warn on error but don't block the request
                            let expire_res: redis::RedisResult<bool> = conn.expire(&key, 60).await;
                            match expire_res {
                                Ok(_) => tracing::trace!("rate_limit: set expire for key={}", key),
                                Err(e) => tracing::warn!(
                                    "rate_limit: expire set error for key {}: {}",
                                    key,
                                    e
                                ),
                            }
                        }
                        // attempt to read TTL for reset header
                        match conn.ttl(&key).await {
                            Ok(ttl) => {
                                maybe_ttl = Some(ttl);
                            }
                            Err(e) => tracing::warn!("rate_limit: ttl read error: {}", e),
                        }

                        if count as usize > limit {
                            tracing::warn!("rate limit exceeded key={} ip={}", key, client_ip);

                            // Build a 429 response but still include the rate limit
                            // headers so clients can see the limits and reset time.
                            use axum::http::header::{HeaderName, HeaderValue};
                            use axum::response::IntoResponse;

                            let limit_val = limit.to_string();
                            let remaining = 0usize;
                            let reset_secs = maybe_ttl.unwrap_or(60).max(0) as i64;

                            let mut resp = StatusCode::TOO_MANY_REQUESTS.into_response();

                            resp.headers_mut().insert(
                                HeaderName::from_static("x-ratelimit-limit"),
                                HeaderValue::from_str(&limit_val)
                                    .unwrap_or_else(|_| HeaderValue::from_static("")),
                            );

                            resp.headers_mut().insert(
                                HeaderName::from_static("x-ratelimit-remaining"),
                                HeaderValue::from_str(&remaining.to_string())
                                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
                            );

                            resp.headers_mut().insert(
                                HeaderName::from_static("x-ratelimit-reset"),
                                HeaderValue::from_str(&reset_secs.to_string())
                                    .unwrap_or_else(|_| HeaderValue::from_static("60")),
                            );

                            return Ok(resp);
                        } else if count as usize + 1 >= limit {
                            // approaching limit
                            tracing::trace!(
                                "rate_limit: client approaching limit key={} count={} limit={}",
                                key,
                                count,
                                limit
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("rate_limit: redis incr error: {}", e);
                        // allow request on redis error (fail-open)
                    }
                }
            }
            Err(e) => tracing::warn!(
                "rate_limit: could not get redis connection for rate limiter: {}",
                e
            ),
        }
    } else {
        tracing::debug!("rate_limit: no AppState found in request extensions (skipping redis)");
    }

    // Run request and attach headers with rate info when available.
    let mut response = next.run(request).await;

    // Always attach rate limit headers (use defaults when Redis was unavailable).
    use axum::http::header::{HeaderName, HeaderValue};

    let limit_val = limit.to_string();
    let (remaining_val, reset_val) = if let Some(count) = maybe_count {
        let remaining = if count as usize >= limit {
            0
        } else {
            limit - count as usize
        };
        let reset_secs = maybe_ttl.unwrap_or(60).max(0) as i64;
        (remaining.to_string(), reset_secs.to_string())
    } else {
        // Redis unavailable or not used; provide conservative defaults
        (limit.to_string(), "60".to_string())
    };

    response.headers_mut().insert(
        HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from_str(&limit_val).unwrap_or_else(|_| HeaderValue::from_static("")),
    );

    response.headers_mut().insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from_str(&remaining_val).unwrap_or_else(|_| HeaderValue::from_static("0")),
    );

    response.headers_mut().insert(
        HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from_str(&reset_val).unwrap_or_else(|_| HeaderValue::from_static("60")),
    );

    Ok(response)
}

/// Adapter middleware that receives `State<AppState>` from axum's
/// `from_fn_with_state` helper, injects the state into the request
/// extensions (as `axum::Extension<AppState>`) so the existing
/// `rate_limit_middleware` can find it, then delegates to it.
pub async fn rate_limit_with_state<T: RateLimitConfig>(
    state: axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Insert the state into request extensions under axum::Extension so the
    // existing middleware lookup will discover it regardless of how the
    // rest of the stack expects it.
    request
        .extensions_mut()
        .insert(axum::Extension(state.0.clone()));

    // Delegate to the main middleware logic
    rate_limit_middleware::<T>(request, next).await
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

/// Programmatic rate-limit check that can be called from non-middleware paths
/// (for example, before performing a WebSocket upgrade). It applies the
/// same counting rules as `rate_limit_middleware` and returns an Err with
/// `(StatusCode, String)` when the limit is exceeded or on fatal Redis errors.
pub async fn check_rate_limit<T: RateLimitConfig>(
    state: &AppState,
    client_ip: &str,
    user_id_opt: Option<Uuid>,
) -> Result<(), (StatusCode, String)> {
    // determine key and limit
    let (key, limit) = match T::name() {
        "API" => {
            if let Some(user_id) = user_id_opt {
                (RedisKey::rate_user_auth(user_id), 300)
            } else {
                (RedisKey::rate_user_ip(client_ip), 60)
            }
        }
        "Auth" | "Strict" => {
            if let Some(user_id) = user_id_opt {
                (RedisKey::rate_user_strict(user_id), 30)
            } else {
                (RedisKey::rate_user_ip(client_ip), 30)
            }
        }
        _ => (RedisKey::rate_user_ip(client_ip), 60),
    };

    match state.redis.get().await {
        Ok(mut conn) => {
            // INCR and set EXPIRE to 60s when count == 1
            let count_res: redis::RedisResult<i64> = conn.incr(&key, 1).await;
            match count_res {
                Ok(count) => {
                    if count == 1 {
                        let _: redis::RedisResult<bool> = conn.expire(&key, 60).await;
                    }

                    if count as usize > limit {
                        return Err((
                            StatusCode::TOO_MANY_REQUESTS,
                            "rate limit exceeded".to_string(),
                        ));
                    }

                    Ok(())
                }
                Err(e) => {
                    tracing::error!("rate_limit: redis incr error: {}", e);
                    // fail-open: allow request when Redis has transient errors
                    Ok(())
                }
            }
        }
        Err(e) => {
            tracing::warn!("rate_limit: could not get redis connection: {}", e);
            // fail-open on missing Redis
            Ok(())
        }
    }
}
