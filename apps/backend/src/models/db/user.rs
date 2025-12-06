use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use super::WalletAddress;

/// User model mapping to PostgreSQL `users` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(skip_serializing)]
    pub(crate) id: Uuid,
    pub wallet_address: WalletAddress,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub trust_rating: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    /// Get user ID.
    pub fn id(&self) -> Uuid {
        self.id
    }
}
