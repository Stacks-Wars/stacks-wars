use crate::{
    models::{game::Player, lobby::LobbyServerMessage},
    state::{ConnectionInfoMap, RedisClient},
    ws::handlers::lobby::message_handler::{
        broadcast_to_lobby,
        handler::{get_pending_players, request_to_join, send_error_to_player, send_to_player},
    },
};
use uuid::Uuid;

pub async fn request_join(
    player: &Player,
    lobby_id: Uuid,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    let user = player.clone().into();

    match request_to_join(lobby_id, user, redis.clone()).await {
        Ok(_) => {
            if let Ok(pending_players) = get_pending_players(lobby_id, redis.clone()).await {
                tracing::info!("Success Adding {} to pending players", player.id);
                let msg = LobbyServerMessage::Pending;
                send_to_player(player.id, lobby_id, &connections, &msg, &redis).await;

                let msg = LobbyServerMessage::PendingPlayers { pending_players };
                broadcast_to_lobby(lobby_id, &msg, &connections, None, redis.clone()).await;
                send_to_player(player.id, lobby_id, &connections, &msg, &redis).await;
            }
        }
        Err(e) => {
            tracing::error!("Failed to mark user as pending: {}", e);
            send_error_to_player(
                player.id,
                lobby_id,
                "Failed to send join request",
                &connections,
                &redis,
            )
            .await;
        }
    }
}
