use crate::{
    db::{
        lobby::{
            get::{get_lobby_info, get_lobby_players},
            join_requests::remove_join_request,
            patch::leave_lobby,
        },
        user::get::get_user_by_id,
    },
    models::{
        game::{LobbyState, Player, PlayerState},
        lobby::LobbyServerMessage,
    },
    state::{ChatConnectionInfoMap, ConnectionInfoMap, RedisClient},
    ws::handlers::lobby::message_handler::{
        broadcast_to_lobby,
        handler::{send_error_to_player, send_to_player},
    },
};
use uuid::Uuid;

pub async fn kick_player(
    player_id: Uuid,
    lobby_id: Uuid,
    player: &Player,
    connections: &ConnectionInfoMap,
    chat_connections: &ChatConnectionInfoMap,
    redis: &RedisClient,
    bot: teloxide::Bot,
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
        tracing::error!("Unauthorized kick attempt by {}", player.id);
        send_error_to_player(
            player.id,
            lobby_id,
            "Only creator can kick players",
            &connections,
            &redis,
        )
        .await;
        return;
    }

    if lobby_info.state != LobbyState::Waiting {
        tracing::error!("Cannot kick players when game is not waiting");
        send_error_to_player(
            player.id,
            lobby_id,
            "Cannot kick player when game is in starting",
            &connections,
            &redis,
        )
        .await;
        return;
    }

    // Remove player
    if let Err(e) = leave_lobby(lobby_id, player_id, redis.clone(), bot).await {
        tracing::error!("Failed to kick player: {}", e);
        send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
    } else if let Ok(players) =
        get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await
    {
        if let Err(e) = remove_join_request(lobby_id, player_id, redis.clone()).await {
            tracing::warn!(
                "Failed to remove join request for kicked player {}: {}",
                player_id,
                e
            );
        }
        let player_updated_msg = LobbyServerMessage::PlayerUpdated { players };
        broadcast_to_lobby(
            lobby_id,
            &player_updated_msg,
            &connections,
            Some(&chat_connections),
            redis.clone(),
        )
        .await;

        tracing::info!("Success kicking {} from {}", player.id, lobby_id);
        let kicked_user = match get_user_by_id(player_id, redis.clone()).await {
            Ok(user) => user,
            Err(e) => {
                tracing::error!("Failed to fetch player info: {}", e);
                send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis)
                    .await;
                return;
            }
        };

        let kicked_msg = LobbyServerMessage::PlayerKicked {
            player: kicked_user,
        };
        broadcast_to_lobby(lobby_id, &kicked_msg, &connections, None, redis.clone()).await;

        let notify_kicked_msg = LobbyServerMessage::NotifyKicked;
        send_to_player(
            player_id,
            lobby_id,
            &connections,
            &notify_kicked_msg,
            &redis,
        )
        .await;
        send_to_player(
            player_id,
            lobby_id,
            &connections,
            &player_updated_msg,
            &redis,
        )
        .await;
    }
}
