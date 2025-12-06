use crate::{errors::AppError, models::db::game::Game};
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Update a game's name (must be unique).
    /// DB constraint handles uniqueness automatically.
    pub async fn update_name(&self, game_id: Uuid, name: &str) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET name = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(name)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::BadRequest(format!("Game name '{}' is already taken", name));
                }
            }
            AppError::DatabaseError(format!("Failed to update game name: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} name to '{}'", game_id, name);

        Ok(game)
    }

    /// Update a game's description.
    pub async fn update_description(
        &self,
        game_id: Uuid,
        description: &str,
    ) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET description = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(description)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game description: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} description", game_id);

        Ok(game)
    }

    /// Update a game's image URL.
    pub async fn update_image_url(&self, game_id: Uuid, image_url: &str) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET image_url = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(image_url)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game image: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} image URL", game_id);

        Ok(game)
    }

    /// Update a game's min/max player limits.
    /// Validates player counts internally.
    pub async fn update_player_limits(
        &self,
        game_id: Uuid,
        min_players: i16,
        max_players: i16,
    ) -> Result<Game, AppError> {
        // Validate at repository level
        let (min_players, max_players) = Game::validate_player_count(min_players, max_players)?;

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

    /// Update a game's category (optional).
    pub async fn update_category(
        &self,
        game_id: Uuid,
        category: Option<&str>,
    ) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET category = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(category)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update game category: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Updated game {} category to {:?}", game_id, category);

        Ok(game)
    }

    /// Set a game's `is_active` status.
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

    /// Partially update a game's fields; unspecified fields are unchanged.
    /// Validates player counts internally. DB constraint handles name uniqueness.
    pub async fn update_game(
        &self,
        game_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        image_url: Option<&str>,
        min_players: Option<i16>,
        max_players: Option<i16>,
        category: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Game, AppError> {
        // Fetch current game
        let current = self.find_by_id(game_id).await?;

        let new_name = name.unwrap_or(&current.name);
        let new_description = description.unwrap_or(&current.description);
        let new_image_url = image_url.unwrap_or(&current.image_url);
        let new_min = min_players.unwrap_or(current.min_players);
        let new_max = max_players.unwrap_or(current.max_players);
        let new_category = category.or(current.category.as_deref());
        let new_active = is_active.unwrap_or(current.is_active);

        // Validate player limits
        let (new_min, new_max) = Game::validate_player_count(new_min, new_max)?;

        let game = sqlx::query_as::<_, Game>(
            "UPDATE games
            SET name = $1, description = $2, image_url = $3, min_players = $4, max_players = $5,
                category = $6, is_active = $7, updated_at = NOW()
            WHERE id = $8
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, updated_at, created_at",
        )
        .bind(new_name)
        .bind(new_description)
        .bind(new_image_url)
        .bind(new_min)
        .bind(new_max)
        .bind(new_category)
        .bind(new_active)
        .bind(game_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::BadRequest(format!("Game name '{}' is already taken", new_name));
                }
            }
            AppError::DatabaseError(format!("Failed to update game: {}", e))
        })?;

        tracing::info!("Updated game {}", game_id);

        Ok(game)
    }
}
