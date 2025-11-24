use std::fmt;

#[derive(Debug)]
pub enum LobbyError {
    LobbyFull,
    NotCreator,
    NotAuthenticated,
    NeedAtLeast(usize),
    JoinFailed(String),
    /// Postgres metadata for the lobby is missing.
    MetadataMissing,
    /// Lobby runtime state or lobby itself was not found.
    NotFound,
    /// Client sent an invalid/uncodable message.
    InvalidMessage,
}

impl fmt::Display for LobbyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LobbyError::LobbyFull => write!(f, "lobby full"),
            LobbyError::NotCreator => write!(f, "only creator can start"),
            LobbyError::NotAuthenticated => write!(f, "authentication required"),
            LobbyError::NeedAtLeast(n) => write!(f, "need at least {} players to start", n),
            LobbyError::JoinFailed(s) => write!(f, "join failed: {}", s),
            LobbyError::MetadataMissing => write!(f, "lobby metadata missing from database"),
            LobbyError::NotFound => write!(f, "lobby not found"),
            LobbyError::InvalidMessage => write!(f, "invalid message"),
        }
    }
}

impl std::error::Error for LobbyError {}

impl LobbyError {
    pub fn code(&self) -> &'static str {
        match self {
            LobbyError::LobbyFull => "LOBBY_FULL",
            LobbyError::NotCreator => "NOT_CREATOR",
            LobbyError::NeedAtLeast(_) => "NEED_AT_LEAST",
            LobbyError::JoinFailed(_) => "JOIN_FAILED",
            LobbyError::NotAuthenticated => "NOT_AUTHENTICATED",
            LobbyError::MetadataMissing => "METADATA_MISSING",
            LobbyError::NotFound => "NOT_FOUND",
            LobbyError::InvalidMessage => "INVALID_MESSAGE",
        }
    }
}
