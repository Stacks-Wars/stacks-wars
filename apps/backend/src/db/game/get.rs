use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        game::GameType,
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};

pub async fn get_game(game_id: Uuid, redis: RedisClient) -> Result<GameType, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::game(KeyPart::Id(game_id));
    let map: HashMap<String, String> = conn
        .hgetall(&key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Game with ID {} not found",
            game_id
        )));
    }

    GameType::from_redis_hash(&map)
}

pub async fn get_all_games(redis: RedisClient) -> Result<Vec<GameType>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let keys: Vec<String> = conn
        .keys(RedisKey::game(KeyPart::Wildcard))
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut games = Vec::new();
    for key in keys {
        let map: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if !map.is_empty() {
            let game = GameType::from_redis_hash(&map)?;
            games.push(game);
        }
    }

    Ok(games)
}
