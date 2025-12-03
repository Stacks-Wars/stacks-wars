use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

/// Refined user model that maps directly to PostgreSQL
/// Maps to `users` table (global user profile)
///
/// This is the primary user model for all new features and endpoints.
/// Contains wallet authentication, profile data, and trust metrics.
///
/// **Note**: Seasonal data (wars points) is managed separately in `UserWarsPoints`.
///
/// # Database Schema
/// - Primary key: `id`
/// - Unique constraint: `wallet_address`
/// - Default trust_rating: 10.0
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserV2 {
    pub id: Uuid,
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub trust_rating: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
