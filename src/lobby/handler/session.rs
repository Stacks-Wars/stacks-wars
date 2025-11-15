use crate::state::AppState;
use uuid::Uuid;

/// Represents a single user's session/context within a lobby.
/// Holds `AppState` so actions can access repos and broadcasting helpers.
#[derive(Clone)]
pub struct LobbySession {
    pub state: AppState,
    pub lobby_id: Uuid,
    pub user_id: Uuid,
}

impl LobbySession {
    pub fn new(state: AppState, lobby_id: Uuid, user_id: Uuid) -> Self {
        Self {
            state,
            lobby_id,
            user_id,
        }
    }
}
