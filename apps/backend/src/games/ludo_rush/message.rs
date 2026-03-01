// Ludo Rush Message Types
//
// Client -> Server: LudoRushAction
// Server -> Client: LudoRushEvent
//
// Note: Shared game events (GameStarted, GameStartFailed, FinalStanding, GameOver)
// are in RoomServerMessage and should be used via broadcast::broadcast_room

use crate::games::{GameAction, GameEvent};
use crate::models::PlayerState;
use serde::{Deserialize, Serialize};

use super::board::{LudoRushBoard, PawnPosition};

// ============================================================================
// Client -> Server Messages
// ============================================================================

/// Ludo Rush game actions (client -> server)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LudoRushAction {
    /// Roll the dice (only valid when it's your turn and you haven't rolled)
    RollDice,
    /// Select a dice value to play with (die1, die2, or sum)
    #[serde(rename_all = "camelCase")]
    SelectDiceValue { dice_value: u8 },
    /// Choose which pawn to move with the currently selected dice value (pawn_id: 0-3)
    #[serde(rename_all = "camelCase")]
    MovePawn { pawn_id: usize },
}

impl GameAction for LudoRushAction {}

// ============================================================================
// Server -> Client Messages (Game-Specific)
// ============================================================================

/// Ludo Rush game events (server -> client)
///
/// These are game-specific events sent via GameMessage wrapper.
/// Shared events (GameStarted, GameStartFailed, FinalStanding, GameOver)
/// are in RoomServerMessage.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LudoRushEvent {
    /// Board state update - broadcast to room
    BoardUpdate { board: LudoRushBoard },

    /// Whose turn it is - broadcast to room
    #[serde(rename_all = "camelCase")]
    Turn {
        player: PlayerState,
        timeout_secs: u64,
    },

    /// Dice roll result (dual dice) - broadcast to room
    #[serde(rename_all = "camelCase")]
    DiceRolled {
        player: PlayerState,
        dice1: u8,
        dice2: u8,
        /// Which values (die1, die2, sum) have at least one movable pawn
        playable_values: Vec<u8>,
    },

    /// Movable pawns for a specific dice value — sent to the requesting player
    #[serde(rename_all = "camelCase")]
    MovablePawns {
        dice_value: u8,
        pawns: Vec<usize>,
    },

    /// A dice value was consumed after a pawn move — broadcast to room
    #[serde(rename_all = "camelCase")]
    DiceValueUsed {
        dice_value: u8,
        /// Remaining playable values after this move
        remaining_values: Vec<u8>,
    },

    /// Pawn moved - broadcast to room
    #[serde(rename_all = "camelCase")]
    PawnMoved {
        player: PlayerState,
        pawn_id: usize,
        from: PawnPosition,
        to: PawnPosition,
        dice_value: u8,
    },

    /// Pawn captured — opponent sent home, attacker pawn finished — broadcast to room
    #[serde(rename_all = "camelCase")]
    PawnCaptured {
        attacker: PlayerState,
        victim: PlayerState,
        victim_pawn_id: usize,
        /// The attacker's pawn that is instantly finished as a capture reward
        attacker_pawn_id: usize,
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

    /// Bonus turn awarded (both dice rolled 6) - broadcast to room
    BonusTurn { player: PlayerState },

    /// Player quit the game - broadcast to room
    PlayerQuit { player: PlayerState, reason: String },

    /// Countdown tick - broadcast to room
    Countdown { time: u64 },

    /// Invalid action - sent to specific user only
    Invalid { reason: String },
}

impl GameEvent for LudoRushEvent {}
