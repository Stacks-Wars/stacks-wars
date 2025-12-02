use axum::{
    Router, middleware as axum_middleware,
    routing::{get, patch, post},
};

use crate::{
    http::handlers::{
        game::{create_game_handler, get_all_games_handler, get_game_handler},
        leaderboard::{get_leaderboard_handler, get_user_stat_handler},
        lobby::{
            create_lobby_handler, get_all_lobbies_extended_handler, get_all_lobbies_info_handler,
            get_lobbies_by_game_id_handler, get_lobby_extended_handler, get_lobby_info_handler,
            get_player_lobbies_handler, get_players_handler, join_lobby_handler,
            kick_player_handler, leave_lobby_handler, update_claim_state_handler,
            update_lobby_state_handler, update_player_state_handler,
        },
        token_info::{get_testnet_token_info_handler, get_token_info_handler},
        user::{
            create_user_handler, get_user_handler, update_display_name_handler,
            update_username_handler,
        },
    },
    middleware::{create_api_rate_limiter, create_auth_rate_limiter, rate_limit_middleware},
    state::AppState,
};

pub fn create_http_routes(state: AppState) -> Router {
    let api_rate_limiter = create_api_rate_limiter();
    let auth_rate_limiter = create_auth_rate_limiter();

    // Routes that need stricter rate limiting (user creation, lobby join/leave)
    let auth_routes = Router::new()
        .route("/user", post(create_user_handler))
        .route("/game", post(create_game_handler))
        .route("/lobby", post(create_lobby_handler))
        .route("/lobby/{lobby_id}/join", patch(join_lobby_handler))
        .route("/lobby/{lobby_id}/leave", patch(leave_lobby_handler))
        .route("/user/username", patch(update_username_handler))
        .route("/user/display_name", patch(update_display_name_handler))
        .route("/lobby/{lobby_id}/kick", patch(kick_player_handler))
        .route("/lobby/{lobby_id}/state", patch(update_lobby_state_handler))
        .route(
            "/lobby/{lobby_id}/player-state",
            patch(update_player_state_handler),
        )
        .route(
            "/lobby/{lobby_id}/claim-state",
            patch(update_claim_state_handler),
        )
        .layer(axum_middleware::from_fn(move |req, next| {
            rate_limit_middleware(auth_rate_limiter.clone(), req, next)
        }));

    // Regular API routes with moderate rate limiting
    let api_routes = Router::new()
        .route("/user/stat", get(get_user_stat_handler))
        .route("/user/{user_id}", get(get_user_handler))
        .route("/user/lobbies", get(get_player_lobbies_handler))
        .route("/game", get(get_all_games_handler))
        .route("/game/{game_id}", get(get_game_handler))
        .route(
            "/game/lobbies/{game_id}",
            get(get_lobbies_by_game_id_handler),
        )
        .route("/lobby", get(get_all_lobbies_info_handler))
        .route("/lobby/{lobby_id}", get(get_lobby_info_handler))
        .route("/lobby/extended", get(get_all_lobbies_extended_handler))
        .route(
            "/lobby/extended/{lobby_id}",
            get(get_lobby_extended_handler),
        )
        .route("/lobby/players/{lobby_id}", get(get_players_handler))
        .route("/leaderboard", get(get_leaderboard_handler))
        .route(
            "/token_info/{contract_address}",
            get(get_token_info_handler),
        )
        .route(
            "/token_info/testnet/{contract_address}",
            get(get_testnet_token_info_handler),
        )
        .layer(axum_middleware::from_fn(move |req, next| {
            rate_limit_middleware(api_rate_limiter.clone(), req, next)
        }));

    Router::new()
        .merge(auth_routes)
        .merge(api_routes)
        .with_state(state)
}
