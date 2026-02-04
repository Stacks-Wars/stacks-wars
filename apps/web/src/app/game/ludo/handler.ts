import { displayUserIdentifier } from "@/lib/utils";
import type { LudoMessage, LudoState, LudoGameState, TurnPhase } from "./types";
import { parseLudoGameState } from "./types";
import { toast } from "sonner";

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
				board: message.board,
			};
		}

		case "turn": {
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
			toast.info(
				`${displayUserIdentifier(message.player)} rolled a ${message.dice}!`
			);
			return {
				...state,
				currentDice: message.dice,
				movablePawns: message.movablePawns,
				turnPhase:
					message.movablePawns.length > 0
						? "WaitingForMove"
						: "Complete",
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

	// Apply board state
	if (gameState.board) {
		newState.board = gameState.board;
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
