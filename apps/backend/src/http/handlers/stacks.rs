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
use bs58;
use hex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing;

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

/// Serialize a Stacks address to Clarity principal hex
fn serialize_principal(address: &str) -> Result<String, AppError> {
    let decoded = bs58::decode(address)
        .into_vec()
        .map_err(|_| AppError::BadRequest("Invalid address".into()))?;
    if decoded.len() != 25 {
        return Err(AppError::BadRequest("Invalid address length".into()));
    }
    let version = decoded[0];
    let hash160 = &decoded[1..21];
    // Clarity principal: 0x05 + version + hash160
    let mut result = vec![0x05, version];
    result.extend_from_slice(hash160);
    Ok(format!("0x{}", hex::encode(result)))
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

/// Check if a player has joined a vault contract
pub async fn has_joined(
    contract_address: &WalletAddress,
    player_address: &WalletAddress,
    state: &AppState,
) -> Result<bool, AppError> {
    tracing::info!(
        "Checking if player has joined contract: {}, player: {}",
        contract_address.as_str(),
        player_address.as_str()
    );

    let network = if state.config.network.is_mainnet() {
        "mainnet"
    } else {
        "testnet"
    };

    // Split contract_address into principal and contract_name
    let addr_str = contract_address.as_str();
    let last_dot = addr_str
        .rfind('.')
        .ok_or_else(|| AppError::BadRequest("Invalid contract address".into()))?;
    let principal = &addr_str[..last_dot];
    let contract_name = &addr_str[last_dot + 1..];

    tracing::info!(
        "Parsed contract - principal: {}, contract_name: {}",
        principal,
        contract_name
    );

    let url = format!(
        "https://api.{}.hiro.so/v2/contracts/call-read/{}/{}/has-joined",
        network, principal, contract_name
    );

    tracing::info!("API URL: {}", url);

    let hex_principal = serialize_principal(player_address.as_str())?;
    tracing::info!("Serialized principal: {}", hex_principal);

    let body = serde_json::json!({
        "sender": player_address.as_str(),
        "arguments": [hex_principal]
    });

    tracing::info!("Request body: {}", body);

    let client = Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to send request to Hiro API: {}", e);
            AppError::FetchError(e.to_string())
        })?;

    tracing::info!("Response status: {}", response.status());

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        tracing::error!("Hiro API returned error status {}: {}", status, error_text);
        return Err(AppError::FetchError("Failed to call contract".into()));
    }

    let json: Value = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse JSON response: {}", e);
        AppError::Deserialization(e.to_string())
    })?;

    tracing::info!("Parsed JSON response: {:?}", json);

    let okay = json.get("okay").and_then(|v| v.as_bool()).ok_or_else(|| {
        tracing::error!("Missing 'okay' field in response");
        AppError::Deserialization("Missing okay".into())
    })?;

    tracing::info!("Contract call okay: {}", okay);

    if !okay {
        let cause = json
            .get("cause")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown cause");
        tracing::warn!("Contract call failed with cause: {}", cause);
        return Err(AppError::BadRequest("Contract call failed".into()));
    }

    let result = json.get("result").and_then(|v| v.as_str()).ok_or_else(|| {
        tracing::error!("Missing 'result' field in response");
        AppError::Deserialization("Missing result".into())
    })?;

    tracing::info!("Contract call result: {}", result);

    // Check if result is true (0x03 for true in Clarity)
    let has_joined = result == "0x03";
    if !has_joined {
        tracing::info!(
            "Player has not joined - expected '0x03' (true), got '{}'",
            result
        );
    } else {
        tracing::info!("Player has joined contract {}", player_address.as_str());
    }

    Ok(has_joined)
}
