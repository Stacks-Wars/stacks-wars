// Room error types
use std::fmt;

#[derive(Debug)]
pub enum RoomError {
    LobbyFull,
    NotCreator,
    NotAuthenticated,
    NotInLobby,
    NeedAtLeast(usize),
    JoinFailed(String),
    LeaveFailed(String),
    LobbyStatusFailed(String),
    ApproveFailed(String),
    RejectFailed(String),
    KickFailed(String),
    SendMessageFailed(String),
    ReactionFailed(String),
    /// Postgres metadata for the lobby is missing.
    MetadataMissing,
    /// Lobby runtime state or lobby itself was not found.
    NotFound,
    /// Client sent an invalid/uncodable message.
    InvalidMessage,
    /// Internal server error with details.
    Internal(String),
}

impl fmt::Display for RoomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoomError::LobbyFull => write!(f, "lobby full"),
            RoomError::NotCreator => write!(f, "only creator can start"),
            RoomError::NotAuthenticated => write!(f, "authentication required"),
            RoomError::NotInLobby => write!(f, "not in lobby"),
            RoomError::NeedAtLeast(n) => write!(f, "need at least {} players to start", n),
            RoomError::JoinFailed(s) => write!(f, "join failed: {}", s),
            RoomError::LeaveFailed(s) => write!(f, "leave failed: {}", s),
            RoomError::LobbyStatusFailed(s) => write!(f, "lobby status update failed: {}", s),
            RoomError::ApproveFailed(s) => write!(f, "approve join failed: {}", s),
            RoomError::RejectFailed(s) => write!(f, "reject join failed: {}", s),
            RoomError::KickFailed(s) => write!(f, "kick failed: {}", s),
            RoomError::SendMessageFailed(s) => write!(f, "send message failed: {}", s),
            RoomError::ReactionFailed(s) => write!(f, "reaction failed: {}", s),
            RoomError::MetadataMissing => write!(f, "lobby metadata missing from database"),
            RoomError::NotFound => write!(f, "lobby not found"),
            RoomError::InvalidMessage => write!(f, "invalid message"),
            RoomError::Internal(s) => write!(f, "internal error: {}", s),
        }
    }
}

impl RoomError {
    pub fn code(&self) -> &'static str {
        match self {
            RoomError::LobbyFull => "LOBBY_FULL",
            RoomError::NotCreator => "NOT_CREATOR",
            RoomError::NotInLobby => "NOT_IN_LOBBY",
            RoomError::NeedAtLeast(_) => "NEED_AT_LEAST",
            RoomError::JoinFailed(_) => "JOIN_FAILED",
            RoomError::LeaveFailed(_) => "LEAVE_FAILED",
            RoomError::LobbyStatusFailed(_) => "LOBBY_STATUS_FAILED",
            RoomError::ApproveFailed(_) => "APPROVE_FAILED",
            RoomError::RejectFailed(_) => "REJECT_FAILED",
            RoomError::KickFailed(_) => "KICK_FAILED",
            RoomError::SendMessageFailed(_) => "SEND_MESSAGE_FAILED",
            RoomError::ReactionFailed(_) => "REACTION_FAILED",
            RoomError::NotAuthenticated => "NOT_AUTHENTICATED",
            RoomError::MetadataMissing => "METADATA_MISSING",
            RoomError::NotFound => "NOT_FOUND",
            RoomError::InvalidMessage => "INVALID_MESSAGE",
            RoomError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl std::error::Error for RoomError {}
