use redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        game::{GameType, GameV2},
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

pub async fn create_game_v2(
    name: String,
    description: String,
    image_url: String,
    min_players: i16,
    max_players: i16,
    category: Option<String>,
    creator_id: Uuid,
    postgres: PgPool,
) -> Result<GameV2, AppError> {
    // Check if game already exists (by name)
    let existing_game = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            i16,
            i16,
            Option<String>,
            Uuid,
            bool,
            chrono::NaiveDateTime,
            chrono::NaiveDateTime,
        ),
    >(
            "SELECT id, name, description, image_url, min_players, max_players, category, creator_id, is_active, created_at, updated_at
            FROM games
            WHERE name = $1",
    )
    .bind(&name)
    .fetch_optional(&postgres)
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to query game: {}", e)))?;

    if let Some((
        id,
        name,
        description,
        image_url,
        min_players,
        max_players,
        category,
        creator_id,
        is_active,
        created_at,
        updated_at,
    )) = existing_game
    {
        tracing::info!("Game already exists: {}", name);
        return Ok(GameV2 {
            id,
            name,
            description,
            image_url,
            min_players,
            max_players,
            category,
            creator_id,
            is_active,
            created_at,
            updated_at,
        });
    }

    // Create new game
    let game_id = Uuid::new_v4();

    // Start transaction
    let mut tx = postgres
        .begin()
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

    // Insert into database
    let game = sqlx::query_as::<_, GameV2>(
        "INSERT INTO games (id, name, description, image_url, min_players, max_players, category, creator_id, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
            RETURNING id, name, description, image_url, min_players, max_players, category, creator_id, is_active, created_at, updated_at",
    )
    .bind(game_id)
    .bind(&name)
    .bind(&description)
    .bind(&image_url)
    .bind(min_players)
    .bind(max_players)
    .bind(&category)
    .bind(creator_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to create game: {}", e)))?;

    tx.commit()
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

    tracing::info!("Created new game: {}", game.name);

    Ok(game)
}
