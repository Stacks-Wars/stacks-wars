use crate::{
    db::season::SeasonRepository,
    errors::AppError,
    http::leaderboards::handlers::{LeaderboardSortBy, SortOrder},
    models::user_game_stats::{GameLeaderBoard, GameStats, UserGameStats, UserTopGame},
};
use uuid::Uuid;

use super::UserGameStatsRepository;

impl UserGameStatsRepository {
    /// Get a single user's game stats for a specific game and season.
    pub async fn get_player_game_stats(
        &self,
        user_id: Uuid,
        game_id: Uuid,
        season_id: Option<i32>,
    ) -> Result<UserGameStats, AppError> {
        let season_id = match season_id {
            Some(id) => id,
            None => {
                let season_repo = SeasonRepository::new(self.pool.clone());
                season_repo.get_current_season_id().await?
            }
        };

        sqlx::query_as::<_, UserGameStats>(
            "SELECT id, user_id, game_id, season_id, points, total_matches, total_wins, total_pnl, win_rate, created_at, updated_at
            FROM user_game_stats
            WHERE user_id = $1 AND game_id = $2 AND season_id = $3",
        )
        .bind(user_id)
        .bind(game_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get player game stats: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Player game stats not found".into()))
    }

    /// Get the leaderboard for a specific game, with pagination and sorting.
    pub async fn get_game_leaderboard(
        &self,
        game_id: Uuid,
        season_id: Option<i32>,
        limit: i64,
        offset: i64,
        sort_by: Option<LeaderboardSortBy>,
        order: Option<SortOrder>,
    ) -> Result<(Vec<GameLeaderBoard>, i64), AppError> {
        let season_id = match season_id {
            Some(id) => id,
            None => {
                let season_repo = SeasonRepository::new(self.pool.clone());
                season_repo.get_current_season_id().await?
            }
        };

        let sort_col = match sort_by.unwrap_or(LeaderboardSortBy::WarsPoints) {
            LeaderboardSortBy::WarsPoints => "ugs.points",
            LeaderboardSortBy::TotalMatches => "ugs.total_matches",
            LeaderboardSortBy::WinRate => "ugs.win_rate",
            LeaderboardSortBy::TotalPnl => "ugs.total_pnl",
        };

        let order_str = match order.unwrap_or(SortOrder::Desc) {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        let query = format!(
            "SELECT ugs.id, ugs.game_id, g.name as game_name, g.image_url as game_image_url,
                    ugs.season_id, ugs.points,
                    u.id as user_id, u.wallet_address, u.username, u.display_name,
                    u.email, u.email_verified, u.trust_rating, u.profile_image,
                    ugs.total_matches, ugs.total_wins, ugs.total_pnl, ugs.win_rate,
                    ugs.created_at, ugs.updated_at
            FROM user_game_stats ugs
            JOIN users u ON ugs.user_id = u.id
            JOIN games g ON ugs.game_id = g.id
            WHERE ugs.game_id = $1 AND ugs.season_id = $2
            ORDER BY {} {}
            LIMIT $3 OFFSET $4",
            sort_col, order_str
        );

        let (leaderboard_result, total_result) = tokio::join!(
            sqlx::query_as::<_, GameLeaderBoard>(&query)
                .bind(game_id)
                .bind(season_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool),
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM user_game_stats WHERE game_id = $1 AND season_id = $2"
            )
            .bind(game_id)
            .bind(season_id)
            .fetch_one(&self.pool)
        );

        let leaderboard = leaderboard_result.map_err(|e| {
            AppError::DatabaseError(format!("Failed to get game leaderboard: {}", e))
        })?;
        let total = total_result.map_err(|e| {
            AppError::DatabaseError(format!("Failed to get game leaderboard total: {}", e))
        })?;

        Ok((leaderboard, total))
    }

    /// Get a user's top/most-played games (across all games for a season).
    pub async fn get_user_top_games(
        &self,
        user_id: Uuid,
        season_id: Option<i32>,
        limit: i64,
    ) -> Result<Vec<UserTopGame>, AppError> {
        let season_id = match season_id {
            Some(id) => id,
            None => {
                let season_repo = SeasonRepository::new(self.pool.clone());
                season_repo.get_current_season_id().await?
            }
        };

        let top_games = sqlx::query_as::<_, UserTopGame>(
            "SELECT ugs.game_id, g.name as game_name, g.image_url as game_image_url,
                    ugs.total_matches, ugs.total_wins, ugs.win_rate, ugs.points, ugs.total_pnl
            FROM user_game_stats ugs
            JOIN games g ON ugs.game_id = g.id
            WHERE ugs.user_id = $1 AND ugs.season_id = $2
            ORDER BY ugs.total_matches DESC
            LIMIT $3",
        )
        .bind(user_id)
        .bind(season_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to get user top games: {}", e))
        })?;

        Ok(top_games)
    }

    /// Get per-game platform stats aggregated across all users for a season.
    pub async fn get_platform_game_stats(
        &self,
        season_id: Option<i32>,
    ) -> Result<Vec<GameStats>, AppError> {
        let season_id = match season_id {
            Some(id) => id,
            None => {
                let season_repo = SeasonRepository::new(self.pool.clone());
                season_repo.get_current_season_id().await?
            }
        };

        let rows = sqlx::query_as::<_, (Uuid, String, String, i64, i64, i64, f64, f64, f64)>(
            "SELECT g.id as game_id, g.name as game_name, g.image_url as game_image_url,
                    COALESCE(SUM(ugs.total_matches), 0)::bigint as total_matches,
                    COUNT(DISTINCT ugs.user_id) as total_players,
                    COALESCE(SUM(ugs.total_wins), 0)::bigint as total_wins,
                    CASE
                        WHEN COALESCE(SUM(ugs.total_matches), 0) > 0
                        THEN (COALESCE(SUM(ugs.total_wins), 0)::double precision / COALESCE(SUM(ugs.total_matches), 0)::double precision) * 100.0
                        ELSE 0.0
                    END as avg_win_rate,
                    COALESCE(SUM(ugs.total_pnl), 0.0) as total_pnl,
                    COALESCE(SUM(ugs.points), 0.0) as total_points
            FROM games g
            LEFT JOIN user_game_stats ugs ON g.id = ugs.game_id AND ugs.season_id = $1
            WHERE g.is_active = true
            GROUP BY g.id, g.name, g.image_url
            ORDER BY total_matches DESC",
        )
        .bind(season_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to get platform game stats: {}", e))
        })?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    game_id,
                    game_name,
                    game_image_url,
                    total_matches,
                    total_players,
                    total_wins,
                    avg_win_rate,
                    total_pnl,
                    total_points,
                )| {
                    GameStats {
                        game_id,
                        game_name,
                        game_image_url,
                        total_matches,
                        total_players,
                        total_wins,
                        avg_win_rate,
                        total_pnl,
                        total_points,
                    }
                },
            )
            .collect())
    }
}
