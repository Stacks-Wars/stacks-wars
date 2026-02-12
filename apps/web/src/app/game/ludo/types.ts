import type { PlayerState } from "@/lib/definitions";

// ============================================================================
// Board Constants
// ============================================================================

export const BOARD_SIZE = 52;
export const HOME_STRETCH_SIZE = 5;
export const PAWNS_PER_PLAYER = 4;
export const MAX_PLAYERS = 4;

/** Player start positions on the main track (where pawns enter from home) */
export const PLAYER_STARTS = [0, 13, 26, 39];

/** Safe squares where pawns cannot be captured */
export const SAFE_SQUARES = [0, 8, 13, 21, 26, 34, 39, 47];

/** Player colors */
export const PLAYER_COLORS = ["red", "green", "yellow", "blue"] as const;
export type PlayerColor = (typeof PLAYER_COLORS)[number];

// ============================================================================
// Pawn Position
// ============================================================================

export type PawnPosition =
	| { type: "home" }
	| { type: "onTrack"; position: number }
	| { type: "homeStretch"; position: number }
	| { type: "finished" };

// ============================================================================
// Pawn
// ============================================================================

export interface Pawn {
	id: number;
	playerIndex: number;
	position: PawnPosition;
}

// ============================================================================
// Player Board State
// ============================================================================

export interface PlayerBoardState {
	userId: string;
	playerIndex: number;
	pawns: Pawn[];
	pawnsFinished: number;
}

// ============================================================================
// Ludo Board
// ============================================================================

export interface LudoBoard {
	players: PlayerBoardState[];
}

// ============================================================================
// Turn Phase
// ============================================================================

export type TurnPhase = "WaitingForRoll" | "WaitingForMove" | "Complete";

// ============================================================================
// Ludo State
// ============================================================================

export interface LudoState {
	/** The board state with all players and pawns */
	board: LudoBoard | null;
	/** Current player whose turn it is */
	currentPlayer: PlayerState | null;
	/** Current turn phase */
	turnPhase: TurnPhase;
	/** Current dice value (null if not yet rolled) */
	currentDice: number | null;
	/** Pawns that can be moved with current dice roll */
	movablePawns: number[];
	/** Time remaining in seconds for current turn */
	timeRemaining: number;
	/** Whether the game has finished */
	finished: boolean;
	/** Final standings when game ends */
	standings: PlayerState[] | null;
	/** Last event for UI feedback */
	lastEvent: LudoUIEvent | null;
}

// ============================================================================
// UI Events (for animations/feedback)
// ============================================================================

export interface LudoUIEvent {
	type:
		| "diceRolled"
		| "pawnMoved"
		| "pawnCaptured"
		| "pawnFinished"
		| "bonusTurn"
		| "noValidMoves";
	data?: unknown;
	timestamp: number;
}

// ============================================================================
// Ludo Messages (Server -> Client)
// ============================================================================

export type LudoMessage =
	| BoardUpdateMessage
	| TurnMessage
	| DiceRolledMessage
	| PawnMovedMessage
	| PawnCapturedMessage
	| PawnFinishedMessage
	| NoValidMovesMessage
	| BonusTurnMessage
	| CountdownMessage
	| InvalidMessage;

export interface BoardUpdateMessage {
	type: "boardUpdate";
	board: LudoBoard;
}

export interface TurnMessage {
	type: "turn";
	player: PlayerState;
	timeoutSecs: number;
}

export interface DiceRolledMessage {
	type: "diceRolled";
	player: PlayerState;
	dice: number;
	movablePawns: number[];
}

export interface PawnMovedMessage {
	type: "pawnMoved";
	player: PlayerState;
	pawnId: number;
	from: PawnPosition;
	to: PawnPosition;
}

export interface PawnCapturedMessage {
	type: "pawnCaptured";
	attacker: PlayerState;
	victim: PlayerState;
	pawnId: number;
}

export interface PawnFinishedMessage {
	type: "pawnFinished";
	player: PlayerState;
	pawnId: number;
	pawnsRemaining: number;
}

export interface NoValidMovesMessage {
	type: "noValidMoves";
	player: PlayerState;
}

export interface BonusTurnMessage {
	type: "bonusTurn";
	player: PlayerState;
}

export interface CountdownMessage {
	type: "countdown";
	time: number;
}

export interface InvalidMessage {
	type: "invalid";
	reason: string;
}

// ============================================================================
// Ludo Game State (for reconnection)
// ============================================================================

export interface LudoGameState {
	board: LudoBoard;
	turn: TurnMessage | null;
	turnPhase: string;
	currentDice: number | null;
	movablePawns: number[];
}

/**
 * Parse raw game state from server into typed LudoGameState
 */
export function parseLudoGameState(raw: unknown): LudoGameState | null {
	if (!raw || typeof raw !== "object") return null;

	const data = raw as Record<string, unknown>;

	return {
		board: data.board as LudoBoard,
		turn: data.turn as TurnMessage | null,
		turnPhase: (data.turnPhase as string) || "WaitingForRoll",
		currentDice: (data.currentDice as number) || null,
		movablePawns: (data.movablePawns as number[]) || [],
	};
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get the color for a player index
 */
export function getPlayerColor(playerIndex: number): PlayerColor {
	return PLAYER_COLORS[playerIndex % PLAYER_COLORS.length];
}

/**
 * Check if a position is a safe square
 */
export function isSafeSquare(position: number): boolean {
	return SAFE_SQUARES.includes(position);
}

/**
 * Check if a pawn is at home
 */
export function isPawnAtHome(position: PawnPosition): boolean {
	return position.type === "home";
}

/**
 * Check if a pawn has finished
 */
export function isPawnFinished(position: PawnPosition): boolean {
	return position.type === "finished";
}

/**
 * Check if a pawn is on the board (track or home stretch)
 */
export function isPawnOnBoard(position: PawnPosition): boolean {
	return position.type === "onTrack" || position.type === "homeStretch";
}
