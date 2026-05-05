// Platform integration helpers for games (persistence, Redis summaries).

use crate::{
    db::{
        lobby::LobbyRepository, lobby_state::LobbyStateRepository,
        player_state::PlayerStateRepository, season::SeasonRepository,
        user_game_stats::UserGameStatsRepository,
        user_wars_points::UserWarsPointsRepository,
    },
    errors::AppError,
    http::{bot::broadcasts::broadcast_lobby_winner_to_tg, chain::stacks::convert_to_stx},
    models::LobbyStatus,
    state::{AppState, RedisClient},
};
use redis::AsyncCommands;
use uuid::Uuid;

pub use stacks_wars_core::{
    calculate_wars_point, GameBootstrap, GamePlayerState, GameResults, GameStatus, GameSummary,
    PlayerRanking, PlayerResult, TurnRotation, WarsPointContext,
};

/// Save a player's game result to Redis and PostgreSQL
pub async fn save_player_result(
    state: &AppState,
    lobby_id: Uuid,
    ctx: &WarsPointContext,
) -> Result<PlayerResult, AppError> {
    save_player_result_with_winner(state, lobby_id, ctx, ctx.rank == 1).await
}

/// Save a player's game result to Redis and PostgreSQL with explicit winner flag.
pub async fn save_player_result_with_winner(
    state: &AppState,
    lobby_id: Uuid,
    ctx: &WarsPointContext,
    is_winner: bool,
) -> Result<PlayerResult, AppError> {
    let wars_point = calculate_wars_point(ctx);

    let player_repo = PlayerStateRepository::new(state.redis.clone());
    player_repo
        .set_result(lobby_id, ctx.user_id, ctx.rank, ctx.prize, wars_point)
        .await?;

    let season_repo = SeasonRepository::new(state.postgres.clone());
    if let Ok(season_id) = season_repo.get_current_season_id().await {
        let wars_points_repo = UserWarsPointsRepository::new(state.postgres.clone());
        let needs_conversion = ctx
            .token_symbol
            .as_deref()
            .is_some_and(|s| !s.eq_ignore_ascii_case("STX"));

        let (stx_entry_amount, stx_prize) = if needs_conversion {
            if let Some(contract_id) = ctx.token_contract_id.as_deref() {
                let entry = match ctx.entry_amount {
                    Some(amt) if amt > 0.0 => {
                        Some(convert_to_stx(contract_id, amt, &state.redis).await)
                    }
                    other => other,
                };
                let prize = match ctx.prize {
                    Some(amt) if amt > 0.0 => {
                        Some(convert_to_stx(contract_id, amt, &state.redis).await)
                    }
                    other => other,
                };
                (entry, prize)
            } else {
                tracing::warn!(
                    "Token symbol is {:?} but no token_contract_id provided, skipping conversion",
                    ctx.token_symbol
                );
                (ctx.entry_amount, ctx.prize)
            }
        } else {
            (ctx.entry_amount, ctx.prize)
        };

        wars_points_repo
            .update_player_stats(
                ctx.user_id,
                Some(season_id),
                Some(wars_point),
                stx_entry_amount,
                stx_prize,
                is_winner,
            )
            .await?;

        if let Some(game_id) = ctx.game_id {
            let game_stats_repo = UserGameStatsRepository::new(state.postgres.clone());
            game_stats_repo
                .update_game_stats(
                    ctx.user_id,
                    game_id,
                    Some(season_id),
                    Some(wars_point),
                    stx_entry_amount,
                    stx_prize,
                    is_winner,
                )
                .await?;
        }
    }

    Ok(PlayerResult {
        rank: ctx.rank,
        prize: ctx.prize,
        wars_point,
    })
}

pub async fn finish_lobby(state: &AppState, lobby_id: Uuid) -> Result<(), AppError> {
    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
    lobby_state_repo.mark_finished(lobby_id).await?;

    let lobby_repo = LobbyRepository::new(state.postgres.clone());
    lobby_repo
        .update_status(lobby_id, LobbyStatus::Finished, state.clone())
        .await?;

    broadcast_lobby_winner_to_tg(state.clone(), lobby_id).await;

    Ok(())
}

pub async fn save_game_summary(
    redis: &RedisClient,
    lobby_id: Uuid,
    results: &GameResults,
    metadata: serde_json::Value,
) -> Result<(), AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

    let key = format!("game:{}:state", lobby_id);
    let summary = GameSummary {
        results: results.clone(),
        metadata,
        finished_at: chrono::Utc::now().timestamp(),
    };

    let json =
        serde_json::to_string(&summary).map_err(|e| AppError::Serialization(e.to_string()))?;

    let _: () = conn
        .set(&key, json)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!("Saved game summary for lobby {}", lobby_id);
    Ok(())
}
