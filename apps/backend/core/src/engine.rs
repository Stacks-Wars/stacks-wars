use crate::error::CoreError;
use crate::host::GameHostRef;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

pub trait GameAction: DeserializeOwned + Send + Sync + 'static {}

pub trait GameEvent: Serialize + Send + Sync + 'static {}

#[async_trait]
pub trait GameEngine: Send + Sync {
    async fn set_host(&mut self, _host: GameHostRef) {}

    async fn set_lobby_context(
        &mut self,
        _game_id: Uuid,
        _entry_amount: Option<f64>,
        _current_amount: Option<f64>,
        _is_sponsored: bool,
        _creator_id: Uuid,
        _token_symbol: Option<String>,
        _token_contract_id: Option<String>,
    ) {
    }

    async fn handle_action(&mut self, user_id: Uuid, action: Value) -> Result<Vec<Value>, CoreError>;

    async fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, CoreError>;

    fn start_loop(&mut self, _host: GameHostRef) {}

    async fn get_game_state(&self, user_id: Option<Uuid>) -> Result<Value, CoreError>;

    async fn handle_player_quit(&mut self, _user_id: Uuid) -> Result<Vec<Value>, CoreError> {
        Ok(vec![])
    }

    fn is_finished(&self) -> bool;
}

pub type GameFactory = fn(Uuid, GameHostRef) -> Box<dyn GameEngine>;
