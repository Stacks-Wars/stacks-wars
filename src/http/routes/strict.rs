//! Sensitive write endpoints subject to the strict rate limiter.
//!
//! Examples: user creation. Mounted under `/api` and wrapped with StrictRateLimit.

use axum::middleware::from_fn_with_state;
use axum::{Router, routing::post};

use crate::middleware::{StrictRateLimit, rate_limit_with_state};
use crate::{http::handlers::user::create_user, state::AppState};

/// Routes that should be subject to the strict limiter.
pub fn routes(state_for_layer: AppState) -> Router<AppState> {
    Router::new()
        .route("/user", post(create_user))
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<StrictRateLimit>,
        ))
}
