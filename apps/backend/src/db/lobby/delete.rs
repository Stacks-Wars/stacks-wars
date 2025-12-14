use sqlx::query;
use uuid::Uuid;

use crate::{errors::AppError, models::LobbyStatus};

use super::LobbyRepository;

impl LobbyRepository {
    /// Delete a lobby by ID (returns number of rows deleted).
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

    /// Delete all lobbies created by a specific user.
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

    /// Delete all lobbies for a specific game.
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

    /// Delete all finished lobbies (cleanup task).
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

    /// Delete lobbies older than the given number of days.
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

    /// Delete specific lobbies by IDs (bulk delete).
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

    /// Delete all lobbies (debug only).
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
