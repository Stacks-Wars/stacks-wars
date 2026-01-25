/**
 * Lexi Wars Game Plugin
 *
 * Word game where players take turns submitting words following rules.
 */

import type { GamePlugin } from "@/lib/definitions";
import LexiWarsGame from "./game";
import type { LexiWarsMessage, LexiWarsState } from "./types";
import { applyLexiWarsGameState, handleLexiWarsMessage } from "./handler";

// ============================================================================
// Initial State
// ============================================================================

const createInitialState = (): LexiWarsState => ({
	currentPlayer: null,
	currentRule: null,
	timeRemaining: 15,
	totalPlayers: 0,
	remainingPlayers: 0,
	finished: false,
	standings: null,
});

// ============================================================================
// Message Handler Wrapper
// ============================================================================

/**
 * Handle incoming game messages
 *
 * New message format: { "game": { "type": "word_entry", ... } }
 * The `message.game` field contains the actual game message object
 */
const handleMessage = (
	state: LexiWarsState,
	message: { game: LexiWarsMessage }
): LexiWarsState => {
	// Extract the game message from the wrapper
	const gameMessage = message.game;
	return handleLexiWarsMessage(state, gameMessage);
};

// ============================================================================
// Export Plugin
// ============================================================================

export const LexiWarsPlugin: GamePlugin<
	LexiWarsState,
	{ game: LexiWarsMessage }
> = {
	path: "lexi-wars",
	name: "Lexi Wars",
	description: "Word battle game - submit words following the rules!",
	createInitialState,
	handleMessage,
	applyGameState: applyLexiWarsGameState,
	GameComponent: LexiWarsGame,
};
