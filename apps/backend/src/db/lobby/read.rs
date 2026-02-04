use sqlx::{FromRow, Row, query, query_as};
use uuid::Uuid;
use crate::state::RedisClient;

use crate::{
    errors::AppError,
    models::{Game, Lobby, LobbyInfo, LobbyExtended, LobbyState, LobbyStatus, User},
};
use crate::db::lobby_state::LobbyStateRepository;

use super::LobbyRepository;

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

    /// Find a lobby by UUID or path.
    pub async fn find_by_identifier(&self, identifier: &str) -> Result<Lobby, AppError> {
        // Try parsing as UUID first
        if let Ok(lobby_id) = Uuid::parse_str(identifier) {
            if let Ok(lobby) = self.find_by_id(lobby_id).await {
                tracing::debug!("Found lobby by UUID: {}", lobby.id);
                return Ok(lobby);
            }
        }

        // Fallback to path lookup
        if let Ok(lobby) = self.find_by_path(identifier).await {
            tracing::debug!("Found lobby by path: {}", lobby.id);
            return Ok(lobby);
        }

        tracing::debug!("Lobby not found for identifier: {}", identifier);
        Err(AppError::NotFound(format!(
            "Lobby not found for identifier: {}",
            identifier
        )))
    }

    /// Get all lobbies created by a specific user.
    pub async fn find_by_creator(
        &self,
        creator_id: Uuid,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        let rows = query(
            "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE creator_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(creator_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch creator lobbies: {}", e)))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);
        let lobbies = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
    }

    /// Get all lobbies for a specific game.
    pub async fn find_by_game_id(
        &self,
        game_id: Uuid,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        let rows = query(
            "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE game_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(game_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch game lobbies: {}", e)))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);
        let lobbies = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
    }

    /// Get all lobbies with a specific status.
    pub async fn find_by_status(
        &self,
        status: LobbyStatus,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        let rows = query(
            "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(status)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch lobbies by status: {}", e))
        })?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);
        let lobbies = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
    }

    /// List lobbies with pagination (limit/offset).
    /// Returns LobbyInfo including game, creator, and state data.
    pub async fn get_all_lobbies(
        &self,
        limit: i64,
        offset: i64,
        redis: &RedisClient,
    ) -> Result<(Vec<LobbyInfo>, i64), AppError> {
        let rows = query(
            "SELECT
                l.id, l.path, l.name, l.description, l.game_id, l.game_path, l.creator_id, l.entry_amount, l.current_amount, l.token_symbol, l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored, l.status, l.created_at, l.updated_at,
                g.id as game_id_col, g.name as game_name, g.path as game_path_col, g.description as game_description, g.image_url, g.min_players, g.max_players, g.category, g.creator_id as game_creator_id, g.is_active, g.updated_at as game_updated_at, g.created_at as game_created_at,
                u.id as user_id_col, u.wallet_address, u.username, u.display_name, u.email, u.email_verified, u.trust_rating, u.profile_image, u.created_at as user_created_at, u.updated_at as user_updated_at,
                COUNT(*) OVER() as total
            FROM lobbies l
            JOIN games g ON l.game_id = g.id
            JOIN users u ON l.creator_id = u.id
            ORDER BY l.created_at DESC
            LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch all lobbies: {}", e)))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);

        if rows.is_empty() {
            return Ok((vec![], total));
        }

        // Parse joined data
        let mut joined_data = Vec::new();
        for row in rows {
            let lobby = Lobby::from_row(&row)
                .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

            let game = Game {
                id: row.get("game_id_col"),
                name: row.get("game_name"),
                path: row.get("game_path_col"),
                description: row.get("game_description"),
                image_url: row.get("image_url"),
                min_players: row.get("min_players"),
                max_players: row.get("max_players"),
                category: row.get("category"),
                creator_id: row.get("game_creator_id"),
                is_active: row.get("is_active"),
                updated_at: row.get("game_updated_at"),
                created_at: row.get("game_created_at"),
            };

            let user = User {
                id: row.get("user_id_col"),
                wallet_address: row.get("wallet_address"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                trust_rating: row.get("trust_rating"),
                profile_image: row.get("profile_image"),
                created_at: row.get("user_created_at"),
                updated_at: row.get("user_updated_at"),
            };

            joined_data.push((lobby, game, user));
        }

        // Get lobby IDs for state fetching
        let lobby_ids: Vec<Uuid> = joined_data.iter().map(|(l, _, _)| l.id).collect();

        // Batch fetch lobby states
        let lobby_state_repo = LobbyStateRepository::new(redis.clone());
        let states_batch = lobby_state_repo
            .get_states_batch(&lobby_ids)
            .await
            .map_err(|e| AppError::RedisError(format!("Failed to fetch lobby states: {}", e)))?;

        // Construct LobbyInfo objects
        let mut lobby_info_list = Vec::new();
        for (lobby, game, creator) in joined_data {
            let state_opt = states_batch
                .iter()
                .find(|(id, _)| *id == lobby.id)
                .map(|(_, s)| s.clone())
                .flatten();
            let state = state_opt.unwrap_or_else(|| LobbyState::new(lobby.id));

            let extended = LobbyExtended::from_parts(lobby, state);

            let lobby_info = LobbyInfo {
                lobby: extended,
                game,
                creator,
            };

            lobby_info_list.push(lobby_info);
        }

        Ok((lobby_info_list, total))
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
    pub async fn get_public_lobbies(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        let rows = query(
            "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE is_private = false ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch public lobbies: {}", e)))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);
        let lobbies = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
    }

    /// Get sponsored (free-entry) lobbies.
    pub async fn get_sponsored_lobbies(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        let rows = query(
            "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE is_sponsored = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch sponsored lobbies: {}", e))
        })?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);
        let lobbies = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
    }

    /// Get lobbies for a given game (by ID or path) and statuses with pagination.
    pub async fn find_by_game_and_status(
        &self,
        game_identifier: &str,
        statuses: &[LobbyStatus],
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        // Try to parse as UUID first
        let query = if let Ok(game_id) = Uuid::parse_str(game_identifier) {
            if statuses.is_empty() {
                sqlx::query(
                    "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE game_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(game_id)
                .bind(limit as i64)
                .bind(offset as i64)
            } else {
                sqlx::query(
                    "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE game_id = $1 AND status = ANY($2) ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(game_id)
                .bind(statuses)
                .bind(limit as i64)
                .bind(offset as i64)
            }
        } else {
            if statuses.is_empty() {
                sqlx::query(
                    "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE game_path = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(game_identifier)
                .bind(limit as i64)
                .bind(offset as i64)
            } else {
                sqlx::query(
                    "SELECT *, COUNT(*) OVER() as total FROM lobbies WHERE game_path = $1 AND status = ANY($2) ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(game_identifier)
                .bind(statuses)
                .bind(limit as i64)
                .bind(offset as i64)
            }
        };

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to fetch lobbies by game and status: {}", e))
            })?;

        let total = if let Some(row) = rows.first() {
            row.get::<i64, _>("total")
        } else {
            0
        };

        let lobbies: Vec<Lobby> = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
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
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        if statuses.is_empty() {
            return self.find_all(offset, limit).await;
        }

        // Build dynamic query with status array
        let rows = query(
            "SELECT *, COUNT(*) OVER() as total FROM lobbies
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

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);
        let lobbies = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
    }

    /// Get all lobbies with pagination (no status filter)
    pub async fn find_all(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Lobby>, i64), AppError> {
        let rows = query(
            "SELECT *, COUNT(*) OVER() as total FROM lobbies ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch all lobbies: {}", e)))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total"))
            .unwrap_or(0);
        let lobbies = rows
            .into_iter()
            .map(|row| Lobby::from_row(&row))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

        Ok((lobbies, total))
    }

    /// TODO: REVIEW
    /// Find lobbies by game identifier (UUID or path) and statuses with pagination.
    /// Returns LobbyInfo including game, creator, and state data.
    pub async fn find_lobbies_by_game_and_status(
        &self,
        game_identifier: &str,
        statuses: &[LobbyStatus],
        limit: usize,
        offset: usize,
        redis: &RedisClient,
    ) -> Result<(Vec<LobbyInfo>, i64), AppError> {
        // Single optimized query with JOINs
        let query = if let Ok(game_id) = Uuid::parse_str(game_identifier) {
            if statuses.is_empty() {
                query(
                    "SELECT
                        l.id, l.path, l.name, l.description, l.game_id, l.game_path, l.creator_id, l.entry_amount, l.current_amount, l.token_symbol, l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored, l.status, l.created_at, l.updated_at,
                        g.id as game_id_col, g.name as game_name, g.path as game_path_col, g.description as game_description, g.image_url, g.min_players, g.max_players, g.category, g.creator_id as game_creator_id, g.is_active, g.updated_at as game_updated_at, g.created_at as game_created_at,
                        u.id as user_id_col, u.wallet_address, u.username, u.display_name, u.email, u.email_verified, u.trust_rating, u.profile_image, u.created_at as user_created_at, u.updated_at as user_updated_at,
                        COUNT(*) OVER() as total
                    FROM lobbies l
                    JOIN games g ON l.game_id = g.id
                    JOIN users u ON l.creator_id = u.id
                    WHERE l.game_id = $1
                    ORDER BY l.created_at DESC
                    LIMIT $2 OFFSET $3"
                )
                .bind(game_id)
                .bind(limit as i64)
                .bind(offset as i64)
            } else {
                query(
                    "SELECT
                        l.id, l.path, l.name, l.description, l.game_id, l.game_path, l.creator_id, l.entry_amount, l.current_amount, l.token_symbol, l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored, l.status, l.created_at, l.updated_at,
                        g.id as game_id_col, g.name as game_name, g.path as game_path_col, g.description as game_description, g.image_url, g.min_players, g.max_players, g.category, g.creator_id as game_creator_id, g.is_active, g.updated_at as game_updated_at, g.created_at as game_created_at,
                        u.id as user_id_col, u.wallet_address, u.username, u.display_name, u.email, u.email_verified, u.trust_rating, u.profile_image, u.created_at as user_created_at, u.updated_at as user_updated_at,
                        COUNT(*) OVER() as total
                    FROM lobbies l
                    JOIN games g ON l.game_id = g.id
                    JOIN users u ON l.creator_id = u.id
                    WHERE l.game_id = $1 AND l.status = ANY($2)
                    ORDER BY l.created_at DESC
                    LIMIT $3 OFFSET $4"
                )
                .bind(game_id)
                .bind(statuses)
                .bind(limit as i64)
                .bind(offset as i64)
            }
        } else {
            if statuses.is_empty() {
                query(
                    "SELECT
                        l.id, l.path, l.name, l.description, l.game_id, l.game_path, l.creator_id, l.entry_amount, l.current_amount, l.token_symbol, l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored, l.status, l.created_at, l.updated_at,
                        g.id as game_id_col, g.name as game_name, g.path as game_path_col, g.description as game_description, g.image_url, g.min_players, g.max_players, g.category, g.creator_id as game_creator_id, g.is_active, g.updated_at as game_updated_at, g.created_at as game_created_at,
                        u.id as user_id_col, u.wallet_address, u.username, u.display_name, u.email, u.email_verified, u.trust_rating, u.profile_image, u.created_at as user_created_at, u.updated_at as user_updated_at,
                        COUNT(*) OVER() as total
                    FROM lobbies l
                    JOIN games g ON l.game_id = g.id
                    JOIN users u ON l.creator_id = u.id
                    WHERE l.game_path = $1
                    ORDER BY l.created_at DESC
                    LIMIT $2 OFFSET $3"
                )
                .bind(game_identifier)
                .bind(limit as i64)
                .bind(offset as i64)
            } else {
                query(
                    "SELECT
                        l.id, l.path, l.name, l.description, l.game_id, l.game_path, l.creator_id, l.entry_amount, l.current_amount, l.token_symbol, l.token_contract_id, l.contract_address, l.is_private, l.is_sponsored, l.status, l.created_at, l.updated_at,
                        g.id as game_id_col, g.name as game_name, g.path as game_path_col, g.description as game_description, g.image_url, g.min_players, g.max_players, g.category, g.creator_id as game_creator_id, g.is_active, g.updated_at as game_updated_at, g.created_at as game_created_at,
                        u.id as user_id_col, u.wallet_address, u.username, u.display_name, u.email, u.email_verified, u.trust_rating, u.profile_image, u.created_at as user_created_at, u.updated_at as user_updated_at,
                        COUNT(*) OVER() as total
                    FROM lobbies l
                    JOIN games g ON l.game_id = g.id
                    JOIN users u ON l.creator_id = u.id
                    WHERE l.game_path = $1 AND l.status = ANY($2)
                    ORDER BY l.created_at DESC
                    LIMIT $3 OFFSET $4"
                )
                .bind(game_identifier)
                .bind(statuses)
                .bind(limit as i64)
                .bind(offset as i64)
            }
        };

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to fetch lobbies: {}", e)))?;

        let total = rows.first().map(|row| row.get::<i64, _>("total")).unwrap_or(0);

        if rows.is_empty() {
            return Ok((vec![], total));
        }

        // Parse joined data
        let mut joined_data = Vec::new();
        for row in rows {
            let lobby = Lobby::from_row(&row).map_err(|e| AppError::DatabaseError(format!("Failed to parse lobby: {}", e)))?;

            let game = Game {
                id: row.get("game_id_col"),
                name: row.get("game_name"),
                path: row.get("game_path_col"),
                description: row.get("game_description"),
                image_url: row.get("image_url"),
                min_players: row.get("min_players"),
                max_players: row.get("max_players"),
                category: row.get("category"),
                creator_id: row.get("game_creator_id"),
                is_active: row.get("is_active"),
                updated_at: row.get("game_updated_at"),
                created_at: row.get("game_created_at"),
            };

            let user = User {
                id: row.get("user_id_col"),
                wallet_address: row.get("wallet_address"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                trust_rating: row.get("trust_rating"),
                profile_image: row.get("profile_image"),
                created_at: row.get("user_created_at"),
                updated_at: row.get("user_updated_at"),
            };

            joined_data.push((lobby, game, user));
        }

        // Get lobby IDs for state fetching
        let lobby_ids: Vec<Uuid> = joined_data.iter().map(|(l, _, _)| l.id).collect();

        // Batch fetch lobby states
        let lobby_state_repo = LobbyStateRepository::new(redis.clone());
        let states_batch = lobby_state_repo
            .get_states_batch(&lobby_ids)
            .await
            .map_err(|e| AppError::RedisError(format!("Failed to fetch lobby states: {}", e)))?;

        // Construct LobbyInfo objects
        let mut lobby_info_list = Vec::new();
        for (lobby, game, creator) in joined_data {
            let state_opt = states_batch.iter().find(|(id, _)| *id == lobby.id).map(|(_, s)| s.clone()).flatten();
            let state = state_opt.unwrap_or_else(|| LobbyState::new(lobby.id));

            let extended = LobbyExtended::from_parts(lobby, state);

            let lobby_info = LobbyInfo {
                lobby: extended,
                game,
                creator,
            };

            lobby_info_list.push(lobby_info);
        }

        Ok((lobby_info_list, total))
    }
}
