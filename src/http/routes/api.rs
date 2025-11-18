//! Read-focused API routes mounted under `/api`.
//!
//! Contains public/read-only handlers: user lookups, game/lobby queries,
//! season info and token metadata. Note: ApiRateLimit is applied at the
//! application/router composition level to avoid duplicate increments.

use axum::{Router, middleware::from_fn_with_state, routing::get};

use crate::{
    http::handlers::{
        game::{get_game, list_games},
        lobby::{get_lobby, list_lobbies_by_game, list_my_lobbies},
        season::{get_current_season, list_seasons},
        token_info::{get_token_info_mainnet, get_token_info_testnet},
        user::get_user,
    },
    middleware::{ApiRateLimit, rate_limit_with_state},
    state::AppState,
};

pub fn routes(state_for_layer: AppState) -> Router<AppState> {
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
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<ApiRateLimit>,
        ))
}
