use crate::{errors::AppError, models::stacks::Token, state::AppState};
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
