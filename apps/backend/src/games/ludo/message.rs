// Ludo Message Types
//
// Client -> Server: LudoAction
// Server -> Client: LudoEvent
//
// Note: Shared game events (GameStarted, GameStartFailed, FinalStanding, GameOver)
// are in RoomServerMessage and should be used via broadcast::broadcast_room

use crate::games::{GameAction, GameEvent};
use crate::models::PlayerState;
use serde::{Deserialize, Serialize};

use super::board::{LudoBoard, PawnPosition};

// ============================================================================
// Client -> Server Messages
// ============================================================================

/// Ludo game actions (client -> server)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LudoAction {
    /// Roll the dice (only valid when it's your turn and you haven't rolled)
    RollDice,
    /// Choose which pawn to move after rolling (pawn_id: 0-3)
    #[serde(rename_all = "camelCase")]
    MovePawn { pawn_id: usize },
}

impl GameAction for LudoAction {}

// ============================================================================
// Server -> Client Messages (Game-Specific)
// ============================================================================

/// Ludo game events (server -> client)
///
/// These are game-specific events sent via GameMessage wrapper.
/// Shared events (GameStarted, GameStartFailed, FinalStanding, GameOver)
/// are in RoomServerMessage.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LudoEvent {
    /// Board state update - broadcast to room
    BoardUpdate { board: LudoBoard },

    /// Whose turn it is - broadcast to room
    #[serde(rename_all = "camelCase")]
    Turn {
        player: PlayerState,
        timeout_secs: u64,
    },

    /// Dice roll result - broadcast to room
    #[serde(rename_all = "camelCase")]
    DiceRolled {
        player: PlayerState,
        dice: u8,
        movable_pawns: Vec<usize>,
    },

    /// Pawn moved - broadcast to room
    #[serde(rename_all = "camelCase")]
    PawnMoved {
        player: PlayerState,
        pawn_id: usize,
        from: PawnPosition,
        to: PawnPosition,
    },

    /// Pawn captured (sent back to home) - broadcast to room
    #[serde(rename_all = "camelCase")]
    PawnCaptured {
        attacker: PlayerState,
        victim: PlayerState,
        pawn_id: usize,
    },

    /// Pawn reached finish - broadcast to room
    #[serde(rename_all = "camelCase")]
    PawnFinished {
        player: PlayerState,
        pawn_id: usize,
        pawns_remaining: usize,
    },

    /// Player has no valid moves (turn skipped) - broadcast to room
    NoValidMoves { player: PlayerState },

    /// Bonus turn awarded (rolled a 6) - broadcast to room
    BonusTurn { player: PlayerState },

    /// Countdown tick - broadcast to room
    Countdown { time: u64 },

    /// Invalid action - sent to specific user only
    Invalid { reason: String },
}

impl GameEvent for LudoEvent {}
