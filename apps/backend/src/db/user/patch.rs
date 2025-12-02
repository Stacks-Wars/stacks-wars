use crate::{
    errors::AppError,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};
use redis::AsyncCommands;
use uuid::Uuid;

fn is_valid_username(username: &str) -> bool {
    let len_ok = (3..=20).contains(&username.len());
    let valid_chars = username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-');

    len_ok && valid_chars
}

pub async fn update_username(
    user_id: Uuid,
    new_username: String,
    redis: RedisClient,
) -> Result<String, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    // Normalize and validate the new username
    let normalized = new_username.trim().to_lowercase();
    if !is_valid_username(&normalized) {
        return Err(AppError::BadRequest("Invalid username".into()));
    }

    // Get the user's current username, if any
    let user_key = RedisKey::user(KeyPart::Id(user_id));
    let current_username: Option<String> = conn
        .hget(&user_key, "username")
        .await
        .map_err(AppError::RedisCommandError)?;

    // Early return if the username is the same (case-insensitive)
    if let Some(ref current) = current_username {
        if current.to_lowercase() == normalized {
            tracing::info!(
                "Username '{}' is already set for user {}, no changes needed",
                new_username,
                user_id
            );
            return Ok(new_username);
        }
    }

    // Check if new username is taken by someone else
    let usernames_hash = RedisKey::users_usernames();
    let existing_user_id: Option<String> = conn
        .hget(&usernames_hash, &normalized)
        .await
        .map_err(AppError::RedisCommandError)?;

    if let Some(existing) = existing_user_id {
        if existing != user_id.to_string() {
            return Err(AppError::BadRequest("Username already taken".into()));
        }
    }

    // Delete the old username from hash if it exists
    if let Some(old_username) = current_username {
        let _: () = conn
            .hdel(&usernames_hash, old_username.to_lowercase())
            .await
            .map_err(AppError::RedisCommandError)?;
    }

    // Set the new username in user's hash
    let _: () = conn
        .hset(&user_key, "username", &new_username)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Create the new username → user_id mapping in hash
    let _: () = conn
        .hset(&usernames_hash, &normalized, user_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(new_username)
}

pub async fn update_display_name(
    user_id: Uuid,
    new_display_name: String,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let trimmed = new_display_name.trim();
    if trimmed.is_empty() || trimmed.len() > 50 {
        return Err(AppError::BadRequest("Invalid display name".into()));
    }

    let user_key = RedisKey::user(KeyPart::Id(user_id));

    let _: () = conn
        .hset(&user_key, "display_name", trimmed)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn _increase_wars_point(
    user_id: Uuid,
    amount: f64,
    redis: RedisClient,
) -> Result<f64, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let user_key = RedisKey::user(KeyPart::Id(user_id));
    let points_key = RedisKey::users_points();
    let user_id_str = user_id.to_string();

    // Use pipeline to update both the user hash and sorted set atomically
    let mut pipe = redis::pipe();
    pipe.cmd("HINCRBYFLOAT")
        .arg(&user_key)
        .arg("wars_point")
        .arg(amount);
    pipe.cmd("ZINCRBY")
        .arg(&points_key)
        .arg(amount)
        .arg(&user_id_str);

    let results: (f64, f64) = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;
    let new_total = results.0;

    Ok(new_total)
}

pub async fn decrease_wars_point(
    user_id: Uuid,
    amount: f64,
    redis: RedisClient,
) -> Result<f64, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let user_key = RedisKey::user(KeyPart::Id(user_id));
    let points_key = RedisKey::users_points();
    let user_id_str = user_id.to_string();

    // Use pipeline to update both the user hash and sorted set atomically
    let mut pipe = redis::pipe();
    pipe.cmd("HINCRBYFLOAT")
        .arg(&user_key)
        .arg("wars_point")
        .arg(-amount);
    pipe.cmd("ZINCRBY")
        .arg(&points_key)
        .arg(-amount)
        .arg(&user_id_str);

    let results: (f64, f64) = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;
    let new_total = results.0;

    Ok(new_total)
}
