// ============================================================================
// Coin Flip State
// ============================================================================

export interface CoinFlipState {
	players: string[];
	currentPlayer: string | null;
	activePlayers: string[];
	currentRound: number;
	guesses: Record<string, "heads" | "tails">;
	eliminatedPlayers: string[];
	lastCoinResult: "heads" | "tails" | null;
	finished: boolean;
	results: {
		rankings: Array<{ userId: string; rank: number; eliminated: boolean }>;
	} | null;
	timeoutSecs: number;
}

// ============================================================================
// Coin Flip Messages
// ============================================================================

export type CoinFlipMessage =
	| {
			type: "game_started";
			payload: {
				players: string[];
				current_player: string;
				timeout_secs: number;
			};
	  }
	| {
			type: "round_started";
			payload: {
				round: number;
				current_player: string;
				timeout_secs: number;
			};
	  }
	| {
			type: "guess_received";
			payload: {
				player_id: string;
			};
	  }
	| {
			type: "player_timed_out";
			payload: {
				player_id: string;
			};
	  }
	| {
			type: "round_complete";
			payload: {
				round: number;
				coin_result: "heads" | "tails";
				results: Array<{
					player_id: string;
					guess: "heads" | "tails" | null;
					correct: boolean;
					eliminated: boolean;
				}>;
				eliminated_players: string[];
				remaining_players: string[];
			};
	  }
	| {
			type: "game_finished";
			payload: {
				results: {
					rankings: Array<{
						userId: string;
						rank: number;
						eliminated: boolean;
					}>;
				};
			};
	  };
