use crate::{errors::AppError, models::UserWarsPoints};
use uuid::Uuid;

use super::UserWarsPointsRepository;

impl UserWarsPointsRepository {
    /// Increment or decrement a user's wars points for a season.
    pub async fn add_wars_points(
        &self,
        user_id: Uuid,
        season_id: i32,
        points_to_add: f64,
    ) -> Result<UserWarsPoints, AppError> {
        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "UPDATE user_wars_points
            SET points = points + $1, updated_at = NOW()
            WHERE user_id = $2 AND season_id = $3
            RETURNING id, user_id, season_id, points, rank_badge, created_at, updated_at",
        )
        .bind(points_to_add)
        .bind(user_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to add wars points: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Wars points entry not found".into()))?;

        tracing::info!(
            "Added {} points to user {} for season {} (new total: {})",
            points_to_add,
            user_id,
            season_id,
            wars_points.points
        );

        Ok(wars_points)
    }

    /// Set a user's wars points to an explicit value.
    pub async fn set_wars_points(
        &self,
        user_id: Uuid,
        season_id: i32,
        new_points: f64,
    ) -> Result<UserWarsPoints, AppError> {
        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "UPDATE user_wars_points
            SET points = $1, updated_at = NOW()
            WHERE user_id = $2 AND season_id = $3
            RETURNING id, user_id, season_id, points, rank_badge, created_at, updated_at",
        )
        .bind(new_points)
        .bind(user_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to set wars points: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Wars points entry not found".into()))?;

        tracing::info!(
            "Set wars points for user {} in season {} to {}",
            user_id,
            season_id,
            new_points
        );

        Ok(wars_points)
    }

    /// Update a user's rank badge for a season.
    pub async fn update_rank_badge(
        &self,
        user_id: Uuid,
        season_id: i32,
        rank_badge: Option<String>,
    ) -> Result<UserWarsPoints, AppError> {
        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "UPDATE user_wars_points
            SET rank_badge = $1, updated_at = NOW()
            WHERE user_id = $2 AND season_id = $3
            RETURNING id, user_id, season_id, points, rank_badge, created_at, updated_at",
        )
        .bind(&rank_badge)
        .bind(user_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update rank badge: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Wars points entry not found".into()))?;

        tracing::info!(
            "Updated rank badge for user {} in season {} to {:?}",
            user_id,
            season_id,
            rank_badge
        );

        Ok(wars_points)
    }

    /// Bulk-update wars points for many users; returns number updated.
    pub async fn bulk_add_points(&self, updates: Vec<(Uuid, i32, f64)>) -> Result<u64, AppError> {
        let mut transaction =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to start transaction: {}", e))
            })?;

        let mut total_updated = 0u64;

        for (user_id, season_id, points_to_add) in updates {
            let result = sqlx::query(
                "UPDATE user_wars_points
                SET points = points + $1, updated_at = NOW()
                WHERE user_id = $2 AND season_id = $3",
            )
            .bind(points_to_add)
            .bind(user_id)
            .bind(season_id)
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to bulk update wars points: {}", e))
            })?;

            total_updated += result.rows_affected();
        }

        transaction
            .commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        tracing::info!("Bulk updated {} wars points entries", total_updated);

        Ok(total_updated)
    }
}
