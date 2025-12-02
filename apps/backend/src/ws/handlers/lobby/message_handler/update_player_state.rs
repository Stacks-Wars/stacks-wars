use crate::{
    db::lobby::{
        get::{get_lobby_info, get_lobby_players},
        patch::{self, update_lobby_state},
    },
    models::{
        game::{LobbyState, Player, PlayerState},
        lobby::LobbyServerMessage,
    },
    state::{ChatConnectionInfoMap, ConnectionInfoMap, RedisClient},
    ws::handlers::lobby::message_handler::{broadcast_to_lobby, handler::send_error_to_player},
};
use uuid::Uuid;

pub async fn update_player_state(
    new_state: PlayerState,
    lobby_id: Uuid,
    player: &Player,
    connections: &ConnectionInfoMap,
    chat_connections: &ChatConnectionInfoMap,
    redis: &RedisClient,
) {
    if let Err(e) =
        patch::update_player_state(lobby_id, player.id, new_state.clone(), redis.clone()).await
    {
        tracing::error!("Failed to update state: {}", e);
        send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
    } else if let Ok(players) =
        get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await
    {
        tracing::info!(
            "Player {} updated state to {:?} in lobby {}",
            player.id,
            new_state,
            lobby_id
        );
        let msg = LobbyServerMessage::PlayerUpdated { players };
        broadcast_to_lobby(
            lobby_id,
            &msg,
            &connections,
            Some(&chat_connections),
            redis.clone(),
        )
        .await;

        if new_state == PlayerState::NotJoined {
            if let Ok(lobby_info) = get_lobby_info(lobby_id, redis.clone()).await {
                if lobby_info.state == LobbyState::Starting {
                    // revert game state to Waiting
                    let _ = update_lobby_state(lobby_id, LobbyState::Waiting, redis.clone()).await;
                    let msg = LobbyServerMessage::LobbyState {
                        state: LobbyState::Waiting,
                        joined_players: None,
                        started: false,
                    };
                    broadcast_to_lobby(lobby_id, &msg, &connections, None, redis.clone()).await;
                }
            }
        }
    }
}
