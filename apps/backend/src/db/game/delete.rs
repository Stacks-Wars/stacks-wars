use crate::errors::AppError;
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Hard-delete a game (permanent). Prefer `deactivate_game` for soft-delete.
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

    /// Soft-delete a game by setting `is_active = false`.
    pub async fn deactivate_game(&self, game_id: Uuid) -> Result<(), AppError> {
        self.set_active(game_id, false).await?;
        tracing::info!("Deactivated game {}", game_id);
        Ok(())
    }

    /// Reactivate a previously deactivated game.
    pub async fn reactivate_game(&self, game_id: Uuid) -> Result<(), AppError> {
        self.set_active(game_id, true).await?;
        tracing::info!("Reactivated game {}", game_id);
        Ok(())
    }

    /// Deactivate multiple games; returns number deactivated.
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

    /// Permanently delete all games created by `creator_id` (use with caution).
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
