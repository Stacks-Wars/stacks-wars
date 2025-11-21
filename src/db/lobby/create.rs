use chrono::Utc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        db::Lobby,
        redis::{LobbyState, LobbyStatus, PlayerState},
    },
    state::RedisClient,
};

use super::LobbyRepository;
use crate::db::{lobby_state::LobbyStateRepository, player_state::PlayerStateRepository};

impl LobbyRepository {
    /// Create a new lobby and return the created `Lobby`.
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
        redis: RedisClient,
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

        let lobby_state_repo = LobbyStateRepository::new(redis.clone());
        let player_repo = PlayerStateRepository::new(redis.clone());

        let lstate = LobbyState::new(lobby.id);
        if let Err(e) = lobby_state_repo.create_state(lstate).await {
            let _ = self.delete_lobby(lobby.id).await;
            return Err(AppError::RedisError(format!(
                "Failed to create lobby state in Redis for {}: {}",
                lobby.id, e
            )));
        }

        let creator_pstate = PlayerState::new(creator_id, lobby.id, None, true);
        if let Err(e) = player_repo.create_state(creator_pstate).await {
            let _ = self.delete_lobby(lobby.id).await;
            return Err(AppError::RedisError(format!(
                "Failed to create creator player state in Redis for {}: {}",
                lobby.id, e
            )));
        }

        tracing::info!("Created lobby: {} ({})", lobby.name, lobby.id);
        Ok(lobby)
    }
}
