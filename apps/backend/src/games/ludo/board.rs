// Ludo Board Logic
//
// Board structure:
// - Main track: 52 squares (0-51) around the board
// - Each player has 4 pawns starting in home base
// - Players enter at their start position and move clockwise
// - Home stretch: 6 squares before finish (player-specific)
// - Safe squares where pawns cannot be captured

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Constants
// ============================================================================

pub const BOARD_SIZE: usize = 52;
pub const HOME_STRETCH_SIZE: usize = 5;
pub const PAWNS_PER_PLAYER: usize = 4;
pub const MAX_PLAYERS: usize = 4;

/// Player start positions on the main track (where pawns enter from home)
pub const PLAYER_STARTS: [usize; 4] = [0, 13, 26, 39];

/// Safe squares where pawns cannot be captured
pub const SAFE_SQUARES: [usize; 8] = [0, 8, 13, 21, 26, 34, 39, 47];

// ============================================================================
// Pawn Position
// ============================================================================

/// Represents the position of a pawn on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PawnPosition {
    /// Pawn is in home base (not yet on the board)
    Home,
    /// Pawn is on the main track at the given position (0-51)
    OnTrack(usize),
    /// Pawn is in the home stretch (0-5, player-specific path to finish)
    HomeStretch(usize),
    /// Pawn has finished (reached the goal)
    Finished,
}

impl PawnPosition {
    /// Check if pawn is still in home base
    pub fn is_home(&self) -> bool {
        matches!(self, PawnPosition::Home)
    }

    /// Check if pawn has finished
    pub fn is_finished(&self) -> bool {
        matches!(self, PawnPosition::Finished)
    }

    /// Check if pawn is on the board (track or home stretch)
    pub fn is_on_board(&self) -> bool {
        matches!(self, PawnPosition::OnTrack(_) | PawnPosition::HomeStretch(_))
    }
}

// ============================================================================
// Pawn
// ============================================================================

/// A single pawn belonging to a player
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pawn {
    pub id: usize,           // 0-3 for each player
    pub player_index: usize, // Which player owns this pawn
    pub position: PawnPosition,
}

impl Pawn {
    pub fn new(id: usize, player_index: usize) -> Self {
        Self {
            id,
            player_index,
            position: PawnPosition::Home,
        }
    }

    /// Get the absolute track position for collision detection
    /// Returns None if pawn is in home, home stretch, or finished
    pub fn track_position(&self) -> Option<usize> {
        match self.position {
            PawnPosition::OnTrack(pos) => Some(pos),
            _ => None,
        }
    }
}

// ============================================================================
// Player Board State
// ============================================================================

/// Board state for a single player
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerBoardState {
    pub user_id: Uuid,
    pub player_index: usize,
    pub pawns: Vec<Pawn>,
    pub pawns_finished: usize,
}

impl PlayerBoardState {
    pub fn new(user_id: Uuid, player_index: usize) -> Self {
        let pawns = (0..PAWNS_PER_PLAYER)
            .map(|id| Pawn::new(id, player_index))
            .collect();

        Self {
            user_id,
            player_index,
            pawns,
            pawns_finished: 0,
        }
    }

    /// Get the start position for this player on the main track
    pub fn start_position(&self) -> usize {
        PLAYER_STARTS[self.player_index]
    }

    /// Get the last track position before entering this player's home stretch.
    /// This is the cell adjacent to HomeStretch(0) — two positions before start.
    pub fn home_stretch_entry(&self) -> usize {
        (self.start_position() + BOARD_SIZE - 2) % BOARD_SIZE
    }

    /// Check if all pawns have finished
    pub fn has_won(&self) -> bool {
        self.pawns_finished == PAWNS_PER_PLAYER
    }

    /// Check if all pawns are still at home (not on board)
    pub fn all_pawns_at_home(&self) -> bool {
        self.pawns.iter().all(|p| p.position.is_home())
    }

    /// Get pawns that can be moved with the given dice roll
    pub fn get_movable_pawns(&self, dice: u8) -> Vec<usize> {
        let mut movable = Vec::new();

        for pawn in &self.pawns {
            if self.can_move_pawn(pawn.id, dice) {
                movable.push(pawn.id);
            }
        }

        movable
    }

    /// Check if a specific pawn can be moved with the given dice roll
    pub fn can_move_pawn(&self, pawn_id: usize, dice: u8) -> bool {
        let pawn = &self.pawns[pawn_id];

        match pawn.position {
            PawnPosition::Home => {
                // Need a 6 to leave home
                dice == 6
            }
            PawnPosition::OnTrack(pos) => {
                // Check if move would enter home stretch or stay on track
                let steps_to_entry = self.steps_to_home_stretch_entry(pos);

                if dice as usize <= steps_to_entry {
                    // Stay on main track
                    true
                } else {
                    // Would enter home stretch — must land exactly on a valid square or finish
                    let home_stretch_pos = dice as usize - steps_to_entry - 1;
                    home_stretch_pos <= HOME_STRETCH_SIZE // == HOME_STRETCH_SIZE means Finished
                }
            }
            PawnPosition::HomeStretch(pos) => {
                // Must roll exact number to reach finish (can't overshoot)
                pos + dice as usize <= HOME_STRETCH_SIZE
            }
            PawnPosition::Finished => false,
        }
    }

    /// Calculate steps needed to reach home stretch entry from current position
    fn steps_to_home_stretch_entry(&self, current_pos: usize) -> usize {
        let entry = self.home_stretch_entry();
        if current_pos <= entry {
            entry - current_pos
        } else {
            BOARD_SIZE - current_pos + entry
        }
    }

    /// Move a pawn by the dice roll, returns the new position
    pub fn move_pawn(&mut self, pawn_id: usize, dice: u8) -> PawnPosition {
        // Read current position first to avoid borrow conflict
        let current_position = self.pawns[pawn_id].position;
        let start_pos = self.start_position();

        let new_position = match current_position {
            PawnPosition::Home => {
                // Moving out of home (dice must be 6)
                PawnPosition::OnTrack(start_pos)
            }
            PawnPosition::OnTrack(pos) => {
                let steps_to_entry = self.steps_to_home_stretch_entry(pos);

                if dice as usize <= steps_to_entry {
                    // Stay on main track
                    PawnPosition::OnTrack((pos + dice as usize) % BOARD_SIZE)
                } else {
                    // Enter home stretch
                    let home_stretch_pos = dice as usize - steps_to_entry - 1;
                    if home_stretch_pos == HOME_STRETCH_SIZE {
                        PawnPosition::Finished
                    } else {
                        PawnPosition::HomeStretch(home_stretch_pos)
                    }
                }
            }
            PawnPosition::HomeStretch(pos) => {
                let new_pos = pos + dice as usize;
                if new_pos == HOME_STRETCH_SIZE {
                    PawnPosition::Finished
                } else {
                    PawnPosition::HomeStretch(new_pos)
                }
            }
            PawnPosition::Finished => PawnPosition::Finished,
        };

        self.pawns[pawn_id].position = new_position;

        if new_position == PawnPosition::Finished {
            self.pawns_finished += 1;
        }

        new_position
    }

    /// Send a pawn back to home (when captured)
    pub fn send_home(&mut self, pawn_id: usize) {
        self.pawns[pawn_id].position = PawnPosition::Home;
    }
}

// ============================================================================
// Board
// ============================================================================

/// The complete Ludo board state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LudoBoard {
    pub players: Vec<PlayerBoardState>,
}

impl LudoBoard {
    pub fn new(user_ids: &[Uuid]) -> Self {
        // Assign player indices so that 2 players are diagonal for fairness
        let indices: Vec<usize> = match user_ids.len() {
            2 => vec![0, 2],       // Red & Yellow — diagonal corners
            3 => vec![0, 1, 2],    // Red, Green, Yellow
            _ => (0..user_ids.len()).collect(), // All four corners
        };

        let players = user_ids
            .iter()
            .zip(indices.iter())
            .map(|(&user_id, &idx)| PlayerBoardState::new(user_id, idx))
            .collect();

        Self { players }
    }

    /// Get a player's board state by user_id
    pub fn get_player(&self, user_id: Uuid) -> Option<&PlayerBoardState> {
        self.players.iter().find(|p| p.user_id == user_id)
    }

    /// Get a mutable reference to a player's board state
    pub fn get_player_mut(&mut self, user_id: Uuid) -> Option<&mut PlayerBoardState> {
        self.players.iter_mut().find(|p| p.user_id == user_id)
    }

    /// Check if a position is a safe square
    pub fn is_safe_square(position: usize) -> bool {
        SAFE_SQUARES.contains(&position)
    }

    /// Check for captures at a track position
    /// Returns the user_id and pawn_id of any captured pawn
    pub fn check_capture(&self, attacker_user_id: Uuid, position: usize) -> Option<(Uuid, usize)> {
        // Can't capture on safe squares
        if Self::is_safe_square(position) {
            return None;
        }

        // Check all other players' pawns
        for player in &self.players {
            if player.user_id == attacker_user_id {
                continue;
            }

            for pawn in &player.pawns {
                if pawn.track_position() == Some(position) {
                    return Some((player.user_id, pawn.id));
                }
            }
        }

        None
    }

    /// Get the winner (first player to finish all pawns)
    pub fn get_winner(&self) -> Option<Uuid> {
        self.players
            .iter()
            .find(|p| p.has_won())
            .map(|p| p.user_id)
    }

    /// Get rankings based on pawns finished (for when game ends)
    pub fn get_rankings(&self) -> Vec<(Uuid, usize)> {
        let mut rankings: Vec<_> = self.players
            .iter()
            .map(|p| (p.user_id, p.pawns_finished))
            .collect();

        // Sort by pawns finished (descending)
        rankings.sort_by(|a, b| b.1.cmp(&a.1));
        rankings
    }
}

// ============================================================================
// Dice
// ============================================================================

/// Roll a dice (1-6)
pub fn roll_dice() -> u8 {
    use rand::Rng;
    rand::rng().random_range(1..=6)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_movement() {
        let user_id = Uuid::new_v4();
        let mut player = PlayerBoardState::new(user_id, 0);

        // Pawn starts at home
        assert!(player.pawns[0].position.is_home());

        // Can't move with anything but 6
        assert!(!player.can_move_pawn(0, 1));
        assert!(!player.can_move_pawn(0, 5));
        assert!(player.can_move_pawn(0, 6));

        // Move out of home with 6
        let new_pos = player.move_pawn(0, 6);
        assert_eq!(new_pos, PawnPosition::OnTrack(0)); // Player 0 starts at position 0
    }

    #[test]
    fn test_home_stretch_entry() {
        let user_id = Uuid::new_v4();
        let player = PlayerBoardState::new(user_id, 0);

        // Player 0's home stretch entry is at position 50 (two before start at 0)
        // Grid: (7,0) — adjacent to HomeStretch(0) at (7,1)
        assert_eq!(player.home_stretch_entry(), 50);

        let player1 = PlayerBoardState::new(user_id, 1);
        // Player 1's home stretch entry is at position 11 (two before start at 13)
        // Grid: (0,7) — adjacent to HomeStretch(0) at (1,7)
        assert_eq!(player1.home_stretch_entry(), 11);

        let player2 = PlayerBoardState::new(user_id, 2);
        // Player 2's home stretch entry is at position 24
        assert_eq!(player2.home_stretch_entry(), 24);

        let player3 = PlayerBoardState::new(user_id, 3);
        // Player 3's home stretch entry is at position 37
        assert_eq!(player3.home_stretch_entry(), 37);
    }

    #[test]
    fn test_capture() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let mut board = LudoBoard::new(&[user1, user2]);

        // Move player 1's pawn to position 5 (not safe)
        board.get_player_mut(user1).unwrap().pawns[0].position = PawnPosition::OnTrack(5);

        // Check for capture
        let capture = board.check_capture(user2, 5);
        assert!(capture.is_some());
        assert_eq!(capture.unwrap(), (user1, 0));

        // No capture on safe square
        board.get_player_mut(user1).unwrap().pawns[0].position = PawnPosition::OnTrack(0);
        let capture = board.check_capture(user2, 0);
        assert!(capture.is_none());
    }
}
