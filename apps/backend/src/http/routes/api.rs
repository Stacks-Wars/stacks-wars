// Read-focused API routes mounted under `/api` (public/read-only)

use axum::{Router, middleware::from_fn_with_state, routing::get};

use crate::{
    http::handlers::{
        contract::{get_contract, get_sponsored_contract},
        game::{get_game, get_game_by_path, get_games_by_creator, list_games},
        lobby::{
            get_all_lobbies, get_lobby, get_lobby_by_path, list_lobbies_by_game, list_my_lobbies,
        },
        platform_rating::{get_rating, list_ratings},
        season::{get_current_season, list_seasons},
        stacks::{get_balance, get_token_info},
        user::get_user,
    },
    middleware::{ApiRateLimit, rate_limit_with_state},
    state::AppState,
};

pub fn routes(state_for_layer: AppState) -> Router<AppState> {
    Router::new()
        .route("/user/{user_id}", get(get_user))
        .route("/platform-rating", get(list_ratings))
        .route("/platform-rating/{user_id}", get(get_rating))
        .route("/games", get(list_games))
        .route("/game/{game_id}", get(get_game))
        .route("/game/by-path/{path}", get(get_game_by_path))
        .route("/game/by-creator/{creator_id}", get(get_games_by_creator))
        .route("/game/{game_id}/lobbies", get(list_lobbies_by_game))
        .route("/lobbies", get(get_all_lobbies))
        .route("/lobby/{lobby_id}", get(get_lobby))
        .route("/lobby/by-path/{path}", get(get_lobby_by_path))
        .route("/lobby/my", get(list_my_lobbies))
        .route("/season/current", get(get_current_season))
        .route("/season", get(list_seasons))
        .route("/token/{contract_address}", get(get_token_info))
        .route("/contract", get(get_contract))
        .route("/sponsored-contract", get(get_sponsored_contract))
        .route("/balance/{wallet_address}", get(get_balance))
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<ApiRateLimit>,
        ))
}
