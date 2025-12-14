use crate::{errors::AppError, models::game::Game};
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Create a new game type.
    pub async fn create_game(
        &self,
        name: &str,
        path: &str,
        description: &str,
        image_url: &str,
        min_players: i16,
        max_players: i16,
        category: Option<&str>,
        creator_id: Uuid,
    ) -> Result<Game, AppError> {
        // Validate player counts
        let (min_players, max_players) = Game::validate_player_count(min_players, max_players)?;

        // Validate path format (lowercase, alphanumeric + hyphens)
        if !path
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(AppError::BadRequest(
                "Game path must be lowercase alphanumeric with hyphens only".to_string(),
            ));
        }

        if path.is_empty() || path.len() > 50 {
            return Err(AppError::BadRequest(
                "Game path must be between 1-50 characters".to_string(),
            ));
        }

        let game = sqlx::query_as::<_, Game>(
            "INSERT INTO games (name, path, description, image_url, min_players, max_players, category, creator_id, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
            RETURNING id, name, path, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(name)
        .bind(path)
        .bind(description)
        .bind(image_url)
        .bind(min_players)
        .bind(max_players)
        .bind(category)
        .bind(creator_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::BadRequest(format!("Game with name '{}' or path '{}' already exists", name, path));
                }
            }
            AppError::DatabaseError(format!("Failed to create game: {}", e))
        })?;

        tracing::info!(
            "Created new game: {} (ID: {}, path: {})",
            game.name,
            game.id(),
            game.path
        );
        Ok(game)
    }
}
