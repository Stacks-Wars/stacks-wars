use std::str::FromStr;

use chrono::NaiveDateTime;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use super::WalletAddress;

/// User model mapping to PostgreSQL `users` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(skip_deserializing)]
    pub(crate) id: Uuid,
    pub wallet_address: WalletAddress,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub trust_rating: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    /// Get user ID.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get email as EmailAddress for validation
    pub fn email(&self) -> Result<EmailAddress, String> {
        EmailAddress::from_str(&self.email).map_err(|e| format!("Invalid email: {}", e))
    }

    /// Validate email string
    pub fn validate_email(email: &str) -> Result<String, String> {
        EmailAddress::from_str(email)
            .map(|e| e.to_string())
            .map_err(|e| format!("Invalid email: {}", e))
    }
}
