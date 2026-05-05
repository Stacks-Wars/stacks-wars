//! Pure game mechanics and result structures (no I/O).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Per-player in-memory game tracking (separate from lobby [`crate::dto::PlayerStateWire`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayerState {
    pub user_id: Uuid,
    pub is_eliminated: bool,
    pub position: Option<usize>,
    pub score: i32,
    pub eliminated_at: Option<i64>,
}

impl GamePlayerState {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            is_eliminated: false,
            position: None,
            score: 0,
            eliminated_at: None,
        }
    }

    pub fn eliminate(&mut self) {
        self.is_eliminated = true;
        self.eliminated_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn is_active(&self) -> bool {
        !self.is_eliminated
    }
}

/// Turn rotation with elimination support.
#[derive(Debug, Clone)]
pub struct TurnRotation {
    players: VecDeque<Uuid>,
    current_index: usize,
    eliminated: HashMap<Uuid, bool>,
}

impl TurnRotation {
    pub fn new(player_ids: Vec<Uuid>) -> Self {
        Self {
            players: player_ids.into_iter().collect(),
            current_index: 0,
            eliminated: HashMap::new(),
        }
    }

    pub fn current_player(&self) -> Option<Uuid> {
        self.active_players().get(self.current_index).copied()
    }

    pub fn active_players(&self) -> Vec<Uuid> {
        self.players
            .iter()
            .filter(|id| !self.eliminated.get(id).unwrap_or(&false))
            .copied()
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.active_players().len()
    }

    pub fn next_turn(&mut self) -> Option<Uuid> {
        if self.active_count() == 0 {
            return None;
        }

        let active = self.active_players();
        self.current_index = (self.current_index + 1) % active.len();
        active.get(self.current_index).copied()
    }

    pub fn eliminate_player(&mut self, player_id: Uuid) {
        self.eliminated.insert(player_id, true);

        let active_count = self.active_count();
        if active_count > 0 && self.current_index >= active_count {
            self.current_index = self.current_index % active_count;
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.active_count() <= 1
    }

    pub fn get_winner(&self) -> Option<Uuid> {
        let active = self.active_players();
        if active.len() == 1 {
            active.first().copied()
        } else {
            None
        }
    }
}

/// Final rankings returned to clients / persistence layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResults {
    pub rankings: Vec<PlayerRanking>,
    pub finished_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRanking {
    pub user_id: Uuid,
    pub rank: usize,
    pub score: Option<i32>,
    pub prize: Option<f64>,
}

impl GameResults {
    pub fn from_ordered_players(player_ids: Vec<Uuid>) -> Self {
        let rankings = player_ids
            .into_iter()
            .enumerate()
            .map(|(idx, user_id)| PlayerRanking {
                user_id,
                rank: idx + 1,
                score: None,
                prize: None,
            })
            .collect();

        Self {
            rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: None,
        }
    }

    pub fn from_game_states(mut states: Vec<GamePlayerState>) -> Self {
        states.sort_by(|a, b| {
            match (a.is_eliminated, b.is_eliminated) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                (false, false) => std::cmp::Ordering::Equal,
                (true, true) => b.eliminated_at.cmp(&a.eliminated_at),
            }
        });

        let rankings = states
            .into_iter()
            .enumerate()
            .map(|(idx, state)| PlayerRanking {
                user_id: state.user_id,
                rank: idx + 1,
                score: Some(state.score),
                prize: None,
            })
            .collect();

        Self {
            rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBootstrap<TState> {
    pub game_id: Uuid,
    pub status: GameStatus,
    pub current_state: TState,
    pub players: Vec<Uuid>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    InProgress,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub results: GameResults,
    pub metadata: serde_json::Value,
    pub finished_at: i64,
}

#[derive(Debug, Clone)]
pub struct PlayerResult {
    pub rank: usize,
    pub prize: Option<f64>,
    pub wars_point: f64,
}

/// Context for calculating wars points for a player result.
#[derive(Debug, Clone)]
pub struct WarsPointContext {
    pub user_id: Uuid,
    pub game_id: Option<Uuid>,
    pub rank: usize,
    pub prize: Option<f64>,
    pub participants: usize,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub is_sponsored: bool,
    pub creator_id: Option<Uuid>,
    pub active_players: usize,
    pub token_symbol: Option<String>,
    pub token_contract_id: Option<String>,
}

pub fn calculate_wars_point(ctx: &WarsPointContext) -> f64 {
    let base_points = (ctx.participants as f64 - ctx.rank as f64 + 1.0) * 2.0;
    base_points.min(50.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_rotation() {
        let players = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let mut rotation = TurnRotation::new(players.clone());

        assert_eq!(rotation.current_player(), Some(players[0]));
        assert_eq!(rotation.active_count(), 3);

        rotation.next_turn();
        assert_eq!(rotation.current_player(), Some(players[1]));

        rotation.eliminate_player(players[1]);
        assert_eq!(rotation.active_count(), 2);
        assert_eq!(rotation.current_player(), Some(players[2]));

        rotation.eliminate_player(players[2]);
        assert_eq!(rotation.active_count(), 1);
        assert!(rotation.is_game_over());
        assert_eq!(rotation.get_winner(), Some(players[0]));
    }
}
