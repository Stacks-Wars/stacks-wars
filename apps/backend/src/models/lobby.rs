use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use super::WalletAddress;
use crate::models::{Game, LobbyState, LobbyStatus, User};

/// Lobby model mapping to the `lobbies` table (room metadata and status).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Lobby {
    #[serde(skip_deserializing)]
    pub(crate) id: Uuid,
    pub path: String,
    pub name: String,
    pub description: Option<String>,
    pub game_id: Uuid,
    pub game_path: String,
    pub creator_id: Uuid,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub token_contract_id: Option<WalletAddress>,
    pub contract_address: Option<WalletAddress>,
    pub is_private: bool,
    pub is_sponsored: bool,
    pub status: LobbyStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Lobby {
    /// Get lobby ID.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Validate amount is positive (if present).
    pub fn validate_amount(amount: Option<f64>) -> Result<Option<f64>, LobbyAmountError> {
        if let Some(amt) = amount {
            if amt < 0.0 {
                return Err(LobbyAmountError::Negative { amount: amt });
            }
            if amt.is_nan() || amt.is_infinite() {
                return Err(LobbyAmountError::Invalid { amount: amt });
            }
        }
        Ok(amount)
    }

    /// Validate lobby creation amounts based on sponsor status.
    ///
    /// Rules:
    /// - If `is_sponsored` is true:
    ///   - `entry_amount` must be None (sponsor pays)
    ///   - `current_amount` must be Some(x) where x > 0
    /// - If `is_sponsored` is false:
    ///   - `entry_amount` must equal `current_amount` (creator has paid)
    ///   - Both can be None (free lobby) or Some(same_value)
    pub fn validate_creation_amounts(
        entry_amount: Option<f64>,
        current_amount: Option<f64>,
        is_sponsored: bool,
    ) -> Result<(Option<f64>, Option<f64>), LobbyAmountError> {
        // First validate individual amounts
        let entry_amount = Self::validate_amount(entry_amount)?;
        let current_amount = Self::validate_amount(current_amount)?;

        if is_sponsored {
            // Sponsored lobby: entry_amount must be None, current_amount must be Some and > 0
            if entry_amount.is_some() {
                return Err(LobbyAmountError::SponsoredWithEntry);
            }
            match current_amount {
                None => return Err(LobbyAmountError::SponsoredWithoutCurrent),
                Some(amt) if amt <= 0.0 => return Err(LobbyAmountError::SponsoredCurrentZero),
                _ => {} // Valid
            }
        } else {
            // Non-sponsored: entry_amount must equal current_amount
            if entry_amount != current_amount {
                return Err(LobbyAmountError::MismatchedAmounts {
                    entry: entry_amount,
                    current: current_amount,
                });
            }
        }

        Ok((entry_amount, current_amount))
    }
}

/// Lobby amount validation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LobbyAmountError {
    #[error("Amount cannot be negative: {amount}")]
    Negative { amount: f64 },

    #[error("Amount is invalid (NaN or Infinite): {amount}")]
    Invalid { amount: f64 },

    #[error(
        "Sponsored lobby cannot have entry_amount set. Entry must be None for sponsored lobbies."
    )]
    SponsoredWithEntry,

    #[error("Sponsored lobby must have current_amount set with the sponsor's contribution.")]
    SponsoredWithoutCurrent,

    #[error("Sponsored lobby current_amount must be greater than 0.")]
    SponsoredCurrentZero,

    #[error(
        "Non-sponsored lobby: entry_amount must equal current_amount (creator pays). Got entry={entry:?}, current={current:?}"
    )]
    MismatchedAmounts {
        entry: Option<f64>,
        current: Option<f64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyInfo {
    pub lobby: LobbyExtended,
    pub game: Game,
    pub creator: User,
}

/// Flattened Lobby payload combining Postgres metadata and Redis runtime fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyExtended {
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub description: Option<String>,
    pub game_id: Uuid,
    pub game_path: String,
    pub creator_id: Uuid,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub token_contract_id: Option<WalletAddress>,
    pub contract_address: Option<WalletAddress>,
    pub is_private: bool,
    pub is_sponsored: bool,
    pub status: LobbyStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    // Runtime fields from Redis
    pub participant_count: usize,
    pub creator_last_ping: Option<u64>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

impl LobbyExtended {
    /// Build a flattened extended lobby payload from Postgres `Lobby` and Redis `LobbyState`.
    pub fn from_parts(lobby: Lobby, state_info: LobbyState) -> Self {
        Self {
            id: lobby.id,
            path: lobby.path,
            name: lobby.name,
            description: lobby.description,
            game_id: lobby.game_id,
            game_path: lobby.game_path,
            creator_id: lobby.creator_id,
            entry_amount: lobby.entry_amount,
            current_amount: lobby.current_amount,
            token_symbol: lobby.token_symbol,
            token_contract_id: lobby.token_contract_id,
            contract_address: lobby.contract_address,
            is_private: lobby.is_private,
            is_sponsored: lobby.is_sponsored,
            status: lobby.status,
            created_at: lobby.created_at,
            updated_at: lobby.updated_at,
            participant_count: state_info.participant_count,
            creator_last_ping: state_info.creator_last_ping,
            started_at: state_info.started_at,
            finished_at: state_info.finished_at,
        }
    }
}
