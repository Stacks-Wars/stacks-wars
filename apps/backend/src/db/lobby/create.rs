use sqlx::query_as;
use uuid::Uuid;

use crate::{
    errors::AppError,
    http::bot::broadcasts::broadcast_lobby_creation_to_tg,
    models::{BotNewLobbyPayload, Lobby, LobbyState, LobbyStatus, PlayerState, WalletAddress, player_state::{ClaimState, PlayerStatus}},
    state::{AppState, RedisClient},
};

use super::LobbyRepository;
use crate::db::{
    lobby_state::LobbyStateRepository, player_state::PlayerStateRepository, user::UserRepository,
};

impl LobbyRepository {
    /// Create a new lobby and return the created `Lobby`.
    pub async fn create_lobby(
        &self,
        name: &str,
        description: Option<&str>,
        creator_id: Uuid,
        game_id: Uuid,
        game_path: &str,
        entry_amount: Option<f64>,
        current_amount: Option<f64>,
        token_symbol: Option<&str>,
        token_contract_id: Option<&str>,
        contract_address: Option<&str>,
        is_private: bool,
        is_sponsored: bool,
        redis: RedisClient,
        state: AppState,
    ) -> Result<Lobby, AppError> {
        // Validate amounts based on sponsor status
        let (entry_amount, current_amount) =
            Lobby::validate_creation_amounts(entry_amount, current_amount, is_sponsored)?;

        // Validate and parse contract addresses
        let token_contract_id = if let Some(addr) = token_contract_id {
            Some(WalletAddress::new(addr)?)
        } else {
            None
        };

        let contract_address = if let Some(addr) = contract_address {
            Some(WalletAddress::new(addr)?)
        } else {
            None
        };
        let lobby_future = query_as::<_, Lobby>(
            r#"
            INSERT INTO lobbies (
                name, description, creator_id, game_id, game_path,
                entry_amount, current_amount, token_symbol, token_contract_id,
                contract_address, is_private, is_sponsored,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, path, name, description, game_id, game_path, creator_id,
                      entry_amount, current_amount, token_symbol, token_contract_id,
                      contract_address, is_private, is_sponsored, status,
                      created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(creator_id)
        .bind(game_id)
        .bind(game_path)
        .bind(entry_amount)
        .bind(current_amount)
        .bind(token_symbol)
        .bind(token_contract_id.as_ref())
        .bind(contract_address.as_ref())
        .bind(is_private)
        .bind(is_sponsored)
        .bind(LobbyStatus::Waiting)
        .fetch_one(&self.pool);

        let user_repo = UserRepository::new(self.pool.clone());
        let user_future = user_repo.find_by_id(creator_id);

        // Run both queries in parallel
        let (lobby_result, creator_result) = tokio::join!(lobby_future, user_future);

        let lobby = lobby_result.map_err(|e| {
            AppError::DatabaseError(format!("Failed to create lobby '{}': {}", name, e))
        })?;

        let creator = creator_result.map_err(|e| {
            let _ = self.delete_lobby(lobby.id(), None);
            AppError::DatabaseError(format!("Failed to fetch creator user: {}", e))
        })?;

        let lobby_state_repo = LobbyStateRepository::new(redis.clone());
        let player_repo = PlayerStateRepository::new(redis.clone());

        // Sponsored lobby creators start as spectators (NotJoined), so participant_count starts at 0.
        // Normal lobby creators are auto-joined, so participant_count starts at 1.
        let initial_count = if is_sponsored { 0 } else { 1 };
        let lstate = LobbyState::new(lobby.id(), initial_count);
        if let Err(e) = lobby_state_repo.create_state(lstate).await {
            let _ = self.delete_lobby(lobby.id(), None).await;
            return Err(AppError::RedisError(format!(
                "Failed to create lobby state in Redis for {}: {}",
                lobby.id(),
                e
            )));
        }

        let claim_state = if contract_address.is_some() {
            Some(ClaimState::NotClaimed)
        } else {
            None
        };

        let creator_username = creator.username.clone();
        let creator_display_name = creator.display_name.clone();

        // Sponsored lobby creators start as spectators (NotJoined) and can opt-in to participate.
        // Normal lobby creators are automatically joined.
        let creator_status = if is_sponsored {
            PlayerStatus::NotJoined
        } else {
            PlayerStatus::Joined
        };

        let creator_pstate = PlayerState::new(
            creator_id,
            lobby.id(),
            creator.wallet_address.to_string(),
            creator.username,
            creator.display_name,
            creator.trust_rating,
            claim_state,
            true,
            creator_status,
        );
        if let Err(e) = player_repo.create_state(creator_pstate, None).await {
            let _ = self.delete_lobby(lobby.id(), None).await;
            return Err(AppError::RedisError(format!(
                "Failed to create creator player state in Redis for {}: {}",
                lobby.id(),
                e
            )));
        }

        tracing::info!("Created lobby: {} (path: {})", lobby.name, lobby.path);

        // Broadcast lobby creation to lobby list subscribers
        crate::ws::broadcast::broadcast_lobby_creation(state.clone(), lobby.id(), game_id, creator_id)
            .await;

        // Broadcast lobby creation to Telegram
        let payload = BotNewLobbyPayload {
            lobby: lobby.clone(),
            creator_name: creator_display_name.or(creator_username),
            wallet_address: creator.wallet_address.as_ref().to_string(),
        };
        broadcast_lobby_creation_to_tg(state, payload).await;

        Ok(lobby)
    }
}
