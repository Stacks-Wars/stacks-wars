use crate::{errors::AppError, models::game::GameV2};
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Create a new game type
    ///
    /// Returns existing game if name already exists (idempotent).
    ///
    /// # Arguments
    /// * `name` - Game name (must be unique)
    /// * `description` - Game description
    /// * `image_url` - Game thumbnail/logo URL
    /// * `min_players` - Minimum players required
    /// * `max_players` - Maximum players allowed
    /// * `category` - Optional game category
    /// * `creator_id` - User who created this game type
    ///
    /// # Returns
    /// * `Ok(GameV2)` - Created or existing game
    ///
    /// # Examples
    /// ```rust,ignore
    /// let game = repo.create_game(
    ///     "Lexi Wars".into(),
    ///     "Word battle game".into(),
    ///     "https://example.com/lexi.png".into(),
    ///     2,
    ///     4,
    ///     Some("Word Games".into()),
    ///     creator_id
    /// ).await?;
    /// ```
    pub async fn create_game(
        &self,
        name: String,
        description: String,
        image_url: String,
        min_players: i16,
        max_players: i16,
        category: Option<String>,
        creator_id: Uuid,
    ) -> Result<GameV2, AppError> {
        // Validate player limits
        if min_players < 1 {
            return Err(AppError::BadRequest(
                "Minimum players must be at least 1".into(),
            ));
        }
        if max_players < min_players {
            return Err(AppError::BadRequest(
                "Maximum players must be >= minimum players".into(),
            ));
        }

        // Check if game already exists (by name)
        let existing_game = sqlx::query_as::<_, GameV2>(
            "SELECT id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at
            FROM games
            WHERE name = $1",
        )
        .bind(&name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query game: {}", e)))?;

        if let Some(game) = existing_game {
            tracing::info!("Game already exists: {}", name);
            return Ok(game);
        }

        // Create new game
        let game_id = Uuid::new_v4();

        let game = sqlx::query_as::<_, GameV2>(
            "INSERT INTO games (id, name, description, image_url, min_players, max_players, category, creator_id, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(game_id)
        .bind(&name)
        .bind(&description)
        .bind(&image_url)
        .bind(min_players)
        .bind(max_players)
        .bind(&category)
        .bind(creator_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create game: {}", e)))?;

        tracing::info!("Created new game: {} (ID: {})", game.name, game.id);

        Ok(game)
    }
}
