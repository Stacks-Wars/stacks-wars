//! `/api/leaderboards/*` — `http::leaderboards`
use reqwest;

#[tokio::test]
async fn get_leaderboard_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/leaderboards", app.base_url))
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");
    let leaderboard = body
        .get("leaderboard")
        .and_then(|v| v.as_array())
        .expect("leaderboard array");
    let total = body.get("total").and_then(|v| v.as_i64()).expect("total");
    assert!(total >= 0);
    if let Some(first) = leaderboard.first() {
        assert!(first.get("userId").is_some());
        assert!(first.get("points").is_some());
    }

    app.stop().await;
}

#[tokio::test]
async fn get_leaderboard_pagination_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "{}/api/leaderboards?limit=5&offset=0",
            app.base_url
        ))
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");
    let arr = body
        .get("leaderboard")
        .and_then(|v| v.as_array())
        .expect("leaderboard array");
    assert!(arr.len() <= 5, "should not return more than 5 entries");

    app.stop().await;
}

#[tokio::test]
async fn get_player_leaderboard_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (user_id, _t) = factory.create_test_user(None).await.expect("user");

    let resp = client
        .get(format!(
            "{}/api/leaderboards/{}",
            app.base_url, user_id
        ))
        .send()
        .await
        .expect("request failed");
    let status = resp.status();
    assert!(
        status.is_success() || status == 404,
        "unexpected status {status}"
    );

    app.stop().await;
}

#[tokio::test]
async fn get_game_leaderboard_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (creator_id, _t) = factory.create_test_user(None).await.expect("user");
    let game_id = factory
        .create_test_game(creator_id, Some("lb-game"))
        .await
        .expect("game");

    let resp = client
        .get(format!(
            "{}/api/leaderboards/game/{}",
            app.base_url, game_id
        ))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}

#[tokio::test]
async fn get_user_top_games_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (user_id, _t) = factory.create_test_user(None).await.expect("user");

    let resp = client
        .get(format!(
            "{}/api/leaderboards/user/{}/top-games?limit=6",
            app.base_url, user_id
        ))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}

#[tokio::test]
async fn get_platform_game_stats_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "{}/api/leaderboards/stats/games",
            app.base_url
        ))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}
