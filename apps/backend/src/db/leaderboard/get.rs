use crate::{
    db::user::get::get_user_by_id,
    errors::AppError,
    models::{leaderboard::LeaderBoard, redis::RedisKey},
    state::RedisClient,
};
use redis::AsyncCommands;
use uuid::Uuid;

pub async fn get_leaderboard(
    limit: Option<u64>,
    redis: RedisClient,
) -> Result<Vec<LeaderBoard>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let points_key = RedisKey::users_points();

    let top_users: Vec<(String, f64)> = if let Some(limit) = limit {
        conn.zrevrange_withscores(&points_key, 0, limit as isize - 1)
            .await
            .map_err(AppError::RedisCommandError)?
    } else {
        conn.zrevrange_withscores(&points_key, 0, -1)
            .await
            .map_err(AppError::RedisCommandError)?
    };

    if top_users.is_empty() {
        return Ok(vec![]);
    }

    // Get user IDs for batch operations
    let user_ids: Vec<String> = top_users.iter().map(|(id, _)| id.clone()).collect();

    let matches_key = RedisKey::users_matches();
    let wins_key = RedisKey::users_wins();
    let pnl_key = RedisKey::users_pnl();

    let mut pipe = redis::pipe();
    for user_id in &user_ids {
        pipe.cmd("ZSCORE").arg(&matches_key).arg(user_id);
        pipe.cmd("ZSCORE").arg(&wins_key).arg(user_id);
        pipe.cmd("HGET").arg(&pnl_key).arg(user_id);
    }

    let results: Vec<(Option<f64>, Option<f64>, Option<String>)> = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Process results
    let mut leaderboard = Vec::new();
    for (idx, ((user_id, wars_point), (matches_opt, wins_opt, pnl_opt))) in
        top_users.into_iter().zip(results.into_iter()).enumerate()
    {
        let user_uuid = match Uuid::parse_str(&user_id) {
            Ok(uuid) => uuid,
            Err(_) => continue,
        };

        let matches = matches_opt.unwrap_or(0.0) as u64;
        let wins = wins_opt.unwrap_or(0.0) as u64;
        let pnl = pnl_opt.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

        // Get user details using existing function
        let user = match get_user_by_id(user_uuid, redis.clone()).await {
            Ok(mut user) => {
                // Update wars_point from the sorted set (in case it's more recent)
                user.wars_point = wars_point;
                user
            }
            Err(_) => continue, // Skip if user doesn't exist
        };

        let win_rate = if matches > 0 {
            (wins as f64 / matches as f64) * 100.0
        } else {
            0.0
        };

        leaderboard.push(LeaderBoard {
            user,
            win_rate,
            rank: (idx + 1) as u64,
            total_match: matches,
            total_wins: wins,
            pnl,
        });
    }

    Ok(leaderboard)
}

pub async fn get_user_stat(user_id: Uuid, redis: RedisClient) -> Result<LeaderBoard, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let user = get_user_by_id(user_id, redis.clone()).await?;

    let user_id_str = user_id.to_string();
    let matches_key = RedisKey::users_matches();
    let wins_key = RedisKey::users_wins();
    let pnl_key = RedisKey::users_pnl();
    let points_key = RedisKey::users_points();

    // Use pipeline for efficiency
    let mut pipe = redis::pipe();
    pipe.cmd("ZSCORE").arg(&matches_key).arg(&user_id_str);
    pipe.cmd("ZSCORE").arg(&wins_key).arg(&user_id_str);
    pipe.cmd("HGET").arg(&pnl_key).arg(&user_id_str);
    pipe.cmd("ZREVRANK").arg(&points_key).arg(&user_id_str); // Get rank by wars_point

    let results: (Option<f64>, Option<f64>, Option<String>, Option<i64>) = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    let matches = results.0.unwrap_or(0.0) as u64;
    let wins = results.1.unwrap_or(0.0) as u64;
    let pnl = results.2.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

    // Redis ZREVRANK returns 0-based rank, so add 1 for 1-based rank
    // If user is not in the sorted set, rank will be 0, hmmm
    let rank = results.3.map(|r| (r + 1) as u64).unwrap_or(0);

    let win_rate = if matches > 0 {
        (wins as f64 / matches as f64) * 100.0
    } else {
        0.0
    };

    Ok(LeaderBoard {
        user,
        win_rate,
        rank,
        total_match: matches,
        total_wins: wins,
        pnl,
    })
}
