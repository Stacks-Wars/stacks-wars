use crate::models::enums::LobbyState;
use uuid::Uuid;

/// Redis key builder for consistent key naming across the application
pub struct RedisKey;

impl RedisKey {
    pub fn user(user_id: impl std::fmt::Display) -> String {
        format!("users:data:{user_id}")
    }

    pub fn users_wallets() -> String {
        "users:wallets".to_string()
    }

    pub fn users_usernames() -> String {
        "users:usernames".to_string()
    }

    pub fn users_matches() -> String {
        "users:matches".to_string()
    }

    pub fn users_wins() -> String {
        "users:wins".to_string()
    }

    pub fn users_pnl() -> String {
        "users:pnl".to_string()
    }

    pub fn users_points() -> String {
        "users:points".to_string()
    }

    pub fn game(game_id: impl std::fmt::Display) -> String {
        format!("games:{game_id}:data")
    }

    pub fn game_lobbies(game_id: impl std::fmt::Display) -> String {
        format!("games:{game_id}:lobbies")
    }

    pub fn lobby(lobby_id: impl std::fmt::Display) -> String {
        format!("lobbies:{lobby_id}:info")
    }

    pub fn lobby_player(
        lobby_id: impl std::fmt::Display,
        player_id: impl std::fmt::Display,
    ) -> String {
        format!("lobbies:{lobby_id}:player:{player_id}")
    }

    pub fn lobby_connected_players(lobby_id: impl std::fmt::Display) -> String {
        format!("lobbies:{lobby_id}:connected_players")
    }

    pub fn lobby_spectators(lobby_id: impl std::fmt::Display) -> String {
        format!("lobbies:{lobby_id}:spectators")
    }

    pub fn lobby_current_players(lobby_id: impl std::fmt::Display) -> String {
        format!("lobbies:{lobby_id}:current_players")
    }

    pub fn lobbies_state(state: &LobbyState) -> String {
        format!("lobbies:{}:state", format!("{state:?}").to_lowercase())
    }

    pub fn lobbies_all() -> String {
        "lobbies:all".to_string()
    }

    pub fn lobby_chat(lobby_id: impl std::fmt::Display) -> String {
        format!("lobbies:{lobby_id}:chats")
    }

    // Temporary keys
    pub fn lobby_countdown(lobby_id: impl std::fmt::Display) -> String {
        format!("lobbies:{lobby_id}:countdown")
    }

    pub fn user_rate_limit_auth(user_id: Uuid) -> String {
        format!("ratelimit:auth:{user_id}")
    }

    pub fn ip_rate_limit_auth(ip: &str) -> String {
        format!("ratelimit:auth:ip:{ip}")
    }
}
