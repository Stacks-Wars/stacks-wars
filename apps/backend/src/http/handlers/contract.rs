use crate::{db::user::UserRepository, errors::AppError, models::WalletAddress, state::AppState};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

// Embed contract templates at compile time
const STX_VAULT_TEMPLATE: &str = include_str!("../../../contract/stacks/contracts/stx-vault.clar");
const FT_VAULT_TEMPLATE: &str = include_str!("../../../contract/stacks/contracts/ft-vault.clar");
const SPONSORED_STX_VAULT_TEMPLATE: &str = include_str!("../../../contract/stacks/contracts/sponsored-stx-vault.clar");
const SPONSORED_FT_VAULT_TEMPLATE: &str = include_str!("../../../contract/stacks/contracts/sponsored-ft-vault.clar");

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
) -> Result<Json<String>, (StatusCode, String)> {
    let user_repo = UserRepository::new(state.postgres);
    let creator_wallet = user_repo
        .find_by_id(query.game_creator_id)
        .await
        .map_err(|e| e.to_response())?
        .wallet_address;

    let contract_template = if query.contract_id.as_str() == "stx" {
        STX_VAULT_TEMPLATE
    } else {
        FT_VAULT_TEMPLATE
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

    Ok(Json(contract))
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
) -> Result<Json<String>, (StatusCode, String)> {
    let user_repo = UserRepository::new(state.postgres);
    let creator_wallet = user_repo
        .find_by_id(query.game_creator_id)
        .await
        .map_err(|e| e.to_response())?
        .wallet_address;

    let contract_template = if query.contract_id.as_str() == "stx" {
        SPONSORED_STX_VAULT_TEMPLATE
    } else {
        SPONSORED_FT_VAULT_TEMPLATE
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

    Ok(Json(contract))
}
