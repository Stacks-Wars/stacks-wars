use std::fmt;
use uuid::Uuid;

/// Represents a fragment of a Redis key. Using `KeyPart::Wildcard` allows
/// building patterns such as `lobbies:*:players:*`.
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

impl fmt::Display for KeyPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyPart::Id(id) => write!(f, "{}", id),
            KeyPart::Str(s) => write!(f, "{}", s),
            KeyPart::Wildcard => write!(f, "*"),
        }
    }
}

/// Redis key builder for consistent key naming across the application
pub struct RedisKey;

impl RedisKey {
    /// Build a key from arbitrary parts joined by ':'
    pub fn build(parts: &[KeyPart]) -> String {
        parts
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(":")
    }

    /// Key for lobby runtime state
    /// Pattern: `lobbies:{lobby_id}:state`
    pub fn lobby_state(lobby_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("state".to_string()),
        ])
    }

    /// Key for a player's state within a lobby
    /// Pattern: `lobbies:{lobby_id}:players:{user_id}`
    pub fn lobby_player(lobby_id: impl Into<KeyPart>, user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("players".to_string()),
            user_id.into(),
        ])
    }

    /// Key for a spectator's state within a lobby
    /// Pattern: `lobbies:{lobby_id}:spectators:{user_id}`
    pub fn lobby_spectator(lobby_id: impl Into<KeyPart>, user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("spectators".to_string()),
            user_id.into(),
        ])
    }

    /// legacy key for game data, kept for hydration purposes and would be cleaned up later
    pub fn game(game_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("games".to_string()),
            game_id.into(),
            KeyPart::Str("data".to_string()),
        ])
    }

    /// legacy key for game data, kept for hydration purposes and would be cleaned up later
    pub fn user(user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("users".to_string()),
            KeyPart::Str("data".to_string()),
            user_id.into(),
        ])
    }

    /// legacy key for game data, kept for hydration purposes and would be cleaned up later
    pub fn lobby(lobby_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("info".to_string()),
        ])
    }
}
