"use client";

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
import Image from "next/image";
import { Dice1, Dice2, Dice3, Dice4, Dice5, Dice6 } from "lucide-react";

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
	safeSquares = SAFE_SQUARES,
}: GamePluginProps<LudoState> & { safeSquares?: number[] }) {
	const user = useUser();
	const roomPlayers = usePlayers();

	const isMyTurn = state.currentPlayer?.userId === user?.id;
	const canRoll = isMyTurn && state.turnPhase === "WaitingForRoll";
	const canMove = isMyTurn && state.turnPhase === "WaitingForMove";

	// Get my playerIndex from the board
	const myPlayerIndex =
		state.board?.players.find((p) => p.userId === user?.id)?.playerIndex ??
		-1;

	// Map userId to PlayerState for display names
	const getPlayerInfo = (userId: string): PlayerState | undefined => {
		return roomPlayers.find((p) => p.userId === userId);
	};

	const handleRollDice = () => {
		if (!canRoll) return;
		sendMessage("rollDice", null);
	};

	const handleSelectDiceValue = (diceValue: number) => {
		if (!canMove) return;
		sendMessage("selectDiceValue", { diceValue });
	};

	const handleSelectPawn = (pawnId: number) => {
		if (!canMove || !state.movablePawns.includes(pawnId)) return;
		// Immediately move the pawn (no confirm button)
		sendMessage("movePawn", { pawnId });
	};

	// Build the three dice value options: die1, sum, die2
	const diceOptions = (() => {
		if (state.dice1 === null || state.dice2 === null) return [];
		const d1 = state.dice1Remaining;
		const d2 = state.dice2Remaining;
		const opts: { label: string; value: number; remaining: boolean }[] = [];

		opts.push({
			label: String(d1 || state.dice1),
			value: state.dice1,
			remaining: d1 > 0,
		});

		// Sum only when both dice are still available
		if (d1 > 0 && d2 > 0) {
			opts.push({
				label: String(d1 + d2),
				value: d1 + d2,
				remaining: true,
			});
		} else {
			// Show sum as 0 when one die is used
			opts.push({
				label: "0",
				value: 0,
				remaining: false,
			});
		}

		opts.push({
			label: String(d2 || state.dice2),
			value: state.dice2,
			remaining: d2 > 0,
		});

		return opts;
	})();

	// Current player color
	const currentPlayerColor = state.board
		? PLAYER_COLOR_CLASSES[
				getPlayerColor(
					state.board.players.find(
						(p) => p.userId === state.currentPlayer?.userId
					)?.playerIndex ?? 0
				)
			]
		: null;

	return (
		<>
			<RoomHeader />
			<div className="mx-auto max-w-4xl space-y-3 py-6">
				{/* Ludo Board */}
				<LudoBoard
					board={state.board}
					myPlayerIndex={myPlayerIndex ?? -1}
					movablePawns={canMove ? state.movablePawns : []}
					selectedPawnId={null}
					onPawnSelect={handleSelectPawn}
					getPlayerInfo={getPlayerInfo}
					isMyTurn={isMyTurn}
					canRoll={canRoll}
					canMove={canMove}
					dice1={state.dice1}
					dice2={state.dice2}
					onRollDice={handleRollDice}
					safeSquares={safeSquares}
				/>

				{/* Player Legend — outside board, below it */}
				{state.board && (
					<div className="flex flex-wrap justify-center gap-3">
						{state.board.players.map((player) => {
							const playerInfo = getPlayerInfo(player.userId);
							const color = getPlayerColor(player.playerIndex);
							const colorClasses = PLAYER_COLOR_CLASSES[color];
							const isCurrentTurn =
								state.currentPlayer?.userId === player.userId;

							return (
								<div
									key={player.playerIndex}
									className={cn(
										"flex items-center gap-2 rounded-full border px-3 py-1 text-xs",
										colorClasses.borderLight,
										isCurrentTurn && "ring-2 ring-offset-1",
										isCurrentTurn && colorClasses.ring
									)}
								>
									<div
										className={cn(
											"h-3 w-3 rounded-full",
											colorClasses.bg
										)}
									/>
									<span>
										{player.userId === user?.id
											? "You"
											: playerInfo
												? displayUserIdentifier(
														playerInfo
													)
												: `Player ${player.playerIndex + 1}`}
									</span>
									<span className="text-muted-foreground">
										(
										{player.pawns?.filter(
											(p) =>
												p.position.type === "finished"
										).length ?? 0}
										/4)
									</span>
								</div>
							);
						})}
					</div>
				)}

				{/* Dice Value Selection Buttons — below legend when in move phase */}
				{canMove && diceOptions.length > 0 && (
					<div className="flex items-center justify-center gap-4">
						{diceOptions.map((opt, i) => {
							const isPlayable =
								opt.remaining &&
								opt.value > 0 &&
								state.playableValues.includes(opt.value);
							const isSelected =
								state.selectedDiceValue === opt.value;
							const isUsed = !opt.remaining;

							return (
								<button
									key={i}
									type="button"
									disabled={!isPlayable}
									onClick={() =>
										isPlayable &&
										handleSelectDiceValue(opt.value)
									}
									className={cn(
										"flex h-12 w-12 items-center justify-center rounded-full text-lg font-bold shadow-md transition-all sm:h-14 sm:w-14 sm:text-xl",
										isUsed &&
											"bg-muted text-muted-foreground cursor-not-allowed opacity-40",
										!isUsed &&
											!isPlayable &&
											"bg-muted text-muted-foreground cursor-not-allowed opacity-60",
										isPlayable &&
											!isSelected &&
											"bg-primary/20 text-primary hover:bg-primary/30 ring-primary/50 cursor-pointer ring-2",
										isSelected &&
											"bg-primary text-primary-foreground ring-primary scale-110 cursor-pointer shadow-lg ring-4"
									)}
								>
									{isUsed ? "0" : opt.label}
								</button>
							);
						})}
					</div>
				)}
			</div>

			{/* Fixed bottom bar: Turn indicator */}
			<div className="pointer-events-none fixed right-0 bottom-0 left-0 z-40 flex justify-center pb-5">
				{state.currentPlayer ? (
					<div
						className={cn(
							"pointer-events-auto flex items-center gap-3 rounded-full border px-4 py-2 shadow-lg backdrop-blur-md",
							isMyTurn
								? "border-primary bg-primary/10"
								: "bg-card/90 border-border"
						)}
					>
						{currentPlayerColor && (
							<div
								className={cn(
									"h-5 w-5 rounded-full",
									currentPlayerColor.bg
								)}
							/>
						)}
						<p className="text-sm font-semibold">
							{isMyTurn
								? "Your turn"
								: `Waiting for ${displayUserIdentifier(state.currentPlayer)}`}
						</p>
						{isMyTurn && (
							<span
								className={cn(
									"text-xs tabular-nums",
									state.timeRemaining <= 3
										? "text-red-500"
										: "text-muted-foreground"
								)}
							>
								· Auto{" "}
								{state.turnPhase === "WaitingForRoll"
									? "roll"
									: "play"}{" "}
								in {state.timeRemaining}s
							</span>
						)}
					</div>
				) : null}
			</div>

			{/* Floating Chat Button */}
			<div className="pointer-events-none fixed right-4 bottom-5 z-50">
				<ChatDialog
					buttonVariant="default"
					buttonClassName="size-12 shadow-lg pointer-events-auto"
				/>
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
	isMyTurn: boolean;
	canRoll: boolean;
	canMove: boolean;
	dice1: number | null;
	dice2: number | null;
	onRollDice: () => void;
	safeSquares: number[];
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
	], // Red - going right toward center
	1: [
		[1, 7],
		[2, 7],
		[3, 7],
		[4, 7],
		[5, 7],
	], // Green - going down toward center
	2: [
		[7, 13],
		[7, 12],
		[7, 11],
		[7, 10],
		[7, 9],
	], // Yellow - going left toward center
	3: [
		[13, 7],
		[12, 7],
		[11, 7],
		[10, 7],
		[9, 7],
	], // Blue - going up toward center
};

function LudoBoard({
	board,
	myPlayerIndex,
	movablePawns,
	selectedPawnId,
	onPawnSelect,
	getPlayerInfo,
	isMyTurn,
	canRoll,
	canMove,
	dice1,
	dice2,
	onRollDice,
	safeSquares,
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

	// Dice icons for center (two dice)
	const Dice1Icon =
		dice1 && dice1 >= 1 && dice1 <= 6 ? DiceIcons[dice1 - 1] : Dice1;
	const Dice2Icon =
		dice2 && dice2 >= 1 && dice2 <= 6 ? DiceIcons[dice2 - 1] : Dice1;

	return (
		<div className="mx-auto w-full max-w-125">
			{/* 15x15 Grid Board */}
			<div className="relative aspect-square w-full">
				<div className="grid h-full w-full grid-cols-15 grid-rows-15 overflow-hidden rounded-lg border-2 border-slate-700">
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
								safeSquares={safeSquares}
							/>
						);
					})}
				</div>

				{/* Center overlay: logo when idle, dice when rolling/moving, tap to roll */}
				<div className="pointer-events-none absolute top-1/2 left-1/2 z-10 -translate-x-1/2 -translate-y-1/2">
					{canRoll ? (
						<button
							onClick={onRollDice}
							className="pointer-events-auto flex animate-bounce flex-col items-center justify-center gap-1 px-4 py-3 transition-transform hover:scale-110 active:scale-95"
						>
							<div className="flex gap-1">
								<Dice1 className="h-5 w-5 text-white sm:h-7 sm:w-7" />
								<Dice1 className="h-5 w-5 text-white sm:h-7 sm:w-7" />
							</div>
							<span className="text-[10px] font-semibold text-white/90 sm:text-xs">
								Tap to roll
							</span>
						</button>
					) : canMove && dice1 && dice2 ? (
						<div className="flex items-center justify-center gap-1.5 sm:gap-2 sm:p-3">
							<Dice1Icon className="h-7 w-7 text-white sm:h-9 sm:w-9" />
							<Dice2Icon className="h-7 w-7 text-white sm:h-9 sm:w-9" />
						</div>
					) : (
						/* Logo in center when idle */
						<Image
							src="/logo.svg"
							alt="Logo"
							width={40}
							height={40}
							className="opacity-40"
						/>
					)}
				</div>
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
	safeSquares: number[];
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
	safeSquares,
}: BoardCellProps) {
	// Determine cell type based on position
	const cellType = getCellType(row, col, safeSquares);

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
		return <div className="bg-slate-800" />;
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

function getCellType(row: number, col: number, safeSquares: number[]): CellType {
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

	// Home stretches (5 cells each, positions 0-4, stop before center)
	// Red home stretch (row 7, cols 1-5)
	if (row === 7 && col >= 1 && col <= 5) {
		return { type: "homeStretch", playerIndex: 0, position: col - 1 };
	}
	// Green home stretch (col 7, rows 1-5)
	if (col === 7 && row >= 1 && row <= 5) {
		return { type: "homeStretch", playerIndex: 1, position: row - 1 };
	}
	// Yellow home stretch (row 7, cols 9-13)
	if (row === 7 && col >= 9 && col <= 13) {
		return { type: "homeStretch", playerIndex: 2, position: 13 - col };
	}
	// Blue home stretch (col 7, rows 9-13)
	if (col === 7 && row >= 9 && row <= 13) {
		return { type: "homeStretch", playerIndex: 3, position: 13 - row };
	}

	// Track cells - map grid position to track position
	const trackPosition = getTrackPosition(row, col);
	if (trackPosition !== -1) {
		const isStart = PLAYER_STARTS.includes(trackPosition);
		const isSafe = safeSquares.includes(trackPosition);
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
				"flex items-center justify-center border border-slate-200",
				isStart && startColor ? startColor.bg : "bg-white",
				isSafe && !isStart && "bg-yellow-100"
			)}
		>
			{pawns.length > 0
				? (() => {
						// Sort: current player's movable pawns first, then current player's other pawns, then opponents
						const sorted = [...pawns].sort((a, b) => {
							const aIsMine =
								a.player.playerIndex === myPlayerIndex ? 1 : 0;
							const bIsMine =
								b.player.playerIndex === myPlayerIndex ? 1 : 0;
							if (aIsMine !== bIsMine) return bIsMine - aIsMine;
							const aMovable = movablePawns.includes(a.pawn.id)
								? 1
								: 0;
							const bMovable = movablePawns.includes(b.pawn.id)
								? 1
								: 0;
							return bMovable - aMovable;
						});
						const top = sorted[0];
						const { pawn, player } = top;
						const color = getPlayerColor(player.playerIndex);
						const colorClasses = PLAYER_COLOR_CLASSES[color];
						const isMyPawn = player.playerIndex === myPlayerIndex;
						const isMovable =
							isMyPawn && movablePawns.includes(pawn.id);
						const isSelected =
							isMyPawn && selectedPawnId === pawn.id;

						return (
							<div className="relative flex items-center justify-center">
								<button
									type="button"
									key={`${player.userId}-${pawn.id}`}
									onClick={() => {
										if (isMovable) onPawnSelect(pawn.id);
									}}
									aria-disabled={!isMovable}
									className={cn(
										"flex aspect-square min-h-4 w-[85%] min-w-4 items-center justify-center rounded-full border-2 bg-white text-[9px] font-bold shadow-sm transition-all select-none",
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
							</div>
						);
					})()
				: isSafe &&
					!isStart && (
						<span className="text-[8px] text-yellow-600">★</span>
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
			className={cn(
				colorClasses.bg,
				"flex items-center justify-center border",
				colorClasses.border
			)}
		>
			{pawns.length > 0 ? (
				(() => {
					// Sort: movable pawns first so they're always visible & clickable
					const sorted = [...pawns].sort((a, b) => {
						const aMovable = movablePawns.includes(a.pawn.id)
							? 1
							: 0;
						const bMovable = movablePawns.includes(b.pawn.id)
							? 1
							: 0;
						return bMovable - aMovable;
					});
					const { pawn } = sorted[0];
					const isMyPawn = playerIndex === myPlayerIndex;
					const isMovable =
						isMyPawn && movablePawns.includes(pawn.id);
					const isSelected = isMyPawn && selectedPawnId === pawn.id;

					return (
						<button
							type="button"
							key={pawn.id}
							onClick={() => {
								if (isMovable) onPawnSelect(pawn.id);
							}}
							aria-disabled={!isMovable}
							className={cn(
								"relative flex aspect-square min-h-4 w-[80%] min-w-4 items-center justify-center rounded-full border-2 border-white bg-white text-[9px] font-bold shadow-sm transition-all select-none",
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
							{pawns.length > 1 && (
								<span className="absolute -top-1 -right-1 flex h-3 w-3 items-center justify-center rounded-full bg-black text-[6px] text-white">
									{pawns.length}
								</span>
							)}
						</button>
					);
				})()
			) : (
				<div className="h-[50%] w-[50%] rounded-full bg-white/30" />
			)}
		</div>
	);
}
