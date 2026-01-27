use crate::{db::user::UserRepository, errors::AppError, models::WalletAddress, state::AppState};
use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractQuery {
    pub game_creator_id: Uuid,
    pub entry_fee: u32,
    pub contract_id: WalletAddress,
}

pub async fn get_contract(
    State(state): State<AppState>,
    Query(query): Query<ContractQuery>,
) -> Result<String, (StatusCode, String)> {
    let user_repo = UserRepository::new(state.postgres);
    let creator_wallet = user_repo
        .find_by_id(query.game_creator_id)
        .await
        .map_err(|e| e.to_response())?
        .wallet_address;

    let contract_template = if query.contract_id.as_str() == "stx" {
        fs::read_to_string("contract/stacks/contracts/stx-vault.clar").map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read contract template".to_string(),
            )
        })?
    } else {
        fs::read_to_string("contract/stacks/contracts/ft-vault.clar").map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read contract template".to_string(),
            )
        })?
    };

    let contract = contract_template
        .replace("u5000000", &format!("u{}000000", query.entry_fee))
        .replace(
            "ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5",
            &creator_wallet.as_str(),
        )
        .replace(
            ".STACKS-WARS-TOKEN",
            &format!("'{}", query.contract_id.as_str()),
        );

    Ok(contract)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsoredContractQuery {
    pub game_creator_id: Uuid,
    pub pool_size: u32,
    pub contract_id: WalletAddress,
}

pub async fn get_sponsored_contract(
    State(state): State<AppState>,
    Query(query): Query<SponsoredContractQuery>,
) -> Result<String, (StatusCode, String)> {
    let user_repo = UserRepository::new(state.postgres);
    let creator_wallet = user_repo
        .find_by_id(query.game_creator_id)
        .await
        .map_err(|e| e.to_response())?
        .wallet_address;

    let contract_template = if query.contract_id.as_str() == "stx" {
        fs::read_to_string("contract/stacks/contracts/sponsored-stx-vault.clar").map_err(|e| {
            AppError::ReadError(format!("Failed to read contract template: {}", e)).to_response()
        })?
    } else {
        fs::read_to_string("contract/stacks/contracts/sponsored-ft-vault.clar").map_err(|e| {
            AppError::ReadError(format!("Failed to read contract template: {}", e)).to_response()
        })?
    };

    let contract = contract_template
        .replace("u10000000", &format!("u{}000000", query.pool_size))
        .replace(
            "ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5",
            &creator_wallet.as_str(),
        )
        .replace(
            ".STACKS-WARS-TOKEN",
            &format!("'{}", query.contract_id.as_str()),
        );

    Ok(contract)
}
