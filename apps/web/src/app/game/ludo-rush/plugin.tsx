/**
 * Ludo Rush Game Plugin
 *
 * Fast-paced Ludo variant where capturing an opponent finishes your pawn.
 * Only entry points are safe squares.
 */

import type { GamePlugin } from "@/lib/definitions";
import type { LudoRushMessage, LudoRushState } from "./types";
import { applyLudoRushGameState, handleLudoRushMessage } from "./handler";
import LudoRushGame from "./game";

// ============================================================================
// Initial State
// ============================================================================

const createInitialState = (): LudoRushState => ({
	board: null,
	currentPlayer: null,
	turnPhase: "WaitingForRoll",
	dice1: null,
	dice2: null,
	dice1Remaining: 0,
	dice2Remaining: 0,
	playableValues: [],
	selectedDiceValue: null,
	movablePawns: [],
	timeRemaining: 30,
	finished: false,
	standings: null,
	lastEvent: null,
});

// ============================================================================
// Message Handler Wrapper
// ============================================================================

/**
 * Handle incoming game messages
 *
 * New message format: { "game": { "type": "boardUpdate", ... } }
 * The `message.game` field contains the actual game message object
 */
const handleMessage = (
	state: LudoRushState,
	message: { game: LudoRushMessage }
): LudoRushState => {
	// Extract the game message from the wrapper
	const gameMessage = message.game;
	return handleLudoRushMessage(state, gameMessage);
};

// ============================================================================
// Export Plugin
// ============================================================================

export const LudoRushPlugin: GamePlugin<
	LudoRushState,
	{ game: LudoRushMessage }
> = {
	path: "ludo-rush",
	name: "Ludo Rush",
	description:
		"Fast-paced Ludo variant, capture opponents to instantly finish your pawn!",
	createInitialState,
	handleMessage,
	applyGameState: applyLudoRushGameState,
	GameComponent: LudoRushGame,
};
