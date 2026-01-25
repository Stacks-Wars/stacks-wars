// Lexi Wars Message Types
//
// Client -> Server: LexiWarsAction
// Server -> Client: LexiWarsEvent
//
// Note: Shared game events (GameStarted, GameStartFailed, FinalStanding, GameOver)
// are in RoomServerMessage and should be used via broadcast::broadcast_room

use crate::games::{GameAction, GameEvent};
use crate::models::PlayerState;
use serde::{Deserialize, Serialize};

use super::rule::ClientRule;

// ============================================================================
// Client -> Server Messages
// ============================================================================

/// LexiWars game actions (client -> server)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LexiWarsAction {
    SubmitWord { word: String },
}

impl GameAction for LexiWarsAction {}

// ============================================================================
// Server -> Client Messages (Game-Specific)
// ============================================================================

/// LexiWars game events (server -> client)
///
/// These are game-specific events sent via GameMessage wrapper.
/// Shared events (GameStarted, GameStartFailed, FinalStanding, GameOver)
/// are in RoomServerMessage.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LexiWarsEvent {
    /// Word has already been used - sent to submitting user only
    UsedWord { word: String },

    /// A valid word was submitted by a player - broadcast to room
    WordEntry { word: String, player: PlayerState },

    /// Invalid word submission - sent to submitting player only
    Invalid { reason: String },

    /// Players count update - broadcast to room
    PlayersCount { remaining: usize, total: usize },

    /// Whose turn it is - broadcast to room
    #[serde(rename_all = "camelCase")]
    Turn {
        player: PlayerState,
        timeout_secs: u64,
    },

    /// Current rule - broadcast to room
    /// Some(rule) for current player, None for others (to clear previous rule)
    Rule { rule: Option<ClientRule> },

    /// Player was eliminated (timeout) - broadcast to room
    Eliminated { player: PlayerState, reason: String },

    /// Countdown tick - broadcast to room
    Countdown { time: u64 },
}

impl GameEvent for LexiWarsEvent {}
