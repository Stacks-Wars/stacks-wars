use crate::{games::{GameAction, GameEvent}, models::PlayerState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PieceColor {
    Red,
    Black,
}

impl PieceColor {
    pub fn opponent(self) -> Self {
        match self {
            Self::Red => Self::Black,
            Self::Black => Self::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PieceKind {
    Man,
    King,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    pub color: PieceColor,
    pub kind: PieceKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CheckersMove {
    pub from: Position,
    pub to: Position,
    pub captured: Option<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckersBoard {
    pub cells: Vec<Vec<Option<Piece>>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CheckersAction {
    #[serde(rename_all = "camelCase")]
    Move { from: Position, to: Position },
}

impl GameAction for CheckersAction {}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CheckersEvent {
    BoardUpdate { board: CheckersBoard },
    #[serde(rename_all = "camelCase")]
    Turn {
        player: PlayerState,
        timeout_secs: u64,
        legal_moves: Vec<CheckersMove>,
    },
    #[serde(rename_all = "camelCase")]
    MoveMade {
        player: PlayerState,
        mv: CheckersMove,
        became_king: bool,
    },
    Countdown { time: u64 },
    #[serde(rename_all = "camelCase")]
    GameDraw { reason: String },
    Invalid { reason: String },
}

impl GameEvent for CheckersEvent {}
