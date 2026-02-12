import { displayUserIdentifier } from "@/lib/utils";
import type {
	LudoMessage,
	LudoState,
	LudoGameState,
	TurnPhase,
	LudoBoard,
	PawnPosition,
} from "./types";
import { parseLudoGameState } from "./types";
import { toast } from "sonner";

/** Normalize board from server (e.g. position.type "Home" -> "home") so UI logic works */
function normalizeBoard(board: LudoBoard): LudoBoard {
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
	console.warn("[Ludo] Unknown position format, returning as-is:", pos);
	return pos as PawnPosition;
}

/**
 * Handle incoming Ludo messages and update state
 */
export const handleLudoMessage = (
	state: LudoState,
	message: LudoMessage
): LudoState => {
	switch (message.type) {
		case "boardUpdate": {
			return {
				...state,
				board: normalizeBoard(message.board),
			};
		}

		case "turn": {
			// Only clear movablePawns when starting a *new* player's turn (don’t wipe if same player / reordered message)
			return {
				...state,
				currentPlayer: message.player,
				timeRemaining: message.timeoutSecs,
				turnPhase: "WaitingForRoll",
				currentDice: null,
				movablePawns: [],
			};
		}

		case "diceRolled": {
			const movablePawns = Array.isArray(message.movablePawns)
				? message.movablePawns
				: [];
			toast.info(
				`${displayUserIdentifier(message.player)} rolled a ${message.dice}!`
			);
			return {
				...state,
				currentDice: message.dice,
				movablePawns,
				turnPhase:
					movablePawns.length > 0 ? "WaitingForMove" : "Complete",
				lastEvent: {
					type: "diceRolled",
					data: { dice: message.dice, player: message.player },
					timestamp: Date.now(),
				},
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
					},
					timestamp: Date.now(),
				},
			};
		}

		case "pawnCaptured": {
			toast.warning(
				`${displayUserIdentifier(message.attacker)} captured ${displayUserIdentifier(message.victim)}'s pawn!`
			);
			return {
				...state,
				lastEvent: {
					type: "pawnCaptured",
					data: {
						attacker: message.attacker,
						victim: message.victim,
						pawnId: message.pawnId,
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
				`${displayUserIdentifier(message.player)} gets a bonus turn! 🎲`
			);
			return {
				...state,
				turnPhase: "WaitingForRoll",
				currentDice: null,
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
			console.warn("[Ludo] Unhandled message type:", message);
			return state;
	}
};

/**
 * Apply game state from reconnection.
 * This hydrates the state when a player connects/reconnects to an in-progress game.
 */
export const applyLudoGameState = (
	state: LudoState,
	rawGameState: unknown
): LudoState => {
	const gameState = parseLudoGameState(rawGameState);
	if (!gameState) {
		console.warn("[Ludo] Invalid game state received:", rawGameState);
		return state;
	}

	let newState = { ...state };

	// Apply board state (normalize position types from server)
	if (gameState.board) {
		newState.board = normalizeBoard(gameState.board);
	}

	// Apply turn phase
	newState.turnPhase = gameState.turnPhase as TurnPhase;

	// Apply current dice
	newState.currentDice = gameState.currentDice;

	// Apply movable pawns
	newState.movablePawns = gameState.movablePawns;

	// Apply turn if present
	if (gameState.turn) {
		newState.currentPlayer = gameState.turn.player;
		newState.timeRemaining = gameState.turn.timeoutSecs;
	}

	return newState;
};
