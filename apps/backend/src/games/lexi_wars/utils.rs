use futures::SinkExt;
use rand::{Rng, rng};

use crate::{
    db::lobby::get::get_spectators,
    models::{game::Player, lexi_wars::LexiWarsServerMessage},
    state::{ConnectionInfoMap, RedisClient},
    ws::handlers::utils::queue_message_for_player,
};
use uuid::Uuid;

pub fn generate_random_letter() -> char {
    let letter = rng().random_range(0..26);
    (b'a' + letter as u8) as char
}

pub async fn broadcast_to_player(
    player_id: Uuid,
    lobby_id: Uuid,
    msg: &LexiWarsServerMessage,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    let serialized = match serde_json::to_string(msg) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to serialize message: {}", e);
            return;
        }
    };

    // Check if player is currently connected
    let conns = connections.lock().await;
    if let Some(conn_info) = conns.get(&player_id) {
        // Player is connected, send directly
        let mut sender_guard = conn_info.sender.lock().await;
        if let Err(e) = sender_guard
            .send(axum::extract::ws::Message::Text(serialized.clone().into()))
            .await
        {
            tracing::debug!(
                "Failed to send direct message to player {}: {}",
                player_id,
                e
            );
            // Connection failed, queue the message if it should be queued
            if msg.should_queue() {
                let _ = queue_message_for_player(player_id, lobby_id, serialized, redis).await;
            }
        }
    } else {
        // Player not connected, queue if message should be queued
        if msg.should_queue() {
            let _ = queue_message_for_player(player_id, lobby_id, serialized, redis).await;
        }
    }
}

pub async fn broadcast_to_lobby(
    msg: &LexiWarsServerMessage,
    players: &[Player],
    lobby_id: Uuid,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    for player in players {
        broadcast_to_player(player.id, lobby_id, msg, connections, redis).await;
    }
}

pub async fn broadcast_to_spectators(
    msg: &LexiWarsServerMessage,
    lobby_id: Uuid,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    if let Ok(spectator_ids) = get_spectators(lobby_id, redis.clone()).await {
        for spectator_id in spectator_ids {
            broadcast_to_player(spectator_id, lobby_id, msg, connections, redis).await;
        }
    }
}

pub async fn broadcast_to_lobby_and_spectators(
    msg: &LexiWarsServerMessage,
    players: &[Player],
    lobby_id: Uuid,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    // Broadcast to players
    for player in players {
        broadcast_to_player(player.id, lobby_id, msg, connections, redis).await;
    }

    // Broadcast to spectators
    broadcast_to_spectators(msg, lobby_id, connections, redis).await;
}

pub async fn broadcast_to_player_and_spectators(
    msg: &LexiWarsServerMessage,
    player_id: Uuid,
    lobby_id: Uuid,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    broadcast_to_player(player_id, lobby_id, msg, connections, redis).await;
    broadcast_to_spectators(msg, lobby_id, connections, redis).await;
}
