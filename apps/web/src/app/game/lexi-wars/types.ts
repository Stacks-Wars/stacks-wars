import type { PlayerState } from "@/lib/definitions";

// ============================================================================
// Client Rule (from backend)
// ============================================================================

export interface ClientRule {
	name: string;
	description: string;
}

// ============================================================================
// Lexi Wars State
// ============================================================================

export interface LexiWarsState {
	/** Current player whose turn it is */
	currentPlayer: PlayerState | null;
	/** Current rule for the turn (only visible to current player) */
	currentRule: ClientRule | null;
	/** Time remaining in seconds for current turn */
	timeRemaining: number;
	/** Total number of players at game start */
	totalPlayers: number;
	/** Number of remaining active players */
	remainingPlayers: number;
	/** Whether the game has finished */
	finished: boolean;
	/** Final standings when game ends */
	standings: PlayerState[] | null;
}

// ============================================================================
// Lexi Wars Messages (Server -> Client)
// ============================================================================

export type LexiWarsMessage =
	| UsedWordMessage
	| WordEntryMessage
	| InvalidMessage
	| PlayersCountMessage
	| TurnMessage
	| RuleMessage
	| EliminatedMessage
	| CountdownMessage;

export interface UsedWordMessage {
	type: "usedWord";
	word: string;
}

export interface WordEntryMessage {
	type: "wordEntry";
	word: string;
	player: PlayerState;
}

export interface InvalidMessage {
	type: "invalid";
	reason: string;
}

export interface PlayersCountMessage {
	type: "playersCount";
	remaining: number;
	total: number;
}

export interface TurnMessage {
	type: "turn";
	player: PlayerState;
	timeoutSecs: number;
}

export interface RuleMessage {
	type: "rule";
	rule: ClientRule | null;
}

export interface EliminatedMessage {
	type: "eliminated";
	player: PlayerState;
	reason: string;
}

export interface CountdownMessage {
	type: "countdown";
	time: number;
}

// ============================================================================
// Lexi Wars Game State (for reconnection)
// ============================================================================

/**
 * Game state sent when a player connects/reconnects to an in-progress game.
 * Each field contains a full message object that can be processed by the handler.
 */
export interface LexiWarsGameState {
	playersCount: PlayersCountMessage;
	turn: TurnMessage | null;
	rule: RuleMessage | null;
	countdown: CountdownMessage;
}

/**
 * Parse raw game state from server into typed LexiWarsGameState
 */
export function parseLexiWarsGameState(raw: unknown): LexiWarsGameState | null {
	if (!raw || typeof raw !== "object") return null;

	const data = raw as Record<string, unknown>;

	return {
		playersCount: data.playersCount as PlayersCountMessage,
		turn: data.turn as TurnMessage | null,
		rule: data.rule as RuleMessage | null,
		countdown: data.countdown as CountdownMessage,
	};
}
