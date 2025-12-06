use crate::errors::AppError;
use crate::models::PlatformRating;
use super::PlatformRatingRepository;

impl PlatformRatingRepository {
    /// Get a platform rating by user id. Returns `Ok(None)` if not found.
    pub async fn get_by_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Option<PlatformRating>, AppError> {
        let rec = sqlx::query_as::<_, PlatformRating>(
            "SELECT id, user_id, rating, comment, created_at, updated_at FROM platform_ratings WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query platform rating: {}", e)))?;

        Ok(rec)
    }

    /// List platform ratings. If `rating_filter` is `Some(n)` returns only
    /// ratings equal to `n` (1-5). If `None` returns all ratings.
    pub async fn list(
        &self,
        rating_filter: Option<i16>,
    ) -> Result<Vec<PlatformRating>, AppError> {
        let recs = sqlx::query_as::<_, PlatformRating>(
            "SELECT id, user_id, rating, comment, created_at, updated_at FROM platform_ratings WHERE ($1 IS NULL OR rating = $1) ORDER BY created_at DESC",
        )
        .bind(rating_filter)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to list platform ratings: {}", e)))?;

        Ok(recs)
    }
}
