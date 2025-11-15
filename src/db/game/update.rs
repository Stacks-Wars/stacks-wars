use crate::{errors::AppError, models::db::game::Game};
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Update game name
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    /// * `name` - New name (must be unique)
    ///
    /// # Returns
    /// * `Ok(Game)` - Updated game
    /// * `Err(AppError::BadRequest)` - Name already taken
    pub async fn update_name(&self, game_id: Uuid, name: String) -> Result<Game, AppError> {
        // Check if name is already taken by another game
        let existing = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT id FROM games WHERE name = $1 AND id != $2",
        )
        .bind(&name)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to check game name: {}", e)))?;

        if existing.is_some() {
            return Err(AppError::BadRequest(format!(
                "Game name '{}' is already taken",
                name
            )));
        }

        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET name = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(&name)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game name: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} name to '{}'", game_id, name);

        Ok(game)
    }

    /// Update game description
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    /// * `description` - New description
    ///
    /// # Returns
    /// * `Ok(Game)` - Updated game
    pub async fn update_description(
        &self,
        game_id: Uuid,
        description: String,
    ) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET description = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(&description)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game description: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} description", game_id);

        Ok(game)
    }

    /// Update game image URL
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    /// * `image_url` - New image URL
    ///
    /// # Returns
    /// * `Ok(Game)` - Updated game
    pub async fn update_image_url(
        &self,
        game_id: Uuid,
        image_url: String,
    ) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET image_url = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(&image_url)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game image: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} image URL", game_id);

        Ok(game)
    }

    /// Update player limits
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    /// * `min_players` - Minimum players (must be >= 1)
    /// * `max_players` - Maximum players (must be >= min_players)
    ///
    /// # Returns
    /// * `Ok(Game)` - Updated game
    /// * `Err(AppError::BadRequest)` - Invalid player limits
    pub async fn update_player_limits(
        &self,
        game_id: Uuid,
        min_players: i16,
        max_players: i16,
    ) -> Result<Game, AppError> {
        // Validate
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

        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET min_players = $1, max_players = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(min_players)
        .bind(max_players)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update player limits: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!(
            "Updated game {} player limits: {}-{}",
            game_id,
            min_players,
            max_players
        );

        Ok(game)
    }

    /// Update game category
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    /// * `category` - New category
    ///
    /// # Returns
    /// * `Ok(Game)` - Updated game
    pub async fn update_category(
        &self,
        game_id: Uuid,
        category: Option<String>,
    ) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET category = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(&category)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game category: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} category to {:?}", game_id, category);

        Ok(game)
    }

    /// Toggle game active status
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    /// * `is_active` - New active status
    ///
    /// # Returns
    /// * `Ok(Game)` - Updated game
    pub async fn set_active(&self, game_id: Uuid, is_active: bool) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET is_active = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(is_active)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game status: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Set game {} active status to {}", game_id, is_active);

        Ok(game)
    }

    /// Update game (partial update)
    ///
    /// Updates only the provided fields. None values are ignored.
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    /// * `name` - Optional new name
    /// * `description` - Optional new description
    /// * `image_url` - Optional new image URL
    /// * `min_players` - Optional new min players
    /// * `max_players` - Optional new max players
    /// * `category` - Optional new category
    /// * `is_active` - Optional new active status
    ///
    /// # Returns
    /// * `Ok(Game)` - Updated game
    pub async fn update_game(
        &self,
        game_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        image_url: Option<String>,
        min_players: Option<i16>,
        max_players: Option<i16>,
        category: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Game, AppError> {
        // Fetch current game
        let current = self.find_by_id(game_id).await?;

        let new_name = name.unwrap_or(current.name.clone());
        let new_description = description.unwrap_or(current.description);
        let new_image_url = image_url.unwrap_or(current.image_url);
        let new_min = min_players.unwrap_or(current.min_players);
        let new_max = max_players.unwrap_or(current.max_players);
        let new_category = category.or(current.category);
        let new_active = is_active.unwrap_or(current.is_active);

        // Validate player limits
        if new_min < 1 {
            return Err(AppError::BadRequest(
                "Minimum players must be at least 1".into(),
            ));
        }
        if new_max < new_min {
            return Err(AppError::BadRequest(
                "Maximum players must be >= minimum players".into(),
            ));
        }

        // Check name uniqueness if changed
        if new_name != current.name {
            let existing = sqlx::query_scalar::<_, Option<Uuid>>(
                "SELECT id FROM games WHERE name = $1 AND id != $2",
            )
            .bind(&new_name)
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to check game name: {}", e)))?;

            if existing.is_some() {
                return Err(AppError::BadRequest(format!(
                    "Game name '{}' is already taken",
                    new_name
                )));
            }
        }

        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET name = $1, description = $2, image_url = $3, min_players = $4, max_players = $5,
                category = $6, is_active = $7, updated_at = NOW()
            WHERE id = $8
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(&new_name)
        .bind(&new_description)
        .bind(&new_image_url)
        .bind(new_min)
        .bind(new_max)
        .bind(&new_category)
        .bind(new_active)
        .bind(game_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game: {}", e)))?;

        tracing::info!("Updated game {}", game_id);

        Ok(game)
    }
}
