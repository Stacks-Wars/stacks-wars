use crate::{errors::AppError, models::user::UserWarsPoints};
use uuid::Uuid;

use super::UserWarsPointsRepository;

impl UserWarsPointsRepository {
    /// Create or update wars points for a season (upsert)
    ///
    /// If entry exists, updates points. Otherwise, creates new entry.
    /// This is the primary method for initializing user wars points.
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `season_id` - Season ID
    /// * `points` - Initial points value (default: 0.0)
    ///
    /// # Returns
    /// * `Ok(UserWarsPoints)` - Created or updated wars points
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Initialize user for new season
    /// let points = repo.upsert_wars_points(user_id, season_id, 0.0).await?;
    /// ```
    pub async fn upsert_wars_points(
        &self,
        user_id: Uuid,
        season_id: i32,
        points: f64,
    ) -> Result<UserWarsPoints, AppError> {
        let wars_point_id = Uuid::new_v4();

        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "INSERT INTO user_wars_points (id, user_id, season_id, points)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, season_id)
            DO UPDATE SET points = $4, updated_at = NOW()
            RETURNING id, user_id, season_id, points, rank_badge, created_at, updated_at",
        )
        .bind(wars_point_id)
        .bind(user_id)
        .bind(season_id)
        .bind(points)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to upsert wars points: {}", e)))?;

        tracing::info!(
            "Upserted wars points for user {} in season {}: {}",
            user_id,
            season_id,
            points
        );

        Ok(wars_points)
    }
}
