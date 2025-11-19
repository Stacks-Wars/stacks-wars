use crate::errors::AppError;
use uuid::Uuid;

use super::UserWarsPointsRepository;

impl UserWarsPointsRepository {
    /// Delete wars points for a specific user and season.
    pub async fn delete_wars_points(&self, user_id: Uuid, season_id: i32) -> Result<(), AppError> {
        let result = sqlx::query(
            "DELETE FROM user_wars_points
            WHERE user_id = $1 AND season_id = $2",
        )
        .bind(user_id)
        .bind(season_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete wars points: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Wars points entry not found".into()));
        }

        tracing::info!(
            "Deleted wars points for user {} in season {}",
            user_id,
            season_id
        );

        Ok(())
    }

    /// Delete all wars points for a user (all seasons).
    pub async fn delete_all_user_wars_points(&self, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query(
            "DELETE FROM user_wars_points
            WHERE user_id = $1",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to delete all user wars points: {}", e))
        })?;

        let deleted = result.rows_affected();
        tracing::info!(
            "Deleted {} wars points entries for user {}",
            deleted,
            user_id
        );

        Ok(deleted)
    }

    /// Delete all wars points for a season (cleanup/reset).
    pub async fn delete_season_wars_points(&self, season_id: i32) -> Result<u64, AppError> {
        let result = sqlx::query(
            "DELETE FROM user_wars_points
            WHERE season_id = $1",
        )
        .bind(season_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to delete season wars points: {}", e))
        })?;

        let deleted = result.rows_affected();
        tracing::info!(
            "Deleted {} wars points entries for season {}",
            deleted,
            season_id
        );

        Ok(deleted)
    }

    /// Reset all points to zero for a season (preserve entries).
    pub async fn reset_season_points(&self, season_id: i32) -> Result<u64, AppError> {
        let result = sqlx::query(
            "UPDATE user_wars_points
            SET points = 0.0, rank_badge = NULL, updated_at = NOW()
            WHERE season_id = $1",
        )
        .bind(season_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to reset season points: {}", e)))?;

        let reset = result.rows_affected();
        tracing::info!(
            "Reset {} wars points entries for season {}",
            reset,
            season_id
        );

        Ok(reset)
    }
}
