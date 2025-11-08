use sqlx::query_as;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::db::{Lobby, lobby::LobbyStatus},
};

use super::LobbyRepository;

impl LobbyRepository {
    /// Find a lobby by its ID
    ///
    /// # Returns
    /// - `Ok(Some(Lobby))` if found
    /// - `Ok(None)` if not found
    /// - `Err` on database error
    pub async fn find_by_id(&self, lobby_id: Uuid) -> Result<Option<Lobby>, AppError> {
        let lobby = query_as::<_, Lobby>("SELECT * FROM lobbies WHERE id = $1")
            .bind(lobby_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to fetch lobby: {}", e)))?;

        Ok(lobby)
    }

    /// Get a lobby by ID, returning error if not found
    ///
    /// Convenience method that returns `AppError::NotFound` instead of `Option`.
    pub async fn get_by_id(&self, lobby_id: Uuid) -> Result<Lobby, AppError> {
        self.find_by_id(lobby_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))
    }

    /// Get all lobbies created by a specific user
    pub async fn find_by_creator(&self, creator_id: Uuid) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies WHERE creator_id = $1 ORDER BY created_at DESC",
        )
        .bind(creator_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch creator lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get all lobbies for a specific game
    pub async fn find_by_game_id(&self, game_id: Uuid) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies WHERE game_id = $1 ORDER BY created_at DESC",
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch game lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get all lobbies with a specific status
    ///
    /// # Example
    /// ```rust
    /// let waiting_lobbies = repo.find_by_status(LobbyStatus::Waiting).await?;
    /// let active_lobbies = repo.find_by_status(LobbyStatus::InProgress).await?;
    /// ```
    pub async fn find_by_status(&self, status: LobbyStatus) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies WHERE status = $1 ORDER BY created_at DESC",
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch lobbies by status: {}", e))
        })?;

        Ok(lobbies)
    }

    /// Get all lobbies (with pagination)
    ///
    /// # Arguments
    /// * `limit` - Maximum number of lobbies to return
    /// * `offset` - Number of lobbies to skip
    pub async fn get_all_lobbies(&self, limit: i64, offset: i64) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch all lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get active lobbies (waiting or in progress)
    ///
    /// Returns lobbies that players can join or are currently playing.
    pub async fn get_active_lobbies(&self) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            r#"
            SELECT * FROM lobbies
            WHERE status IN ('waiting', 'starting', 'inprogress')
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch active lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get public lobbies (not private)
    pub async fn get_public_lobbies(&self) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies WHERE is_private = false ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch public lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get sponsored lobbies (free entry)
    pub async fn get_sponsored_lobbies(&self) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies WHERE is_sponsored = true ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch sponsored lobbies: {}", e))
        })?;

        Ok(lobbies)
    }

    /// Get lobbies by game and status
    ///
    /// Useful for finding joinable lobbies for a specific game.
    pub async fn find_by_game_and_status(
        &self,
        game_id: Uuid,
        status: LobbyStatus,
    ) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies WHERE game_id = $1 AND status = $2 ORDER BY created_at DESC",
        )
        .bind(game_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch lobbies by game and status: {}", e))
        })?;

        Ok(lobbies)
    }

    /// Count total lobbies
    pub async fn count_lobbies(&self) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lobbies")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to count lobbies: {}", e)))?;

        Ok(count.0)
    }

    /// Count lobbies by status
    pub async fn count_by_status(&self, status: LobbyStatus) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lobbies WHERE status = $1")
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to count lobbies by status: {}", e))
            })?;

        Ok(count.0)
    }

    /// Count lobbies by game
    pub async fn count_by_game(&self, game_id: Uuid) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lobbies WHERE game_id = $1")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to count lobbies by game: {}", e))
            })?;

        Ok(count.0)
    }

    /// Check if a lobby exists
    pub async fn exists(&self, lobby_id: Uuid) -> Result<bool, AppError> {
        let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM lobbies WHERE id = $1)")
            .bind(lobby_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to check lobby existence: {}", e))
            })?;

        Ok(exists.0)
    }
}
