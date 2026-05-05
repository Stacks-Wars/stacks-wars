use serde::{Deserialize, Serialize};

/// Represents a token balance for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// Token name (e.g., "stacks" for STX, "sbtc-token" for fungible tokens)
    pub name: String,
    /// Token balance in actual units (converted from micro units)
    pub balance: f64,
    /// Contract ID (e.g., "stx" for STX, full contract address for fungible tokens)
    pub contract_id: String,
}

/// Token information with pricing and minimum amount for $10 USD equivalent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Current USD price per token
    pub price: f64,
    /// Calculated minimum token amount for $10 USD equivalent
    pub minimum_amount: f64,
}
