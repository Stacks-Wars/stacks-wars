use crate::{errors::AppError, models::db::game::Game};
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Create a new game type.
    pub async fn create_game(
        &self,
        name: &str,
        description: &str,
        image_url: &str,
        min_players: i16,
        max_players: i16,
        category: Option<&str>,
        creator_id: Uuid,
    ) -> Result<Game, AppError> {
        // Validate player counts
        let (min_players, max_players) = Game::validate_player_count(min_players, max_players)?;

        let game = sqlx::query_as::<_, Game>(
            "INSERT INTO games (name, description, image_url, min_players, max_players, category, creator_id, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE)
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(name)
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
                    return AppError::BadRequest(format!("Game with name '{}' already exists", name));
                }
            }
            AppError::DatabaseError(format!("Failed to create game: {}", e))
        })?;

        tracing::info!("Created new game: {} (ID: {})", game.name, game.id());
        Ok(game)
    }
}
