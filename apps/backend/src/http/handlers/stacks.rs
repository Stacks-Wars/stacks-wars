use crate::{
    errors::AppError,
    models::{
        WalletAddress,
        stacks::{Token, TokenInfo},
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use reqwest::Client;
use serde::Deserialize;

/// Hiro API response for STX balance
#[derive(Debug, Deserialize)]
struct StxBalance {
    balance: String,
}

/// Hiro API response for fungible tokens
#[derive(Debug, Deserialize)]
struct FungibleTokens {
    #[serde(flatten)]
    tokens: std::collections::HashMap<String, TokenBalance>,
}

/// Hiro API response for individual token balance
#[derive(Debug, Deserialize)]
struct TokenBalance {
    balance: String,
}

/// Hiro API full response
#[derive(Debug, Deserialize)]
struct HiroBalancesResponse {
    stx: StxBalance,
    fungible_tokens: FungibleTokens,
}

/// StxTools API metrics
#[derive(Debug, Deserialize)]
struct StxToolsMetrics {
    price_usd: f64,
}

/// StxTools API response
#[derive(Debug, Deserialize)]
struct StxToolsResponse {
    metrics: StxToolsMetrics,
}

/// Get user balance from Hiro API
pub async fn get_balance(
    Path(wallet_address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Token>>, (StatusCode, String)> {
    let network = if state.config.network.is_mainnet() {
        "mainnet"
    } else {
        "testnet"
    };
    let url = format!(
        "https://api.{}.hiro.so/extended/v1/address/{}/balances",
        network, wallet_address
    );

    let client = Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header("x-api-key", &state.config.hiro_api_key)
        .send()
        .await
        .map_err(|e| AppError::FetchError(e.to_string()).to_response())?;

    if !response.status().is_success() {
        return Err(AppError::FetchError("Failed to fetch balance".to_string()).to_response());
    }

    let balances: HiroBalancesResponse = response
        .json()
        .await
        .map_err(|e| AppError::Deserialization(e.to_string()).to_response())?;

    let mut tokens = Vec::new();

    // Add STX balance
    let stx_balance = balances
        .stx
        .balance
        .parse::<f64>()
        .map_err(|e| AppError::Deserialization(e.to_string()).to_response())?
        / 1_000_000.0;
    tokens.push(Token {
        name: "stacks".to_string(),
        balance: stx_balance,
        contract_id: "stx".to_string(),
    });

    // Add fungible tokens
    for (key, token_balance) in balances.fungible_tokens.tokens {
        if let Some((contract_id, name)) = parse_token_key(&key) {
            let balance = token_balance
                .balance
                .parse::<f64>()
                .map_err(|e| AppError::Deserialization(e.to_string()).to_response())?
                / 1_000_000.0;
            tokens.push(Token {
                name,
                balance,
                contract_id,
            });
        }
    }

    Ok(Json(tokens))
}

/// Parse the token key to extract contract_id and name
fn parse_token_key(key: &str) -> Option<(String, String)> {
    if let Some(colon_pos) = key.rfind("::") {
        let contract_id = key[..colon_pos].to_string();
        let name = key[colon_pos + 2..].to_string();
        Some((contract_id, name))
    } else {
        None
    }
}

/// Get token information including price and minimum amount for $10 USD
pub async fn get_token_info(
    Path(contract_address_str): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<TokenInfo>, (StatusCode, String)> {
    let contract_address =
        WalletAddress::try_from(contract_address_str.as_str()).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid contract address".to_string(),
            )
        })?;
    if !state.config.network.is_mainnet() {
        // Return hardcoded values for testnet
        return Ok(Json(TokenInfo {
            price: 0.01,
            minimum_amount: 1000.0,
        }));
    }

    let url = format!(
        "https://api.stxtools.io/tokens/{}",
        contract_address.as_str()
    );

    let client = Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| AppError::FetchError(e.to_string()).to_response())?;

    if !response.status().is_success() {
        return Err(AppError::NotFound("Token not found".to_string()).to_response());
    }

    let token_data: StxToolsResponse = response
        .json()
        .await
        .map_err(|e| AppError::Deserialization(e.to_string()).to_response())?;

    let price = token_data.metrics.price_usd;
    let minimum_amount = if price > 0.0 { 10.0 / price } else { 0.0 };

    Ok(Json(TokenInfo {
        price,
        minimum_amount,
    }))
}
