use crate::{
    errors::AppError,
    models::{
        WalletAddress, keys::RedisKey,
        stacks::{Token, TokenInfo},
    },
    state::{AppState, RedisClient},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use hex;
use redis::AsyncCommands;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
    image_url: Option<String>,
    metrics: StxToolsMetrics,
}

/// Cached token price data (stored in Redis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTokenPrice {
    pub price_usd: f64,
    pub image_url: Option<String>,
}

/// Request body for token info endpoint
#[derive(Debug, Deserialize)]
pub struct TokenInfoRequest {
    /// Optional STX amount to calculate minimum equivalent in this token.
    /// Defaults to 5 if not provided.
    #[serde(default = "default_stx_amount")]
    pub stx_amount: f64,
}

fn default_stx_amount() -> f64 {
    5.0
}

/// Fetch token price + image from stxtools.io with Redis caching (5-min TTL).
///
/// Returns `CachedTokenPrice` with price_usd and image_url.
/// On any failure, returns price 0.0 and no image.
pub async fn get_token_price_data(
    token_contract_id: &str,
    redis_pool: &RedisClient,
) -> CachedTokenPrice {
    let cache_key = RedisKey::token_price_data(token_contract_id);
    let default = CachedTokenPrice { price_usd: 0.0, image_url: None };

    // Try cache first
    if let Ok(mut conn) = redis_pool.get().await {
        let cached: Result<Option<String>, _> = conn.get(&cache_key).await;
        if let Ok(Some(json_str)) = cached {
            if let Ok(data) = serde_json::from_str::<CachedTokenPrice>(&json_str) {
                return data;
            }
        }
    }

    // Fetch from API
    let client = Client::new();
    let url = format!("https://api.stxtools.io/tokens/{}", token_contract_id);

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to fetch token price for {}: {}", token_contract_id, e);
            return default;
        }
    };

    if !resp.status().is_success() {
        return default;
    }

    let body = match resp.text().await {
        Ok(t) => t,
        Err(_) => return default,
    };

    let parsed: StxToolsResponse = match serde_json::from_str(&body) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to parse token price for {}: {}", token_contract_id, e);
            return default;
        }
    };

    let result = CachedTokenPrice {
        price_usd: parsed.metrics.price_usd,
        image_url: parsed.image_url,
    };

    // Cache for 5 minutes
    if let Ok(mut conn) = redis_pool.get().await {
        if let Ok(json_str) = serde_json::to_string(&result) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json_str, 300).await;
        }
    }

    result
}

/// Convert a token amount to USD using stxtools.io prices.
///
/// Returns 0.0 on any failure (network error, missing price, etc.)
pub async fn convert_to_usd(
    token_contract_id: &str,
    amount: f64,
    redis_pool: &RedisClient,
) -> f64 {
    if amount <= 0.0 {
        return 0.0;
    }
    let price = get_token_price_data(token_contract_id, redis_pool).await.price_usd;
    amount * price
}

/// Convert a token amount to its STX equivalent using stxtools.io prices with Redis caching.
///
/// Fetches the USD price of both the token and STX (using Redis cache with 5-min TTL),
/// then computes:
///   stx_amount = amount * (token_price_usd / stx_price_usd)
///
/// Returns 0.0 on any failure (network error, missing price, etc.)
pub async fn convert_to_stx(
    token_contract_id: &str,
    amount: f64,
    redis_pool: &RedisClient,
) -> f64 {
    if amount <= 0.0 {
        return 0.0;
    }

    // Fetch token price from cache
    let token_price = get_token_price_data(token_contract_id, redis_pool).await.price_usd;
    if token_price <= 0.0 {
        tracing::warn!("Token price is zero for {}, skipping conversion", token_contract_id);
        return 0.0;
    }

    // Fetch STX price from cache
    let stx_price = get_token_price_data("stx", redis_pool).await.price_usd;
    if stx_price <= 0.0 {
        tracing::warn!("STX price is zero, skipping conversion");
        return 0.0;
    }

    let stx_equivalent = amount * (token_price / stx_price);
    tracing::debug!(
        "Converted {} {} → {} STX (token_usd={}, stx_usd={})",
        amount, token_contract_id, stx_equivalent, token_price, stx_price
    );
    stx_equivalent
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
        name: "STX".to_string(),
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
            if balance > 0.0 {
                tokens.push(Token {
                    name,
                    balance,
                    contract_id,
                });
            }
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

/// C32 alphabet used by Stacks addresses (Crockford's Base32 variant).
const C32_CHARS: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Decode a c32-encoded string to bytes (base32 → base256 big-number conversion).
fn c32_decode(input: &str) -> Result<Vec<u8>, AppError> {
    let input = input.to_uppercase();
    // Count leading c32 zeros (the character '0')
    let leading_zeros = input.chars().take_while(|&c| c == '0').count();

    // Big-number base conversion: base32 → base256
    let mut acc: Vec<u8> = Vec::new();
    for ch in input.chars() {
        let val = C32_CHARS
            .find(ch)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid c32 character: {}", ch)))?;

        let mut carry = val;
        for byte in acc.iter_mut().rev() {
            let wide = (*byte as usize) * 32 + carry;
            *byte = (wide % 256) as u8;
            carry = wide / 256;
        }
        while carry > 0 {
            acc.insert(0, (carry % 256) as u8);
            carry /= 256;
        }
    }

    // Prepend zero bytes for leading '0' characters
    let mut result = vec![0u8; leading_zeros];
    result.extend(acc);
    Ok(result)
}

/// Serialize a Stacks address to Clarity principal hex.
///
/// Stacks addresses use c32check encoding:
///   Format: S + <version_char> + <c32_encoded(hash160 + checksum)>
///
/// Produces the Clarity serialized standard principal:
///   0x05 + version_byte + hash160 (22 bytes total)
fn serialize_principal(address: &str) -> Result<String, AppError> {
    let address = address.to_uppercase();

    if address.len() < 5 || !address.starts_with('S') {
        return Err(AppError::BadRequest(
            "Invalid Stacks address format".into(),
        ));
    }

    // Second character encodes the version byte via c32
    let version_char = address.chars().nth(1).unwrap();
    let version = C32_CHARS
        .find(version_char)
        .ok_or_else(|| AppError::BadRequest("Invalid address version character".into()))?
        as u8;

    // Remaining characters encode hash160 (20 bytes) + checksum (4 bytes)
    let data_str = &address[2..];
    let decoded = c32_decode(data_str)?;

    // Normalize to exactly 24 bytes (20 hash160 + 4 checksum)
    let hash160_with_checksum = if decoded.len() < 24 {
        let mut padded = vec![0u8; 24 - decoded.len()];
        padded.extend(&decoded);
        padded
    } else {
        decoded[decoded.len() - 24..].to_vec()
    };

    let hash160 = &hash160_with_checksum[..20];

    // Clarity standard principal: 0x05 + version + hash160
    let mut principal_bytes = vec![0x05, version];
    principal_bytes.extend_from_slice(hash160);
    Ok(format!("0x{}", hex::encode(principal_bytes)))
}

/// Get token information including price and minimum amount for the requested STX amount
pub async fn get_token_info(
    Path(contract_address_str): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<TokenInfoRequest>,
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

    // Special case: STX token just returns the requested amount directly, no API call needed
    if contract_address_str.eq_ignore_ascii_case("stx") {
        return Ok(Json(TokenInfo {
            price: get_token_price_data("stx", &state.redis).await.price_usd,
            minimum_amount: req.stx_amount,
        }));
    }

    // Fetch token price from cache
    let token_price = get_token_price_data(contract_address.as_str(), &state.redis).await.price_usd;
    if token_price <= 0.0 {
        return Err(AppError::NotFound("Token not found or price unavailable".to_string()).to_response());
    }

    // Fetch STX price from cache to calculate minimum
    let stx_price = get_token_price_data("stx", &state.redis).await.price_usd;
    if stx_price <= 0.0 {
        return Err(AppError::FetchError("STX price unavailable".to_string()).to_response());
    }

    // Minimum is the requested STX amount (defaults to 5) in equivalent token
    let minimum_amount = req.stx_amount * (stx_price / token_price);

    Ok(Json(TokenInfo {
        price: token_price,
        minimum_amount,
    }))
}

/// Check if a player has joined a vault contract
pub async fn has_joined(
    contract_address: &WalletAddress,
    player_address: &WalletAddress,
    state: &AppState,
) -> Result<bool, AppError> {
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

    let url = format!(
        "https://api.{}.hiro.so/v2/contracts/call-read/{}/{}/has-joined",
        network, principal, contract_name
    );

    let hex_principal = serialize_principal(player_address.as_str())?;

    let body = serde_json::json!({
        "sender": player_address.as_str(),
        "arguments": [hex_principal]
    });

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

    let okay = json.get("okay").and_then(|v| v.as_bool()).ok_or_else(|| {
        tracing::error!("Missing 'okay' field in response");
        AppError::Deserialization("Missing okay".into())
    })?;

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

    // Check if result is true (0x03 for true in Clarity)
    let has_joined = result == "0x03";
    if !has_joined {
        tracing::warn!(
            "Player has not joined - expected '0x03' (true), got '{}'",
            result
        );
    } else {
        tracing::info!("Player has joined contract {}", player_address.as_str());
    }

    Ok(has_joined)
}
