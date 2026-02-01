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

        // Try to fetch existing entry
        let existing = sqlx::query_as::<_, UserWarsPoints>(
            "SELECT id, user_id, season_id, points, rank_badge, total_matches, total_wins, total_pnl, win_rate, created_at, updated_at
            FROM user_wars_points
            WHERE user_id = $1 AND season_id = $2",
        )
        .bind(user_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get user wars points: {}", e)))?;

        if let Some(mut points) = existing {
            // Apply wars points if provided
            if let Some(wp) = wars_point {
                points.points += wp;
            }

            // Update stats
            points.total_matches += 1;
            if is_winner {
                points.total_wins += 1;
            }

            let pnl_addition = if let Some(entry) = entry_amount {
                let prize_amount = prize.unwrap_or(0.0);
                prize_amount - entry
            } else {
                0.0
            };

            points.total_pnl += pnl_addition;
            points.win_rate = if points.total_matches > 0 {
                (points.total_wins as f64 / points.total_matches as f64) * 100.0
            } else {
                0.0
            };

            // Persist
            sqlx::query(
                "UPDATE user_wars_points
                SET points = $1, total_matches = $2, total_wins = $3, total_pnl = $4, win_rate = $5, updated_at = NOW()
                WHERE id = $6",
            )
            .bind(points.points)
            .bind(points.total_matches)
            .bind(points.total_wins)
            .bind(points.total_pnl)
            .bind(points.win_rate)
            .bind(points.id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to update player stats: {}", e)))?;
        } else {
            // Create new entry
            let initial_points = wars_point.unwrap_or(0.0);
            let total_matches = 1;
            let total_wins = if is_winner { 1 } else { 0 };

            let total_pnl = if let Some(entry) = entry_amount {
                let prize_amount = prize.unwrap_or(0.0);
                prize_amount - entry
            } else {
                0.0
            };

            let win_rate = if total_matches > 0 {
                (total_wins as f64 / total_matches as f64) * 100.0
            } else {
                0.0
            };

            let _new_id = sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO user_wars_points (user_id, season_id, points, total_matches, total_wins, total_pnl, win_rate)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id",
            )
            .bind(user_id)
            .bind(season_id)
            .bind(initial_points)
            .bind(total_matches)
            .bind(total_wins)
            .bind(total_pnl)
            .bind(win_rate)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create player stats: {}", e)))?;
        }

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
