import type { CoinFlipMessage, CoinFlipState } from "./types";

/**
 * Handle incoming Coin Flip messages and update state
 *
 * Note: The new message format wraps game messages as:
 * { "game": { "type": "round_started", ... } }
 *
 * The message parameter here is already the inner game object.
 */
export const handleCoinFlipMessage = (
	state: CoinFlipState,
	message: CoinFlipMessage
): CoinFlipState => {
	switch (message.type) {
		case "game_started": {
			return {
				...state,
				players: message.payload.players,
				currentPlayer: message.payload.current_player,
				activePlayers: message.payload.players,
				currentRound: 1,
				timeoutSecs: message.payload.timeout_secs,
				guesses: {},
			};
		}

		case "round_started": {
			return {
				...state,
				currentRound: message.payload.round,
				currentPlayer: message.payload.current_player,
				timeoutSecs: message.payload.timeout_secs,
				guesses: {},
				lastCoinResult: null,
			};
		}

		case "guess_received": {
			return {
				...state,
				guesses: {
					...state.guesses,
					[message.payload.player_id]: "heads", // We don't know the guess yet
				},
			};
		}

		case "player_timed_out": {
			return {
				...state,
				eliminatedPlayers: [
					...state.eliminatedPlayers,
					message.payload.player_id,
				],
			};
		}

		case "round_complete": {
			// Update guesses with actual values from results
			const newGuesses: Record<string, "heads" | "tails"> = {};
			message.payload.results.forEach((result) => {
				if (result.guess) {
					newGuesses[result.player_id] = result.guess;
				}
			});

			return {
				...state,
				lastCoinResult: message.payload.coin_result,
				guesses: newGuesses,
				eliminatedPlayers: message.payload.eliminated_players,
				activePlayers: message.payload.remaining_players,
			};
		}

		case "game_finished": {
			return {
				...state,
				finished: true,
				results: message.payload.results,
			};
		}

		default:
			console.warn("[CoinFlip] Unhandled message type:", message);
			return state;
	}
};
