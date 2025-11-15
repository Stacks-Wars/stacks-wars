use chrono::Utc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{db::Lobby, redis::LobbyStatus},
};

use super::LobbyRepository;

impl LobbyRepository {
    /// Create a new lobby in the database
    ///
    /// # Arguments
    /// * `name` - Lobby display name
    /// * `description` - Optional lobby description
    /// * `creator_id` - User ID of the lobby creator
    /// * `game_id` - Game type ID
    /// * `entry_amount` - Optional entry fee amount
    /// * `token_symbol` - Token symbol (e.g., "STX")
    /// * `token_contract_id` - Token contract identifier
    /// * `contract_address` - Smart contract address
    /// * `is_private` - Whether lobby is private (requires approval)
    /// * `is_sponsored` - Whether lobby is sponsored (free entry)
    ///
    /// # Returns
    /// Created `Lobby` with generated UUID and timestamps
    ///
    /// # Errors
    /// - `AppError::DatabaseError` if insertion fails
    /// - Foreign key violations if creator_id or game_id don't exist
    pub async fn create_lobby(
        &self,
        name: String,
        description: Option<String>,
        creator_id: Uuid,
        game_id: Uuid,
        entry_amount: Option<f64>,
        token_symbol: Option<String>,
        token_contract_id: Option<String>,
        contract_address: Option<String>,
        is_private: bool,
        is_sponsored: bool,
    ) -> Result<Lobby, AppError> {
        let lobby = query_as::<_, Lobby>(
            r#"
            INSERT INTO lobbies (
                name, description, creator_id, game_id,
                entry_amount, token_symbol, token_contract_id,
                contract_address, is_private, is_sponsored,
                status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)
            RETURNING *
            "#,
        )
        .bind(&name)
        .bind(description)
        .bind(creator_id)
        .bind(game_id)
        .bind(entry_amount)
        .bind(token_symbol)
        .bind(token_contract_id)
        .bind(contract_address)
        .bind(is_private)
        .bind(is_sponsored)
        .bind(LobbyStatus::Waiting)
        .bind(Utc::now().naive_utc())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to create lobby '{}': {}", name, e))
        })?;

        tracing::info!("Created lobby: {} ({})", lobby.name, lobby.id);
        Ok(lobby)
    }

    /// Create a sponsored lobby (free entry, no fees)
    ///
    /// Convenience method for creating lobbies without entry fees.
    /// Sets `is_sponsored = true` and `entry_amount = 0`.
    pub async fn create_sponsored_lobby(
        &self,
        name: String,
        description: Option<String>,
        creator_id: Uuid,
        game_id: Uuid,
        is_private: bool,
    ) -> Result<Lobby, AppError> {
        self.create_lobby(
            name,
            description,
            creator_id,
            game_id,
            Some(0.0),
            None,
            None,
            None,
            is_private,
            true,
        )
        .await
    }

    /// Create a private lobby (requires join approval)
    ///
    /// Convenience method for creating invite-only lobbies.
    /// Sets `is_private = true`.
    pub async fn create_private_lobby(
        &self,
        name: String,
        description: Option<String>,
        creator_id: Uuid,
        game_id: Uuid,
        entry_amount: Option<f64>,
        token_symbol: Option<String>,
    ) -> Result<Lobby, AppError> {
        self.create_lobby(
            name,
            description,
            creator_id,
            game_id,
            entry_amount,
            token_symbol,
            None,
            None,
            true,
            false,
        )
        .await
    }
}
