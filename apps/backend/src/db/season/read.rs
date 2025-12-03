use crate::{errors::AppError, models::db::Season};

use super::SeasonRepository;

impl SeasonRepository {
    /// Return the ID of the currently active season, if any.
    pub async fn get_current_season(&self) -> Result<i32, AppError> {
        let now = chrono::Utc::now();

        let current_season_id = sqlx::query_scalar::<_, i32>(
            "SELECT id
            FROM seasons
            WHERE start_date <= $1 AND end_date >= $1
            ORDER BY start_date DESC
            LIMIT 1",
        )
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch current season: {}", e)))?
        .ok_or_else(|| AppError::NotFound("No active season found".into()))?;

        Ok(current_season_id)
    }

    /// Return the full `Season` for the currently active season.
    pub async fn get_current_season_full(&self) -> Result<Season, AppError> {
        let now = chrono::Utc::now();

        let season = sqlx::query_as::<_, Season>(
            "SELECT id, name, description, start_date, end_date, created_at
            FROM seasons
            WHERE start_date <= $1 AND end_date >= $1
            ORDER BY start_date DESC
            LIMIT 1",
        )
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch current season: {}", e)))?
        .ok_or_else(|| AppError::NotFound("No active season found".into()))?;

        Ok(season)
    }

    /// Find a `Season` by its ID.
    pub async fn find_by_id(&self, season_id: i32) -> Result<Season, AppError> {
        let season = sqlx::query_as::<_, Season>(
            "SELECT id, name, description, start_date, end_date, created_at
            FROM seasons
            WHERE id = $1",
        )
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch season: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Season not found".into()))?;

        Ok(season)
    }

    /// Find a `Season` by its name.
    pub async fn find_by_name(&self, name: &str) -> Result<Season, AppError> {
        let season = sqlx::query_as::<_, Season>(
            "SELECT id, name, description, start_date, end_date, created_at
            FROM seasons
            WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch season by name: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Season not found".into()))?;

        Ok(season)
    }

    /// List seasons (most recent first) with `limit` and `offset`.
    pub async fn get_all_seasons(&self, limit: i64, offset: i64) -> Result<Vec<Season>, AppError> {
        let seasons = sqlx::query_as::<_, Season>(
            "SELECT id, name, description, start_date, end_date, created_at
            FROM seasons
            ORDER BY start_date DESC
            LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch seasons: {}", e)))?;

        Ok(seasons)
    }

    /// Return past seasons (already ended), limited by `limit`.
    pub async fn get_past_seasons(&self, limit: i64) -> Result<Vec<Season>, AppError> {
        let now = chrono::Utc::now();

        let seasons = sqlx::query_as::<_, Season>(
            "SELECT id, name, description, start_date, end_date, created_at
            FROM seasons
            WHERE end_date < $1
            ORDER BY end_date DESC
            LIMIT $2",
        )
        .bind(now)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch past seasons: {}", e)))?;

        Ok(seasons)
    }

    /// Get upcoming seasons (starting after now)
    ///
    /// # Arguments
    /// * `limit` - Maximum number of seasons to return
    ///
    /// # Returns
    /// * `Ok(Vec<Season>)` - Upcoming seasons
    pub async fn get_upcoming_seasons(&self, limit: i64) -> Result<Vec<Season>, AppError> {
        let now = chrono::Utc::now();

        let seasons = sqlx::query_as::<_, Season>(
            "SELECT id, name, description, start_date, end_date, created_at
            FROM seasons
            WHERE start_date > $1
            ORDER BY start_date ASC
            LIMIT $2",
        )
        .bind(now)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch upcoming seasons: {}", e)))?;

        Ok(seasons)
    }

    /// Count total seasons
    ///
    /// # Returns
    /// * `Ok(i64)` - Total number of seasons
    pub async fn count_seasons(&self) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM seasons")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to count seasons: {}", e)))?;

        Ok(count)
    }

    /// Check if a season exists by ID
    ///
    /// # Arguments
    /// * `season_id` - Season ID
    ///
    /// # Returns
    /// * `Ok(bool)` - true if season exists
    pub async fn exists(&self, season_id: i32) -> Result<bool, AppError> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM seasons WHERE id = $1)")
                .bind(season_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to check season existence: {}", e))
                })?;

        Ok(exists)
    }
}
