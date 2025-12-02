use crate::{
    db::lobby::{get::get_lobby_info, join_requests::get_player_join_request},
    models::{
        game::{LobbyState, Player},
        lobby::{JoinState, LobbyServerMessage},
    },
    state::{ConnectionInfoMap, RedisClient},
    ws::handlers::lobby::message_handler::{
        broadcast_to_lobby,
        handler::{get_pending_players, send_error_to_player, send_to_player, set_join_state},
    },
};
use uuid::Uuid;

pub async fn permit_join(
    allow: bool,
    user_id: Uuid,
    player: Player,
    lobby_id: Uuid,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    let lobby_info = match get_lobby_info(lobby_id, redis.clone()).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Failed to fetch lobby info: {}", e);
            send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
            return;
        }
    };

    if lobby_info.creator.id != player.id {
        tracing::error!("Unauthorized permit attempt by {}", player.id);
        send_error_to_player(
            player.id,
            lobby_id,
            "Only lobby creator can permit joins".to_string(),
            &connections,
            &redis,
        )
        .await;
        return;
    }

    if lobby_info.state != LobbyState::Waiting {
        tracing::error!("Cannot permit joins when game is not waiting");
        send_error_to_player(
            player.id,
            lobby_id,
            "Cannot permit joins when game is starting".to_string(),
            &connections,
            &redis,
        )
        .await;
        return;
    }

    // Check if user has a join request
    let join_request = match get_player_join_request(lobby_id, user_id, redis.clone()).await {
        Ok(Some(request)) => request,
        Ok(None) => {
            send_error_to_player(
                player.id,
                lobby_id,
                "No join request found for this user",
                &connections,
                &redis,
            )
            .await;
            return;
        }
        Err(e) => {
            tracing::error!("Failed to get join request: {}", e);
            send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
            return;
        }
    };

    let new_state = if allow {
        JoinState::Allowed
    } else {
        JoinState::Rejected
    };

    if let Err(e) = set_join_state(
        lobby_id,
        join_request.user.clone(),
        new_state.clone(),
        redis.clone(),
    )
    .await
    {
        tracing::error!("Failed to update join state: {}", e);
        send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
        return;
    }

    // Send response to the user who requested to join
    let response_msg = if allow {
        LobbyServerMessage::Allowed
    } else {
        LobbyServerMessage::Rejected
    };
    send_to_player(user_id, lobby_id, &connections, &response_msg, &redis).await;

    // Get updated pending players
    if let Ok(pending_players) = get_pending_players(lobby_id, redis.clone()).await {
        let pending_msg = LobbyServerMessage::PendingPlayers { pending_players };
        broadcast_to_lobby(lobby_id, &pending_msg, &connections, None, redis.clone()).await;
        send_to_player(user_id, lobby_id, &connections, &pending_msg, &redis).await;
    }

    tracing::info!(
        "Player {} {} user {} for lobby {}",
        player.id,
        if allow { "allowed" } else { "rejected" },
        user_id,
        lobby_id
    );
}
