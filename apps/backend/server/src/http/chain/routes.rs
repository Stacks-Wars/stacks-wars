use axum::{Router, routing::{get, post}};

use crate::state::AppState;

use super::{contract, stacks};

/// Mounted at `/api/chain`.
pub fn read_routes() -> Router<AppState> {
    Router::new()
        .route("/token/{contract_address}", post(stacks::get_token_info))
        .route("/contract", get(contract::get_contract))
        .route("/sponsored-contract", get(contract::get_sponsored_contract))
        .route("/balance/{wallet_address}", get(stacks::get_balance))
}
