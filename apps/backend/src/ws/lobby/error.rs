// Lobby list error types
use std::fmt;

#[derive(Debug)]
pub enum LobbyError {
    /// Failed to fetch lobbies from database
    FetchFailed(String),
    /// Internal server error with details
    Internal(String),
}

impl fmt::Display for LobbyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LobbyError::FetchFailed(s) => write!(f, "failed to fetch lobbies: {}", s),
            LobbyError::Internal(s) => write!(f, "internal error: {}", s),
        }
    }
}

impl LobbyError {
    pub fn code(&self) -> &'static str {
        match self {
            LobbyError::FetchFailed(_) => "FETCH_FAILED",
            LobbyError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl std::error::Error for LobbyError {}
