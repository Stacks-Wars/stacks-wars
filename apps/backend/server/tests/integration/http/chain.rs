//! `/api/chain/*` — `http::chain`
use reqwest;
use serde_json::json;

#[tokio::test]
async fn get_contract_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (creator_id, _t) = factory.create_test_user(None).await.expect("user");

    let resp = client
        .get(format!(
            "{}/api/chain/contract?gameCreatorId={}&entryFee=1&contractId=stx",
            app.base_url, creator_id
        ))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let text = resp.text().await.expect("body");
    assert!(text.contains(";;") || text.len() > 10);

    app.stop().await;
}

#[tokio::test]
async fn get_sponsored_contract_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (creator_id, _t) = factory.create_test_user(None).await.expect("user");

    let resp = client
        .get(format!(
            "{}/api/chain/sponsored-contract?gameCreatorId={}&poolSize=1&contractId=stx",
            app.base_url, creator_id
        ))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}

#[tokio::test]
async fn post_token_info_testnet_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let contract = "SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.leo-token";
    let resp = client
        .post(format!(
            "{}/api/chain/token/{}",
            app.base_url, contract
        ))
        .json(&json!({ "stxAmount": 5.0 }))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("json");
    assert!(body.get("price").is_some());
    assert!(body.get("minimumAmount").is_some());

    app.stop().await;
}

#[tokio::test]
async fn get_balance_hits_hiro_or_errors_gracefully() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let wallet = "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7";
    let resp = client
        .get(format!("{}/api/chain/balance/{}", app.base_url, wallet))
        .send()
        .await
        .expect("request failed");
    // External Hiro dependency: accept success or server-side failure, not client/routing error.
    let s = resp.status();
    assert_ne!(s, 404);
    assert_ne!(s, 401);
    assert_ne!(s, 400);

    app.stop().await;
}
