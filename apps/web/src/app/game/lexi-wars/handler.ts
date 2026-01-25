import { displayUserIdentifier } from "@/lib/utils";
import type { LexiWarsMessage, LexiWarsState } from "./types";
import { parseLexiWarsGameState } from "./types";
import { toast } from "sonner";

/**
 * Handle incoming Lexi Wars messages and update state
 *
 * Note: The new message format wraps game messages as:
 * { "game": { "type": "word_entry", ... } }
 *
 * The message parameter here is already the inner game object.
 */
export const handleLexiWarsMessage = (
	state: LexiWarsState,
	message: LexiWarsMessage
): LexiWarsState => {
	switch (message.type) {
		case "turn": {
			return {
				...state,
				currentPlayer: message.player,
				timeRemaining: message.timeoutSecs,
			};
		}

		case "rule": {
			return {
				...state,
				currentRule: message.rule,
			};
		}

		case "countdown": {
			return {
				...state,
				timeRemaining: message.time,
			};
		}

		case "wordEntry": {
			const isCurrentUser =
				message.player.userId === state.currentPlayer?.userId;
			toast.info(
				`${isCurrentUser ? "You" : displayUserIdentifier(message.player)} entered: ${message.word}`
			);
			return state;
		}

		case "usedWord": {
			toast.warning(`"${message.word}" has already been used`);
			return state;
		}

		case "invalid": {
			toast.warning(message.reason);
			return state;
		}

		case "playersCount": {
			return {
				...state,
				remainingPlayers: message.remaining,
				totalPlayers: message.total,
			};
		}

		case "eliminated": {
			toast.error(`${message.player.username} was eliminated`, {
				description: `${message.reason}`,
			});
			return state;
		}

		default:
			console.warn("[LexiWars] Unhandled message type:", message);
			return state;
	}
};

/**
 * Apply game state from reconnection.
 * This hydrates the state when a player connects/reconnects to an in-progress game.
 */
export const applyLexiWarsGameState = (
	state: LexiWarsState,
	rawGameState: unknown
): LexiWarsState => {
	const gameState = parseLexiWarsGameState(rawGameState);
	if (!gameState) {
		console.warn("[LexiWars] Invalid game state received:", rawGameState);
		return state;
	}

	let newState = { ...state };

	// Apply players count
	newState.remainingPlayers = gameState.playersCount.remaining;
	newState.totalPlayers = gameState.playersCount.total;

	// Apply countdown
	newState.timeRemaining = gameState.countdown.time;

	// Apply turn if present
	if (gameState.turn) {
		newState.currentPlayer = gameState.turn.player;
	}

	// Apply rule if present (only sent to current player)
	if (gameState.rule) {
		newState.currentRule = gameState.rule.rule;
	}

	return newState;
};
