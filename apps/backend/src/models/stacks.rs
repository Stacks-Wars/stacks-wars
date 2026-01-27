use serde::{Deserialize, Serialize};

/// Represents a token balance for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token name (e.g., "stacks" for STX, "sbtc-token" for fungible tokens)
    pub name: String,
    /// Token balance in actual units (converted from micro units)
    pub balance: f64,
    /// Contract ID (e.g., "stx" for STX, full contract address for fungible tokens)
    pub contract_id: String,
}