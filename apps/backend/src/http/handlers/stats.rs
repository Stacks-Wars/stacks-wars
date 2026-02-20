use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::{
    db::{game::GameRepository, lobby::LobbyRepository, user::UserRepository},
    http::handlers::stacks::get_token_price_data,
    models::LobbyStatus,
    state::AppState,
};

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
    pub total_lobbies: i64,
    pub active_lobbies: i64,
    pub finished_lobbies: i64,
    pub total_games: i64,
    pub total_volume_usd: f64,
    pub active_volume_usd: f64,
    pub finished_volume_usd: f64,
    pub token_breakdown: Vec<TokenVolume>,
}

/// GET /api/stats — Platform analytics (public, no auth)
pub async fn get_platform_stats(
    State(state): State<AppState>,
) -> Result<Json<PlatformStats>, (StatusCode, String)> {
    let pool = state.postgres.clone();

    let user_repo = UserRepository::new(pool.clone());
    let lobby_repo = LobbyRepository::new(pool.clone());
    let game_repo = GameRepository::new(pool.clone());

    // Counts
    let total_users = user_repo.count_users().await.unwrap_or(0);
    let total_lobbies = lobby_repo.count_lobbies().await.unwrap_or(0);
    let total_games = game_repo.count_games(true).await.unwrap_or(0);

    // Active lobbies (waiting + starting + in_progress)
    let waiting = lobby_repo
        .count_by_status(LobbyStatus::Waiting)
        .await
        .unwrap_or(0);
    let starting = lobby_repo
        .count_by_status(LobbyStatus::Starting)
        .await
        .unwrap_or(0);
    let in_progress = lobby_repo
        .count_by_status(LobbyStatus::InProgress)
        .await
        .unwrap_or(0);
    let finished_lobbies = lobby_repo
        .count_by_status(LobbyStatus::Finished)
        .await
        .unwrap_or(0);
    let active_lobbies = waiting + starting + in_progress;

    // Token breakdown with status info for volume aggregation
    // current_amount = actual pool value (non-zero when funded)
    // entry_amount = per-player fee (not part of volume)
    let token_rows: Vec<(Option<String>, Option<String>, String, f64, i64)> = sqlx::query_as::<_, (Option<String>, Option<String>, String, f64, i64)>(
        "SELECT token_symbol, token_contract_id, status::text, \
         COALESCE(SUM(COALESCE(current_amount, 0)), 0) as volume, \
         COUNT(*) as lobby_count \
         FROM lobbies \
         WHERE token_symbol IS NOT NULL \
         GROUP BY token_symbol, token_contract_id, status",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    // Aggregate volumes by token and calculate USD equivalents
    // HashMap key is symbol, value is (total_vol, lobby_count, contract_id, active_vol, finished_vol)
    let mut token_map: std::collections::HashMap<String, (f64, i64, String, f64, f64)> = std::collections::HashMap::new();

    for (symbol_opt, contract_id_opt, status, volume, count) in token_rows {
        let symbol = symbol_opt.unwrap_or_else(|| "STX".to_string());
        let contract_id = contract_id_opt.unwrap_or_else(|| "stx".to_string());

        let entry = token_map.entry(symbol.clone()).or_insert((0.0, 0, contract_id.clone(), 0.0, 0.0));
        entry.0 += volume; // total volume
        entry.1 += count;  // lobby count
        // entry.2 is contract_id (already set)

        // Track by status
        match status.as_str() {
            "waiting" | "starting" | "in_progress" => entry.3 += volume, // active
            "finished" => entry.4 += volume, // finished
            _ => {}
        }
    }

    // Convert to USD and build token breakdown
    let mut token_breakdown = Vec::new();
    let mut total_volume_usd = 0.0;
    let mut active_volume_usd = 0.0;
    let mut finished_volume_usd = 0.0;

    for (symbol, (volume, lobby_count, contract_id, active_vol, finished_vol)) in token_map.iter() {
        let price_data = get_token_price_data(contract_id, &state.redis).await;
        let volume_usd = *volume * price_data.price_usd;
        let active_usd = *active_vol * price_data.price_usd;
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
        active_volume_usd += active_usd;
        finished_volume_usd += finished_usd;
    }

    // Sort by USD volume descending
    token_breakdown.sort_by(|a, b| b.volume_usd.partial_cmp(&a.volume_usd).unwrap_or(std::cmp::Ordering::Equal));

    Ok(Json(PlatformStats {
        total_users,
        total_lobbies,
        active_lobbies,
        finished_lobbies,
        total_games,
        total_volume_usd,
        active_volume_usd,
        finished_volume_usd,
        token_breakdown,
    }))
}
