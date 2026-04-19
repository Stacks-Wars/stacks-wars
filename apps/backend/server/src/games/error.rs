use std::fmt;

#[derive(Debug, Clone)]
pub enum GameError {
    /// Player attempted action when it's not their turn
    NotYourTurn,
    /// Player is not in the game
    NotInGame,
    /// Game has already finished
    GameFinished,
    /// Game hasn't started yet
    GameNotStarted,
    /// Invalid action for current game state
    InvalidAction(String),
    /// Player already eliminated
    AlreadyEliminated,
    /// Insufficient players to start
    InsufficientPlayers { required: usize, actual: usize },
    /// Internal game error
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

// Convert GameError to AppError for compatibility
impl From<GameError> for crate::errors::AppError {
    fn from(err: GameError) -> Self {
        match err {
            GameError::NotYourTurn
            | GameError::NotInGame
            | GameError::InvalidAction(_)
            | GameError::AlreadyEliminated => crate::errors::AppError::BadRequest(err.to_string()),
            GameError::GameFinished | GameError::GameNotStarted => {
                crate::errors::AppError::BadRequest(err.to_string())
            }
            GameError::InsufficientPlayers { .. } => {
                crate::errors::AppError::BadRequest(err.to_string())
            }
            GameError::Internal(_msg) => crate::errors::AppError::InternalError,
        }
    }
}
