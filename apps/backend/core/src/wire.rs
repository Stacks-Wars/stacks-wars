//! Room-level messages games emit (must match `RoomServerMessage` JSON on the wire).

use crate::dto::PlayerStateWire;
use serde::Serialize;

/// Subset of room server messages constructed by game engines.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum GameRoomBroadcast {
    GameStarted,
    GameStartFailed { reason: String },
    FinalStanding {
        standings: Vec<PlayerStateWire>,
    },
}

/// Per-user message (matches `RoomServerMessage::GameOver` on the wire).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UserRoomMessage {
    #[serde(rename_all = "camelCase")]
    GameOver {
        rank: usize,
        prize: Option<f64>,
        wars_point: f64,
    },
}
