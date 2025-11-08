use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a user rating/review on the platform
/// Maps to `platform_ratings` table in PostgreSQL
///
/// Used for trust evaluation, reputation systems, and player feedback.
/// Each user can receive multiple ratings from different users.
///
/// # Database Schema
/// - Primary key: `id`
/// - Foreign key: `user_id` (users)
/// - Rating scale: typically 1-5 stars
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
