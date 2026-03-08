use crate::db::season::SeasonRepository;
use crate::errors::AppError;
use uuid::Uuid;

use super::UserGameStatsRepository;

impl UserGameStatsRepository {
    /// Update per-game player statistics after a match.
    /// Upserts into `user_game_stats` table scoped by (user_id, game_id, season_id).
    pub async fn update_game_stats(
        &self,
        user_id: Uuid,
        game_id: Uuid,
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
            "INSERT INTO user_game_stats (
                user_id,
                game_id,
                season_id,
                points,
                total_matches,
                total_wins,
                total_pnl,
                win_rate
            )
            VALUES ($1, $2, $3, $4, 1, $5, $6, $7)
            ON CONFLICT (user_id, game_id, season_id)
            DO UPDATE SET
                points = user_game_stats.points + EXCLUDED.points,
                total_matches = user_game_stats.total_matches + 1,
                total_wins = user_game_stats.total_wins + EXCLUDED.total_wins,
                total_pnl = user_game_stats.total_pnl + EXCLUDED.total_pnl,
                win_rate = CASE
                    WHEN user_game_stats.total_matches + 1 > 0
                    THEN ((user_game_stats.total_wins + EXCLUDED.total_wins)::double precision / (user_game_stats.total_matches + 1)::double precision) * 100.0
                    ELSE 0.0
                END,
                updated_at = NOW()",
        )
        .bind(user_id)
        .bind(game_id)
        .bind(season_id)
        .bind(points_delta)
        .bind(wins_delta)
        .bind(pnl_delta)
        .bind(initial_win_rate)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to upsert game stats: {}", e)))?;

        tracing::debug!(
            "Updated game stats for user={} game={} season={}",
            user_id,
            game_id,
            season_id,
        );

        Ok(())
    }
}
