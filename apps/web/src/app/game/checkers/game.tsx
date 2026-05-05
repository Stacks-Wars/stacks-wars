"use client";

import { useEffect, useState } from "react";
import RoomHeader from "@/components/room/room-header";
import ChatDialog from "@/components/room/chat";
import { PiCrown } from "react-icons/pi";
import { useUser } from "@/lib/stores/user";
import { usePlayers } from "@/lib/stores/room";
import { playSound } from "@/lib/audio/play-sound";
import { cn, displayUserIdentifier } from "@/lib/utils";
import { PlayerIdentifierPill } from "@/app/game/_components/player-identifier-pill";
import { FixedTurnStatusBar } from "@/app/game/_components/fixed-turn-status-bar";
import type { GamePluginProps } from "@/lib/definitions";
import type { CheckersState, Position } from "./types";

type LocalPieceColor = "red" | "black";

function oppositeColor(color: LocalPieceColor): LocalPieceColor {
	return color === "red" ? "black" : "red";
}

function samePosition(a: Position | null, b: Position | null) {
	if (!a || !b) return false;
	return a.row === b.row && a.col === b.col;
}

function countPawnsLeftByColor(
	board: CheckersState["board"]
): Record<LocalPieceColor, number> {
	const counts: Record<LocalPieceColor, number> = {
		red: 0,
		black: 0,
	};

	if (!board) return counts;

	for (const row of board.cells) {
		for (const piece of row) {
			if (piece?.color === "red") counts.red += 1;
			if (piece?.color === "black") counts.black += 1;
		}
	}

	return counts;
}

export default function CheckersGame({
	state,
	sendMessage,
}: GamePluginProps<CheckersState>) {
	const user = useUser();
	const roomPlayers = usePlayers();
	const joinedPlayers = roomPlayers.filter((p) => p.status === "joined");
	const isSpectator = !joinedPlayers.some((p) => p.userId === user?.id);
	const isMyTurn = state.currentPlayer?.userId === user?.id;
	const [selected, setSelected] = useState<Position | null>(null);
	const [myColor, setMyColor] = useState<LocalPieceColor>("red");

	useEffect(() => {
		if (isSpectator) {
			return;
		}

		if (
			!state.board ||
			!state.currentPlayer ||
			state.legalMoves.length === 0
		) {
			return;
		}

		const firstMove = state.legalMoves[0];
		const pieceAtFrom =
			state.board.cells[firstMove.from.row]?.[firstMove.from.col];
		if (!pieceAtFrom) {
			return;
		}

		const turnColor = pieceAtFrom.color as LocalPieceColor;
		if (state.currentPlayer.userId === user?.id) {
			setMyColor(turnColor);
		} else {
			setMyColor(oppositeColor(turnColor));
		}
	}, [
		isSpectator,
		state.board,
		state.currentPlayer,
		state.legalMoves,
		user?.id,
	]);

	const opponentColor = oppositeColor(myColor);
	const isClientRed = isSpectator ? true : myColor === "red";
	const pawnsLeftByColor = countPawnsLeftByColor(state.board);

	const currentTurnColor: LocalPieceColor | null = (() => {
		if (!state.board || state.legalMoves.length === 0) return null;
		const firstMove = state.legalMoves[0];
		const pieceAtFrom =
			state.board.cells[firstMove.from.row]?.[firstMove.from.col];
		return (pieceAtFrom?.color as LocalPieceColor | undefined) ?? null;
	})();

	const getPlayerColor = (
		playerUserId: string,
		fallback: LocalPieceColor
	): LocalPieceColor => {
		if (state.currentPlayer?.userId === playerUserId && currentTurnColor) {
			return currentTurnColor;
		}

		if (state.currentPlayer?.userId && currentTurnColor) {
			return oppositeColor(currentTurnColor);
		}

		return fallback;
	};

	// Get opponent info from room players
	const opponentInfo = joinedPlayers.find((p) => p.userId !== user?.id);

	useEffect(() => {
		setSelected(null);
	}, [state.currentPlayer?.userId, state.lastMove]);

	const legalFromSet = new Set(
		state.legalMoves.map((move) => `${move.from.row}-${move.from.col}`)
	);
	const legalToSet = new Set(
		state.legalMoves
			.filter((move) =>
				selected
					? move.from.row === selected.row &&
						move.from.col === selected.col
					: false
			)
			.map((move) => `${move.to.row}-${move.to.col}`)
	);

	const onSquareClick = (row: number, col: number) => {
		if (!isMyTurn) return;

		const current = selected;
		const fromKey = `${row}-${col}`;

		if (!current) {
			if (legalFromSet.has(fromKey)) {
				playSound("/audio/click.wav");
				setSelected({ row, col });
			}
			return;
		}

		if (current.row === row && current.col === col) {
			setSelected(null);
			return;
		}

		const move = state.legalMoves.find(
			(m) =>
				m.from.row === current.row &&
				m.from.col === current.col &&
				m.to.row === row &&
				m.to.col === col
		);

		if (move) {
			sendMessage("move", { from: move.from, to: move.to });
			setSelected(null);
			return;
		}

		if (legalFromSet.has(fromKey)) {
			playSound("/audio/click.wav");
			setSelected({ row, col });
		}
	};

	// Keep local player perspective at the bottom.
	const boardRows = state.board?.cells
		? isClientRed
			? state.board.cells
			: [...state.board.cells].reverse().map((row) => [...row].reverse())
		: [];

	return (
		<>
			<RoomHeader />
			<div className="mx-auto max-w-4xl space-y-4 py-6 pb-20">
				{/* Game board */}
				<div className="">
					<div className="border-border mx-auto grid w-full max-w-140 grid-cols-8 gap-1 rounded-xl border p-3">
						{boardRows.map((rowData, rowIdx) => {
							// Adjust row indices based on orientation
							const actualRow = isClientRed ? rowIdx : 7 - rowIdx;
							return rowData.map((piece, colIdx) => {
								const actualCol = isClientRed
									? colIdx
									: 7 - colIdx;
								const dark = (actualRow + actualCol) % 2 === 1;
								const fromKey = `${actualRow}-${actualCol}`;
								const isSelectable =
									isMyTurn && legalFromSet.has(fromKey);
								const isDestination =
									isMyTurn && legalToSet.has(fromKey);
								const isSelected = samePosition(selected, {
									row: actualRow,
									col: actualCol,
								});

								return (
									<button
										key={`${actualRow}-${actualCol}`}
										type="button"
										onClick={() =>
											onSquareClick(actualRow, actualCol)
										}
										className={cn(
											"aspect-square rounded-md border transition",
											dark
												? "border-amber-700/80 bg-amber-700/90"
												: "border-amber-100 bg-amber-100/95",
											isSelectable &&
												"ring-primary ring-2",
											isDestination &&
												"ring-2 ring-emerald-500",
											isSelected &&
												"ring-2 ring-yellow-400"
										)}
									>
										{piece ? (
											<div
												className={cn(
													"mx-auto flex h-5 w-5 items-center justify-center rounded-full text-[10px] font-bold sm:h-10 sm:w-10 sm:text-sm",
													piece.color === "red"
														? "bg-rose-500 text-white ring-1 ring-rose-200/70"
														: "bg-neutral-900 text-white ring-1 ring-white/60"
												)}
											>
												{piece.kind === "king" ? (
													<PiCrown className="h-3.5 w-3.5 sm:h-5 sm:w-5" />
												) : null}
											</div>
										) : null}
									</button>
								);
							});
						})}
					</div>

					{state.isDraw && (
						<p className="text-muted-foreground mt-3 text-center text-sm">
							Draw: {state.drawReason ?? "No winner"}
						</p>
					)}
				</div>

				{/* Player identifiers (below board) */}
				<div className="flex flex-wrap justify-center gap-3">
					{isSpectator
						? joinedPlayers.slice(0, 2).map((player, index) => {
								const playerColor = getPlayerColor(
									player.userId,
									index === 0 ? "red" : "black"
								);

								return (
									<PlayerIdentifierPill
										key={player.userId}
										label={displayUserIdentifier(player)}
										dotClassName={
											playerColor === "red"
												? "bg-rose-500"
												: "bg-neutral-900"
										}
										pawnsLeft={
											pawnsLeftByColor[playerColor]
										}
										isActiveTurn={
											state.currentPlayer?.userId ===
											player.userId
										}
										activeRingClassName="ring-2 ring-offset-1 ring-emerald-500"
									/>
								);
							})
						: [
								<PlayerIdentifierPill
									key="opponent"
									label={
										opponentInfo
											? displayUserIdentifier(
													opponentInfo
												)
											: "Opponent"
									}
									isYou={opponentInfo?.userId === user?.id}
									dotClassName={
										opponentColor === "red"
											? "bg-rose-500"
											: "bg-neutral-900"
									}
									pawnsLeft={pawnsLeftByColor[opponentColor]}
									isActiveTurn={
										state.currentPlayer?.userId ===
										opponentInfo?.userId
									}
									activeRingClassName="ring-2 ring-offset-1 ring-emerald-500"
								/>,
								<PlayerIdentifierPill
									key="you"
									label={user?.username ?? "You"}
									isYou={true}
									dotClassName={
										myColor === "red"
											? "bg-rose-500"
											: "bg-neutral-900"
									}
									pawnsLeft={pawnsLeftByColor[myColor]}
									isActiveTurn={isMyTurn}
									activeRingClassName="ring-2 ring-offset-1 ring-blue-500"
								/>,
							]}
				</div>
			</div>

			<FixedTurnStatusBar
				isMyTurn={isMyTurn}
				hasCurrentPlayer={!!state.currentPlayer}
				currentPlayerLabel={
					state.currentPlayer
						? displayUserIdentifier(state.currentPlayer)
						: ""
				}
				timeRemaining={state.timeRemaining}
				hideCountdownWhenNotMyTurn={true}
			/>

			<div className="pointer-events-none fixed right-4 bottom-5 z-50">
				<ChatDialog
					buttonVariant="default"
					buttonClassName="size-12 shadow-lg pointer-events-auto"
				/>
			</div>
		</>
	);
}
