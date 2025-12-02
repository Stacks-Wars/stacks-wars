use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    db::lobby::get::{get_lobby_info, get_lobby_players},
    errors::AppError,
    models::{
        game::{Player, PlayerState},
        lobby::LobbyServerMessage,
        redis::{KeyPart, RedisKey},
    },
    state::{ConnectionInfoMap, RedisClient},
    ws::handlers::lobby::message_handler::broadcast_to_lobby,
};

pub async fn last_ping(
    ts: u64,
    lobby_id: Uuid,
    player: &Player,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    let is_creator = match get_lobby_info(lobby_id, redis.clone()).await {
        Ok(lobby_info) => lobby_info.creator.id == player.id,
        Err(e) => {
            tracing::error!("Failed to get lobby info for {}: {}", lobby_id, e);
            false
        }
    };

    match update_player_last_ping(lobby_id, player.id, ts, is_creator, redis.clone()).await {
        Ok(()) => {
            if let Ok(players) =
                get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await
            {
                let msg = LobbyServerMessage::PlayerUpdated { players };
                broadcast_to_lobby(lobby_id, &msg, connections, None, redis.clone()).await;
            }
        }
        Err(e) => {
            tracing::error!(
                "Failed to update last_ping for player {} in lobby {}: {}",
                player.id,
                lobby_id,
                e
            );
        }
    }
}

async fn update_player_last_ping(
    lobby_id: Uuid,
    player_id: Uuid,
    last_ping: u64,
    is_creator: bool,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    let exists: bool = conn
        .exists(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if !exists {
        return Ok(());
    }

    let _: () = conn
        .hset(&player_key, "last_ping", last_ping.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    if is_creator {
        let lobby_key = RedisKey::lobby(KeyPart::Id(lobby_id));
        let _: () = conn
            .hset(&lobby_key, "creator_last_ping", last_ping.to_string())
            .await
            .map_err(AppError::RedisCommandError)?;

        tracing::debug!(
            "Updated creator_last_ping for lobby {} to {}",
            lobby_id,
            last_ping
        );
    }

    tracing::debug!(
        "Updated last_ping for player {} in lobby {} to {}",
        player_id,
        lobby_id,
        last_ping
    );

    Ok(())
}
