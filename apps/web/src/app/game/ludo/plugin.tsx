/**
 * Ludo Game Plugin
 *
 * Classic board game where players race to get all pawns home.
 */

import type { GamePlugin } from "@/lib/definitions";
//import LudoGame from "./game";
import type { LudoMessage, LudoState } from "./types";
import { applyLudoGameState, handleLudoMessage } from "./handler";
import LudoGame from "./game";

// ============================================================================
// Initial State
// ============================================================================

const createInitialState = (): LudoState => ({
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
	state: LudoState,
	message: { game: LudoMessage }
): LudoState => {
	// Extract the game message from the wrapper
	const gameMessage = message.game;
	// Skip logging countdown messages (too frequent)
	if (gameMessage.type !== "countdown") {
		console.log("[Ludo] handleMessage: received message", {
			messageType: gameMessage.type,
			rawMessage: gameMessage,
		});
	}
	return handleLudoMessage(state, gameMessage);
};

// ============================================================================
// Export Plugin
// ============================================================================

export const LudoPlugin: GamePlugin<LudoState, { game: LudoMessage }> = {
	path: "ludo",
	name: "Ludo",
	description: "Classic board game - race to get all your pawns home!",
	createInitialState,
	handleMessage,
	applyGameState: applyLudoGameState,
	GameComponent: LudoGame,
};
