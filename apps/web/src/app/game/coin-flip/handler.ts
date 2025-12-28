import type { GameMessage } from "@/lib/definitions";
import type { CoinFlipMessage, CoinFlipState } from "./types";

export const handleCoinFlipMessage = (
	state: CoinFlipState,
	message: GameMessage<CoinFlipMessage>
): CoinFlipState => {
	// Extract the actual message from payload (since GameMessage wraps it)
	const msg = message.payload as CoinFlipMessage;

	switch (msg.type) {
		case "game_started": {
			return {
				...state,
				players: msg.payload.players,
				currentPlayer: msg.payload.current_player,
				activePlayers: msg.payload.players,
				currentRound: 1,
				timeoutSecs: msg.payload.timeout_secs,
				guesses: {},
			};
		}

		case "round_started": {
			return {
				...state,
				currentRound: msg.payload.round,
				currentPlayer: msg.payload.current_player,
				timeoutSecs: msg.payload.timeout_secs,
				guesses: {},
				lastCoinResult: null,
			};
		}

		case "guess_received": {
			return {
				...state,
				guesses: {
					...state.guesses,
					[msg.payload.player_id]: "heads", // We don't know the guess yet
				},
			};
		}

		case "player_timed_out": {
			return {
				...state,
				eliminatedPlayers: [
					...state.eliminatedPlayers,
					msg.payload.player_id,
				],
			};
		}

		case "round_complete": {
			// Update guesses with actual values from results
			const newGuesses: Record<string, "heads" | "tails"> = {};
			msg.payload.results.forEach((result) => {
				if (result.guess) {
					newGuesses[result.player_id] = result.guess;
				}
			});

			return {
				...state,
				lastCoinResult: msg.payload.coin_result,
				guesses: newGuesses,
				eliminatedPlayers: msg.payload.eliminated_players,
				activePlayers: msg.payload.remaining_players,
			};
		}

		case "game_finished": {
			return {
				...state,
				finished: true,
				results: msg.payload.results,
			};
		}

		default:
			console.warn("[CoinFlip] Unhandled message type:", msg);
			return state;
	}
};
