use crate::{
    db::{
        lobby::{
            get::{get_lobby_info, get_lobby_players},
            join_requests::remove_join_request,
            patch,
        },
        user::patch::decrease_wars_point,
    },
    models::{
        game::{Player, PlayerState},
        lobby::LobbyServerMessage,
    },
    state::{ChatConnectionInfoMap, ConnectionInfoMap, RedisClient},
    ws::handlers::lobby::message_handler::{
        broadcast_to_lobby,
        handler::{send_error_to_player, send_to_player},
    },
};
use uuid::Uuid;

pub async fn leave_lobby(
    lobby_id: Uuid,
    player: &Player,
    connections: &ConnectionInfoMap,
    chat_connections: &ChatConnectionInfoMap,
    redis: &RedisClient,
    bot: teloxide::Bot,
) {
    if let Err(e) = patch::leave_lobby(lobby_id, player.id, redis.clone(), bot).await {
        tracing::error!("Failed to leave lobby: {}", e);
        send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
    } else if let Ok(players) =
        get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await
    {
        tracing::info!("Player {} left lobby {}", player.id, lobby_id);

        // Remove join request when leaving
        if let Err(e) = remove_join_request(lobby_id, player.id, redis.clone()).await {
            tracing::warn!(
                "Failed to remove join request for player {}: {}",
                player.id,
                e
            );
        }

        match get_lobby_info(lobby_id, redis.clone()).await {
            Ok(lobby_info) => {
                if lobby_info.creator.id != player.id {
                    // Subtract 10 wars points for leaving the lobby (only for non-creators)
                    match decrease_wars_point(player.id, 10.0, redis.clone()).await {
                        Ok(new_total) => {
                            tracing::info!(
                                "Subtracted 10 wars points from player {} for leaving lobby. New total: {}",
                                player.id,
                                new_total
                            );

                            let wars_point_msg = LobbyServerMessage::WarsPointDeduction {
                                amount: 10.0,
                                new_total,
                                reason: "Left lobby".to_string(),
                            };
                            send_to_player(
                                player.id,
                                lobby_id,
                                connections,
                                &wars_point_msg,
                                redis,
                            )
                            .await;
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to subtract wars points from player {} for leaving lobby: {}",
                                player.id,
                                e
                            );
                        }
                    }
                } else {
                    tracing::debug!(
                        "Player {} is the lobby creator, no wars points deducted for leaving",
                        player.id
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to get lobby info to check creator: {}. Proceeding without wars point deduction.",
                    e
                );
            }
        }

        let msg = LobbyServerMessage::PlayerUpdated { players };
        broadcast_to_lobby(
            lobby_id,
            &msg,
            &connections,
            Some(&chat_connections),
            redis.clone(),
        )
        .await;
        send_to_player(player.id, lobby_id, &connections, &msg, redis).await;
        let left_msg = LobbyServerMessage::Left;
        send_to_player(player.id, lobby_id, &connections, &left_msg, redis).await;
    }
}
