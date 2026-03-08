use crate::{errors::AppError, models::UserWarsPoints};
use crate::db::season::SeasonRepository;
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

    /// Update player statistics and optionally add wars points for a season.
    ///
    /// If `season_id` is `None` the current season will be resolved. If
    /// `wars_point` is `Some`, it will be added to the user's `points`.
    /// This also increments `total_matches`, optionally `total_wins`, updates
    /// `total_pnl` and recalculates `win_rate`.
    pub async fn update_player_stats(
        &self,
        user_id: Uuid,
        season_id: Option<i32>,
        wars_point: Option<f64>,
        entry_amount: Option<f64>,
        prize: Option<f64>,
        is_winner: bool,
    ) -> Result<(), AppError> {
        // Resolve season
        let season_id = if let Some(s) = season_id {
            s
        } else {
            let season_repo = SeasonRepository::new(self.pool.clone());
            season_repo.get_current_season_id().await?
        };

        let points_delta = wars_point.unwrap_or(0.0);
        let wins_delta = if is_winner { 1 } else { 0 };

        let entry = entry_amount.unwrap_or(0.0);
        let prize_amount = prize.unwrap_or(0.0);
        let pnl_delta = prize_amount - entry;

        let initial_win_rate = if wins_delta > 0 { 100.0 } else { 0.0 };

        sqlx::query(
            "INSERT INTO user_wars_points (
                user_id,
                season_id,
                points,
                total_matches,
                total_wins,
                total_pnl,
                win_rate
            )
            VALUES ($1, $2, $3, 1, $4, $5, $6)
            ON CONFLICT (user_id, season_id)
            DO UPDATE SET
                points = user_wars_points.points + EXCLUDED.points,
                total_matches = user_wars_points.total_matches + 1,
                total_wins = user_wars_points.total_wins + EXCLUDED.total_wins,
                total_pnl = user_wars_points.total_pnl + EXCLUDED.total_pnl,
                win_rate = CASE
                    WHEN user_wars_points.total_matches + 1 > 0
                    THEN ((user_wars_points.total_wins + EXCLUDED.total_wins)::double precision / (user_wars_points.total_matches + 1)::double precision) * 100.0
                    ELSE 0.0
                END,
                updated_at = NOW()",
        )
        .bind(user_id)
        .bind(season_id)
        .bind(points_delta)
        .bind(wins_delta)
        .bind(pnl_delta)
        .bind(initial_win_rate)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to upsert player stats: {}", e)))?;

        tracing::debug!(
            "Updated player stats for {}: season={}, wars_point={:?}",
            user_id,
            season_id,
            wars_point
        );

        Ok(())
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
