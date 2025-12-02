use crate::{
    errors::AppError,
    models::{
        User,
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};
use bb8::PooledConnection;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn get_user_by_id(user_id: Uuid, redis: RedisClient) -> Result<User, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::user(KeyPart::Id(user_id));

    let data: HashMap<String, String> = conn
        .hgetall(&key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if data.is_empty() {
        return Err(AppError::NotFound("User not found".into()));
    }

    let user = User {
        id: user_id,
        wallet_address: data
            .get("wallet_address")
            .cloned()
            .unwrap_or_else(|| "".into()),
        display_name: data.get("display_name").cloned(),
        wars_point: data
            .get("wars_point")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0),
        username: data.get("username").cloned(),
    };

    Ok(user)
}

pub async fn get_user_by_id_with_conn(
    user_id: Uuid,
    conn: &mut PooledConnection<'_, RedisConnectionManager>,
) -> Result<User, AppError> {
    let key = RedisKey::user(KeyPart::Id(user_id));

    let data: HashMap<String, String> = conn
        .hgetall(&key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if data.is_empty() {
        return Err(AppError::NotFound("User not found".into()));
    }

    let user = User {
        id: user_id,
        wallet_address: data
            .get("wallet_address")
            .cloned()
            .unwrap_or_else(|| "".into()),
        display_name: data.get("display_name").cloned(),
        wars_point: data
            .get("wars_point")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0),
        username: data.get("username").cloned(),
    };

    Ok(user)
}

pub async fn get_user_id(identifier: String, redis: RedisClient) -> Result<Uuid, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    // Try wallet lookup first using hash
    let wallets_hash = RedisKey::users_wallets();

    match conn
        .hget::<_, _, Option<String>>(&wallets_hash, &identifier)
        .await
    {
        Ok(Some(user_id_str)) => {
            tracing::info!("Found user ID via wallet hash lookup: {}", user_id_str);
            return Uuid::parse_str(&user_id_str).map_err(|e| {
                AppError::Deserialization(format!("Invalid UUID from wallet lookup: {}", e))
            });
        }
        Ok(None) => {
            tracing::debug!("No user found for wallet address: {}", identifier);
        }
        Err(e) => {
            tracing::error!("Error during wallet lookup: {}", e);
        }
    }

    // Fallback to username lookup using hash
    let usernames_hash = RedisKey::users_usernames();
    let normalized_username = identifier.to_lowercase();

    match conn
        .hget::<_, _, Option<String>>(&usernames_hash, &normalized_username)
        .await
    {
        Ok(Some(user_id_str)) => {
            tracing::info!("Found user ID via username hash lookup: {}", user_id_str);
            Uuid::parse_str(&user_id_str).map_err(|e| {
                AppError::Deserialization(format!("Invalid UUID from username lookup: {}", e))
            })
        }
        Ok(None) => {
            tracing::warn!(
                "User not found for identifier '{}' in both wallet and username hashes",
                identifier
            );
            Err(AppError::NotFound(format!(
                "User not found for identifier: {}",
                identifier
            )))
        }
        Err(e) => {
            tracing::error!("Error during username lookup for '{}': {}", identifier, e);
            Err(AppError::RedisCommandError(e))
        }
    }
}

pub async fn _get_all_users(redis: RedisClient) -> Result<Vec<User>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    // Get all user keys
    let pattern = RedisKey::user(KeyPart::Wildcard);
    let user_keys: Vec<String> = redis::cmd("KEYS")
        .arg(pattern)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut users = Vec::new();

    for key in user_keys {
        if let Some(user_id) = RedisKey::_extract_user_id_from_user_key(&key) {
            if let Ok(user) = get_user_by_id(user_id, redis.clone()).await {
                users.push(user);
            }
        }
    }

    Ok(users)
}
