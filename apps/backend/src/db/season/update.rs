use crate::{errors::AppError, models::db::Season};
use chrono::NaiveDateTime;

use super::SeasonRepository;

impl SeasonRepository {
    /// Update season name (ensures uniqueness).
    pub async fn update_name(&self, season_id: i32, name: String) -> Result<Season, AppError> {
        // Check if name is already taken by another season
        let existing = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT id FROM seasons WHERE name = $1 AND id != $2",
        )
        .bind(&name)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to check season name: {}", e)))?;

        if existing.is_some() {
            return Err(AppError::BadRequest(format!(
                "Season name '{}' is already taken",
                name
            )));
        }

        let season = sqlx::query_as::<_, Season>(
            "UPDATE seasons
            SET name = $1
            WHERE id = $2
            RETURNING id, name, description, start_date, end_date, created_at",
        )
        .bind(&name)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update season name: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Season not found".into()))?;

        tracing::info!("Updated season {} name to '{}'", season_id, name);

        Ok(season)
    }

    /// Update season description.
    pub async fn update_description(
        &self,
        season_id: i32,
        description: Option<String>,
    ) -> Result<Season, AppError> {
        let season = sqlx::query_as::<_, Season>(
            "UPDATE seasons
            SET description = $1
            WHERE id = $2
            RETURNING id, name, description, start_date, end_date, created_at",
        )
        .bind(&description)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update season description: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound("Season not found".into()))?;

        tracing::info!("Updated season {} description", season_id);

        Ok(season)
    }

    /// Update season dates (validates end_date > start_date).
    pub async fn update_dates(
        &self,
        season_id: i32,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<Season, AppError> {
        // Validate dates
        if end_date <= start_date {
            return Err(AppError::BadRequest(
                "End date must be after start date".into(),
            ));
        }

        let season = sqlx::query_as::<_, Season>(
            "UPDATE seasons
            SET start_date = $1, end_date = $2
            WHERE id = $3
            RETURNING id, name, description, start_date, end_date, created_at",
        )
        .bind(start_date)
        .bind(end_date)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update season dates: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Season not found".into()))?;

        tracing::info!("Updated season {} dates", season_id);

        Ok(season)
    }

    /// Partially update a season (only provided fields are changed).
    pub async fn update_season(
        &self,
        season_id: i32,
        name: Option<String>,
        description: Option<String>,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>,
    ) -> Result<Season, AppError> {
        // Fetch current season
        let current = self.find_by_id(season_id).await?;

        let new_name = name.unwrap_or(current.name.clone());
        let new_description = description.or(current.description);
        let new_start = start_date.unwrap_or(current.start_date);
        let new_end = end_date.unwrap_or(current.end_date);

        // Validate dates
        if new_end <= new_start {
            return Err(AppError::BadRequest(
                "End date must be after start date".into(),
            ));
        }

        // Check name uniqueness if changed
        if new_name != current.name {
            let existing = sqlx::query_scalar::<_, Option<i32>>(
                "SELECT id FROM seasons WHERE name = $1 AND id != $2",
            )
            .bind(&new_name)
            .bind(season_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to check season name: {}", e)))?;

            if existing.is_some() {
                return Err(AppError::BadRequest(format!(
                    "Season name '{}' is already taken",
                    new_name
                )));
            }
        }

        let season = sqlx::query_as::<_, Season>(
            "UPDATE seasons
            SET name = $1, description = $2, start_date = $3, end_date = $4
            WHERE id = $5
            RETURNING id, name, description, start_date, end_date, created_at",
        )
        .bind(&new_name)
        .bind(&new_description)
        .bind(new_start)
        .bind(new_end)
        .bind(season_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update season: {}", e)))?;

        tracing::info!("Updated season {}", season_id);

        Ok(season)
    }

    /// Extend a season's end date by a number of days.
    pub async fn extend_season(&self, season_id: i32, days: i64) -> Result<Season, AppError> {
        let current = self.find_by_id(season_id).await?;
        let new_end = current.end_date + chrono::Duration::days(days);

        self.update_dates(season_id, current.start_date, new_end)
            .await
    }
}
