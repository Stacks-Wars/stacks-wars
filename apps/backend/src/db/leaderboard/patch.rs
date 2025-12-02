use crate::{
    errors::AppError,
    models::{
        game::ClaimState,
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};
use redis::AsyncCommands;
use uuid::Uuid;

pub async fn update_user_stats(
    user_id: Uuid,
    lobby_id: Uuid,
    rank: usize,
    prize: Option<f64>,
    wars_point: f64,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let matches_key = RedisKey::users_matches();
    let wins_key = RedisKey::users_wins();
    let pnl_key = RedisKey::users_pnl();
    let points_key = RedisKey::users_points();
    let user_key = RedisKey::user(crate::models::redis::KeyPart::Id(user_id));
    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(user_id));
    let user_id_str = user_id.to_string();

    // Use pipeline for efficiency
    let mut pipe = redis::pipe();

    // Increment match count
    pipe.cmd("ZINCRBY")
        .arg(&matches_key)
        .arg(1.0)
        .arg(&user_id_str);

    if rank == 1 {
        pipe.cmd("ZINCRBY")
            .arg(&wins_key)
            .arg(1.0)
            .arg(&user_id_str);
    }

    // Update wars point in both user hash and sorted set
    pipe.cmd("HINCRBYFLOAT")
        .arg(&user_key)
        .arg("wars_point")
        .arg(wars_point);
    pipe.cmd("ZINCRBY")
        .arg(&points_key)
        .arg(wars_point)
        .arg(&user_id_str);

    // Update player rank in lobby player hash
    pipe.cmd("HSET")
        .arg(&player_key)
        .arg("rank")
        .arg(rank.to_string());

    if let Some(prize_amount) = prize {
        if prize_amount != 0.0 {
            pipe.cmd("HINCRBYFLOAT")
                .arg(&pnl_key)
                .arg(&user_id_str)
                .arg(prize_amount);
            pipe.cmd("HSET")
                .arg(&player_key)
                .arg("prize")
                .arg(prize_amount.to_string());

            let claim_json = serde_json::to_string(&ClaimState::NotClaimed)
                .unwrap_or_else(|_| "null".to_string());
            pipe.cmd("HSET")
                .arg(&player_key)
                .arg("claim")
                .arg(claim_json);
        }
    }

    let _: () = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!(
        "Updated user stats for {}: rank={}, prize={:?}, wars_point={}",
        user_id,
        rank,
        prize,
        wars_point
    );

    Ok(())
}

/// Batch update stats for multiple users (useful for lobby completion)
pub async fn _batch_update_user_stats(
    user_stats: Vec<(Uuid, Uuid, usize, Option<f64>, f64)>, // (user_id, lobby_id, rank, prize, wars_point)
    redis: RedisClient,
) -> Result<(), AppError> {
    if user_stats.is_empty() {
        return Ok(());
    }

    let user_count = user_stats.len();

    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let matches_key = RedisKey::users_matches();
    let wins_key = RedisKey::users_wins();
    let pnl_key = RedisKey::users_pnl();
    let points_key = RedisKey::users_points();

    let mut pipe = redis::pipe();

    for (user_id, lobby_id, rank, prize, wars_point) in user_stats {
        let user_id_str = user_id.to_string();
        let user_key = RedisKey::user(crate::models::redis::KeyPart::Id(user_id));
        let player_key = RedisKey::lobby_player(
            crate::models::redis::KeyPart::Id(lobby_id),
            crate::models::redis::KeyPart::Id(user_id),
        );

        pipe.cmd("ZINCRBY")
            .arg(&matches_key)
            .arg(1.0)
            .arg(&user_id_str);

        if rank == 1 {
            pipe.cmd("ZINCRBY")
                .arg(&wins_key)
                .arg(1.0)
                .arg(&user_id_str);
        }

        // Update wars point in both user hash and sorted set
        pipe.cmd("HINCRBYFLOAT")
            .arg(&user_key)
            .arg("wars_point")
            .arg(wars_point);
        pipe.cmd("ZINCRBY")
            .arg(&points_key)
            .arg(wars_point)
            .arg(&user_id_str);

        // Update player rank in lobby player hash
        pipe.cmd("HSET")
            .arg(&player_key)
            .arg("rank")
            .arg(rank.to_string());

        // Update player prize in lobby player hash if applicable
        if let Some(prize_amount) = prize {
            if prize_amount != 0.0 {
                pipe.cmd("HINCRBYFLOAT")
                    .arg(&pnl_key)
                    .arg(&user_id_str)
                    .arg(prize_amount);
                pipe.cmd("HSET")
                    .arg(&player_key)
                    .arg("prize")
                    .arg(prize_amount.to_string());

                let claim_json = serde_json::to_string(&ClaimState::NotClaimed)
                    .unwrap_or_else(|_| "null".to_string());
                pipe.cmd("HSET")
                    .arg(&player_key)
                    .arg("claim")
                    .arg(claim_json);
            }
        }
    }

    let _: () = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!("Batch updated stats for {} users", user_count);

    Ok(())
}

pub async fn _update_user_wars_point(
    user_id: Uuid,
    new_wars_point: f64,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let points_key = RedisKey::users_points();
    let user_id_str = user_id.to_string();

    // Update wars_point in sorted set
    let _: () = conn
        .zadd(&points_key, &user_id_str, new_wars_point)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!(
        "Updated wars point for user {}: {}",
        user_id,
        new_wars_point
    );

    Ok(())
}
