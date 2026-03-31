import type { GamePlugin } from "@/lib/definitions";
import CheckersGame from "./game";
import { applyCheckersGameState, handleCheckersMessage } from "./handler";
import type { CheckersMessage, CheckersState } from "./types";

const createInitialState = (): CheckersState => ({
	board: null,
	currentPlayer: null,
	timeRemaining: 15,
	legalMoves: [],
	lastMove: null,
	isDraw: false,
	drawReason: null,
});

const handleMessage = (
	state: CheckersState,
	message: { game: CheckersMessage }
): CheckersState => {
	return handleCheckersMessage(state, message.game);
};

export const CheckersPlugin: GamePlugin<
	CheckersState,
	{ game: CheckersMessage }
> = {
	path: "checkers",
	name: "Checkers",
	description: "Classic 2-player checkers with automatic move timeout.",
	createInitialState,
	handleMessage,
	applyGameState: applyCheckersGameState,
	GameComponent: CheckersGame,
};
