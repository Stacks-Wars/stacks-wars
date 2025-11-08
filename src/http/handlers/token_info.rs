//! Token Information Handlers
//!
//! Provides cryptocurrency token information for Stacks-based tokens.
//! Fetches real-time pricing data from external APIs for mainnet tokens
//! and provides mock data for testnet environments.

use axum::{extract::Path, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

// ============================================================================
// External API Types
// ============================================================================

/// Response structure from stxtools.io API
#[derive(Debug, Deserialize)]
struct TokenApiResponse {
    contract_id: String,
    symbol: String,
    decimals: u8,
    name: String,
    metrics: TokenMetrics,
}

/// Token price metrics from external API
#[derive(Debug, Deserialize)]
struct TokenMetrics {
    price_usd: f64,
}

// ============================================================================
// Response Types
// ============================================================================

/// Token information with pricing data
///
/// Contains comprehensive token details including real-time pricing
/// and calculated minimum amounts for platform entry fees.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Stacks contract address (e.g., "SP...::token-name")
    pub contract_id: String,
    /// Token symbol (e.g., "STX", "ALEX", "WELSH")
    pub symbol: String,
    /// Full token name
    pub name: String,
    /// Number of decimal places for the token
    pub decimals: u8,
    /// Current USD price per token
    pub price_usd: f64,
    /// Calculated minimum token amount for $10 USD equivalent
    pub minimum_amount: f64,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Fetch token information from external API and calculate minimums
///
/// Queries stxtools.io API for real-time token data and calculates
/// the minimum token amount needed for a $10 USD entry fee.
///
/// # Arguments
/// * `contract_address` - Stacks contract address (format: "SP...::token")
///
/// # Returns
/// * `Ok(TokenInfo)` - Token details with pricing
/// * `Err(AppError)` - API errors or token not found
///
/// # Minimum Amount Calculation
/// The minimum amount represents how many tokens are needed for a $10 USD entry:
/// - Expensive tokens (≥$1): Up to 6 decimal places
/// - Medium tokens ($0.01-$1): Up to 2 decimal places
/// - Low-value tokens ($0.001-$0.01): Up to 1 decimal place
/// - Very cheap tokens (<$0.001): Whole numbers only
pub async fn get_token_info(contract_address: String) -> Result<TokenInfo, AppError> {
    let url = format!("https://api.stxtools.io/tokens/{}", contract_address);

    // Fetch token data from external API
    let res = reqwest::get(&url).await.map_err(|e| {
        tracing::error!("Failed to fetch token info from stxtools.io: {}", e);
        AppError::BadRequest(format!("Failed to fetch token info: {}", e))
    })?;

    if !res.status().is_success() {
        let error_msg = format!("Token not found or API error: {}", contract_address);
        tracing::error!("{}", error_msg);
        return Err(AppError::NotFound(error_msg));
    }

    let token_data: TokenApiResponse = res.json().await.map_err(|e| {
        tracing::error!("Invalid JSON response from stxtools.io: {}", e);
        AppError::BadRequest(format!("Invalid JSON response: {}", e))
    })?;

    // Calculate minimum amount for $10 USD worth of tokens
    let minimum_usd_value = 10.0;
    let minimum_amount = if token_data.metrics.price_usd > 0.0 {
        let min_token_amount = minimum_usd_value / token_data.metrics.price_usd;

        // Smart rounding based on token price (prevents fractional display issues)
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
        tracing::warn!("Token has zero price, returning 0 minimum amount");
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

// ============================================================================
// Handlers
// ============================================================================

/// Get token information (mainnet)
///
/// Fetches real-time token information including current USD price and
/// calculated minimum amounts for platform entry fees.
///
/// # Authentication
/// - **Required**: No
///
/// # Path Parameters
/// - `contract_address` - Stacks contract address (e.g., "SP3K8BC0PPEVCV7NZ6QSRWPQ2JE9E5B6N3PA0KBR9.alex-token")
///
/// # Response
/// - **200 OK**: Token information retrieved
/// ```json
/// {
///   "contractId": "SP3K8BC0PPEVCV7NZ6QSRWPQ2JE9E5B6N3PA0KBR9.alex-token",
///   "symbol": "ALEX",
///   "name": "ALEX Token",
///   "decimals": 8,
///   "priceUsd": 0.15,
///   "minimumAmount": 66.67
/// }
/// ```
///
/// # Errors
/// - **404 Not Found**: Token not found in registry
/// - **500 Internal Server Error**: External API error
pub async fn get_token_info_mainnet(
    Path(contract_address): Path<String>,
) -> Result<Json<TokenInfo>, (StatusCode, String)> {
    get_token_info(contract_address)
        .await
        .map(Json)
        .map_err(|e| {
            let status = match e {
                AppError::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string())
        })
}

/// Get token information (testnet)
///
/// Returns mock token data for testnet environments. Provides fixed values
/// for STX or generic test tokens for development and testing.
///
/// # Authentication
/// - **Required**: No
///
/// # Path Parameters
/// - `contract_address` - Token identifier ("stx" for STX, any other for test tokens)
///
/// # Response
/// - **200 OK**: Token information (mock data)
/// ```json
/// {
///   "contractId": "ST...::test-token",
///   "symbol": "TEST",
///   "name": "Test Token",
///   "decimals": 6,
///   "priceUsd": 0.01,
///   "minimumAmount": 3000.0
/// }
/// ```
///
/// # Notes
/// - For "stx": Returns real STX token data from mainnet API
/// - For other addresses: Returns mock test token data
pub async fn get_token_info_testnet(
    Path(contract_address): Path<String>,
) -> Result<Json<TokenInfo>, (StatusCode, String)> {
    // For STX, fetch real mainnet data even in testnet
    if contract_address.to_lowercase() == "stx" {
        return get_token_info(contract_address)
            .await
            .map(Json)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    // Return fixed testnet token info for development
    let token_info = TokenInfo {
        contract_id: contract_address.clone(),
        symbol: "TEST".to_string(),
        name: "Test Token".to_string(),
        decimals: 6,
        price_usd: 0.01,
        minimum_amount: 3000.0,
    };

    tracing::debug!(
        "Returning mock testnet token info for: {}",
        contract_address
    );
    Ok(Json(token_info))
}
