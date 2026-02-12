"use client";

import { useState } from "react";
import type { GamePluginProps, PlayerState } from "@/lib/definitions";
import type { LudoState, Pawn, PlayerBoardState, LudoBoard } from "./types";
import {
	PAWNS_PER_PLAYER,
	PLAYER_STARTS,
	SAFE_SQUARES,
	getPlayerColor,
} from "./types";
import { useUser } from "@/lib/stores/user";
import { usePlayers } from "@/lib/stores/room";
import { cn, displayUserIdentifier } from "@/lib/utils";
import RoomHeader from "@/components/room/room-header";
import ChatDialog from "@/components/room/chat";
import { Dice1, Dice2, Dice3, Dice4, Dice5, Dice6, Move } from "lucide-react";

// Dice icons mapping
const DiceIcons = [Dice1, Dice2, Dice3, Dice4, Dice5, Dice6];

// Player color classes for styling
const PLAYER_COLOR_CLASSES = {
	red: {
		bg: "bg-red-500",
		bgLight: "bg-red-500/20",
		border: "border-red-500",
		borderLight: "border-red-500/50",
		text: "text-red-500",
		ring: "ring-red-500",
	},
	green: {
		bg: "bg-green-500",
		bgLight: "bg-green-500/20",
		border: "border-green-500",
		borderLight: "border-green-500/50",
		text: "text-green-500",
		ring: "ring-green-500",
	},
	yellow: {
		bg: "bg-yellow-500",
		bgLight: "bg-yellow-500/20",
		border: "border-yellow-500",
		borderLight: "border-yellow-500/50",
		text: "text-yellow-500",
		ring: "ring-yellow-500",
	},
	blue: {
		bg: "bg-blue-500",
		bgLight: "bg-blue-500/20",
		border: "border-blue-500",
		borderLight: "border-blue-500/50",
		text: "text-blue-500",
		ring: "ring-blue-500",
	},
};

export default function LudoGame({
	state,
	sendMessage,
	lobby,
	game,
}: GamePluginProps<LudoState>) {
	const user = useUser();
	const roomPlayers = usePlayers();
	const [selectedPawnId, setSelectedPawnId] = useState<number | null>(null);

	const isMyTurn = state.currentPlayer?.userId === user?.id;
	const canRoll = isMyTurn && state.turnPhase === "WaitingForRoll";
	const canMove = isMyTurn && state.turnPhase === "WaitingForMove";

	// Get my player index for the board (match by user id or current player when it's my turn)
	const myPlayerIndex =
		state.board?.players.findIndex((p) => p.userId === user?.id) ??
		(isMyTurn && state.currentPlayer
			? state.board?.players.findIndex(
					(p) => p.userId === state.currentPlayer?.userId
				)
			: -1) ??
		-1;

	// Map userId to PlayerState for display names
	const getPlayerInfo = (userId: string): PlayerState | undefined => {
		return roomPlayers.find((p) => p.userId === userId);
	};

	const handleRollDice = () => {
		if (!canRoll) return;
		setSelectedPawnId(null);
		sendMessage("rollDice", null);
	};

	const handleSelectPawn = (pawnId: number) => {
		if (!canMove || !state.movablePawns.includes(pawnId)) return;
		setSelectedPawnId(pawnId);
	};

	const handleConfirmMove = () => {
		if (!canMove || selectedPawnId === null) return;
		sendMessage("movePawn", { pawnId: selectedPawnId });
		setSelectedPawnId(null);
	};

	// Timer color based on time remaining
	const timerColor =
		state.timeRemaining <= 5 && isMyTurn
			? "text-red-500"
			: state.timeRemaining <= 10 && isMyTurn
				? "text-yellow-500"
				: "text-primary";

	// Get dice icon component
	const DiceIcon = state.currentDice
		? DiceIcons[state.currentDice - 1]
		: Dice1;

	return (
		<>
			<RoomHeader />
			<div className="mx-auto max-w-4xl space-y-4 px-4 py-4">
				{/* Turn Indicator with Timer */}
				<div
					className={cn(
						"rounded-lg border p-4 transition-all",
						isMyTurn
							? "border-primary bg-primary/10 animate-pulse"
							: "bg-card"
					)}
				>
					{state.currentPlayer ? (
						<div className="flex items-center justify-between">
							<div className="flex items-center gap-3">
								{/* Current player color indicator */}
								{state.board && (
									<div
										className={cn(
											"h-10 w-10 rounded-full",
											PLAYER_COLOR_CLASSES[
												getPlayerColor(
													state.board.players.findIndex(
														(p) =>
															p.userId ===
															state.currentPlayer
																?.userId
													)
												)
											]?.bg
										)}
									/>
								)}
								<div>
									<p
										className={cn(
											"text-sm font-medium",
											isMyTurn
												? "text-primary"
												: "text-muted-foreground"
										)}
									>
										{isMyTurn
											? "🎲 Your Turn!"
											: "Current Turn"}
									</p>
									<p className="text-lg font-semibold">
										{displayUserIdentifier(
											state.currentPlayer
										)}
									</p>
									{isMyTurn && (
										<p className="text-muted-foreground mt-1 text-xs">
											{state.turnPhase ===
											"WaitingForRoll"
												? "Roll the dice!"
												: state.turnPhase ===
													  "WaitingForMove"
													? selectedPawnId !== null
														? "Press Move to confirm"
														: "Select a pawn to move"
													: "Processing..."}
										</p>
									)}
								</div>
							</div>
							<div className="flex items-center gap-4">
								{/* Timer */}
								<div
									className={cn(
										"flex h-14 w-14 items-center justify-center rounded-full border-2 text-xl font-bold transition-colors",
										state.timeRemaining <= 5 && isMyTurn
											? "border-red-500 bg-red-500/10"
											: state.timeRemaining <= 10 &&
												  isMyTurn
												? "border-yellow-500 bg-yellow-500/10"
												: "border-primary bg-primary/10"
									)}
								>
									<span className={timerColor}>
										{state.timeRemaining}
									</span>
								</div>
							</div>
						</div>
					) : (
						<p className="text-muted-foreground text-center">
							Waiting for game to start...
						</p>
					)}
				</div>

				{/* Ludo Board with Center Dice */}
				<LudoBoard
					board={state.board}
					myPlayerIndex={myPlayerIndex ?? -1}
					movablePawns={canMove ? state.movablePawns : []}
					selectedPawnId={selectedPawnId}
					onPawnSelect={handleSelectPawn}
					getPlayerInfo={getPlayerInfo}
					currentPlayerId={state.currentPlayer?.userId}
					isMyTurn={isMyTurn}
					canRoll={canRoll}
					canMove={canMove}
					currentDice={state.currentDice}
					onRollDice={handleRollDice}
					onConfirmMove={handleConfirmMove}
				/>
			</div>

			{/* Floating Chat Button */}
			<div className="pointer-events-none fixed right-0 bottom-6 left-0 z-40">
				<div className="container mx-auto flex max-w-4xl justify-end px-4">
					<ChatDialog
						buttonVariant="default"
						buttonClassName="size-12 shadow-lg pointer-events-auto"
					/>
				</div>
			</div>
		</>
	);
}

// ============================================================================
// Ludo Board Component - Traditional Board Layout
// ============================================================================

interface LudoBoardProps {
	board: LudoState["board"];
	myPlayerIndex: number;
	movablePawns: number[];
	selectedPawnId: number | null;
	onPawnSelect: (pawnId: number) => void;
	getPlayerInfo: (userId: string) => PlayerState | undefined;
	currentPlayerId?: string;
	isMyTurn: boolean;
	canRoll: boolean;
	canMove: boolean;
	currentDice: number | null;
	onRollDice: () => void;
	onConfirmMove: () => void;
}

// Board layout: 15x15 grid
// Corners (0-5, 0-5) are home bases
// Center cross is the track
// Home stretches lead to center

// Track position mapping to grid coordinates (clockwise from red start at position 0)
const TRACK_COORDS: [number, number][] = [
	// Red's starting column (top, going down) - positions 0-5
	[6, 1],
	[6, 2],
	[6, 3],
	[6, 4],
	[6, 5],
	// Top row going right - positions 5-11
	[5, 6],
	[4, 6],
	[3, 6],
	[2, 6],
	[1, 6],
	[0, 6],
	// Green's column (going down) - positions 11-12
	[0, 7],
	[0, 8],
	// Green's starting column (right side, going down) - positions 13-17
	[1, 8],
	[2, 8],
	[3, 8],
	[4, 8],
	[5, 8],
	// Right column going down - positions 18-24
	[6, 9],
	[6, 10],
	[6, 11],
	[6, 12],
	[6, 13],
	[6, 14],
	// Yellow's row (going left) - positions 25-26
	[7, 14],
	[8, 14],
	// Yellow's starting row (bottom, going left) - positions 26-31
	[8, 13],
	[8, 12],
	[8, 11],
	[8, 10],
	[8, 9],
	// Bottom row going left - positions 32-38
	[9, 8],
	[10, 8],
	[11, 8],
	[12, 8],
	[13, 8],
	[14, 8],
	// Blue's column (going up) - positions 39-40
	[14, 7],
	[14, 6],
	// Blue's starting column (left side, going up) - positions 39-44
	[13, 6],
	[12, 6],
	[11, 6],
	[10, 6],
	[9, 6],
	// Left column going up - positions 45-51
	[8, 5],
	[8, 4],
	[8, 3],
	[8, 2],
	[8, 1],
	[8, 0],
	// Back to red - position 51 wraps to 0
	[7, 0],
	[6, 0],
];

// Home stretch coordinates for each player (leading to center)
const HOME_STRETCH_COORDS: Record<number, [number, number][]> = {
	0: [
		[7, 1],
		[7, 2],
		[7, 3],
		[7, 4],
		[7, 5],
		[7, 6],
	], // Red - going down
	1: [
		[1, 7],
		[2, 7],
		[3, 7],
		[4, 7],
		[5, 7],
		[6, 7],
	], // Green - going right
	2: [
		[7, 13],
		[7, 12],
		[7, 11],
		[7, 10],
		[7, 9],
		[7, 8],
	], // Yellow - going up
	3: [
		[13, 7],
		[12, 7],
		[11, 7],
		[10, 7],
		[9, 7],
		[8, 7],
	], // Blue - going left
};

function LudoBoard({
	board,
	myPlayerIndex,
	movablePawns,
	selectedPawnId,
	onPawnSelect,
	getPlayerInfo,
	currentPlayerId,
	isMyTurn,
	canRoll,
	canMove,
	currentDice,
	onRollDice,
	onConfirmMove,
}: LudoBoardProps) {
	if (!board) {
		return (
			<div className="bg-card rounded-lg border p-8 text-center">
				<p className="text-muted-foreground">Loading board...</p>
			</div>
		);
	}

	// Create maps for pawns at each position
	const trackPawnMap = new Map<
		string,
		{ pawn: Pawn; player: PlayerBoardState }[]
	>();
	const homeStretchPawnMap = new Map<
		string,
		{ pawn: Pawn; player: PlayerBoardState }[]
	>();
	const homePawnMap = new Map<number, Pawn[]>(); // playerIndex -> pawns at home

	board.players.forEach((player) => {
		player.pawns.forEach((pawn) => {
			if (pawn.position.type === "onTrack") {
				const pos = pawn.position.position;
				const key = `track-${pos}`;
				if (!trackPawnMap.has(key)) trackPawnMap.set(key, []);
				trackPawnMap.get(key)!.push({ pawn, player });
			} else if (pawn.position.type === "homeStretch") {
				const key = `stretch-${player.playerIndex}-${pawn.position.position}`;
				if (!homeStretchPawnMap.has(key))
					homeStretchPawnMap.set(key, []);
				homeStretchPawnMap.get(key)!.push({ pawn, player });
			} else if (pawn.position.type === "home") {
				if (!homePawnMap.has(player.playerIndex))
					homePawnMap.set(player.playerIndex, []);
				homePawnMap.get(player.playerIndex)!.push(pawn);
			}
		});
	});

	// Get player info for each position
	const getPlayer = (playerIndex: number) =>
		board.players.find((p) => p.playerIndex === playerIndex);

	// Dice icon for center
	const DiceIcon = currentDice ? DiceIcons[currentDice - 1] : Dice1;

	return (
		<div className="bg-card rounded-xl border-2 p-2 sm:p-4">
			{/* 15x15 Grid Board */}
			<div className="relative mx-auto aspect-square w-full max-w-125">
				<div className="grid h-full w-full grid-cols-15 grid-rows-15 gap-px rounded-lg bg-slate-800 p-1">
					{Array.from({ length: 15 * 15 }, (_, idx) => {
						const row = Math.floor(idx / 15);
						const col = idx % 15;

						return (
							<BoardCell
								key={`${row}-${col}`}
								row={row}
								col={col}
								board={board}
								trackPawnMap={trackPawnMap}
								homeStretchPawnMap={homeStretchPawnMap}
								homePawnMap={homePawnMap}
								myPlayerIndex={myPlayerIndex}
								movablePawns={movablePawns}
								selectedPawnId={selectedPawnId}
								onPawnSelect={onPawnSelect}
								getPlayerInfo={getPlayerInfo}
								getPlayer={getPlayer}
							/>
						);
					})}
				</div>

				{/* Center Dice/Move Button - positioned only in center so board cells stay clickable */}
				{isMyTurn && (
					<div className="absolute top-1/2 left-1/2 z-10 flex -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2 rounded-2xl bg-slate-800/90 p-2 shadow-xl">
						{canRoll ? (
							<button
								onClick={onRollDice}
								className="flex h-16 w-16 animate-bounce items-center justify-center rounded-xl border-4 border-white bg-linear-to-br from-slate-700 to-slate-900 shadow-xl transition-transform hover:scale-110 active:scale-95 sm:h-20 sm:w-20"
							>
								<Dice1 className="h-8 w-8 text-white sm:h-10 sm:w-10" />
							</button>
						) : canMove ? (
							<>
								{/* Show dice result only when waiting for move (so never stale) */}
								<div className="flex h-14 w-14 items-center justify-center rounded-xl border-4 border-white bg-linear-to-br from-slate-700 to-slate-900 shadow-xl sm:h-16 sm:w-16">
									<DiceIcon
										className={cn(
											"h-7 w-7 sm:h-9 sm:w-9",
											currentDice === 6
												? "text-yellow-400"
												: "text-white"
										)}
									/>
								</div>
								{selectedPawnId !== null && (
									<button
										onClick={onConfirmMove}
										className="flex items-center gap-1 rounded-full bg-green-500 px-4 py-2 text-sm font-bold text-white shadow-lg transition-transform hover:scale-105 active:scale-95"
									>
										<Move className="h-4 w-4" />
										Move
									</button>
								)}
							</>
						) : null}
					</div>
				)}
			</div>

			{/* Player Legend */}
			<div className="mt-4 flex flex-wrap justify-center gap-3">
				{[0, 1, 2, 3].map((playerIndex) => {
					const player = getPlayer(playerIndex);
					const playerInfo = player
						? getPlayerInfo(player.userId)
						: undefined;
					const color = getPlayerColor(playerIndex);
					const colorClasses = PLAYER_COLOR_CLASSES[color];
					const isActive = !!player;
					const isCurrentTurn =
						player && currentPlayerId === player.userId;

					return (
						<div
							key={playerIndex}
							className={cn(
								"flex items-center gap-2 rounded-full border px-3 py-1 text-xs",
								isActive
									? colorClasses.borderLight
									: "border-muted",
								isCurrentTurn && "ring-2 ring-offset-1",
								isCurrentTurn && colorClasses.ring
							)}
						>
							<div
								className={cn(
									"h-3 w-3 rounded-full",
									isActive ? colorClasses.bg : "bg-muted"
								)}
							/>
							<span
								className={
									isActive ? "" : "text-muted-foreground"
								}
							>
								{isActive
									? playerInfo
										? displayUserIdentifier(playerInfo)
										: `Player ${playerIndex + 1}`
									: "Empty"}
							</span>
							{isActive && (
								<span className="text-muted-foreground">
									({player?.pawnsFinished || 0}/4)
								</span>
							)}
						</div>
					);
				})}
			</div>
		</div>
	);
}

// ============================================================================
// Board Cell Component - Renders individual cells
// ============================================================================

interface BoardCellProps {
	row: number;
	col: number;
	board: LudoBoard;
	trackPawnMap: Map<string, { pawn: Pawn; player: PlayerBoardState }[]>;
	homeStretchPawnMap: Map<string, { pawn: Pawn; player: PlayerBoardState }[]>;
	homePawnMap: Map<number, Pawn[]>;
	myPlayerIndex: number;
	movablePawns: number[];
	selectedPawnId: number | null;
	onPawnSelect: (pawnId: number) => void;
	getPlayerInfo: (userId: string) => PlayerState | undefined;
	getPlayer: (playerIndex: number) => PlayerBoardState | undefined;
}

function BoardCell({
	row,
	col,
	board,
	trackPawnMap,
	homeStretchPawnMap,
	homePawnMap,
	myPlayerIndex,
	movablePawns,
	selectedPawnId,
	onPawnSelect,
	getPlayerInfo,
	getPlayer,
}: BoardCellProps) {
	// Determine cell type based on position
	const cellType = getCellType(row, col);

	if (cellType.type === "empty") {
		return <div className="bg-slate-900" />;
	}

	if (cellType.type === "home") {
		return (
			<HomeCell
				row={row}
				col={col}
				playerIndex={cellType.playerIndex}
				homePawnMap={homePawnMap}
				myPlayerIndex={myPlayerIndex}
				movablePawns={movablePawns}
				selectedPawnId={selectedPawnId}
				onPawnSelect={onPawnSelect}
				getPlayer={getPlayer}
			/>
		);
	}

	if (cellType.type === "center") {
		return <div className="bg-linear-to-br from-slate-700 to-slate-800" />;
	}

	if (cellType.type === "track") {
		const pawns = trackPawnMap.get(`track-${cellType.position}`) || [];
		return (
			<TrackCell
				position={cellType.position}
				pawns={pawns}
				isStart={cellType.isStart}
				isSafe={cellType.isSafe}
				startPlayerIndex={cellType.startPlayerIndex}
				myPlayerIndex={myPlayerIndex}
				movablePawns={movablePawns}
				selectedPawnId={selectedPawnId}
				onPawnSelect={onPawnSelect}
			/>
		);
	}

	if (cellType.type === "homeStretch") {
		const pawns =
			homeStretchPawnMap.get(
				`stretch-${cellType.playerIndex}-${cellType.position}`
			) || [];
		return (
			<HomeStretchCell
				position={cellType.position}
				playerIndex={cellType.playerIndex}
				pawns={pawns}
				myPlayerIndex={myPlayerIndex}
				movablePawns={movablePawns}
				selectedPawnId={selectedPawnId}
				onPawnSelect={onPawnSelect}
			/>
		);
	}

	return <div className="bg-slate-900" />;
}

type CellType =
	| { type: "empty" }
	| { type: "home"; playerIndex: number }
	| { type: "center" }
	| {
			type: "track";
			position: number;
			isStart: boolean;
			isSafe: boolean;
			startPlayerIndex: number;
	  }
	| { type: "homeStretch"; playerIndex: number; position: number };

function getCellType(row: number, col: number): CellType {
	// Home bases (6x6 corners)
	// Red: top-left (rows 0-5, cols 0-5)
	if (row <= 5 && col <= 5) {
		return { type: "home", playerIndex: 0 };
	}
	// Green: top-right (rows 0-5, cols 9-14)
	if (row <= 5 && col >= 9) {
		return { type: "home", playerIndex: 1 };
	}
	// Blue: bottom-left (rows 9-14, cols 0-5)
	if (row >= 9 && col <= 5) {
		return { type: "home", playerIndex: 3 };
	}
	// Yellow: bottom-right (rows 9-14, cols 9-14)
	if (row >= 9 && col >= 9) {
		return { type: "home", playerIndex: 2 };
	}

	// Center (rows 6-8, cols 6-8)
	if (row >= 6 && row <= 8 && col >= 6 && col <= 8) {
		return { type: "center" };
	}

	// Home stretches
	// Red home stretch (row 7, cols 1-6)
	if (row === 7 && col >= 1 && col <= 6) {
		return { type: "homeStretch", playerIndex: 0, position: col - 1 };
	}
	// Green home stretch (col 7, rows 1-6)
	if (col === 7 && row >= 1 && row <= 6) {
		return { type: "homeStretch", playerIndex: 1, position: row - 1 };
	}
	// Yellow home stretch (row 7, cols 8-13)
	if (row === 7 && col >= 8 && col <= 13) {
		return { type: "homeStretch", playerIndex: 2, position: 13 - col };
	}
	// Blue home stretch (col 7, rows 8-13)
	if (col === 7 && row >= 8 && row <= 13) {
		return { type: "homeStretch", playerIndex: 3, position: 13 - row };
	}

	// Track cells - map grid position to track position
	const trackPosition = getTrackPosition(row, col);
	if (trackPosition !== -1) {
		const isStart = PLAYER_STARTS.includes(trackPosition);
		const isSafe = SAFE_SQUARES.includes(trackPosition);
		const startPlayerIndex = PLAYER_STARTS.indexOf(trackPosition);
		return {
			type: "track",
			position: trackPosition,
			isStart,
			isSafe,
			startPlayerIndex,
		};
	}

	return { type: "empty" };
}

function getTrackPosition(row: number, col: number): number {
	// Standard Ludo board track mapping (52 positions, 0-51)
	// PLAYER_STARTS = [0, 13, 26, 39]
	// Red (top-left) starts at 0, Green (top-right) at 13, Yellow (bottom-right) at 26, Blue (bottom-left) at 39

	// Track flows clockwise starting from Red's start position

	// ===== RED QUADRANT (positions 0-12) =====
	// Red start position 0 is at row 6, col 1 (just outside red home)
	if (row === 6 && col === 1) return 0;
	if (row === 6 && col === 2) return 1;
	if (row === 6 && col === 3) return 2;
	if (row === 6 && col === 4) return 3;
	if (row === 6 && col === 5) return 4;
	// Going up the left side of center
	if (row === 5 && col === 6) return 5;
	if (row === 4 && col === 6) return 6;
	if (row === 3 && col === 6) return 7;
	if (row === 2 && col === 6) return 8;
	if (row === 1 && col === 6) return 9;
	if (row === 0 && col === 6) return 10;
	// Turn right at top
	if (row === 0 && col === 7) return 11;
	if (row === 0 && col === 8) return 12;

	// ===== GREEN QUADRANT (positions 13-25) =====
	// Green start position 13 is at row 1, col 8 (just outside green home)
	if (row === 1 && col === 8) return 13;
	if (row === 2 && col === 8) return 14;
	if (row === 3 && col === 8) return 15;
	if (row === 4 && col === 8) return 16;
	if (row === 5 && col === 8) return 17;
	// Going right along top of center
	if (row === 6 && col === 9) return 18;
	if (row === 6 && col === 10) return 19;
	if (row === 6 && col === 11) return 20;
	if (row === 6 && col === 12) return 21;
	if (row === 6 && col === 13) return 22;
	if (row === 6 && col === 14) return 23;
	// Turn down at right edge
	if (row === 7 && col === 14) return 24;
	if (row === 8 && col === 14) return 25;

	// ===== YELLOW QUADRANT (positions 26-38) =====
	// Yellow start position 26 is at row 8, col 13 (just outside yellow home)
	if (row === 8 && col === 13) return 26;
	if (row === 8 && col === 12) return 27;
	if (row === 8 && col === 11) return 28;
	if (row === 8 && col === 10) return 29;
	if (row === 8 && col === 9) return 30;
	// Going down the right side of center
	if (row === 9 && col === 8) return 31;
	if (row === 10 && col === 8) return 32;
	if (row === 11 && col === 8) return 33;
	if (row === 12 && col === 8) return 34;
	if (row === 13 && col === 8) return 35;
	if (row === 14 && col === 8) return 36;
	// Turn left at bottom
	if (row === 14 && col === 7) return 37;
	if (row === 14 && col === 6) return 38;

	// ===== BLUE QUADRANT (positions 39-51) =====
	// Blue start position 39 is at row 13, col 6 (just outside blue home)
	if (row === 13 && col === 6) return 39;
	if (row === 12 && col === 6) return 40;
	if (row === 11 && col === 6) return 41;
	if (row === 10 && col === 6) return 42;
	if (row === 9 && col === 6) return 43;
	// Going left along bottom of center
	if (row === 8 && col === 5) return 44;
	if (row === 8 && col === 4) return 45;
	if (row === 8 && col === 3) return 46;
	if (row === 8 && col === 2) return 47;
	if (row === 8 && col === 1) return 48;
	if (row === 8 && col === 0) return 49;
	// Turn up at left edge
	if (row === 7 && col === 0) return 50;
	if (row === 6 && col === 0) return 51;

	return -1;
}

// ============================================================================
// Home Cell Component
// ============================================================================

interface HomeCellProps {
	row: number;
	col: number;
	playerIndex: number;
	homePawnMap: Map<number, Pawn[]>;
	myPlayerIndex: number;
	movablePawns: number[];
	selectedPawnId: number | null;
	onPawnSelect: (pawnId: number) => void;
	getPlayer: (playerIndex: number) => PlayerBoardState | undefined;
}

function HomeCell({
	row,
	col,
	playerIndex,
	homePawnMap,
	myPlayerIndex,
	movablePawns,
	selectedPawnId,
	onPawnSelect,
	getPlayer,
}: HomeCellProps) {
	const color = getPlayerColor(playerIndex);
	const colorClasses = PLAYER_COLOR_CLASSES[color];
	const player = getPlayer(playerIndex);
	const isActive = !!player;

	// Determine which pawn slot this cell represents within the 6x6 home
	// Each slot corresponds to a specific pawn ID (0-3)
	// Slot positions are absolute grid coordinates
	const homeSlots: Record<number, Record<string, number>> = {
		0: {
			// Red (top-left): pawn slots at specific positions
			"1-1": 0,
			"1-4": 1,
			"4-1": 2,
			"4-4": 3,
		},
		1: {
			// Green (top-right)
			"1-10": 0,
			"1-13": 1,
			"4-10": 2,
			"4-13": 3,
		},
		2: {
			// Yellow (bottom-right)
			"10-10": 0,
			"10-13": 1,
			"13-10": 2,
			"13-13": 3,
		},
		3: {
			// Blue (bottom-left)
			"10-1": 0,
			"10-4": 1,
			"13-1": 2,
			"13-4": 3,
		},
	};

	const slotKey = `${row}-${col}`;
	const slotPawnId = homeSlots[playerIndex]?.[slotKey];

	if (slotPawnId !== undefined && isActive) {
		// This is a pawn slot - find the pawn with this ID that's at home
		const homePawns = homePawnMap.get(playerIndex) || [];
		const pawn = homePawns.find((p) => p.id === slotPawnId);
		const isMovable =
			pawn &&
			myPlayerIndex === playerIndex &&
			movablePawns.includes(pawn.id);
		const isSelected =
			pawn && myPlayerIndex === playerIndex && selectedPawnId === pawn.id;

		return (
			<div
				className={cn(
					colorClasses.bg,
					"flex items-center justify-center"
				)}
			>
				{pawn ? (
					<button
						type="button"
						onClick={() => {
							if (isMovable) onPawnSelect(pawn.id);
						}}
						aria-disabled={!isMovable}
						className={cn(
							"flex h-[80%] w-[80%] items-center justify-center rounded-full border-2 border-white bg-white text-xs font-bold shadow-md transition-all select-none",
							color === "yellow"
								? "text-yellow-600"
								: colorClasses.text,
							isMovable && "cursor-pointer hover:scale-110",
							!isMovable && "cursor-default",
							isSelected && "scale-110 ring-4 ring-amber-400",
							isMovable &&
								!isSelected &&
								"animate-pulse shadow-[0_0_16px_rgba(56,189,248,0.8)] ring-4 ring-sky-400 ring-offset-2 ring-offset-slate-800"
						)}
					>
						{pawn.id + 1}
					</button>
				) : (
					<div className="h-[60%] w-[60%] rounded-full border-2 border-white/50 bg-white/20" />
				)}
			</div>
		);
	}

	// Regular home cell
	return (
		<div
			className={cn(
				isActive ? colorClasses.bg : "bg-slate-700",
				"transition-colors"
			)}
		/>
	);
}

// ============================================================================
// Track Cell Component
// ============================================================================

interface TrackCellProps {
	position: number;
	pawns: { pawn: Pawn; player: PlayerBoardState }[];
	isStart: boolean;
	isSafe: boolean;
	startPlayerIndex: number;
	myPlayerIndex: number;
	movablePawns: number[];
	selectedPawnId: number | null;
	onPawnSelect: (pawnId: number) => void;
}

function TrackCell({
	position,
	pawns,
	isStart,
	isSafe,
	startPlayerIndex,
	myPlayerIndex,
	movablePawns,
	selectedPawnId,
	onPawnSelect,
}: TrackCellProps) {
	const startColor =
		startPlayerIndex >= 0
			? PLAYER_COLOR_CLASSES[getPlayerColor(startPlayerIndex)]
			: null;

	return (
		<div
			className={cn(
				"flex items-center justify-center",
				isStart && startColor ? startColor.bg : "bg-white",
				isSafe && !isStart && "bg-yellow-100"
			)}
		>
			{pawns.length > 0 ? (
				<div className="relative flex items-center justify-center">
					{pawns.slice(0, 1).map(({ pawn, player }) => {
						const color = getPlayerColor(player.playerIndex);
						const colorClasses = PLAYER_COLOR_CLASSES[color];
						const isMyPawn = player.playerIndex === myPlayerIndex;
						const isMovable =
							isMyPawn && movablePawns.includes(pawn.id);
						const isSelected =
							isMyPawn && selectedPawnId === pawn.id;

						return (
							<button
								type="button"
								key={`${player.userId}-${pawn.id}`}
								onClick={() => {
									if (isMovable) onPawnSelect(pawn.id);
								}}
								aria-disabled={!isMovable}
								className={cn(
									"flex aspect-square min-h-4 min-w-4 w-[85%] items-center justify-center rounded-full border-2 bg-white text-[9px] font-bold shadow-sm transition-all select-none",
									colorClasses.border,
									color === "yellow"
										? "text-yellow-600"
										: colorClasses.text,
									isMovable &&
										"cursor-pointer hover:scale-110",
									!isMovable && "cursor-default",
									isSelected &&
										"scale-110 ring-4 ring-amber-400",
									isMovable &&
										!isSelected &&
										"animate-pulse shadow-[0_0_16px_rgba(56,189,248,0.8)] ring-4 ring-sky-400 ring-offset-1"
								)}
							>
								{pawn.id + 1}
								{pawns.length > 1 && (
									<span className="absolute -top-1 -right-1 flex h-3 w-3 items-center justify-center rounded-full bg-black text-[6px] text-white">
										{pawns.length}
									</span>
								)}
							</button>
						);
					})}
				</div>
			) : (
				isSafe &&
				!isStart && (
					<span className="text-[8px] text-yellow-600">★</span>
				)
			)}
		</div>
	);
}

// ============================================================================
// Home Stretch Cell Component
// ============================================================================

interface HomeStretchCellProps {
	position: number;
	playerIndex: number;
	pawns: { pawn: Pawn; player: PlayerBoardState }[];
	myPlayerIndex: number;
	movablePawns: number[];
	selectedPawnId: number | null;
	onPawnSelect: (pawnId: number) => void;
}

function HomeStretchCell({
	position,
	playerIndex,
	pawns,
	myPlayerIndex,
	movablePawns,
	selectedPawnId,
	onPawnSelect,
}: HomeStretchCellProps) {
	const color = getPlayerColor(playerIndex);
	const colorClasses = PLAYER_COLOR_CLASSES[color];

	return (
		<div
			className={cn(colorClasses.bg, "flex items-center justify-center")}
		>
			{pawns.length > 0 ? (
				pawns.slice(0, 1).map(({ pawn }) => {
					const isMyPawn = playerIndex === myPlayerIndex;
					const isMovable =
						isMyPawn && movablePawns.includes(pawn.id);
					const isSelected =
						isMyPawn && selectedPawnId === pawn.id;

					return (
						<button
							type="button"
							key={pawn.id}
							onClick={() => {
								if (isMovable) onPawnSelect(pawn.id);
							}}
							aria-disabled={!isMovable}
							className={cn(
								"flex aspect-square min-h-4 min-w-4 w-[80%] items-center justify-center rounded-full border-2 border-white bg-white text-[9px] font-bold shadow-sm transition-all select-none",
								color === "yellow"
									? "text-yellow-600"
									: colorClasses.text,
								isMovable && "cursor-pointer hover:scale-110",
								!isMovable && "cursor-default",
								isSelected && "scale-110 ring-4 ring-amber-400",
								isMovable &&
									!isSelected &&
									"animate-pulse shadow-[0_0_16px_rgba(56,189,248,0.8)] ring-4 ring-sky-400 ring-offset-1"
							)}
						>
							{pawn.id + 1}
						</button>
					);
				})
			) : (
				<div className="h-[50%] w-[50%] rounded-full bg-white/30" />
			)}
		</div>
	);
}
