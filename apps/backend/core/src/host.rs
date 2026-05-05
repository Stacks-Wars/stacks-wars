use crate::dto::PlayerStateWire;
use crate::error::CoreError;
use crate::kit::{PlayerResult, WarsPointContext};
use crate::wire::{GameRoomBroadcast, UserRoomMessage};
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use std::sync::Arc;
use uuid::Uuid;

/// Platform services available to game engines (implemented by the server kernel).
#[async_trait]
pub trait GameHost: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;

    async fn broadcast_room_game(&self, lobby_id: Uuid, msg: &GameRoomBroadcast);

    async fn broadcast_user_room(&self, user_id: Uuid, msg: &UserRoomMessage);

    async fn broadcast_game_message(&self, lobby_id: Uuid, payload: Value);

    async fn broadcast_game_message_to_user(&self, lobby_id: Uuid, user_id: Uuid, payload: Value);

    async fn broadcast_game_message_to_room_except(
        &self,
        lobby_id: Uuid,
        except_user_id: Uuid,
        payload: Value,
    );

    async fn save_player_result(
        &self,
        lobby_id: Uuid,
        ctx: &WarsPointContext,
    ) -> Result<PlayerResult, CoreError>;

    async fn save_player_result_with_winner(
        &self,
        lobby_id: Uuid,
        ctx: &WarsPointContext,
        is_winner: bool,
    ) -> Result<PlayerResult, CoreError>;

    async fn finish_lobby(&self, lobby_id: Uuid) -> Result<(), CoreError>;

    async fn get_player_states_in_lobby(&self, lobby_id: Uuid) -> Result<Vec<PlayerStateWire>, CoreError>;
}

pub type GameHostRef = Arc<dyn GameHost>;
