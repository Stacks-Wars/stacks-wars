use sqlx::{FromRow, Row, query, query_as};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{Lobby, LobbyStatus, WalletAddress},
};

use super::LobbyRepository;

/// Intermediate struct for fetching lobbies with joined user and game data
#[derive(Debug, FromRow)]
pub struct LobbyWithJoins {
    // Lobby fields
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub description: Option<String>,
    pub game_id: Uuid,
    pub game_path: String,
    pub creator_id: Uuid,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub token_contract_id: Option<WalletAddress>,
    pub contract_address: Option<WalletAddress>,
    pub is_private: bool,
    pub is_sponsored: bool,
    pub status: LobbyStatus,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,

    // User fields
    pub creator_wallet_address: WalletAddress,
    pub creator_username: Option<String>,
    pub creator_display_name: Option<String>,

    // Game fields
    pub game_image_url: String,
    pub game_min_players: i16,
    pub game_max_players: i16,
}

impl LobbyRepository {
    /// Find a lobby by its ID.
    pub async fn find_by_id(&self, lobby_id: Uuid) -> Result<Lobby, AppError> {
        let lobby = query_as::<_, Lobby>("SELECT * FROM lobbies WHERE id = $1")
            .bind(lobby_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to fetch lobby: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Find a lobby by its path.
    pub async fn find_by_path(&self, path: &str) -> Result<Lobby, AppError> {
        let lobby = query_as::<_, Lobby>("SELECT * FROM lobbies WHERE path = $1")
            .bind(path)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to fetch lobby by path: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Lobby with path '{}' not found", path)))?;

        Ok(lobby)
    }

    /// Get all lobbies created by a specific user.
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

    /// Get all lobbies for a specific game.
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

    /// Get all lobbies with a specific status.
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

    /// List lobbies with pagination (limit/offset).
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

    /// Get active lobbies (waiting or in-progress).
    pub async fn get_active_lobbies(&self) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            r#"
            SELECT * FROM lobbies
            WHERE status IN ('waiting', 'starting', 'in_progress')
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch active lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get public (non-private) lobbies.
    pub async fn get_public_lobbies(&self) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies WHERE is_private = false ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch public lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get sponsored (free-entry) lobbies.
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

    /// Get lobbies for a given game and status.
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

    /// Count total lobbies.
    pub async fn count_lobbies(&self) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lobbies")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to count lobbies: {}", e)))?;

        Ok(count.0)
    }

    /// Count lobbies by status.
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

    /// Count lobbies by game.
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

    /// Check if a lobby exists by ID.
    pub async fn exists(&self, lobby_id: Uuid) -> Result<bool, AppError> {
        let result = query("SELECT EXISTS(SELECT 1 FROM lobbies WHERE lobby_id = $1)")
            .bind(lobby_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to check lobby existence: {}", e))
            })?;

        Ok(result.get(0))
    }

    /// Get lobbies by multiple statuses with pagination
    pub async fn find_by_statuses(
        &self,
        statuses: &[LobbyStatus],
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Lobby>, AppError> {
        if statuses.is_empty() {
            return self.find_all(offset, limit).await;
        }

        // Build dynamic query with status array
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies
             WHERE status = ANY($1)
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(statuses)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch lobbies by statuses: {}", e))
        })?;

        Ok(lobbies)
    }

    /// Get all lobbies with pagination (no status filter)
    pub async fn find_all(&self, offset: usize, limit: usize) -> Result<Vec<Lobby>, AppError> {
        let lobbies = query_as::<_, Lobby>(
            "SELECT * FROM lobbies ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch all lobbies: {}", e)))?;

        Ok(lobbies)
    }

    /// Get lobbies with creator and game information using optimized JOIN query
    pub async fn find_all_with_joins(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<LobbyWithJoins>, AppError> {
        let lobbies = query_as::<_, LobbyWithJoins>(
            r#"
            SELECT
                l.id, l.path, l.name, l.description, l.game_id, l.game_path,
                l.creator_id, l.entry_amount, l.current_amount, l.token_symbol,
                l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored,
                l.status, l.created_at, l.updated_at,
                u.wallet_address as creator_wallet_address,
                u.username as creator_username,
                u.display_name as creator_display_name,
                g.image_url as game_image_url,
                g.min_players as game_min_players,
                g.max_players as game_max_players
            FROM lobbies l
            INNER JOIN users u ON l.creator_id = u.id
            INNER JOIN games g ON l.game_id = g.id
            ORDER BY l.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch lobbies with joins: {}", e))
        })?;

        Ok(lobbies)
    }

    /// Get lobbies by multiple statuses with creator and game information using optimized JOIN query
    pub async fn find_by_statuses_with_joins(
        &self,
        statuses: &[LobbyStatus],
        offset: usize,
        limit: usize,
    ) -> Result<Vec<LobbyWithJoins>, AppError> {
        if statuses.is_empty() {
            return Ok(vec![]);
        }

        let lobbies = query_as::<_, LobbyWithJoins>(
            r#"
            SELECT
                l.id, l.path, l.name, l.description, l.game_id, l.game_path,
                l.creator_id, l.entry_amount, l.current_amount, l.token_symbol,
                l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored,
                l.status, l.created_at, l.updated_at,
                u.wallet_address as creator_wallet_address,
                u.username as creator_username,
                u.display_name as creator_display_name,
                g.image_url as game_image_url,
                g.min_players as game_min_players,
                g.max_players as game_max_players
            FROM lobbies l
            INNER JOIN users u ON l.creator_id = u.id
            INNER JOIN games g ON l.game_id = g.id
            WHERE l.status = ANY($1)
            ORDER BY l.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(statuses)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!(
                "Failed to fetch lobbies by statuses with joins: {}",
                e
            ))
        })?;

        Ok(lobbies)
    }

    /// Find a lobby by path with creator and game information using optimized JOIN query
    pub async fn find_by_path_with_joins(&self, path: &str) -> Result<LobbyWithJoins, AppError> {
        let lobby = query_as::<_, LobbyWithJoins>(
            r#"
            SELECT
                l.id, l.path, l.name, l.description, l.game_id, l.game_path,
                l.creator_id, l.entry_amount, l.current_amount, l.token_symbol,
                l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored,
                l.status, l.created_at, l.updated_at,
                u.wallet_address as creator_wallet_address,
                u.username as creator_username,
                u.display_name as creator_display_name,
                g.image_url as game_image_url,
                g.min_players as game_min_players,
                g.max_players as game_max_players
            FROM lobbies l
            INNER JOIN users u ON l.creator_id = u.id
            INNER JOIN games g ON l.game_id = g.id
            WHERE l.path = $1
            "#,
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch lobby by path with joins: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound(format!("Lobby with path '{}' not found", path)))?;

        Ok(lobby)
    }

    /// Find a lobby by ID with creator and game information using optimized JOIN query
    pub async fn find_by_id_with_joins(&self, lobby_id: Uuid) -> Result<LobbyWithJoins, AppError> {
        let lobby = query_as::<_, LobbyWithJoins>(
            r#"
            SELECT
                l.id, l.path, l.name, l.description, l.game_id, l.game_path,
                l.creator_id, l.entry_amount, l.current_amount, l.token_symbol,
                l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored,
                l.status, l.created_at, l.updated_at,
                u.wallet_address as creator_wallet_address,
                u.username as creator_username,
                u.display_name as creator_display_name,
                g.image_url as game_image_url,
                g.min_players as game_min_players,
                g.max_players as game_max_players
            FROM lobbies l
            INNER JOIN users u ON l.creator_id = u.id
            INNER JOIN games g ON l.game_id = g.id
            WHERE l.id = $1
            "#,
        )
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch lobby by ID with joins: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound(format!("Lobby with ID '{}' not found", lobby_id)))?;

        Ok(lobby)
    }
}

impl LobbyWithJoins {
    /// Convert to a plain Lobby struct
    pub fn to_lobby(self) -> Lobby {
        Lobby {
            id: self.id,
            path: self.path,
            name: self.name,
            description: self.description,
            game_id: self.game_id,
            game_path: self.game_path,
            creator_id: self.creator_id,
            entry_amount: self.entry_amount,
            current_amount: self.current_amount,
            token_symbol: self.token_symbol,
            token_contract_id: self.token_contract_id,
            contract_address: self.contract_address,
            is_private: self.is_private,
            is_sponsored: self.is_sponsored,
            status: self.status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    /// Extract creator information
    pub fn creator_info(&self) -> (WalletAddress, Option<String>, Option<String>) {
        (
            self.creator_wallet_address.clone(),
            self.creator_username.clone(),
            self.creator_display_name.clone(),
        )
    }

    /// Extract game information
    pub fn game_info(&self) -> (String, i16, i16) {
        (
            self.game_image_url.clone(),
            self.game_min_players,
            self.game_max_players,
        )
    }
}
