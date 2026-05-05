use crate::CoreError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum GameError {
    NotYourTurn,
    NotInGame,
    GameFinished,
    GameNotStarted,
    InvalidAction(String),
    AlreadyEliminated,
    InsufficientPlayers { required: usize, actual: usize },
    Internal(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::NotYourTurn => write!(f, "Not your turn"),
            GameError::NotInGame => write!(f, "You are not in this game"),
            GameError::GameFinished => write!(f, "Game has already finished"),
            GameError::GameNotStarted => write!(f, "Game has not started yet"),
            GameError::InvalidAction(msg) => write!(f, "Invalid action: {}", msg),
            GameError::AlreadyEliminated => write!(f, "You have been eliminated"),
            GameError::InsufficientPlayers { required, actual } => {
                write!(f, "Need at least {} players, got {}", required, actual)
            }
            GameError::Internal(msg) => write!(f, "Internal game error: {}", msg),
        }
    }
}

impl std::error::Error for GameError {}

impl GameError {
    pub fn code(&self) -> &'static str {
        match self {
            GameError::NotYourTurn => "NOT_YOUR_TURN",
            GameError::NotInGame => "NOT_IN_GAME",
            GameError::GameFinished => "GAME_FINISHED",
            GameError::GameNotStarted => "GAME_NOT_STARTED",
            GameError::InvalidAction(_) => "INVALID_ACTION",
            GameError::AlreadyEliminated => "ALREADY_ELIMINATED",
            GameError::InsufficientPlayers { .. } => "INSUFFICIENT_PLAYERS",
            GameError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl From<GameError> for CoreError {
    fn from(err: GameError) -> Self {
        match err {
            GameError::Internal(_) => CoreError::Internal,
            _ => CoreError::BadRequest(err.to_string()),
        }
    }
}
