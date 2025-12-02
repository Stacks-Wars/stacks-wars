use uuid::Uuid;

use crate::models::game::LobbyState;

pub struct RedisKey;

impl RedisKey {
    pub fn user(user_id: KeyPart) -> String {
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

    pub fn game(game_id: KeyPart) -> String {
        format!("games:{game_id}:data")
    }

    pub fn game_lobbies(game_id: KeyPart) -> String {
        format!("games:{game_id}:lobbies")
    }

    pub fn lobby(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:info")
    }

    pub fn lobby_player(lobby_id: KeyPart, player_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:player:{player_id}")
    }

    pub fn lobby_connected_players(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:connected_players")
    }

    pub fn lobby_spectators(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:spectators")
    }

    pub fn lobby_current_players(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:current_players")
    }

    pub fn lobbies_state(state: &LobbyState) -> String {
        format!("lobbies:{}:state", format!("{state:?}").to_lowercase())
    }

    pub fn lobbies_all() -> String {
        "lobbies:all".to_string()
    }

    pub fn lobby_chat(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:chats")
    }

    // temporary keys
    pub fn lobby_countdown(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:countdown")
    }

    pub fn lobby_used_words(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:used_words")
    }

    pub fn lobby_rule_context(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:rule_context")
    }

    pub fn lobby_rule_index(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:rule_index")
    }

    pub fn lobby_current_turn(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:current_turn")
    }

    pub fn lobby_eliminated_players(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:eliminated_players")
    }

    pub fn lobby_game_started(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:game_started")
    }

    pub fn lobby_current_rule(lobby_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:current_rule")
    }

    pub fn words_set() -> String {
        "games:word_set".to_string()
    }

    pub fn lobby_join_requests(lobby_id: KeyPart) -> String {
        format!("lobbies:{}:join_requests", lobby_id)
    }

    pub fn lobby_join_request_user(lobby_id: KeyPart, user_id: KeyPart) -> String {
        format!("lobbies:{}:join_requests:{}", lobby_id, user_id)
    }

    pub fn temp_union() -> String {
        let id = Uuid::new_v4();
        format!("temp:union:{id}")
    }

    pub fn temp_inter() -> String {
        let id = Uuid::new_v4();
        format!("temp:inter:{id}")
    }

    pub fn player_missed_msgs(lobby_id: KeyPart, player_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:missed_msgs:{player_id}")
    }

    pub fn player_missed_chat_msgs(lobby_id: KeyPart, player_id: KeyPart) -> String {
        format!("lobbies:{lobby_id}:missed_chat_msgs:{player_id}")
    }

    // Key parsing utilities
    pub fn _extract_user_id_from_user_key(key: &str) -> Option<Uuid> {
        // Parse "users:{uuid}" to extract user_id
        if let Some(user_id_str) = key.strip_prefix("users:data:") {
            Uuid::parse_str(user_id_str).ok()
        } else {
            None
        }
    }

    pub fn extract_lobby_id_from_player_key(key: &str) -> Option<Uuid> {
        // Parse "lobbies:{lobby_id}:player:{user_id}" to extract lobby_id
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() >= 2 && parts[0] == "lobbies" {
            Uuid::parse_str(parts[1]).ok()
        } else {
            None
        }
    }

    pub fn _extract_user_id_from_player_key(key: &str) -> Option<Uuid> {
        // Parse "lobbies:{lobby_id}:player:{user_id}" to extract user_id
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() >= 4 && parts[0] == "lobbies" && parts[2] == "player" {
            Uuid::parse_str(parts[3]).ok()
        } else {
            None
        }
    }

    pub fn _extract_ids_from_player_key(key: &str) -> Option<(Uuid, Uuid)> {
        // Parse "lobbies:{lobby_id}:player:{user_id}" to extract (lobby_id, user_id)
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() >= 4 && parts[0] == "lobbies" && parts[2] == "player" {
            if let (Ok(lobby_id), Ok(user_id)) =
                (Uuid::parse_str(parts[1]), Uuid::parse_str(parts[3]))
            {
                return Some((lobby_id, user_id));
            }
        }
        None
    }

    pub fn _extract_user_ids_from_connected_set(key: &str) -> Option<Uuid> {
        // Parse "lobbies:{lobby_id}:connected_players" to extract lobby_id if needed
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() >= 2 && parts[0] == "lobbies" {
            Uuid::parse_str(parts[1]).ok()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeyPart {
    Id(Uuid),
    Str(String),
    Wildcard,
}

impl From<Uuid> for KeyPart {
    fn from(id: Uuid) -> Self {
        KeyPart::Id(id)
    }
}

impl From<&str> for KeyPart {
    fn from(s: &str) -> Self {
        if s == "*" {
            KeyPart::Wildcard
        } else {
            KeyPart::Str(s.to_string())
        }
    }
}

impl std::fmt::Display for KeyPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyPart::Id(id) => write!(f, "{}", id),
            KeyPart::Str(s) => write!(f, "{}", s),
            KeyPart::Wildcard => write!(f, "*"),
        }
    }
}
