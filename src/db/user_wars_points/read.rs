use crate::{errors::AppError, models::db::UserWarsPoints};
use uuid::Uuid;

use super::UserWarsPointsRepository;

impl UserWarsPointsRepository {
    /// Get user wars points for a specific season
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `season_id` - Season ID
    ///
    /// # Returns
    /// * `Ok(UserWarsPoints)` - Wars points data for the season
    /// * `Err(AppError::NotFound)` - No entry for this user/season combo
    ///
    /// # Examples
    /// ```rust,ignore
    /// let points = repo.get_wars_points(user_id, season_id).await?;
    /// println!("User has {} points in season {}", points.points, season_id);
    /// ```
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

    /// Get all wars points for a user across all seasons
    ///
    /// Returns wars points history ordered by season (most recent first).
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    ///
    /// # Returns
    /// * `Ok(Vec<UserWarsPoints>)` - All wars points entries for this user
    ///
    /// # Examples
    /// ```rust,ignore
    /// let history = repo.get_all_wars_points(user_id).await?;
    /// for entry in history {
    ///     println!("Season {}: {} points", entry.season_id, entry.points);
    /// }
    /// ```
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

    /// Get top users by wars points for a season (leaderboard)
    ///
    /// Returns the leaderboard with wars points and wallet addresses for display.
    ///
    /// # Arguments
    /// * `season_id` - Season ID
    /// * `limit` - Number of top users to return
    ///
    /// # Returns
    /// * `Ok(Vec<(UserWarsPoints, String)>)` - Wars points with wallet addresses
    ///
    /// # Examples
    /// ```rust,ignore
    /// let top_10 = repo.get_leaderboard(season_id, 10).await?;
    /// for (rank, (points, wallet)) in top_10.iter().enumerate() {
    ///     println!("{}. {} - {} points", rank + 1, wallet, points.points);
    /// }
    /// ```
    pub async fn get_leaderboard(
        &self,
        season_id: i32,
        limit: i64,
    ) -> Result<Vec<(UserWarsPoints, String)>, AppError> {
        let results = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                i32,
                f64,
                Option<String>,
                chrono::NaiveDateTime,
                chrono::NaiveDateTime,
                String,
            ),
        >(
            "SELECT uwp.id, uwp.user_id, uwp.season_id, uwp.points, uwp.rank_badge,
                    uwp.created_at, uwp.updated_at, u.wallet_address
            FROM user_wars_points uwp
            JOIN users u ON uwp.user_id = u.id
            WHERE uwp.season_id = $1
            ORDER BY uwp.points DESC
            LIMIT $2",
        )
        .bind(season_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get leaderboard: {}", e)))?;

        let leaderboard: Vec<(UserWarsPoints, String)> = results
            .into_iter()
            .map(
                |(id, user_id, season_id, points, rank_badge, created_at, updated_at, wallet)| {
                    (
                        UserWarsPoints {
                            id,
                            user_id,
                            season_id,
                            points,
                            rank_badge,
                            created_at,
                            updated_at,
                        },
                        wallet,
                    )
                },
            )
            .collect();

        Ok(leaderboard)
    }

    /// Get all users' wars points for a specific season
    ///
    /// Useful for batch operations or analytics.
    ///
    /// # Arguments
    /// * `season_id` - Season ID
    ///
    /// # Returns
    /// * `Ok(Vec<UserWarsPoints>)` - All wars points for this season
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
