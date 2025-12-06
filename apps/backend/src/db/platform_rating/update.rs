use super::PlatformRatingRepository;
use crate::errors::AppError;
use crate::models::db::PlatformRating;

impl PlatformRatingRepository {
    /// Update an existing platform rating (by user_id). Returns the updated row.
    pub async fn update_rating(
        &self,
        user_id: uuid::Uuid,
        rating: i16,
        comment: Option<&str>,
    ) -> Result<PlatformRating, AppError> {
        let rec = sqlx::query_as::<_, PlatformRating>(
            r#"UPDATE platform_ratings SET rating = $1, comment = $2, updated_at = NOW()
            WHERE user_id = $3
            RETURNING id, user_id, rating, comment, created_at, updated_at"#,
        )
        .bind(rating)
        .bind(comment)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update platform rating: {}", e)))?;

        Ok(rec)
    }
}
