import { displayUserIdentifier } from "@/lib/utils";
import type {
	LudoRushMessage,
	LudoRushState,
	LudoRushGameState,
	TurnPhase,
	LudoRushBoard,
	PawnPosition,
} from "./types";
import { parseLudoRushGameState } from "./types";
import { toast } from "sonner";

/** Normalize board from server (e.g. position.type "Home" -> "home") so UI logic works */
function normalizeBoard(board: LudoRushBoard): LudoRushBoard {
	return {
		...board,
		players: board.players.map((p) => ({
			...p,
			pawns: p.pawns.map((pawn) => ({
				...pawn,
				position: normalizePosition(pawn.position),
			})),
		})),
	};
}

/**
 * Normalize a pawn position from any server format into the client format.
 *
 * Server sends (serde camelCase externally-tagged enum):
 *   - Unit variants: the string "home" or "finished"
 *   - Newtype variants: { "onTrack": 5 } or { "homeStretch": 3 }
 *
 * Client expects: { type: "home" } | { type: "finished" } | { type: "onTrack", position: N } | { type: "homeStretch", position: N }
 */
function normalizePosition(pos: unknown): PawnPosition {
	// Already-normalized client format: { type: "home" }, { type: "onTrack", position: N }, etc.
	if (pos !== null && typeof pos === "object" && "type" in (pos as object)) {
		const p = pos as Record<string, unknown>;
		const t = (p.type as string).toLowerCase();
		if (t === "home") return { type: "home" };
		if (t === "finished") return { type: "finished" };
		if (t === "ontrack" && typeof p.position === "number")
			return { type: "onTrack", position: p.position };
		if (t === "homestretch" && typeof p.position === "number")
			return { type: "homeStretch", position: p.position };
	}
	// Server unit variants are plain strings: "home", "finished"
	if (typeof pos === "string") {
		const s = pos.toLowerCase();
		if (s === "home") return { type: "home" };
		if (s === "finished") return { type: "finished" };
	}
	// Server newtype variants: { "onTrack": 5 } or { "homeStretch": 3 }
	if (pos !== null && typeof pos === "object") {
		const p = pos as Record<string, unknown>;
		if (p.onTrack !== undefined && typeof p.onTrack === "number")
			return { type: "onTrack", position: p.onTrack };
		if (p.homeStretch !== undefined && typeof p.homeStretch === "number")
			return { type: "homeStretch", position: p.homeStretch };
	}
	// Fallback
	console.warn(
		"[LudoRush] Unknown position format, returning as-is:",
		pos
	);
	return pos as PawnPosition;
}

/**
 * Handle incoming Ludo Rush messages and update state
 */
export const handleLudoRushMessage = (
	state: LudoRushState,
	message: LudoRushMessage
): LudoRushState => {
	switch (message.type) {
		case "boardUpdate": {
			return {
				...state,
				board: normalizeBoard(message.board),
			};
		}

		case "turn": {
			return {
				...state,
				currentPlayer: message.player,
				timeRemaining: message.timeoutSecs,
				turnPhase: "WaitingForRoll",
				dice1: null,
				dice2: null,
				dice1Remaining: 0,
				dice2Remaining: 0,
				playableValues: [],
				selectedDiceValue: null,
				movablePawns: [],
			};
		}

		case "diceRolled": {
			const playableValues = Array.isArray(message.playableValues)
				? message.playableValues
				: [];
			toast.info(
				`${displayUserIdentifier(message.player)} rolled ${message.dice1} and ${message.dice2}!`
			);
			return {
				...state,
				dice1: message.dice1,
				dice2: message.dice2,
				dice1Remaining: message.dice1,
				dice2Remaining: message.dice2,
				playableValues,
				selectedDiceValue: null,
				movablePawns: [],
				turnPhase:
					playableValues.length > 0 ? "WaitingForMove" : "Complete",
				lastEvent: {
					type: "diceRolled",
					data: {
						dice1: message.dice1,
						dice2: message.dice2,
						player: message.player,
					},
					timestamp: Date.now(),
				},
			};
		}

		case "movablePawns": {
			return {
				...state,
				selectedDiceValue: message.diceValue,
				movablePawns: Array.isArray(message.pawns) ? message.pawns : [],
			};
		}

		case "diceValueUsed": {
			const remaining = Array.isArray(message.remainingValues)
				? message.remainingValues
				: [];

			// Figure out which dice remain based on remaining values
			let d1Rem = state.dice1Remaining;
			let d2Rem = state.dice2Remaining;
			const usedValue = message.diceValue;
			const sum = d1Rem > 0 && d2Rem > 0 ? d1Rem + d2Rem : 0;

			if (d1Rem > 0 && d2Rem > 0 && usedValue === sum) {
				// Sum used — both consumed
				d1Rem = 0;
				d2Rem = 0;
			} else if (usedValue === d1Rem) {
				d1Rem = 0;
			} else if (usedValue === d2Rem) {
				d2Rem = 0;
			}

			return {
				...state,
				dice1Remaining: d1Rem,
				dice2Remaining: d2Rem,
				playableValues: remaining,
				selectedDiceValue: null,
				movablePawns: [],
				// If no remaining values, turn will end via "Complete" or next event
				turnPhase:
					remaining.length > 0 ? "WaitingForMove" : state.turnPhase,
			};
		}

		case "pawnMoved": {
			return {
				...state,
				lastEvent: {
					type: "pawnMoved",
					data: {
						player: message.player,
						pawnId: message.pawnId,
						from: message.from,
						to: message.to,
						diceValue: message.diceValue,
					},
					timestamp: Date.now(),
				},
			};
		}

		case "pawnCaptured": {
			toast.warning(
				`${displayUserIdentifier(message.attacker)} captured ${displayUserIdentifier(message.victim)}'s pawn and finished theirs! ⚡`
			);
			return {
				...state,
				lastEvent: {
					type: "pawnCaptured",
					data: {
						attacker: message.attacker,
						victim: message.victim,
						victimPawnId: message.victimPawnId,
						attackerPawnId: message.attackerPawnId,
					},
					timestamp: Date.now(),
				},
			};
		}

		case "pawnFinished": {
			toast.success(
				`${displayUserIdentifier(message.player)} got a pawn home! ${message.pawnsRemaining} remaining.`
			);
			return {
				...state,
				lastEvent: {
					type: "pawnFinished",
					data: {
						player: message.player,
						pawnId: message.pawnId,
						pawnsRemaining: message.pawnsRemaining,
					},
					timestamp: Date.now(),
				},
			};
		}

		case "noValidMoves": {
			toast.info(
				`${displayUserIdentifier(message.player)} has no valid moves.`
			);
			return {
				...state,
				turnPhase: "Complete",
				lastEvent: {
					type: "noValidMoves",
					data: { player: message.player },
					timestamp: Date.now(),
				},
			};
		}

		case "bonusTurn": {
			toast.info(
				`${displayUserIdentifier(message.player)} gets a bonus turn! 🎲🎲`
			);
			return {
				...state,
				turnPhase: "WaitingForRoll",
				dice1: null,
				dice2: null,
				dice1Remaining: 0,
				dice2Remaining: 0,
				playableValues: [],
				selectedDiceValue: null,
				movablePawns: [],
				lastEvent: {
					type: "bonusTurn",
					data: { player: message.player },
					timestamp: Date.now(),
				},
			};
		}

		case "countdown": {
			return {
				...state,
				timeRemaining: message.time,
			};
		}

		case "invalid": {
			toast.error(message.reason);
			return state;
		}

		default:
			console.warn("[LudoRush] Unhandled message type:", message);
			return state;
	}
};

/**
 * Apply game state from reconnection.
 * This hydrates the state when a player connects/reconnects to an in-progress game.
 */
export const applyLudoRushGameState = (
	state: LudoRushState,
	rawGameState: unknown
): LudoRushState => {
	const gameState = parseLudoRushGameState(rawGameState);
	if (!gameState) {
		console.warn(
			"[LudoRush] Invalid game state received:",
			rawGameState
		);
		return state;
	}

	let newState = { ...state };

	// Apply board state (normalize position types from server)
	if (gameState.board) {
		newState.board = normalizeBoard(gameState.board);
	}

	// Apply turn phase
	newState.turnPhase = gameState.turnPhase as TurnPhase;

	// Apply dual dice state
	newState.dice1 = gameState.dice1;
	newState.dice2 = gameState.dice2;
	newState.dice1Remaining = gameState.dice1Remaining;
	newState.dice2Remaining = gameState.dice2Remaining;
	newState.selectedDiceValue = gameState.selectedDiceValue;
	newState.movablePawns = gameState.movablePawns;
	newState.playableValues = gameState.playableValues;

	// Apply turn if present
	if (gameState.turn) {
		newState.currentPlayer = gameState.turn.player;
		newState.timeRemaining = gameState.turn.timeoutSecs;
	}

	return newState;
};
