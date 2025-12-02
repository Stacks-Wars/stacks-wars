use crate::models::game::Player;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LexiWarsClientMessage {
    WordEntry { word: String },
    Ping { ts: u64 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerStanding {
    pub player: Player,
    pub rank: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LexiWarsServerMessage {
    #[serde(rename_all = "camelCase")]
    Turn {
        current_turn: Player,
        countdown: u64,
    },
    Rule {
        rule: String,
    },
    Countdown {
        time: u64,
    },
    Rank {
        rank: String,
    },
    Validate {
        msg: String,
    },
    WordEntry {
        word: String,
        sender: Player,
    },
    UsedWord {
        word: String,
    },
    GameOver,
    FinalStanding {
        standing: Vec<PlayerStanding>,
    },
    Prize {
        amount: f64,
    },
    #[serde(rename_all = "camelCase")]
    WarsPoint {
        wars_point: f64,
    },
    Pong {
        ts: u64,
        pong: u64,
    },
    Start {
        time: u32,
        started: bool,
    },
    StartFailed,
    Spectator,
    #[serde(rename_all = "camelCase")]
    PlayersCount {
        connected_players: usize,
        remaining_players: usize,
    },
}

impl LexiWarsServerMessage {
    pub fn should_queue(&self) -> bool {
        match self {
            // Time-sensitive messages that should NOT be queued
            LexiWarsServerMessage::Countdown { .. } => false,
            LexiWarsServerMessage::Pong { .. } => false,
            LexiWarsServerMessage::Start { started: false, .. } => false,
            LexiWarsServerMessage::Turn { .. } => false,
            LexiWarsServerMessage::Rule { .. } => false,

            // Important messages that SHOULD be queued
            LexiWarsServerMessage::Rank { .. } => true,
            LexiWarsServerMessage::Validate { .. } => true,
            LexiWarsServerMessage::WordEntry { .. } => true,
            LexiWarsServerMessage::UsedWord { .. } => true,
            LexiWarsServerMessage::GameOver => true,
            LexiWarsServerMessage::FinalStanding { .. } => true,
            LexiWarsServerMessage::Prize { .. } => true,
            LexiWarsServerMessage::WarsPoint { .. } => true,
            LexiWarsServerMessage::Start { started: true, .. } => true, // Game actually started
            LexiWarsServerMessage::StartFailed => true,
            LexiWarsServerMessage::Spectator => true,
            LexiWarsServerMessage::PlayersCount { .. } => true,
        }
    }
}
