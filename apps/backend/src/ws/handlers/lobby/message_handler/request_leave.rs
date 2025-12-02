use uuid::Uuid;

use crate::{
    db::lobby::get::get_connected_players_ids,
    models::{game::Player, lobby::LobbyServerMessage},
    state::{ConnectionInfoMap, RedisClient},
    ws::handlers::lobby::message_handler::handler::send_to_player,
};

pub async fn request_leave(
    player: &Player,
    lobby_id: Uuid,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    let is_connected = match get_connected_players_ids(lobby_id, redis.clone()).await {
        Ok(connected_players) => connected_players.contains(&player.id),
        Err(e) => {
            tracing::error!(
                "Failed to get connected players for lobby {}: {}",
                lobby_id,
                e
            );
            // If we can't check, assume they're connected to be safe
            true
        }
    };

    let response_msg = LobbyServerMessage::IsConnectedPlayer {
        response: is_connected,
    };

    send_to_player(player.id, lobby_id, connections, &response_msg, redis).await;

    tracing::debug!(
        "Player {} requested to leave lobby {}, is_connected: {}",
        player.id,
        lobby_id,
        is_connected
    );
}
