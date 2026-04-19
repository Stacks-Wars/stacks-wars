//! [`stacks_wars_core::GameHost`] implementation for [`crate::state::AppState`].

use crate::db::player_state::PlayerStateRepository;
use crate::games::common::{finish_lobby, save_player_result, save_player_result_with_winner};
use crate::models::player_state::{ClaimState as ModelClaimState, PlayerState, PlayerStatus as ModelPlayerStatus};
use crate::state::AppState;
use crate::ws::broadcast;
use crate::ws::room::messages::RoomServerMessage;
use async_trait::async_trait;
use stacks_wars_core::{
    ClaimState as WireClaimState, CoreError, GameHost, GameHostRef, GameRoomBroadcast,
    JoinRequestState as WireJoinState, PlayerResult, PlayerStateWire, PlayerStatus as WirePlayerStatus,
    UserRoomMessage, WarsPointContext,
};
use uuid::Uuid;

/// Newtype so `Arc<KernelGameHost>` can be downcast from [`GameHostRef`] while legacy engines keep using [`AppState`].
#[derive(Clone)]
pub struct KernelGameHost(pub AppState);

/// Recover [`AppState`] from the kernel [`GameHostRef`] (panics if not `KernelGameHost`).
#[allow(dead_code)]
pub fn kernel_app_state(host: &GameHostRef) -> AppState {
    GameHost::as_any(host.as_ref())
        .downcast_ref::<KernelGameHost>()
        .expect("Stacks Wars kernel GameHost")
        .0
        .clone()
}

fn app_err_to_core(e: crate::errors::AppError) -> CoreError {
    match e {
        crate::errors::AppError::BadRequest(m) => CoreError::BadRequest(m),
        crate::errors::AppError::NotFound(m) => CoreError::NotFound(m),
        crate::errors::AppError::RedisError(m) | crate::errors::AppError::RedisPoolError(m) => {
            CoreError::Redis(m)
        }
        crate::errors::AppError::DatabaseError(m) => CoreError::Database(m),
        crate::errors::AppError::Serialization(m) => CoreError::Serialization(m),
        crate::errors::AppError::Deserialization(m) => CoreError::Deserialization(m),
        _ => CoreError::Internal,
    }
}

fn wire_join_to_model(s: &WireJoinState) -> crate::db::join_request::JoinRequestState {
    match s {
        WireJoinState::Pending => crate::db::join_request::JoinRequestState::Pending,
        WireJoinState::Accepted => crate::db::join_request::JoinRequestState::Accepted,
        WireJoinState::Rejected => crate::db::join_request::JoinRequestState::Rejected,
    }
}

fn model_join_to_wire(s: &crate::db::join_request::JoinRequestState) -> WireJoinState {
    match s {
        crate::db::join_request::JoinRequestState::Pending => WireJoinState::Pending,
        crate::db::join_request::JoinRequestState::Accepted => WireJoinState::Accepted,
        crate::db::join_request::JoinRequestState::Rejected => WireJoinState::Rejected,
    }
}

fn wire_status_to_model(s: &WirePlayerStatus) -> ModelPlayerStatus {
    match s {
        WirePlayerStatus::NotJoined => ModelPlayerStatus::NotJoined,
        WirePlayerStatus::Joined => ModelPlayerStatus::Joined,
    }
}

fn model_status_to_wire(s: &ModelPlayerStatus) -> WirePlayerStatus {
    match s {
        ModelPlayerStatus::NotJoined => WirePlayerStatus::NotJoined,
        ModelPlayerStatus::Joined => WirePlayerStatus::Joined,
    }
}

fn wire_claim_to_model(c: &WireClaimState) -> ModelClaimState {
    match c {
        WireClaimState::Claimed { tx_id } => ModelClaimState::Claimed {
            tx_id: tx_id.clone(),
        },
        WireClaimState::NotClaimed => ModelClaimState::NotClaimed,
    }
}

fn model_claim_to_wire(c: &ModelClaimState) -> WireClaimState {
    match c {
        ModelClaimState::Claimed { tx_id } => WireClaimState::Claimed {
            tx_id: tx_id.clone(),
        },
        ModelClaimState::NotClaimed => WireClaimState::NotClaimed,
    }
}

pub fn player_state_to_wire(p: &PlayerState) -> PlayerStateWire {
    PlayerStateWire {
        user_id: p.user_id,
        wallet_address: p.wallet_address.clone(),
        username: p.username.clone(),
        display_name: p.display_name.clone(),
        trust_rating: p.trust_rating,
        lobby_id: p.lobby_id,
        status: model_status_to_wire(&p.status),
        state: model_join_to_wire(&p.state),
        rank: p.rank,
        prize: p.prize,
        wars_point: p.wars_point,
        claim_state: p.claim_state.as_ref().map(model_claim_to_wire),
        last_ping: p.last_ping,
        joined_at: p.joined_at,
        updated_at: p.updated_at,
        is_creator: p.is_creator,
    }
}

fn player_wire_to_model(w: &PlayerStateWire) -> PlayerState {
    PlayerState {
        user_id: w.user_id,
        wallet_address: w.wallet_address.clone(),
        username: w.username.clone(),
        display_name: w.display_name.clone(),
        trust_rating: w.trust_rating,
        lobby_id: w.lobby_id,
        status: wire_status_to_model(&w.status),
        state: wire_join_to_model(&w.state),
        rank: w.rank,
        prize: w.prize,
        wars_point: w.wars_point,
        claim_state: w.claim_state.as_ref().map(wire_claim_to_model),
        last_ping: w.last_ping,
        joined_at: w.joined_at,
        updated_at: w.updated_at,
        is_creator: w.is_creator,
    }
}

fn room_broadcast_to_message(msg: &GameRoomBroadcast) -> RoomServerMessage {
    match msg {
        GameRoomBroadcast::GameStarted => RoomServerMessage::GameStarted,
        GameRoomBroadcast::GameStartFailed { reason } => RoomServerMessage::GameStartFailed {
            reason: reason.clone(),
        },
        GameRoomBroadcast::FinalStanding { standings } => RoomServerMessage::FinalStanding {
            standings: standings.iter().map(player_wire_to_model).collect(),
        },
    }
}

fn user_room_to_message(msg: &UserRoomMessage) -> RoomServerMessage {
    match msg {
        UserRoomMessage::GameOver {
            rank,
            prize,
            wars_point,
        } => RoomServerMessage::GameOver {
            rank: *rank,
            prize: *prize,
            wars_point: *wars_point,
        },
    }
}

#[async_trait]
impl GameHost for KernelGameHost {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn broadcast_room_game(&self, lobby_id: Uuid, msg: &GameRoomBroadcast) {
        let m = room_broadcast_to_message(msg);
        broadcast::broadcast_room(&self.0, lobby_id, &m).await;
    }

    async fn broadcast_user_room(&self, user_id: Uuid, msg: &UserRoomMessage) {
        let m = user_room_to_message(msg);
        broadcast::broadcast_user(&self.0, user_id, &m).await;
    }

    async fn broadcast_game_message(&self, lobby_id: Uuid, payload: serde_json::Value) {
        broadcast::broadcast_game_message(&self.0, lobby_id, payload).await;
    }

    async fn broadcast_game_message_to_user(
        &self,
        _lobby_id: Uuid,
        user_id: Uuid,
        payload: serde_json::Value,
    ) {
        broadcast::broadcast_game_message_to_user(&self.0, user_id, payload).await;
    }

    async fn broadcast_game_message_to_room_except(
        &self,
        lobby_id: Uuid,
        except_user_id: Uuid,
        payload: serde_json::Value,
    ) {
        broadcast::broadcast_game_message_to_room_except(&self.0, lobby_id, except_user_id, payload)
            .await;
    }

    async fn save_player_result(
        &self,
        lobby_id: Uuid,
        ctx: &WarsPointContext,
    ) -> Result<PlayerResult, CoreError> {
        save_player_result(&self.0, lobby_id, ctx)
            .await
            .map_err(app_err_to_core)
    }

    async fn save_player_result_with_winner(
        &self,
        lobby_id: Uuid,
        ctx: &WarsPointContext,
        is_winner: bool,
    ) -> Result<PlayerResult, CoreError> {
        save_player_result_with_winner(&self.0, lobby_id, ctx, is_winner)
            .await
            .map_err(app_err_to_core)
    }

    async fn finish_lobby(&self, lobby_id: Uuid) -> Result<(), CoreError> {
        finish_lobby(&self.0, lobby_id).await.map_err(app_err_to_core)
    }

    async fn get_player_states_in_lobby(&self, lobby_id: Uuid) -> Result<Vec<PlayerStateWire>, CoreError> {
        let repo = PlayerStateRepository::new(self.0.redis.clone());
        let rows = repo
            .get_all_in_lobby(lobby_id)
            .await
            .map_err(app_err_to_core)?;
        Ok(rows.iter().map(player_state_to_wire).collect())
    }
}
