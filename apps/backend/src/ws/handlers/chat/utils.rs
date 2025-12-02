use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        chat::ChatServerMessage,
        redis::{KeyPart, RedisKey},
    },
    state::{ChatConnectionInfo, ChatConnectionInfoMap, RedisClient},
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn queue_chat_message_for_player(
    player_id: Uuid,
    lobby_id: Uuid,
    message: String,
    redis: &RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::player_missed_chat_msgs(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    // Use Redis list to store messages with 5-minute TTL
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

    tracing::debug!("Queued chat message for player {} in Redis", player_id);
    Ok(())
}

pub async fn get_queued_chat_messages_for_player(
    player_id: Uuid,
    lobby_id: Uuid,
    redis: &RedisClient,
) -> Result<Vec<String>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::player_missed_chat_msgs(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    let messages: Vec<String> = redis::cmd("LRANGE")
        .arg(&key)
        .arg(0)
        .arg(-1)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    if !messages.is_empty() {
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        tracing::info!(
            "Retrieved {} queued chat messages for player {}",
            messages.len(),
            player_id
        );
    }

    Ok(messages.into_iter().rev().collect())
}

pub async fn store_chat_connection_and_send_queued_messages(
    lobby_id: Uuid,
    player_id: Uuid,
    sender: SplitSink<WebSocket, Message>,
    connections: &ChatConnectionInfoMap,
    redis: &RedisClient,
) {
    // Store the connection
    let conn_info = Arc::new(ChatConnectionInfo {
        sender: Arc::new(Mutex::new(sender)),
    });
    connections
        .lock()
        .await
        .insert(player_id, conn_info.clone());

    // Send queued messages
    match get_queued_chat_messages_for_player(player_id, lobby_id, redis).await {
        Ok(messages) => {
            if !messages.is_empty() {
                tracing::info!(
                    "Sending {} queued chat messages to player {} in lobby {}",
                    messages.len(),
                    player_id,
                    lobby_id
                );

                let mut sender_guard = conn_info.sender.lock().await;
                let mut sent_count = 0;
                for message in messages {
                    if let Err(e) = sender_guard.send(Message::Text(message.into())).await {
                        tracing::error!(
                            "Failed to send queued chat message to player {} in lobby {}: {}",
                            player_id,
                            lobby_id,
                            e
                        );
                        break;
                    }
                    sent_count += 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }

                tracing::info!(
                    "Successfully sent {} queued chat messages to player {} in lobby {}",
                    sent_count,
                    player_id,
                    lobby_id
                );
            }
        }
        Err(e) => {
            tracing::error!(
                "Failed to retrieve queued chat messages for player {} in lobby {}: {}",
                player_id,
                lobby_id,
                e
            );
        }
    }
}

pub async fn remove_chat_connection(player_id: Uuid, chat_connections: &ChatConnectionInfoMap) {
    let mut conn_map = chat_connections.lock().await;
    if conn_map.remove(&player_id).is_some() {
        tracing::debug!("Removed chat connection for player {}", player_id);
    }
}

pub async fn send_chat_message_to_player(
    player_id: Uuid,
    message: &ChatServerMessage,
    connections: &ChatConnectionInfoMap,
) {
    let serialized = match serde_json::to_string(message) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Failed to serialize message: {}", e);
            return;
        }
    };

    let connection_guard = connections.lock().await;
    if let Some(conn_info) = connection_guard.get(&player_id) {
        let mut sender = conn_info.sender.lock().await;
        if let Err(e) = sender.send(Message::Text(serialized.into())).await {
            tracing::debug!("Failed to send message to player {}: {}", player_id, e);
        }
    }
}
