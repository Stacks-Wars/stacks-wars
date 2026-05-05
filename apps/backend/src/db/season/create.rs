use crate::{errors::AppError, models::Season};

use super::SeasonRepository;

impl SeasonRepository {
    /// Create a new season.
    pub async fn create_season(
        &self,
        name: &str,
        description: Option<&str>,
        start_date: &str,
        end_date: &str,
    ) -> Result<Season, AppError> {
        let (start_date, end_date) = Season::parse_date_range(start_date, end_date)?;

        // Try to insert season
        let season = sqlx::query_as::<_, Season>(
            "INSERT INTO seasons (name, description, start_date, end_date)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, start_date, end_date, created_at",
        )
        .bind(name)
        .bind(description)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::BadRequest(format!(
                        "Season with name '{}' already exists",
                        name
                    ));
                }
            }
            tracing::error!("Database error creating season: {}", e);
            AppError::DatabaseError(format!("Failed to create season: {}", e))
        })?;

        tracing::info!("Created new season: {} (ID: {})", season.name, season.id());
        Ok(season)
    }
}
