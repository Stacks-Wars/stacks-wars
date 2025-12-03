use crate::errors::AppError;
use crate::models::db::PlatformRating;
use super::PlatformRatingRepository;

impl PlatformRatingRepository {
    /// Create or replace a platform rating for the given user.
    /// Since `platform_ratings.user_id` is unique, this will return an
    /// error if the row already exists — callers can use `update_rating`
    /// to change an existing rating.
    pub async fn create_rating(
        &self,
        user_id: uuid::Uuid,
        rating: i16,
        comment: Option<String>,
    ) -> Result<PlatformRating, AppError> {
        let rec = sqlx::query_as::<_, PlatformRating>(
            r#"INSERT INTO platform_ratings (user_id, rating, comment)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, rating, comment, created_at, updated_at"#,
        )
        .bind(user_id)
        .bind(rating)
        .bind(comment)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create platform rating: {}", e)))?;

        tracing::info!("Created platform rating for user {}", rec.user_id);
        Ok(rec)
    }
}
