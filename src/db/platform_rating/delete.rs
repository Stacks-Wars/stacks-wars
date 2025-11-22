use super::PlatformRatingRepository;
use crate::errors::AppError;

impl PlatformRatingRepository {
    /// Delete a user's platform rating. Returns Ok(()) even if the row
    /// did not exist.
    pub async fn delete_by_user(&self, user_id: uuid::Uuid) -> Result<(), AppError> {
        sqlx::query_scalar::<_, i64>("DELETE FROM platform_ratings WHERE user_id = $1 RETURNING 1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to delete platform rating: {}", e))
            })?;

        Ok(())
    }
}
