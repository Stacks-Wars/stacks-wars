use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Chat message in a lobby - stored in Redis for real-time access
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub message_id: Uuid,
    pub lobby_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Uuid>, // ID of message being replied to
    pub reactions: Vec<Reaction>,
    pub created_at: DateTime<Utc>,
}

impl ChatMessage {
    /// Create a new chat message with validation
    pub fn new(
        lobby_id: Uuid,
        user_id: Uuid,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Result<Self, ChatMessageError> {
        // Validate content length (max 500 chars)
        if content.trim().is_empty() {
            return Err(ChatMessageError::EmptyMessage);
        }
        if content.len() > 500 {
            return Err(ChatMessageError::MessageTooLong { max: 500 });
        }

        Ok(Self {
            message_id: Uuid::new_v4(),
            lobby_id,
            user_id,
            content: content.to_string(),
            reply_to,
            reactions: Vec::new(),
            created_at: Utc::now(),
        })
    }

    /// Add a reaction to this message
    pub fn add_reaction(&mut self, user_id: Uuid, emoji: &str) {
        // Remove existing reaction from this user for this emoji
        self.reactions
            .retain(|r| !(r.user_id == user_id && r.emoji == emoji));

        self.reactions.push(Reaction { user_id, emoji: emoji.to_string() });
    }

    /// Remove a reaction from this message
    pub fn remove_reaction(&mut self, user_id: Uuid, emoji: &str) {
        self.reactions
            .retain(|r| !(r.user_id == user_id && r.emoji == emoji));
    }
}

/// Reaction to a chat message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub user_id: Uuid,
    pub emoji: String,
}

/// Supported reaction types (5 basic reactions)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ReactionType {
    ThumbsUp,
    ThumbsDown,
    Heart,
    Laugh,
    Fire,
}

/// Errors related to chat messages
#[derive(Debug, thiserror::Error)]
pub enum ChatMessageError {
    #[error("Message cannot be empty")]
    EmptyMessage,
    #[error("Message too long: maximum {max} characters")]
    MessageTooLong { max: usize },
}
