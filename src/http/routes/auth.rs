//! # Authenticated Write Routes
//!
//! Protected endpoints requiring JWT authentication.
//!
//! ## Rate Limiting
//! - **Limit**: 300 requests per minute per IP
//!
//! ## Authentication
//! All routes require valid JWT token:
//! ```
//! Authorization: Bearer <jwt_token>
//! ```

use axum::{
    Router, middleware as axum_middleware,
    routing::{delete, patch, post},
};

use crate::{
    http::handlers::{
        game::create_game,
        lobby::{create_lobby, delete_lobby},
        //season::create_season,
        user::{create_user, update_display_name, update_profile, update_username},
    },
    middleware::{AuthRateLimit, rate_limit_middleware},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user", post(create_user))
        .route("/user/profile", patch(update_profile))
        .route("/user/username", patch(update_username))
        .route("/user/display-name", patch(update_display_name))
        .route("/game", post(create_game))
        .route("/lobby", post(create_lobby))
        .route("/lobby/{lobby_id}", delete(delete_lobby))
        //.route("/season", post(create_season))
        .layer(axum_middleware::from_fn(
            rate_limit_middleware::<AuthRateLimit>,
        ))
}
