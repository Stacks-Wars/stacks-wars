/**
 * Coin Flip Game Plugin
 *
 * Simple coin flip game to demonstrate the plugin system.
 */

import type { GamePlugin } from "@/lib/definitions";
import CoinFlipGame from "./game";
import type { CoinFlipMessage, CoinFlipState } from "./types";
import { handleCoinFlipMessage } from "./handler";

// ============================================================================
// Plugin Implementation
// ============================================================================

const createInitialState = (): CoinFlipState => ({
	players: [],
	currentPlayer: null,
	activePlayers: [],
	currentRound: 0,
	guesses: {},
	eliminatedPlayers: [],
	lastCoinResult: null,
	finished: false,
	results: null,
	timeoutSecs: 5,
});

// ============================================================================
// Export Plugin
// ============================================================================

export const CoinFlipPlugin: GamePlugin<CoinFlipState, CoinFlipMessage> = {
	id: "coin-flip",
	name: "Coin Flip",
	description: "Simple coin flip betting game",
	createInitialState,
	handleMessage: handleCoinFlipMessage,
	GameComponent: CoinFlipGame,
};
