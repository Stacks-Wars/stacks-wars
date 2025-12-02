use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::errors::AppError;
use crate::models::redis::{KeyPart, RedisKey};
use crate::state::ConnectionInfoMap;
use crate::state::{ConnectionInfo, RedisClient};
use uuid::Uuid;

// Redis message queue functions
pub async fn queue_message_for_player(
    player_id: Uuid,
    lobby_id: Uuid,
    message: String,
    redis: &RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::player_missed_msgs(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    // Use Redis list to store messages with 2-minute TTL
    let _: () = redis::cmd("LPUSH")
        .arg(&key)
        .arg(&message)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Set TTL to 2 minutes (120 seconds)
    let _: () = redis::cmd("EXPIRE")
        .arg(&key)
        .arg(120)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::debug!("Queued message for player {} in Redis", player_id);
    Ok(())
}

pub async fn get_queued_messages_for_player(
    player_id: Uuid,
    lobby_id: Uuid,
    redis: &RedisClient,
) -> Result<Vec<String>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::player_missed_msgs(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    // Get all messages and delete the key atomically
    let messages: Vec<String> = redis::cmd("LRANGE")
        .arg(&key)
        .arg(0)
        .arg(-1)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    if !messages.is_empty() {
        // Delete the key after retrieving messages
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        tracing::info!(
            "Retrieved {} queued messages for player {}",
            messages.len(),
            player_id
        );
    }

    // Reverse the messages since LPUSH adds to the front but we want chronological order
    Ok(messages.into_iter().rev().collect())
}

async fn store_connection(
    player_id: Uuid,
    sender: SplitSink<WebSocket, Message>,
    connections: &ConnectionInfoMap,
) {
    let mut conns = connections.lock().await;
    let conn_info = ConnectionInfo {
        sender: Arc::new(Mutex::new(sender)),
    };
    conns.insert(player_id, Arc::new(conn_info));
    tracing::debug!("Stored connection for player {}", player_id);
}

pub async fn store_connection_and_send_queued_messages(
    player_id: Uuid,
    lobby_id: Uuid,

    sender: SplitSink<WebSocket, Message>,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    // Store the connection first
    store_connection(player_id, sender, connections).await;

    // Check for queued messages and send them
    match get_queued_messages_for_player(player_id, lobby_id, redis).await {
        Ok(messages) => {
            if !messages.is_empty() {
                tracing::info!(
                    "Sending {} queued messages to player {} in lobby {}",
                    messages.len(),
                    player_id,
                    lobby_id
                );

                let conns = connections.lock().await;
                if let Some(conn_info) = conns.get(&player_id) {
                    let mut sender_guard = conn_info.sender.lock().await;

                    for message in messages {
                        if let Err(e) = sender_guard.send(Message::Text(message.into())).await {
                            tracing::error!(
                                "Failed to send queued message to player {}: {}",
                                player_id,
                                e
                            );
                            break;
                        }

                        // Small delay to avoid overwhelming the client
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!(
                "Failed to retrieve queued messages for player {}: {}",
                player_id,
                e
            );
        }
    }
}

pub async fn remove_connection(player_id: Uuid, connections: &ConnectionInfoMap) {
    let mut conns = connections.lock().await;
    if conns.remove(&player_id).is_some() {
        tracing::debug!("Removed connection for player {}", player_id);
    }
}
