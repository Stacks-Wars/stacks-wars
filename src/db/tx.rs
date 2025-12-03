use crate::errors::AppError;

pub async fn validate_payment_tx(
    tx_id: &str,
    expected_sender: &str,
    expected_contract: &str,
    expected_amount: f64,
) -> Result<(), AppError> {
    let network = std::env::var("STACKS_NETWORK").unwrap_or("testnet".to_string());
    let url = format!("https://api.{network}.hiro.so/extended/v1/tx/{}", tx_id);

    let res = reqwest::get(&url)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to fetch tx: {}", e)))?;

    if !res.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "Transaction not found or failed: {}",
            tx_id
        )));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| AppError::Deserialization(format!("Invalid JSON response: {}", e)))?;

    // Validate sender
    let sender_address = json
        .get("sender_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing sender address".into()))?;

    if sender_address != expected_sender {
        return Err(AppError::BadRequest(format!(
            "Unexpected sender address: {}",
            sender_address
        )));
    }

    // Validate transaction status
    let status = json
        .get("tx_status")
        .and_then(|v| v.as_str())
        .unwrap_or("failed");

    if status != "success" {
        return Err(AppError::BadRequest("Transaction failed".into()));
    }

    // Validate amount and recipient
    let empty_vec = Vec::new();
    let events = json
        .get("events")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);
    tracing::info!("Processing {:#?}", events);
    let mut matched = None;

    for event in events {
        let Some(event_type) = event.get("event_type").and_then(|et| et.as_str()) else {
            tracing::warn!("Skipping event: missing event_type");
            continue;
        };

        if event_type != "stx_asset" && event_type != "fungible_token_asset" {
            tracing::debug!(
                "Skipping event: not stx_asset or fungible_token_asset, got {event_type}"
            );
            continue;
        }

        let Some(asset) = event.get("asset") else {
            tracing::warn!("stx_asset/fungible_token_asset event missing 'asset' field");
            continue;
        };

        let recipient_matches = asset
            .get("recipient")
            .and_then(|r| r.as_str())
            .map(|r| {
                let m = r == expected_contract;
                if !m {
                    tracing::debug!("Recipient mismatch: expected {expected_contract}, got {r}");
                }
                m
            })
            .unwrap_or_else(|| {
                tracing::warn!("Missing recipient in asset event");
                false
            });

        let amount_matches = asset
            .get("amount")
            .and_then(|a| a.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|a| {
                let m = a == expected_amount * 1_000_000.0;
                if !m {
                    tracing::debug!("Amount mismatch: expected {expected_amount}, got {a}");
                }
                m
            })
            .unwrap_or_else(|| {
                tracing::warn!("Missing or invalid amount in asset event");
                false
            });

        if recipient_matches && amount_matches {
            matched = Some(event);
            break;
        }
    }

    if matched.is_none() {
        return Err(AppError::BadRequest(
            "No matching STX asset transfer event found".into(),
        ));
    }

    Ok(())
}

pub async fn validate_fee_transfer(
    tx_id: &str,
    expected_sender: &str,
    fee_wallet: &str,
) -> Result<(), AppError> {
    let network = std::env::var("STACKS_NETWORK").unwrap_or("testnet".to_string());
    let url = format!("https://api.{network}.hiro.so/extended/v1/tx/{}", tx_id);

    let res = reqwest::get(&url)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to fetch tx: {}", e)))?;

    if !res.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "Transaction not found or failed: {}",
            tx_id
        )));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| AppError::Deserialization(format!("Invalid JSON response: {}", e)))?;

    // Validate sender
    let sender_address = json
        .get("sender_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing sender address".into()))?;

    if sender_address != expected_sender {
        return Err(AppError::BadRequest(format!(
            "Unexpected sender address: {}, expected: {}",
            sender_address, expected_sender
        )));
    }

    // Validate transaction status
    let status = json
        .get("tx_status")
        .and_then(|v| v.as_str())
        .unwrap_or("failed");

    if status != "success" {
        return Err(AppError::BadRequest("Transaction failed".into()));
    }

    // Validate transaction type (should be token_transfer for STX transfers)
    let tx_type = json
        .get("tx_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing transaction type".into()))?;

    if tx_type != "token_transfer" {
        return Err(AppError::BadRequest(
            "Invalid transaction type, expected token_transfer".into(),
        ));
    }

    // Validate recipient and amount from token_transfer field
    let token_transfer = json
        .get("token_transfer")
        .ok_or_else(|| AppError::BadRequest("Missing token_transfer data".into()))?;

    let recipient_address = token_transfer
        .get("recipient_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing recipient address".into()))?;

    if recipient_address != fee_wallet {
        return Err(AppError::BadRequest(format!(
            "Unexpected recipient address: expected {}, got {}",
            fee_wallet, recipient_address
        )));
    }

    let amount_str = token_transfer
        .get("amount")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing transfer amount".into()))?;

    let amount = amount_str
        .parse::<f64>()
        .map_err(|_| AppError::BadRequest("Invalid amount format".into()))?;

    // Expected amount is 0.2 STX = 200,000 microSTX
    let expected_amount = 200_000.0;
    if amount != expected_amount {
        return Err(AppError::BadRequest(format!(
            "Invalid fee amount: expected {} microSTX (0.2 STX), got {} microSTX",
            expected_amount, amount
        )));
    }

    Ok(())
}
