//! Wire types shared between the server and game crates (must match JSON shape).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Player participation status (lobby / room wire shape).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PlayerStatus {
    NotJoined,
    Joined,
}

/// Prize claim status for finished games.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "data", rename_all = "camelCase")]
pub enum ClaimState {
    Claimed { tx_id: String },
    NotClaimed,
}

/// Join request state stored on [`PlayerStateWire`].
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum JoinRequestState {
    Pending,
    Accepted,
    Rejected,
}

/// Runtime player row in a lobby (matches server `models::PlayerState` JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStateWire {
    pub user_id: Uuid,
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub trust_rating: f64,
    pub lobby_id: Uuid,
    pub status: PlayerStatus,
    pub state: JoinRequestState,
    pub rank: Option<usize>,
    pub prize: Option<f64>,
    pub wars_point: Option<f64>,
    pub claim_state: Option<ClaimState>,
    pub last_ping: Option<u64>,
    pub joined_at: i64,
    pub updated_at: i64,
    pub is_creator: bool,
}
