use std::fmt;

#[derive(Debug)]
pub enum LobbyError {
    LobbyFull,
    NotCreator,
    NeedAtLeast(usize),
    JoinFailed(String),
    Generic(String),
}

impl fmt::Display for LobbyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LobbyError::LobbyFull => write!(f, "lobby full"),
            LobbyError::NotCreator => write!(f, "only creator can start"),
            LobbyError::NeedAtLeast(n) => write!(f, "need at least {} players to start", n),
            LobbyError::JoinFailed(s) => write!(f, "join failed: {}", s),
            LobbyError::Generic(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for LobbyError {}
