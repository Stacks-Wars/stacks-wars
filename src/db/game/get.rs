use redis::AsyncCommands;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        game::{GameType, GameV2, Order, Pagination},
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

pub async fn get_game_v2(game_id: Uuid, postgres: PgPool) -> Result<GameV2, AppError> {
    // Fetch game from database
    let game = sqlx::query_as::<_, GameV2>(
        "SELECT id, name, description, image_url, min_players, max_players, category,
                creator_id, is_active, updated_at, created_at
            FROM games
            WHERE id = $1",
    )
    .bind(game_id)
    .fetch_optional(&postgres)
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to query game: {}", e)))?
    .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

    Ok(game)
}

pub async fn get_games(
    pagination: Pagination,
    order: Order,
    postgres: PgPool,
) -> Result<Vec<GameV2>, AppError> {
    let offset = pagination.offset();
    let limit = pagination.limit;
    let order_sql = order.to_sql();

    let query = format!(
        "SELECT id, name, description, image_url, min_players, max_players, category,
            creator_id, is_active, updated_at, created_at
        FROM games
        ORDER BY created_at {}
        LIMIT $1 OFFSET $2",
        order_sql
    );

    let games = sqlx::query_as::<_, GameV2>(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&postgres)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch games: {}", e)))?;

    Ok(games)
}
