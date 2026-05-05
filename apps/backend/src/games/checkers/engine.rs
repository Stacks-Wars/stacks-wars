use crate::{
    db::player_state::PlayerStateRepository,
    errors::AppError,
    games::{
        GameEngine,
        common::{
            GamePlayerState, GameResults, PlayerRanking, TurnRotation, WarsPointContext,
            finish_lobby, save_player_result_with_winner,
        },
    },
    models::PlayerState,
    state::AppState,
    ws::{broadcast, room::messages::RoomServerMessage},
};
use async_trait::async_trait;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Notify, RwLock};
use uuid::Uuid;

use super::message::{
    CheckersAction, CheckersBoard, CheckersEvent, CheckersMove, Piece, PieceColor, PieceKind,
    Position,
};

const BOARD_SIZE: usize = 8;
const MOVE_TIMEOUT_SECS: u64 = 15;
const DRAW_NO_PROGRESS_HALF_MOVES: u32 = 80;
const DRAW_REPETITION_THRESHOLD: u8 = 3;

struct CheckersInner {
    lobby_id: Uuid,
    players: HashMap<Uuid, GamePlayerState>,
    player_states: HashMap<Uuid, PlayerState>,
    player_colors: HashMap<Uuid, PieceColor>,
    turn_rotation: TurnRotation,
    board: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
    total_players: usize,
    finished: bool,
    results: Option<GameResults>,
    move_generation: u64,
    no_progress_half_moves: u32,
    repetition_counts: HashMap<String, u8>,
    // Multi-jump tracking: if Some((row, col)), player is forced to continue jumping from that square
    forced_jump_from: Option<(usize, usize)>,

    game_id: Option<Uuid>,
    entry_amount: Option<f64>,
    current_amount: Option<f64>,
    is_sponsored: bool,
    creator_id: Option<Uuid>,
    token_symbol: Option<String>,
    token_contract_id: Option<String>,

    turn_advance_notify: Arc<Notify>,
    state: AppState,
}

impl CheckersInner {
    fn new(lobby_id: Uuid, state: AppState) -> Self {
        Self {
            lobby_id,
            players: HashMap::new(),
            player_states: HashMap::new(),
            player_colors: HashMap::new(),
            turn_rotation: TurnRotation::new(Vec::new()),
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            total_players: 0,
            finished: false,
            results: None,
            move_generation: 0,
            no_progress_half_moves: 0,
            repetition_counts: HashMap::new(),
            forced_jump_from: None,
            game_id: None,
            entry_amount: None,
            current_amount: None,
            is_sponsored: false,
            creator_id: None,
            token_symbol: None,
            token_contract_id: None,
            turn_advance_notify: Arc::new(Notify::new()),
            state,
        }
    }

    fn initialize_board(&mut self) {
        self.board = [[None; BOARD_SIZE]; BOARD_SIZE];

        for row in 0..3 {
            for col in 0..BOARD_SIZE {
                if (row + col) % 2 == 1 {
                    self.board[row][col] = Some(Piece {
                        color: PieceColor::Black,
                        kind: PieceKind::Man,
                    });
                }
            }
        }

        for row in 5..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if (row + col) % 2 == 1 {
                    self.board[row][col] = Some(Piece {
                        color: PieceColor::Red,
                        kind: PieceKind::Man,
                    });
                }
            }
        }
    }

    fn to_board_event(&self) -> CheckersEvent {
        CheckersEvent::BoardUpdate {
            board: self.serialize_board(),
        }
    }

    fn serialize_board(&self) -> CheckersBoard {
        CheckersBoard {
            cells: self
                .board
                .iter()
                .map(|row| row.iter().copied().collect::<Vec<_>>())
                .collect(),
        }
    }

    fn current_player_id(&self) -> Option<Uuid> {
        self.turn_rotation.current_player()
    }

    fn player_state(&self, user_id: Uuid) -> Option<PlayerState> {
        self.player_states.get(&user_id).cloned()
    }

    fn in_bounds(row: i32, col: i32) -> bool {
        row >= 0 && row < BOARD_SIZE as i32 && col >= 0 && col < BOARD_SIZE as i32
    }

    fn directions(piece: Piece) -> &'static [(i32, i32)] {
        match piece.kind {
            PieceKind::King => &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
            PieceKind::Man => match piece.color {
                PieceColor::Red => &[(-1, -1), (-1, 1)],
                PieceColor::Black => &[(1, -1), (1, 1)],
            },
        }
    }

    fn piece_moves_from(
        &self,
        row: usize,
        col: usize,
        color: PieceColor,
        capture_only: bool,
    ) -> Vec<CheckersMove> {
        let Some(piece) = self.board[row][col] else {
            return Vec::new();
        };

        if piece.color != color {
            return Vec::new();
        }

        let mut moves = Vec::new();

        for (dr, dc) in Self::directions(piece) {
            // For men: check only 1 square
            // For kings: check all squares along the diagonal
            let max_steps = match piece.kind {
                PieceKind::Man => 1,
                PieceKind::King => BOARD_SIZE,
            };

            for step_count in 1..=max_steps {
                let r1 = row as i32 + dr * step_count as i32;
                let c1 = col as i32 + dc * step_count as i32;

                if !Self::in_bounds(r1, c1) {
                    break; // Hit board edge, stop looking in this direction
                }

                let step = self.board[r1 as usize][c1 as usize];

                // Regular move: empty square
                if step.is_none() {
                    if !capture_only {
                        moves.push(CheckersMove {
                            from: Position { row, col },
                            to: Position {
                                row: r1 as usize,
                                col: c1 as usize,
                            },
                            captured: None,
                        });
                    }
                    continue; // Can keep moving in this direction
                }

                // Piece in the way
                if let Some(mid_piece) = step {
                    // If it's our own piece, stop looking in this direction
                    if mid_piece.color == color {
                        break;
                    }

                    // It's an opponent piece - try to capture it
                    // For men: capture only the immediately adjacent piece
                    // For kings: capture and potentially continue
                    let r2 = r1 + dr;
                    let c2 = c1 + dc;

                    if !Self::in_bounds(r2, c2) {
                        break; // No space to land after capture
                    }

                    if self.board[r2 as usize][c2 as usize].is_none() {
                        moves.push(CheckersMove {
                            from: Position { row, col },
                            to: Position {
                                row: r2 as usize,
                                col: c2 as usize,
                            },
                            captured: Some(Position {
                                row: r1 as usize,
                                col: c1 as usize,
                            }),
                        });
                    }

                    // Stop looking in this direction after hitting a piece
                    break;
                }
            }
        }

        moves
    }

    fn legal_moves_for_color(&self, color: PieceColor) -> Vec<CheckersMove> {
        // If player is forced to continue a multi-jump from a specific square, only allow moves from that square
        if let Some((forced_row, forced_col)) = self.forced_jump_from {
            let piece = self.board[forced_row][forced_col];
            if piece.is_some_and(|p| p.color == color) {
                let moves = self.piece_moves_from(forced_row, forced_col, color, true); // capture_only=true
                if !moves.is_empty() {
                    return moves; // Forced to continue jumping from this square
                }
                // If no more captures from forced square, turn ends normally (shouldn't happen in well-formed game)
                return Vec::new();
            }
        }

        // Normal turn: generate all moves, prioritize captures (forced jump rule)
        let mut captures = Vec::new();
        let mut regular = Vec::new();

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let piece_moves = self.piece_moves_from(row, col, color, false);
                for mv in piece_moves {
                    if mv.captured.is_some() {
                        captures.push(mv);
                    } else {
                        regular.push(mv);
                    }
                }
            }
        }

        if captures.is_empty() {
            regular
        } else {
            captures // If captures available, only captures are legal moves (forced jump rule)
        }
    }

    fn legal_moves_for_user(&self, user_id: Uuid) -> Vec<CheckersMove> {
        let Some(color) = self.player_colors.get(&user_id).copied() else {
            return Vec::new();
        };

        self.legal_moves_for_color(color)
    }

    fn board_signature(&self) -> String {
        let mut sig = String::with_capacity(BOARD_SIZE * BOARD_SIZE + 40);

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let ch = match self.board[row][col] {
                    None => '.',
                    Some(Piece {
                        color: PieceColor::Red,
                        kind: PieceKind::Man,
                    }) => 'r',
                    Some(Piece {
                        color: PieceColor::Red,
                        kind: PieceKind::King,
                    }) => 'R',
                    Some(Piece {
                        color: PieceColor::Black,
                        kind: PieceKind::Man,
                    }) => 'b',
                    Some(Piece {
                        color: PieceColor::Black,
                        kind: PieceKind::King,
                    }) => 'B',
                };
                sig.push(ch);
            }
        }

        sig.push(':');
        if let Some(current) = self.current_player_id() {
            sig.push_str(&current.to_string());
        }

        sig
    }

    fn record_repetition(&mut self) -> bool {
        let signature = self.board_signature();
        let count = self.repetition_counts.entry(signature).or_insert(0);
        *count = count.saturating_add(1);
        *count >= DRAW_REPETITION_THRESHOLD
    }

    fn promote_if_needed(piece: &mut Piece, row: usize) -> bool {
        if piece.kind == PieceKind::King {
            return false;
        }

        let should_promote = match piece.color {
            PieceColor::Red => row == 0,
            PieceColor::Black => row == BOARD_SIZE - 1,
        };

        if should_promote {
            piece.kind = PieceKind::King;
            true
        } else {
            false
        }
    }

    fn apply_move(&mut self, mv: CheckersMove) -> Result<(bool, bool), AppError> {
        let from = mv.from;
        let to = mv.to;

        if from.row >= BOARD_SIZE
            || from.col >= BOARD_SIZE
            || to.row >= BOARD_SIZE
            || to.col >= BOARD_SIZE
        {
            return Err(AppError::BadRequest("Move is out of bounds".to_string()));
        }

        let mut piece = self.board[from.row][from.col]
            .ok_or_else(|| AppError::BadRequest("No piece at source square".to_string()))?;

        if self.board[to.row][to.col].is_some() {
            return Err(AppError::BadRequest(
                "Destination square is already occupied".to_string(),
            ));
        }

        self.board[from.row][from.col] = None;

        let mut captured_piece = false;
        if let Some(captured) = mv.captured {
            if captured.row >= BOARD_SIZE || captured.col >= BOARD_SIZE {
                return Err(AppError::BadRequest(
                    "Captured square is out of bounds".to_string(),
                ));
            }
            if self.board[captured.row][captured.col].is_some() {
                self.board[captured.row][captured.col] = None;
                captured_piece = true;
            }
        }

        let became_king = Self::promote_if_needed(&mut piece, to.row);
        self.board[to.row][to.col] = Some(piece);

        if captured_piece || became_king {
            self.no_progress_half_moves = 0;
        } else {
            self.no_progress_half_moves = self.no_progress_half_moves.saturating_add(1);
        }

        self.move_generation = self.move_generation.saturating_add(1);

        Ok((captured_piece, became_king))
    }

    fn piece_count(&self, color: PieceColor) -> usize {
        let mut count = 0usize;
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self
                    .board[row][col]
                    .is_some_and(|piece| piece.color == color)
                {
                    count += 1;
                }
            }
        }
        count
    }

    async fn broadcast_turn(&self) {
        let Some(current_id) = self.current_player_id() else {
            return;
        };

        let Some(player) = self.player_state(current_id) else {
            return;
        };

        let event = CheckersEvent::Turn {
            player,
            timeout_secs: MOVE_TIMEOUT_SECS,
            legal_moves: self.legal_moves_for_user(current_id),
        };

        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&event).unwrap_or_default(),
        )
        .await;
    }

    fn calculate_winner_prize(&self, rank: usize) -> Option<f64> {
        if rank == 1 {
            self.current_amount.filter(|amount| *amount > 0.0)
        } else {
            None
        }
    }

    fn calculate_draw_refund(&self) -> Option<f64> {
        self.entry_amount.filter(|amount| *amount > 0.0)
    }

    fn build_ctx(&self, user_id: Uuid, rank: usize, prize: Option<f64>) -> WarsPointContext {
        WarsPointContext {
            user_id,
            game_id: self.game_id,
            rank,
            prize,
            participants: self.total_players,
            entry_amount: self.entry_amount,
            current_amount: self.current_amount,
            is_sponsored: self.is_sponsored,
            creator_id: self.creator_id,
            active_players: self.turn_rotation.active_count(),
            token_symbol: self.token_symbol.clone(),
            token_contract_id: self.token_contract_id.clone(),
        }
    }

    async fn send_game_over(&self, user_id: Uuid, rank: usize, prize: Option<f64>, wars_point: f64) {
        let message = RoomServerMessage::GameOver {
            rank,
            prize,
            wars_point,
        };
        broadcast::broadcast_user(&self.state, user_id, &message).await;
    }

    async fn finish_with_standings(&self, standings: Vec<PlayerState>) {
        let final_standing = RoomServerMessage::FinalStanding { standings };
        broadcast::broadcast_room(&self.state, self.lobby_id, &final_standing).await;

        if let Err(e) = finish_lobby(&self.state, self.lobby_id).await {
            tracing::error!("Failed to finish lobby {}: {}", self.lobby_id, e);
        }
    }

    async fn end_with_winner(&mut self, winner_id: Uuid) {
        if self.finished {
            return;
        }

        self.finished = true;

        let active_players = self.turn_rotation.active_players();
        let loser_id = active_players.into_iter().find(|uid| *uid != winner_id);

        let mut standings = Vec::new();
        let mut rankings = Vec::new();

        let mut outcomes = vec![(winner_id, 1usize, true)];
        if let Some(loser_id) = loser_id {
            outcomes.push((loser_id, 2usize, false));
        }

        for (user_id, rank, is_winner) in outcomes {
            if !self.player_states.contains_key(&user_id) {
                continue;
            }

            let prize = self.calculate_winner_prize(rank);
            let ctx = self.build_ctx(user_id, rank, prize);
            let result = match save_player_result_with_winner(&self.state, self.lobby_id, &ctx, is_winner).await {
                Ok(saved) => saved,
                Err(e) => {
                    tracing::error!("Failed to save result for {}: {}", user_id, e);
                    continue;
                }
            };

            if let Some(ps) = self.player_states.get_mut(&user_id) {
                ps.rank = Some(rank);
                ps.prize = prize;
                ps.wars_point = Some(result.wars_point);
                standings.push(ps.clone());
            }

            rankings.push(PlayerRanking {
                user_id,
                rank,
                score: None,
                prize,
            });

            self.send_game_over(user_id, rank, prize, result.wars_point).await;
        }

        standings.sort_by_key(|player| player.rank.unwrap_or(usize::MAX));
        self.finish_with_standings(standings).await;

        self.results = Some(GameResults {
            rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: None,
        });
    }

    async fn end_draw(&mut self, reason: String) {
        if self.finished {
            return;
        }

        self.finished = true;

        let event = CheckersEvent::GameDraw { reason };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&event).unwrap_or_default(),
        )
        .await;

        let mut standings = Vec::new();
        let mut rankings = Vec::new();

        for user_id in self.turn_rotation.active_players() {
            let rank = 1usize;
            let prize = self.calculate_draw_refund();
            let ctx = self.build_ctx(user_id, rank, prize);
            let result = match save_player_result_with_winner(&self.state, self.lobby_id, &ctx, false).await {
                Ok(saved) => saved,
                Err(e) => {
                    tracing::error!("Failed to save draw result for {}: {}", user_id, e);
                    continue;
                }
            };

            if let Some(ps) = self.player_states.get_mut(&user_id) {
                ps.rank = Some(rank);
                ps.prize = prize;
                ps.wars_point = Some(result.wars_point);
                standings.push(ps.clone());
            }

            rankings.push(PlayerRanking {
                user_id,
                rank,
                score: None,
                prize,
            });

            self.send_game_over(user_id, rank, prize, result.wars_point).await;
        }

        standings.sort_by_key(|player| player.rank.unwrap_or(usize::MAX));
        self.finish_with_standings(standings).await;

        self.results = Some(GameResults {
            rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: Some(serde_json::json!({ "outcome": "draw" })),
        });
    }

    async fn perform_move(&mut self, user_id: Uuid, from: Position, to: Position) -> Result<bool, AppError> {
        if self.finished {
            return Err(AppError::BadRequest("Game already finished".to_string()));
        }

        if self.current_player_id() != Some(user_id) {
            let event = CheckersEvent::Invalid {
                reason: "Not your turn".to_string(),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        }

        let legal_moves = self.legal_moves_for_user(user_id);
        let Some(selected_move) = legal_moves
            .iter()
            .find(|mv| mv.from == from && mv.to == to)
            .copied()
        else {
            let event = CheckersEvent::Invalid {
                reason: "Illegal move".to_string(),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        };

        let (captured, became_king) = self.apply_move(selected_move)?;

        let Some(player) = self.player_state(user_id) else {
            return Err(AppError::NotFound("Player state not found".to_string()));
        };

        let moved_event = CheckersEvent::MoveMade {
            player,
            mv: selected_move,
            became_king,
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&moved_event).unwrap_or_default(),
        )
        .await;

        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&self.to_board_event()).unwrap_or_default(),
        )
        .await;

        if self.no_progress_half_moves >= DRAW_NO_PROGRESS_HALF_MOVES {
            self.end_draw("No captures or promotions for too long".to_string())
                .await;
            self.turn_advance_notify.notify_one();
            return Ok(true);
        }

        // Check for forced multi-jump continuation (if this move was a capture)
        if captured {
            let Some(color) = self.player_colors.get(&user_id).copied() else {
                return Err(AppError::BadRequest("Player color missing".to_string()));
            };

            let additional_captures = self.piece_moves_from(selected_move.to.row, selected_move.to.col, color, true);

            if !additional_captures.is_empty() {
                // Forced to continue jumping from the destination square
                self.forced_jump_from = Some((selected_move.to.row, selected_move.to.col));
                self.turn_advance_notify.notify_one();
                return Ok(true);
            }
            // If no more captures available, forced_jump_from stays None and we advance turn normally
            self.forced_jump_from = None;
        } else {
            // Non-capture move ends the turn normally
            self.forced_jump_from = None;
        }

        self.turn_rotation.next_turn();

        if self.record_repetition() {
            self.end_draw("Threefold repetition detected".to_string()).await;
            self.turn_advance_notify.notify_one();
            return Ok(true);
        }

        if let Some(current_id) = self.current_player_id() {
            let current_color = self
                .player_colors
                .get(&current_id)
                .copied()
                .ok_or_else(|| AppError::BadRequest("Current player color missing".to_string()))?;

            if self.piece_count(current_color) == 0 || self.legal_moves_for_user(current_id).is_empty() {
                self.end_with_winner(user_id).await;
                self.turn_advance_notify.notify_one();
                return Ok(true);
            }
        }

        self.turn_advance_notify.notify_one();
        Ok(true)
    }

    async fn auto_move_current(&mut self) {
        let Some(current_id) = self.current_player_id() else {
            return;
        };

        let legal_moves = self.legal_moves_for_user(current_id);
        if let Some(first_move) = legal_moves.first().copied() {
            let _ = self.perform_move(current_id, first_move.from, first_move.to).await;
        } else {
            let winner = self
                .turn_rotation
                .active_players()
                .into_iter()
                .find(|uid| *uid != current_id);
            if let Some(winner_id) = winner {
                self.end_with_winner(winner_id).await;
                self.turn_advance_notify.notify_one();
            }
        }
    }

    async fn handle_quit(&mut self, user_id: Uuid) {
        if self.finished {
            return;
        }

        if !self.turn_rotation.active_players().contains(&user_id) {
            return;
        }

        self.turn_rotation.eliminate_player(user_id);
        if let Some(player) = self.players.get_mut(&user_id) {
            player.eliminate();
        }

        if let Some(player_state) = self.player_states.get(&user_id).cloned() {
            let event = CheckersEvent::Invalid {
                reason: format!("Player {} left the game", player_state.user_id),
            };
            broadcast::broadcast_game_message(
                &self.state,
                self.lobby_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
        }

        if let Some(winner_id) = self.turn_rotation.active_players().first().copied() {
            self.end_with_winner(winner_id).await;
        }

        self.turn_advance_notify.notify_one();
    }
}

pub struct CheckersEngine {
    inner: Arc<RwLock<CheckersInner>>,
}

impl CheckersEngine {
    pub fn new(lobby_id: Uuid, state: AppState) -> Self {
        Self {
            inner: Arc::new(RwLock::new(CheckersInner::new(lobby_id, state))),
        }
    }

    fn get_inner(&self) -> Arc<RwLock<CheckersInner>> {
        self.inner.clone()
    }
}

#[async_trait]
impl GameEngine for CheckersEngine {
    async fn set_lobby_context(
        &mut self,
        game_id: Uuid,
        entry_amount: Option<f64>,
        current_amount: Option<f64>,
        is_sponsored: bool,
        creator_id: Uuid,
        token_symbol: Option<String>,
        token_contract_id: Option<String>,
    ) {
        let mut inner = self.inner.write().await;
        inner.game_id = Some(game_id);
        inner.entry_amount = entry_amount;
        inner.current_amount = current_amount;
        inner.is_sponsored = is_sponsored;
        inner.creator_id = Some(creator_id);
        inner.token_symbol = token_symbol;
        inner.token_contract_id = token_contract_id;
    }

    async fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, AppError> {
        if player_ids.len() != 2 {
            let error_event = RoomServerMessage::GameStartFailed {
                reason: "Checkers requires exactly 2 players".to_string(),
            };
            return Ok(vec![serde_json::to_value(&error_event).unwrap_or_default()]);
        }

        let mut inner = self.inner.write().await;

        inner.total_players = player_ids.len();
        inner.players = player_ids
            .iter()
            .map(|id| (*id, GamePlayerState::new(*id)))
            .collect();
        inner.turn_rotation = TurnRotation::new(player_ids.clone());
        inner.player_colors.insert(player_ids[0], PieceColor::Red);
        inner.player_colors.insert(player_ids[1], PieceColor::Black);
        inner.initialize_board();

        let player_repo = PlayerStateRepository::new(inner.state.redis.clone());
        if let Ok(states) = player_repo.get_all_in_lobby(inner.lobby_id).await {
            for state in states {
                inner.player_states.insert(state.user_id, state);
            }
        }

        inner.repetition_counts.clear();
        inner.record_repetition();

        broadcast::broadcast_room(&inner.state, inner.lobby_id, &RoomServerMessage::GameStarted).await;
        broadcast::broadcast_game_message(
            &inner.state,
            inner.lobby_id,
            serde_json::to_value(&inner.to_board_event()).unwrap_or_default(),
        )
        .await;

        Ok(Vec::new())
    }

    async fn handle_action(&mut self, user_id: Uuid, action: Value) -> Result<Vec<Value>, AppError> {
        let action: CheckersAction = serde_json::from_value(action)
            .map_err(|e| AppError::BadRequest(format!("Invalid checkers action: {}", e)))?;

        let mut inner = self.inner.write().await;

        match action {
            CheckersAction::Move { from, to } => {
                let _ = inner.perform_move(user_id, from, to).await?;
            }
        }

        Ok(Vec::new())
    }

    async fn get_game_state(&self, _user_id: Option<Uuid>) -> Result<Value, AppError> {
        let inner = self.inner.read().await;

        let turn = if let Some(current_id) = inner.current_player_id() {
            if let Some(player) = inner.player_state(current_id) {
                Some(CheckersEvent::Turn {
                    player,
                    timeout_secs: MOVE_TIMEOUT_SECS,
                    legal_moves: inner.legal_moves_for_user(current_id),
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(serde_json::json!({
            "board": inner.serialize_board(),
            "turn": turn.map(|event| serde_json::to_value(event).unwrap_or_default()),
        }))
    }

    async fn handle_player_quit(&mut self, user_id: Uuid) -> Result<Vec<Value>, AppError> {
        let mut inner = self.inner.write().await;
        inner.handle_quit(user_id).await;
        Ok(Vec::new())
    }

    fn is_finished(&self) -> bool {
        self.inner
            .try_read()
            .map(|inner| inner.finished)
            .unwrap_or(false)
    }

    fn start_loop(&mut self, state: AppState) {
        let inner = self.get_inner();
        tokio::spawn(run_game_loop(inner, state));
    }
}

async fn run_game_loop(inner: Arc<RwLock<CheckersInner>>, state: AppState) {
    let (notify, lobby_id) = {
        let inner_guard = inner.read().await;
        (inner_guard.turn_advance_notify.clone(), inner_guard.lobby_id)
    };

    loop {
        {
            let inner_guard = inner.read().await;
            if inner_guard.finished {
                break;
            }
        }

        {
            let inner_guard = inner.read().await;
            inner_guard.broadcast_turn().await;
        }

        let start_generation = {
            let inner_guard = inner.read().await;
            inner_guard.move_generation
        };

        let mut remaining = MOVE_TIMEOUT_SECS;
        while remaining > 0 {
            {
                let inner_guard = inner.read().await;
                if inner_guard.finished {
                    return;
                }
            }

            let countdown = CheckersEvent::Countdown { time: remaining };
            broadcast::broadcast_game_message(
                &state,
                lobby_id,
                serde_json::to_value(&countdown).unwrap_or_default(),
            )
            .await;

            if tokio::time::timeout(std::time::Duration::from_secs(1), notify.notified())
                .await
                .is_ok()
            {
                let inner_guard = inner.read().await;
                if inner_guard.move_generation != start_generation || inner_guard.finished {
                    break;
                }
            }

            remaining -= 1;
        }

        let timeout_zero = CheckersEvent::Countdown { time: 0 };
        broadcast::broadcast_game_message(
            &state,
            lobby_id,
            serde_json::to_value(&timeout_zero).unwrap_or_default(),
        )
        .await;

        let should_auto_move = {
            let inner_guard = inner.read().await;
            !inner_guard.finished && inner_guard.move_generation == start_generation
        };

        if should_auto_move {
            let mut inner_guard = inner.write().await;
            inner_guard.auto_move_current().await;
        }
    }
}

pub fn create_checkers(lobby_id: Uuid, state: AppState) -> Box<dyn GameEngine> {
    Box::new(CheckersEngine::new(lobby_id, state))
}
