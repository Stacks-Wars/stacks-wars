use std::fmt;
use uuid::Uuid;

/// Fragment of a Redis key (Id, Str, or Wildcard).
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

/// Helper to build Redis keys consistently.
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

    /// Key for lobby runtime state (pattern: `lobbies:{lobby_id}:state`).
    pub fn lobby_state(lobby_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("state".to_string()),
        ])
    }

    /// Key for a player's state in a lobby (pattern: `lobbies:{lobby_id}:players:{user_id}`).
    pub fn lobby_player(lobby_id: impl Into<KeyPart>, user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("players".to_string()),
            user_id.into(),
        ])
    }

    /// Key for a spectator's state in a lobby (pattern: `lobbies:{lobby_id}:spectators:{user_id}`).
    pub fn lobby_spectator(lobby_id: impl Into<KeyPart>, user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("spectators".to_string()),
            user_id.into(),
        ])
    }

    /// Legacy key for game data (kept for hydration).
    pub fn game(game_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("games".to_string()),
            game_id.into(),
            KeyPart::Str("data".to_string()),
        ])
    }

    /// Legacy key for user data (kept for hydration).
    pub fn user(user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("users".to_string()),
            KeyPart::Str("data".to_string()),
            user_id.into(),
        ])
    }

    /// Legacy key for lobby info (kept for hydration).
    pub fn lobby(lobby_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("info".to_string()),
        ])
    }

    /// Key for lobby join requests (hash keyed by user id)
    pub fn lobby_join_requests(lobby_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("join_requests".to_string()),
        ])
    }

    /// Key for lobby countdown state
    pub fn lobby_countdown(lobby_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("countdown".to_string()),
        ])
    }

    /// Key for lobby chat messages sorted set (pattern: `lobbies:{lobby_id}:chat`).
    /// Uses Redis sorted set with timestamp as score for chronological ordering.
    pub fn lobby_chat(lobby_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("chat".to_string()),
        ])
    }

    /// Key for individual chat message data (pattern: `lobbies:{lobby_id}:chat:messages:{message_id}`).
    pub fn lobby_chat_message(
        lobby_id: impl Into<KeyPart>,
        message_id: impl Into<KeyPart>,
    ) -> String {
        Self::build(&[
            KeyPart::Str("lobbies".to_string()),
            lobby_id.into(),
            KeyPart::Str("chat".to_string()),
            KeyPart::Str("messages".to_string()),
            message_id.into(),
        ])
    }

    /// Rate limiter key for unauthenticated users by IP.
    pub fn rate_user_ip(ip: &str) -> String {
        Self::build(&[
            KeyPart::Str("rate".to_string()),
            KeyPart::Str("user".to_string()),
            KeyPart::Str("ip".to_string()),
            KeyPart::Str(ip.to_string()),
        ])
    }

    /// Rate limiter key for authenticated users (public APIs).
    pub fn rate_user_auth(user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("rate".to_string()),
            KeyPart::Str("user".to_string()),
            KeyPart::Str("auth".to_string()),
            user_id.into(),
        ])
    }

    /// Rate limiter key for strict/write operations (authenticated users).
    pub fn rate_user_strict(user_id: impl Into<KeyPart>) -> String {
        Self::build(&[
            KeyPart::Str("rate".to_string()),
            KeyPart::Str("user".to_string()),
            KeyPart::Str("strict".to_string()),
            user_id.into(),
        ])
    }
}
