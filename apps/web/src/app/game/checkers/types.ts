import type { PlayerState } from "@/lib/definitions";

export interface Position {
	row: number;
	col: number;
}

export type PieceColor = "red" | "black";
export type PieceKind = "man" | "king";

export interface Piece {
	color: PieceColor;
	kind: PieceKind;
}

export interface CheckersMove {
	from: Position;
	to: Position;
	captured?: Position | null;
}

export interface CheckersBoard {
	cells: (Piece | null)[][];
}

export interface CheckersState {
	board: CheckersBoard | null;
	currentPlayer: PlayerState | null;
	timeRemaining: number;
	legalMoves: CheckersMove[];
	lastMove: CheckersMove | null;
	isDraw: boolean;
	drawReason: string | null;
}

export type CheckersMessage =
	| BoardUpdateMessage
	| TurnMessage
	| MoveMadeMessage
	| CountdownMessage
	| GameDrawMessage
	| InvalidMessage;

export interface BoardUpdateMessage {
	type: "boardUpdate";
	board: CheckersBoard;
}

export interface TurnMessage {
	type: "turn";
	player: PlayerState;
	timeoutSecs: number;
	legalMoves: CheckersMove[];
}

export interface MoveMadeMessage {
	type: "moveMade";
	player: PlayerState;
	mv: CheckersMove;
	becameKing: boolean;
}

export interface CountdownMessage {
	type: "countdown";
	time: number;
}

export interface GameDrawMessage {
	type: "gameDraw";
	reason: string;
}

export interface InvalidMessage {
	type: "invalid";
	reason: string;
}

export interface CheckersGameState {
	board: CheckersBoard;
	turn: TurnMessage | null;
}

export function parseCheckersGameState(raw: unknown): CheckersGameState | null {
	if (!raw || typeof raw !== "object") return null;
	const data = raw as Record<string, unknown>;
	return {
		board: data.board as CheckersBoard,
		turn: (data.turn as TurnMessage | null) ?? null,
	};
}
