use crate::models::game::Player;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatClientMessage {
    Chat { text: String },
    Ping { ts: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub text: String,
    pub sender: Player,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatServerMessage {
    PermitChat { allowed: bool },
    Chat { message: ChatMessage },
    ChatHistory { messages: Vec<ChatMessage> },
    Pong { ts: u64, pong: u64 },
    Error { message: String },
}

impl ChatServerMessage {
    pub fn should_queue(&self) -> bool {
        match self {
            // Time-sensitive messages that should NOT be queued
            ChatServerMessage::Pong { .. } => false,

            // Important messages that SHOULD be queued
            ChatServerMessage::PermitChat { .. } => true,
            ChatServerMessage::Chat { .. } => true,
            ChatServerMessage::ChatHistory { .. } => true,
            ChatServerMessage::Error { .. } => true,
        }
    }
}
