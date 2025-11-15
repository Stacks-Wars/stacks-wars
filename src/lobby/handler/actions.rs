//! Lobby action helpers. These methods perform repository updates and
//! broadcasting. They return typed `LobbyError` on failure and may return
//! an optional `LobbyServerMessage` that should be sent to the initiating
//! connection in addition to any broadcasts.
use super::LobbySession;
use crate::db::{
    game::GameRepository, lobby::LobbyRepository, lobby_state::LobbyStateRepository,
    player_state::PlayerStateRepository,
};
use crate::lobby::handler::error::LobbyError;
use crate::lobby::handler::messages::LobbyServerMessage;
use crate::lobby::manager;
use crate::models::redis::PlayerState as PlayerStateModel;
impl LobbySession {
    /// Attempt to add the handler's `user_id` as a player in `lobby_id`.
    /// Returns an optional server message to send directly to the caller on success.
    pub async fn join_lobby(&self) -> Result<Option<LobbyServerMessage>, LobbyError> {
        let lobby_id = self.lobby_id;
        let user_id = self.user_id;

        // validate lobby exists and capacity
        let lobby_repo = LobbyRepository::new(self.state.postgres.clone());
        if let Ok(Some(lobby_row)) = lobby_repo.find_by_id(lobby_id).await.map(|r| r) {
            let game_repo = GameRepository::new(self.state.postgres.clone());
            if let Ok(game) = game_repo.find_by_id(lobby_row.game_id).await {
                let max_players_opt = Some(game.max_players as usize);
                if let Ok(state_info) = LobbyStateRepository::new(self.state.redis.clone())
                    .get_state(lobby_id)
                    .await
                {
                    let full = max_players_opt
                        .map(|max| state_info.participant_count >= max)
                        .unwrap_or(false);
                    if full {
                        return Err(LobbyError::LobbyFull);
                    }

                    // create player state
                    let player_repo = PlayerStateRepository::new(self.state.redis.clone());
                    let new_state = PlayerStateModel::new(user_id, lobby_id, None);
                    if let Err(e) = player_repo.create_state(new_state).await {
                        return Err(LobbyError::JoinFailed(e.to_string()));
                    }

                    // broadcast updated player list
                    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                        manager::broadcast(
                            &self.state,
                            lobby_id,
                            &LobbyServerMessage::PlayerUpdated {
                                players: players.clone(),
                            },
                        )
                        .await;
                        return Ok(Some(LobbyServerMessage::PlayerUpdated { players }));
                    }
                }
            }
        }

        Err(LobbyError::Generic("join failed".to_string()))
    }

    /// Remove the handler's `user_id` from the lobby and broadcast the updated list.
    pub async fn leave_lobby(&self) -> Result<Option<LobbyServerMessage>, LobbyError> {
        let lobby_id = self.lobby_id;
        let user_id = self.user_id;

        let player_repo = PlayerStateRepository::new(self.state.redis.clone());
        let _ = player_repo.remove_from_lobby(lobby_id, user_id).await;
        if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
            manager::broadcast(
                &self.state,
                lobby_id,
                &LobbyServerMessage::PlayerUpdated {
                    players: players.clone(),
                },
            )
            .await;
            return Ok(Some(LobbyServerMessage::PlayerUpdated { players }));
        }
        Ok(None)
    }

    /// Trigger the start countdown if the caller is the creator and the lobby
    /// has enough players. This updates lobby state and broadcasts progress.
    pub async fn toggle_start_countdown(&self) -> Result<Option<LobbyServerMessage>, LobbyError> {
        let lobby_id = self.lobby_id;
        let user_id = self.user_id;

        let lobby_repo = LobbyRepository::new(self.state.postgres.clone());
        if let Ok(Some(lobby)) = lobby_repo.find_by_id(lobby_id).await.map(|r| r) {
            if lobby.creator_id != user_id {
                return Err(LobbyError::NotCreator);
            }

            let repo = GameRepository::new(self.state.postgres.clone());
            if let Ok(game) = repo.find_by_id(lobby.game_id).await {
                let min_players = game.min_players as usize;

                if let Ok(state_info) = LobbyStateRepository::new(self.state.redis.clone())
                    .get_state(lobby_id)
                    .await
                {
                    if state_info.participant_count < min_players {
                        return Err(LobbyError::NeedAtLeast(min_players));
                    }

                    let lobby_state_repo = LobbyStateRepository::new(self.state.redis.clone());
                    let _ = lobby_state_repo
                        .update_status(
                            lobby_id,
                            crate::models::redis::lobby_state::LobbyStatus::Starting,
                        )
                        .await;
                    if let Ok(new_info) = lobby_state_repo.get_state(lobby_id).await {
                        manager::broadcast(
                            &self.state,
                            lobby_id,
                            &LobbyServerMessage::LobbyState {
                                state: new_info.status.clone(),
                                joined_players: None,
                                started: false,
                            },
                        )
                        .await;
                        // return an explicit state message to the caller
                        return Ok(Some(LobbyServerMessage::LobbyState {
                            state: new_info.status.clone(),
                            joined_players: None,
                            started: false,
                        }));
                    }

                    let state_clone = self.state.clone();
                    let redis = self.state.redis.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        if let Ok(info2) =
                            LobbyStateRepository::new(redis).get_state(lobby_id).await
                        {
                            if info2.participant_count >= min_players {
                                let lobby_state_repo =
                                    LobbyStateRepository::new(state_clone.redis.clone());
                                let _ = lobby_state_repo
                                    .update_status(
                                        lobby_id,
                                        crate::models::redis::lobby_state::LobbyStatus::InProgress,
                                    )
                                    .await;
                                if let Ok(info3) = lobby_state_repo.get_state(lobby_id).await {
                                    manager::broadcast(
                                        &state_clone,
                                        lobby_id,
                                        &LobbyServerMessage::LobbyState {
                                            state: info3.status.clone(),
                                            joined_players: None,
                                            started: true,
                                        },
                                    )
                                    .await;
                                }
                            } else {
                                let lobby_state_repo =
                                    LobbyStateRepository::new(state_clone.redis.clone());
                                let _ = lobby_state_repo
                                    .update_status(
                                        lobby_id,
                                        crate::models::redis::lobby_state::LobbyStatus::Waiting,
                                    )
                                    .await;
                                if let Ok(info4) = lobby_state_repo.get_state(lobby_id).await {
                                    manager::broadcast(
                                        &state_clone,
                                        lobby_id,
                                        &LobbyServerMessage::LobbyState {
                                            state: info4.status.clone(),
                                            joined_players: None,
                                            started: false,
                                        },
                                    )
                                    .await;
                                }
                            }
                        }
                    });

                    return Ok(None);
                }
            }
        }
        Err(LobbyError::Generic("toggle failed".to_string()))
    }
}
