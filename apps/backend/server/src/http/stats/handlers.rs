use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{season::SeasonRepository, user::UserRepository},
    http::chain::stacks::get_token_price_data,
    state::AppState,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsQuery {
    pub season_id: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenVolume {
    pub symbol: String,
    pub volume: f64,
    pub volume_usd: f64,
    pub lobby_count: i64,
    pub price_usd: f64,
    pub image_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStats {
    pub total_users: i64,
    pub new_users_count: i64,
    pub total_lobbies: i64,
    pub active_lobbies: i64,
    pub finished_lobbies: i64,
    pub fee_lobbies: i64,
    pub active_fee_lobbies: i64,
    pub total_games: i64,
    pub total_volume_usd: f64,
    pub finished_volume_usd: f64,
    pub token_breakdown: Vec<TokenVolume>,
}

/// GET /api/stats — Platform analytics (public, no auth)
pub async fn get_platform_stats(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<PlatformStats>, (StatusCode, String)> {
    let pool = state.postgres.clone();
    let season_id = query.season_id;

    let user_repo = UserRepository::new(pool.clone());
    let season_repo = SeasonRepository::new(pool.clone());
    let season_window = if let Some(season_id) = season_id {
        match season_repo.find_by_id(season_id).await {
            Ok(season) => Some((season.start_date, season.end_date)),
            Err(_) => None,
        }
    } else {
        None
    };

    // Counts
    let total_users = user_repo.count_users().await.unwrap_or(0);

    let new_users_count = if let Some(season_id) = season_id {
        match season_repo.find_by_id(season_id).await {
            Ok(season) => sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE created_at >= $1 AND created_at <= $2",
            )
            .bind(season.start_date)
            .bind(season.end_date)
            .fetch_one(&pool)
            .await
            .unwrap_or(0),
            Err(_) => 0,
        }
    } else {
        total_users
    };

    let (total_lobbies, active_lobbies, finished_lobbies, fee_lobbies, active_fee_lobbies, total_games) =
        if let Some((season_start, season_end)) = season_window {
            sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64)>(
                "SELECT
                    COUNT(*)::bigint,
                    COUNT(*) FILTER (
                        WHERE status::text IN ('waiting', 'starting', 'in_progress')
                    )::bigint,
                    COUNT(*) FILTER (
                        WHERE status::text = 'finished'
                    )::bigint,
                    COUNT(*) FILTER (
                        WHERE COALESCE(entry_amount, 0) > 0
                    )::bigint,
                    COUNT(*) FILTER (
                        WHERE COALESCE(entry_amount, 0) > 0
                            AND status::text IN ('waiting', 'starting', 'in_progress')
                    )::bigint,
                    COUNT(DISTINCT game_id)::bigint
                 FROM lobbies
                 WHERE created_at >= $1 AND created_at <= $2",
            )
            .bind(season_start)
            .bind(season_end)
            .fetch_one(&pool)
            .await
            .unwrap_or((0, 0, 0, 0, 0, 0))
        } else {
            sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64)>(
                "SELECT
                    COUNT(*)::bigint,
                    COUNT(*) FILTER (
                        WHERE status::text IN ('waiting', 'starting', 'in_progress')
                    )::bigint,
                    COUNT(*) FILTER (
                        WHERE status::text = 'finished'
                    )::bigint,
                    COUNT(*) FILTER (
                        WHERE COALESCE(entry_amount, 0) > 0
                    )::bigint,
                    COUNT(*) FILTER (
                        WHERE COALESCE(entry_amount, 0) > 0
                            AND status::text IN ('waiting', 'starting', 'in_progress')
                    )::bigint,
                    COUNT(DISTINCT game_id)::bigint
                 FROM lobbies",
            )
            .fetch_one(&pool)
            .await
            .unwrap_or((0, 0, 0, 0, 0, 0))
        };

    // Token breakdown with status info for volume aggregation
    // current_amount = actual pool value (non-zero when funded)
    // entry_amount = per-player fee (not part of volume)
    let token_rows: Vec<(Option<String>, Option<String>, String, f64, i64)> = if let Some((season_start, season_end)) = season_window {
        sqlx::query_as::<_, (Option<String>, Option<String>, String, f64, i64)>(
            "SELECT token_symbol, token_contract_id, status::text, \
             COALESCE(SUM(COALESCE(current_amount, 0)), 0) as volume, \
             COUNT(*) as lobby_count \
             FROM lobbies \
             WHERE token_symbol IS NOT NULL \
             AND created_at >= $1 AND created_at <= $2 \
             GROUP BY token_symbol, token_contract_id, status",
        )
        .bind(season_start)
        .bind(season_end)
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, (Option<String>, Option<String>, String, f64, i64)>(
            "SELECT token_symbol, token_contract_id, status::text, \
             COALESCE(SUM(COALESCE(current_amount, 0)), 0) as volume, \
             COUNT(*) as lobby_count \
             FROM lobbies \
             WHERE token_symbol IS NOT NULL \
             GROUP BY token_symbol, token_contract_id, status",
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
    };

    // Aggregate volumes by token and calculate USD equivalents
    // HashMap key is symbol, value is (total_vol, lobby_count, contract_id, finished_vol)
    let mut token_map: std::collections::HashMap<String, (f64, i64, String, f64)> =
        std::collections::HashMap::new();

    for (symbol_opt, contract_id_opt, status, volume, count) in token_rows {
        let symbol = symbol_opt.unwrap_or_else(|| "STX".to_string());
        let contract_id = contract_id_opt.unwrap_or_else(|| "stx".to_string());

        let entry = token_map
            .entry(symbol.clone())
            .or_insert((0.0, 0, contract_id.clone(), 0.0));
        entry.0 += volume; // total volume
        entry.1 += count;  // lobby count
        // entry.2 is contract_id (already set)

        // Track by status
        if status.as_str() == "finished" {
            entry.3 += volume;
        }
    }

    // Convert to USD and build token breakdown
    let mut token_breakdown = Vec::new();
    let mut total_volume_usd = 0.0;
    let mut finished_volume_usd = 0.0;

    for (symbol, (volume, lobby_count, contract_id, finished_vol)) in token_map.iter() {
        let price_data = get_token_price_data(contract_id, &state.redis).await;
        let volume_usd = *volume * price_data.price_usd;
        let finished_usd = *finished_vol * price_data.price_usd;

        token_breakdown.push(TokenVolume {
            symbol: symbol.clone(),
            volume: *volume,
            volume_usd,
            lobby_count: *lobby_count,
            price_usd: price_data.price_usd,
            image_url: price_data.image_url,
        });

        total_volume_usd += volume_usd;
        finished_volume_usd += finished_usd;
    }

    // Sort by USD volume descending
    token_breakdown.sort_by(|a, b| b.volume_usd.partial_cmp(&a.volume_usd).unwrap_or(std::cmp::Ordering::Equal));

    Ok(Json(PlatformStats {
        total_users,
        new_users_count,
        total_lobbies,
        active_lobbies,
        finished_lobbies,
        fee_lobbies,
        active_fee_lobbies,
        total_games,
        total_volume_usd,
        finished_volume_usd,
        token_breakdown,
    }))
}
