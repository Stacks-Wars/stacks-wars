use axum::{extract::Path, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug, Deserialize)]
struct TokenApiResponse {
    contract_id: String,
    symbol: String,
    decimals: u8,
    name: String,
    metrics: TokenMetrics,
}

#[derive(Debug, Deserialize)]
struct TokenMetrics {
    price_usd: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub contract_id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub minimum_amount: f64,
}

pub async fn get_token_info(contract_address: String) -> Result<TokenInfo, AppError> {
    let url = format!("https://api.stxtools.io/tokens/{}", contract_address);

    let res = reqwest::get(&url).await.map_err(|e| {
        tracing::error!("Failed to fetch token info: {}", e);
        AppError::BadRequest(format!("Failed to fetch token info: {}", e))
    })?;

    if !res.status().is_success() {
        let error_msg = format!("Token not found or API error: {}", contract_address);
        tracing::error!("{}", error_msg);
        return Err(AppError::BadRequest(format!("{}", error_msg)));
    }

    let token_data: TokenApiResponse = res.json().await.map_err(|e| {
        tracing::error!("Invalid JSON response: {}", e);
        AppError::BadRequest(format!("Invalid JSON response: {}", e))
    })?;

    // Calculate minimum amount for $10 worth of tokens
    let minimum_usd_value = 10.0;
    let minimum_amount = if token_data.metrics.price_usd > 0.0 {
        let min_token_amount = minimum_usd_value / token_data.metrics.price_usd;

        // Smart rounding based on token price
        if token_data.metrics.price_usd >= 1.0 {
            // Expensive tokens (≥$1): keep up to 6 decimal places
            (min_token_amount * 1_000_000.0).ceil() / 1_000_000.0
        } else if token_data.metrics.price_usd >= 0.01 {
            // Medium tokens ($0.01-$1): keep up to 2 decimal places
            (min_token_amount * 100.0).ceil() / 100.0
        } else if token_data.metrics.price_usd >= 0.001 {
            // Low-value tokens ($0.001-$0.01): keep up to 1 decimal place
            (min_token_amount * 10.0).ceil() / 10.0
        } else {
            // Very cheap tokens (<$0.001): round to whole numbers
            min_token_amount.ceil()
        }
    } else {
        0.0
    };

    Ok(TokenInfo {
        contract_id: token_data.contract_id,
        symbol: token_data.symbol,
        name: token_data.name,
        decimals: token_data.decimals,
        price_usd: token_data.metrics.price_usd,
        minimum_amount,
    })
}

pub async fn get_token_info_handler(
    Path(contract_address): Path<String>,
) -> Result<Json<TokenInfo>, (StatusCode, String)> {
    get_token_info(contract_address)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_testnet_token_info_handler(
    Path(contract_address): Path<String>,
) -> Result<Json<TokenInfo>, (StatusCode, String)> {
    if &contract_address == "stx" {
        return get_token_info(contract_address)
            .await
            .map(Json)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }
    // Return fixed testnet token info for any other contract address
    let token_info = TokenInfo {
        contract_id: contract_address.clone(),
        symbol: "TEST".to_string(),
        name: "Test Token".to_string(),
        decimals: 6,
        price_usd: 0.01,
        minimum_amount: 3000.0,
    };

    Ok(Json(token_info))
}
