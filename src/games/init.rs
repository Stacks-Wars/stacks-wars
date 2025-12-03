use crate::{
    db::game::{get::get_all_games, post::create_game, words::add_word_set},
    errors::AppError,
    state::RedisClient,
};

pub async fn initialize_games(redis: RedisClient) -> Result<(), AppError> {
    tracing::info!("Initializing games...");

    // Initialize word set
    add_word_set(redis.clone()).await?;

    // Try to get all games from Redis
    match get_all_games(redis.clone()).await {
        Ok(games) => {
            if games.is_empty() {
                tracing::info!("No games found in database, adding default games");
                add_default_games(redis).await?;
            } else {
                tracing::info!("Found {} existing games in database", games.len());
            }
        }
        Err(e) => {
            tracing::warn!("Failed to get games from database: {}", e);
        }
    }

    Ok(())
}

async fn add_default_games(redis: RedisClient) -> Result<(), AppError> {
    let game_id = create_game(
        "Lexi Wars".to_string(),
        "A word battle game where players compete with words.".to_string(),
        "https://res.cloudinary.com/dapbvli1v/image/upload/Lexi_Wars2_yuuoam.png".to_string(),
        Some(vec![
            "word".to_string(),
            "strategy".to_string(),
            "multiplayer".to_string(),
        ]),
        2,
        redis,
    )
    .await?;

    tracing::info!("Added Lexi Wars game with ID: {}", game_id);
    Ok(())
}
