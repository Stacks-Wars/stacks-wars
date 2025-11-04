use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a user rating/review on the platform
/// Used for trust evaluation, reputation systems, and player feedback
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PlatformRating {
    pub id: Uuid,
    pub user_id: Uuid,
    pub rating: i16,
    pub comment: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
