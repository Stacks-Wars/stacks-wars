//! Read-focused API routes (public).
//!
//! These endpoints are intended for read/query operations and are rate limited
//! at a moderate level (default 1000 requests/min per IP). Most routes are
//! public; a few query handlers may require authentication - check the handler
//! docs for details.

use axum::{Router, middleware as axum_middleware, routing::get};

use crate::{
    http::handlers::{
        game::{get_game, list_games},
        lobby::{get_lobby, list_lobbies_by_game, list_my_lobbies},
        season::{get_current_season, list_seasons},
        token_info::{get_token_info_mainnet, get_token_info_testnet},
        user::get_user,
    },
    middleware::{ApiRateLimit, rate_limit_middleware},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user/{user_id}", get(get_user))
        .route("/game", get(list_games))
        .route("/game/{game_id}", get(get_game))
        .route("/game/{game_id}/lobbies", get(list_lobbies_by_game))
        .route("/lobby/{lobby_id}", get(get_lobby))
        .route("/lobby/my", get(list_my_lobbies))
        .route("/season/current", get(get_current_season))
        .route("/season", get(list_seasons))
        .route("/token/{contract_address}", get(get_token_info_mainnet))
        .route(
            "/token/testnet/{contract_address}",
            get(get_token_info_testnet),
        )
        .layer(axum_middleware::from_fn(
            rate_limit_middleware::<ApiRateLimit>,
        ))
}
