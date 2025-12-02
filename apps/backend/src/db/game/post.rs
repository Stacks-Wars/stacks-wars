use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        game::GameType,
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};

pub async fn create_game(
    name: String,
    description: String,
    image_url: String,
    tags: Option<Vec<String>>,
    min_players: u8,
    redis: RedisClient,
) -> Result<Uuid, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let game_id = Uuid::new_v4();

    let game = GameType {
        id: game_id,
        name,
        description,
        image_url,
        tags,
        min_players,
    };

    let key = RedisKey::game(KeyPart::Id(game_id));
    let hash = game.to_redis_hash();
    let fields: Vec<(&str, &str)> = hash.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    let _: () = conn
        .hset_multiple(&key, &fields)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(game_id)
}
