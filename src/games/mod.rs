// Game module: extensible GameState trait and helpers

use crate::errors::AppError;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Base trait for game-specific state
///
/// Each game implements this trait to define its own state structure.
/// Allows the platform to be generic while games can be specific.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Serialize, Deserialize)]
/// pub struct ChessGameState {
///     pub board: [[Option<Piece>; 8]; 8],
///     pub captured_white: Vec<Piece>,
///     pub captured_black: Vec<Piece>,
///     pub move_history: Vec<Move>,
///     pub current_turn: Color,
/// }
///
/// impl GameState for ChessGameState {
///     fn initialize() -> Self {
///         // Create initial board state
///     }
///
///     fn validate(&self) -> Result<(), AppError> {
///         // Validate board state
///     }
/// }
/// ```
pub trait GameState: Serialize + DeserializeOwned + Clone + Send + Sync {
    /// Serialize to JSON for Redis storage
    ///
    /// Stored in Redis at: `lobbies:{lobby_id}:game_state`
    fn to_json(&self) -> Result<String, AppError> {
        serde_json::to_string(self).map_err(|e| AppError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON
    fn from_json(json: &str) -> Result<Self, AppError> {
        serde_json::from_str(json).map_err(|e| AppError::Deserialization(e.to_string()))
    }

    /// Initialize new game state with default values
    ///
    /// Called when a lobby starts a new game.
    fn initialize() -> Self;

    /// Validate state is consistent
    ///
    /// Called before saving to ensure data integrity.
    /// Return `Err` if state is invalid.
    fn validate(&self) -> Result<(), AppError>;

    /// Get a human-readable summary of the game state
    ///
    /// Useful for debugging and logging.
    fn summary(&self) -> String {
        format!("GameState: {}", std::any::type_name::<Self>())
    }
}
