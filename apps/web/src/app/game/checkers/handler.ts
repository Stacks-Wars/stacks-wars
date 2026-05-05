import { toast } from "sonner";
import { playSound } from "@/lib/audio/play-sound";
import type { CheckersMessage, CheckersState } from "./types";
import { parseCheckersGameState } from "./types";

export const handleCheckersMessage = (
	state: CheckersState,
	message: CheckersMessage
): CheckersState => {
	switch (message.type) {
		case "boardUpdate":
			return {
				...state,
				board: message.board,
			};
		case "turn":
			return {
				...state,
				currentPlayer: message.player,
				timeRemaining: message.timeoutSecs,
				legalMoves: message.legalMoves ?? [],
			};
		case "moveMade":
			playSound(
				message.mv.captured ? "/audio/pawn-capture.wav" : "/audio/pawn-move.mp3"
			);
			return {
				...state,
				lastMove: message.mv,
			};
		case "countdown":
			return {
				...state,
				timeRemaining: message.time,
			};
		case "gameDraw":
			toast.info("Draw detected", {
				description: message.reason,
			});
			return {
				...state,
				isDraw: true,
				drawReason: message.reason,
			};
		case "invalid":
			toast.error(message.reason);
			return state;
		default:
			return state;
	}
};

export const applyCheckersGameState = (
	state: CheckersState,
	rawGameState: unknown
): CheckersState => {
	const gameState = parseCheckersGameState(rawGameState);
	if (!gameState) {
		return state;
	}

	return {
		...state,
		board: gameState.board,
		currentPlayer: gameState.turn?.player ?? null,
		timeRemaining: gameState.turn?.timeoutSecs ?? state.timeRemaining,
		legalMoves: gameState.turn?.legalMoves ?? [],
	};
};
