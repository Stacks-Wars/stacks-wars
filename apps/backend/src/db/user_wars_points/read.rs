use crate::{
    db::season::SeasonRepository, errors::AppError, http::handlers::player_stats::{LeaderboardSortBy, SortOrder}, models::user_wars_point::{LeaderBoard, PlayerStats, UserWarsPoints}
};
use uuid::Uuid;

use super::UserWarsPointsRepository;



impl UserWarsPointsRepository {
    /// Get a user's wars points for a specific season.
    pub async fn get_wars_points(
        &self,
        user_id: Uuid,
        season_id: i32,
    ) -> Result<UserWarsPoints, AppError> {
        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "SELECT id, user_id, season_id, points, rank_badge, created_at, updated_at
            FROM user_wars_points
            WHERE user_id = $1 AND season_id = $2",
        )
        .bind(user_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user wars points: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Wars points not found for this season".into()))?;

        Ok(wars_points)
    }


    /// Get player statistics from PostgreSQL for the current season
    pub async fn get_player_stats(&self, user_id: Uuid) -> Result<PlayerStats, AppError> {
        // Get current season ID
        let season_repo = SeasonRepository::new(self.pool.clone());
        let season_id = season_repo.get_current_season_id().await?;

        // Get user wars points for current season
        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "SELECT id, user_id, season_id, points, rank_badge, total_matches, total_wins, total_pnl, win_rate, created_at, updated_at
            FROM user_wars_points
            WHERE user_id = $1 AND season_id = $2",
        )
        .bind(user_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get player stats: {}", e)))?;

        match wars_points {
            Some(points) => Ok(PlayerStats {
                total_matches: points.total_matches,
                total_wins: points.total_wins,
                total_pnl: points.total_pnl,
                win_rate: points.win_rate,
            }),
            None => Ok(PlayerStats {
                total_matches: 0,
                total_wins: 0,
                total_pnl: 0.0,
                win_rate: 0.0,
            }),
        }
    }

    /// Get the leaderboard (top users by wars points) for a season.
    pub async fn get_leaderboard(
        &self,
        season_id: Option<i32>,
        limit: i64,
        offset: i64,
        sort_by: Option<LeaderboardSortBy>,
        order: Option<SortOrder>,
    ) -> Result<Vec<LeaderBoard>, AppError> {
        // Get current season ID if not provided
        let season_id = match season_id {
            Some(id) => id,
            None => {
                let season_repo = SeasonRepository::new(self.pool.clone());
                season_repo.get_current_season_id().await?
            }
        };

        // Build ORDER BY clause based on requested sort and order.
        let sort_col = match sort_by.unwrap_or(LeaderboardSortBy::WarsPoints) {
            LeaderboardSortBy::WarsPoints => "uwp.points",
            LeaderboardSortBy::TotalMatches => "uwp.total_matches",
            LeaderboardSortBy::WinRate => "uwp.win_rate",
            LeaderboardSortBy::TotalPnl => "uwp.total_pnl",
        };

        let order_str = match order.unwrap_or(SortOrder::Desc) {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };

        let query = format!(
            "SELECT uwp.id, uwp.season_id, uwp.points, uwp.rank_badge,
                    u.id as user_id, u.wallet_address, u.username, u.display_name,
                    u.email, u.email_verified, u.trust_rating,
                    uwp.total_matches, uwp.total_wins, uwp.total_pnl, uwp.win_rate,
                    uwp.created_at, uwp.updated_at
            FROM user_wars_points uwp
            JOIN users u ON uwp.user_id = u.id
            WHERE uwp.season_id = $1
            ORDER BY {} {}
            LIMIT $2 OFFSET $3",
            sort_col, order_str
        );

        let leaderboard = sqlx::query_as::<_, LeaderBoard>(&query)
            .bind(season_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to get leaderboard: {}", e)))?;

        Ok(leaderboard)
    }

    /// Get a single player's leaderboard entry for a season.
    pub async fn get_player_leaderboard(
        &self,
        user_id: Uuid,
        season_id: Option<i32>,
    ) -> Result<LeaderBoard, AppError> {
        // Get current season ID if not provided
        let season_id = match season_id {
            Some(id) => id,
            None => {
                let season_repo = SeasonRepository::new(self.pool.clone());
                season_repo.get_current_season_id().await?
            }
        };

        // Get the leaderboard data from database for this specific user
        let leaderboard_entry = sqlx::query_as::<_, LeaderBoard>(
            "SELECT uwp.id, uwp.season_id, uwp.points, uwp.rank_badge,
                    u.id as user_id, u.wallet_address, u.username, u.display_name,
                    u.email, u.email_verified, u.trust_rating,
                    uwp.total_matches, uwp.total_wins, uwp.total_pnl, uwp.win_rate,
                    uwp.created_at, uwp.updated_at
            FROM user_wars_points uwp
            JOIN users u ON uwp.user_id = u.id
            WHERE uwp.user_id = $1 AND uwp.season_id = $2",
        )
        .bind(user_id)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get player leaderboard entry: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Player not found in this season".into()))?;
        Ok(leaderboard_entry)
    }

    /// Get all wars points for a user across all seasons.
    pub async fn get_all_wars_points(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserWarsPoints>, AppError> {
        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "SELECT id, user_id, season_id, points, rank_badge, created_at, updated_at
            FROM user_wars_points
            WHERE user_id = $1
            ORDER BY season_id DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch all wars points: {}", e)))?;

        Ok(wars_points)
    }


    /// Get all users' wars points for a specific season.
    pub async fn get_season_wars_points(
        &self,
        season_id: i32,
    ) -> Result<Vec<UserWarsPoints>, AppError> {
        let wars_points = sqlx::query_as::<_, UserWarsPoints>(
            "SELECT id, user_id, season_id, points, rank_badge, created_at, updated_at
            FROM user_wars_points
            WHERE season_id = $1
            ORDER BY points DESC",
        )
        .bind(season_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch season wars points: {}", e))
        })?;

        Ok(wars_points)
    }
}
