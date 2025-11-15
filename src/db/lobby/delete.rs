use sqlx::query;
use uuid::Uuid;

use crate::{errors::AppError, models::redis::LobbyStatus};

use super::LobbyRepository;

impl LobbyRepository {
    /// Delete a lobby by ID
    ///
    /// # Cascade Behavior
    /// Due to foreign key constraints, this will also delete:
    /// - Related chat messages (if cascade configured)
    /// - Join requests (if cascade configured)
    ///
    /// # Returns
    /// Number of rows deleted (0 or 1)
    pub async fn delete_lobby(&self, lobby_id: Uuid) -> Result<u64, AppError> {
        let result = query("DELETE FROM lobbies WHERE id = $1")
            .bind(lobby_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete lobby: {}", e)))?;

        if result.rows_affected() > 0 {
            tracing::info!("Deleted lobby: {}", lobby_id);
        }

        Ok(result.rows_affected())
    }

    /// Delete all lobbies created by a specific user
    ///
    /// # Use Case
    /// When a user account is deleted, clean up their lobbies.
    pub async fn delete_by_creator(&self, creator_id: Uuid) -> Result<u64, AppError> {
        let result = query("DELETE FROM lobbies WHERE creator_id = $1")
            .bind(creator_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to delete lobbies by creator: {}", e))
            })?;

        tracing::info!(
            "Deleted {} lobbies for creator {}",
            result.rows_affected(),
            creator_id
        );

        Ok(result.rows_affected())
    }

    /// Delete all lobbies for a specific game
    ///
    /// # Use Case
    /// When a game type is deprecated or removed.
    pub async fn delete_by_game(&self, game_id: Uuid) -> Result<u64, AppError> {
        let result = query("DELETE FROM lobbies WHERE game_id = $1")
            .bind(game_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to delete lobbies by game: {}", e))
            })?;

        tracing::info!(
            "Deleted {} lobbies for game {}",
            result.rows_affected(),
            game_id
        );

        Ok(result.rows_affected())
    }

    /// Delete all finished lobbies
    ///
    /// # Use Case
    /// Cleanup old finished lobbies to free up database space.
    /// Run this as a scheduled maintenance task.
    pub async fn delete_finished_lobbies(&self) -> Result<u64, AppError> {
        let result = query("DELETE FROM lobbies WHERE status = $1")
            .bind(LobbyStatus::Finished)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to delete finished lobbies: {}", e))
            })?;

        tracing::info!("Deleted {} finished lobbies", result.rows_affected());

        Ok(result.rows_affected())
    }

    /// Delete lobbies older than specified days
    ///
    /// # Arguments
    /// * `days` - Number of days to retain lobbies
    ///
    /// # Example
    /// ```rust
    /// // Delete lobbies older than 30 days
    /// repo.delete_old_lobbies(30).await?;
    /// ```
    pub async fn delete_old_lobbies(&self, days: i32) -> Result<u64, AppError> {
        let result = query("DELETE FROM lobbies WHERE created_at < NOW() - INTERVAL '1 day' * $1")
            .bind(days)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete old lobbies: {}", e)))?;

        tracing::info!(
            "Deleted {} lobbies older than {} days",
            result.rows_affected(),
            days
        );

        Ok(result.rows_affected())
    }

    /// Delete specific lobbies by IDs
    ///
    /// # Use Case
    /// Bulk deletion based on a list of lobby IDs.
    pub async fn delete_bulk(&self, lobby_ids: &[Uuid]) -> Result<u64, AppError> {
        if lobby_ids.is_empty() {
            return Ok(0);
        }

        let result = query("DELETE FROM lobbies WHERE id = ANY($1)")
            .bind(lobby_ids)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to bulk delete lobbies: {}", e))
            })?;

        tracing::info!("Bulk deleted {} lobbies", result.rows_affected());

        Ok(result.rows_affected())
    }

    /// Delete all lobbies (dangerous - use with caution!)
    ///
    /// # Warning
    /// This deletes ALL lobbies. Only use for testing or complete resets.
    #[cfg(debug_assertions)]
    pub async fn delete_all(&self) -> Result<u64, AppError> {
        let result = query("DELETE FROM lobbies")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete all lobbies: {}", e)))?;

        tracing::warn!("Deleted all {} lobbies", result.rows_affected());

        Ok(result.rows_affected())
    }
}
