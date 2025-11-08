use crate::errors::AppError;
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Delete a game (hard delete)
    ///
    /// **Warning**: This permanently removes the game from the database.
    /// Consider using `set_active(game_id, false)` instead for soft delete.
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    ///
    /// # Returns
    /// * `Ok(())` - Successfully deleted
    /// * `Err(AppError::NotFound)` - Game not found
    ///
    /// # Examples
    /// ```rust,ignore
    /// repo.delete_game(game_id).await?;
    /// ```
    pub async fn delete_game(&self, game_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "DELETE FROM games
            WHERE id = $1",
        )
        .bind(game_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete game: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Game not found".into()));
        }

        tracing::info!("Deleted game {}", game_id);

        Ok(())
    }

    /// Deactivate a game (soft delete)
    ///
    /// Sets `is_active = false`. Preferred over hard delete to maintain data integrity.
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    ///
    /// # Returns
    /// * `Ok(())` - Successfully deactivated
    pub async fn deactivate_game(&self, game_id: Uuid) -> Result<(), AppError> {
        self.set_active(game_id, false).await?;
        tracing::info!("Deactivated game {}", game_id);
        Ok(())
    }

    /// Reactivate a game
    ///
    /// Sets `is_active = true`.
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    ///
    /// # Returns
    /// * `Ok(())` - Successfully reactivated
    pub async fn reactivate_game(&self, game_id: Uuid) -> Result<(), AppError> {
        self.set_active(game_id, true).await?;
        tracing::info!("Reactivated game {}", game_id);
        Ok(())
    }

    /// Bulk deactivate games
    ///
    /// Deactivates multiple games at once.
    ///
    /// # Arguments
    /// * `game_ids` - List of game IDs
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of games deactivated
    pub async fn bulk_deactivate(&self, game_ids: Vec<Uuid>) -> Result<u64, AppError> {
        if game_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query(
            "UPDATE games
            SET is_active = FALSE, updated_at = NOW()
            WHERE id = ANY($1)",
        )
        .bind(&game_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to bulk deactivate games: {}", e)))?;

        let deactivated = result.rows_affected();
        tracing::info!("Bulk deactivated {} games", deactivated);

        Ok(deactivated)
    }

    /// Delete games by creator
    ///
    /// **Warning**: Permanently deletes all games created by a user.
    ///
    /// # Arguments
    /// * `creator_id` - Creator user ID
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of games deleted
    pub async fn delete_by_creator(&self, creator_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query(
            "DELETE FROM games
            WHERE creator_id = $1",
        )
        .bind(creator_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to delete games by creator: {}", e))
        })?;

        let deleted = result.rows_affected();
        tracing::info!("Deleted {} games by creator {}", deleted, creator_id);

        Ok(deleted)
    }
}
