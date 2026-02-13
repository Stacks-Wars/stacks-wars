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
	/** Die 1 value (null if not yet rolled) */
	dice1: number | null;
	/** Die 2 value (null if not yet rolled) */
	dice2: number | null;
	/** Remaining value of die 1 (0 = used) */
	dice1Remaining: number;
	/** Remaining value of die 2 (0 = used) */
	dice2Remaining: number;
	/** Which dice values (die1, die2, sum) are playable (have movable pawns) */
	playableValues: number[];
	/** Currently selected dice value for pawn movement */
	selectedDiceValue: number | null;
	/** Pawns that can be moved with the currently selected dice value */
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
	| MovablePawnsMessage
	| DiceValueUsedMessage
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
	dice1: number;
	dice2: number;
	playableValues: number[];
}

export interface MovablePawnsMessage {
	type: "movablePawns";
	diceValue: number;
	pawns: number[];
}

export interface DiceValueUsedMessage {
	type: "diceValueUsed";
	diceValue: number;
	remainingValues: number[];
}

export interface PawnMovedMessage {
	type: "pawnMoved";
	player: PlayerState;
	pawnId: number;
	from: PawnPosition;
	to: PawnPosition;
	diceValue: number;
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
	dice1: number | null;
	dice2: number | null;
	dice1Remaining: number;
	dice2Remaining: number;
	selectedDiceValue: number | null;
	movablePawns: number[];
	playableValues: number[];
}

/**
 * Parse raw game state from server into typed LudoGameState
 */
export function parseLudoGameState(raw: unknown): LudoGameState | null {
	if (!raw || typeof raw !== "object") {
		console.warn("[Ludo] parseLudoGameState: raw is not an object", raw);
		return null;
	}

	const data = raw as Record<string, unknown>;

	console.log("[Ludo] parseLudoGameState: parsing game state", {
		hasBoard: !!data.board,
		dice1: data.dice1,
		dice2: data.dice2,
		dice1Type: typeof data.dice1,
		dice2Type: typeof data.dice2,
		turnPhase: data.turnPhase,
		boardPlayers: data.board ? (data.board as any).players?.length : null,
	});

	// Log board player indices if available
	if (data.board && typeof data.board === "object") {
		const board = data.board as any;
		if (Array.isArray(board.players)) {
			console.log("[Ludo] parseLudoGameState: board players", {
				playerCount: board.players.length,
				players: board.players.map((p: any) => ({
					userId: p.userId,
					playerIndex: p.playerIndex,
					startPosition: p.playerIndex !== undefined ? PLAYER_STARTS[p.playerIndex] : null,
				})),
			});
		}
	}

	return {
		board: data.board as LudoBoard,
		turn: data.turn as TurnMessage | null,
		turnPhase: (data.turnPhase as string) || "WaitingForRoll",
		dice1: (data.dice1 as number) ?? null,
		dice2: (data.dice2 as number) ?? null,
		dice1Remaining: (data.dice1Remaining as number) ?? 0,
		dice2Remaining: (data.dice2Remaining as number) ?? 0,
		selectedDiceValue: (data.selectedDiceValue as number) ?? null,
		movablePawns: (data.movablePawns as number[]) || [],
		playableValues: (data.playableValues as number[]) || [],
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
