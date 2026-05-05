import type { Metadata } from "next";
import { notFound } from "next/navigation";
import Image from "next/image";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions/game";
import type { LobbyInfo, PaginatedLobbiesResponse } from "@/lib/definitions";
import GameLobbies from "../_components/game-lobbies";

const GAME_IDENTIFIER = "checkers";

export const metadata: Metadata = {
	title: "Checkers",
	description:
		"Checkers is a classic strategy board game where two players compete to capture all opponent pieces or block their moves. Plan ahead, force captures, and promote your pieces to kings to dominate the board on Stacks Wars.",
};

async function getGame(identifier: string): Promise<Game> {
	try {
		const res = await ApiClient.get<Game>(`/api/games/${identifier}`);
		if (!res.data) {
			throw new Error("No game data received");
		}
		return res.data;
	} catch (error) {
		console.error("Failed to fetch game:", error);
		notFound();
	}
}

async function getGameLobbies(identifier: string): Promise<{
	lobbies: LobbyInfo[];
	total: number;
}> {
	try {
		const res = await ApiClient.get<PaginatedLobbiesResponse>(
			`/api/lobbies/game/${identifier}/lobbies?statuses=waiting,starting,inProgress&limit=6&offset=0`
		);
		if (!res.data) {
			return { lobbies: [], total: 0 };
		}
		return {
			lobbies: res.data.data,
			total: res.data.total,
		};
	} catch (error) {
		console.error("Failed to fetch game lobbies:", error);
		return { lobbies: [], total: 0 };
	}
}

export default async function CheckersPage() {
	const [game, { lobbies, total }] = await Promise.all([
		getGame(GAME_IDENTIFIER),
		getGameLobbies(GAME_IDENTIFIER),
	]);

	return (
		<div>
			<div className="mb-8 text-center sm:mb-12">
				<div className="mx-auto mb-6 w-full max-w-md sm:max-w-lg">
					<Image
						src={game.imageUrl}
						alt={game.name}
						width={400}
						height={200}
						className="h-auto w-full rounded-2xl"
						priority
					/>
				</div>
				<h1 className="mb-4 text-3xl font-bold sm:text-4xl lg:text-5xl">
					{game.name}
				</h1>
				<p className="text-muted-foreground mx-auto max-w-2xl text-lg sm:text-xl">
					{game.description}
				</p>
			</div>

			<div className="mb-12">
				<div className="mx-auto max-w-4xl space-y-6">
					<h2 className="text-2xl font-bold sm:text-3xl">
						What is Checkers?
					</h2>
					<div className="space-y-4 text-base sm:text-lg">
						<p>
							Checkers is a 2-player strategy board game played on
							an 8x8 board. Players move diagonally and capture
							opposing pieces by jumping over them. Captures are
							mandatory and chain captures are enforced when
							available.
						</p>
						<p>
							Reach the opposite side to promote a pawn into a
							king. Kings can move in both directions. Win by
							capturing all opponent pawns or leaving them with no
							legal move.
						</p>
					</div>

					<h3 className="text-xl font-semibold sm:text-2xl">
						How to Play
					</h3>
					<div className="space-y-4">
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								1
							</div>
							<div>
								<h4 className="font-semibold">
									Select and Move
								</h4>
								<p className="text-muted-foreground">
									Click a highlighted pawn and then choose a
									highlighted destination square.
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								2
							</div>
							<div>
								<h4 className="font-semibold">
									Capture is Mandatory
								</h4>
								<p className="text-muted-foreground">
									If a capture exists, you must take it. If
									another capture is available after jumping,
									you must continue.
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								3
							</div>
							<div>
								<h4 className="font-semibold">
									Beat the Clock
								</h4>
								<p className="text-muted-foreground">
									Each turn has a 15-second timer. If time
									runs out, a legal move is auto-played.
								</p>
							</div>
						</div>
					</div>

					<div className="bg-background/50 mt-8 rounded-2xl p-4 sm:p-6">
						<h4 className="mb-2 font-semibold">Game Details</h4>
						<div className="grid grid-cols-1 gap-2 text-sm sm:grid-cols-2 sm:text-base">
							<p>
								<span className="font-medium">Players:</span>{" "}
								{game.minPlayers} - {game.maxPlayers}
							</p>
							<p>
								<span className="font-medium">Category:</span>{" "}
								{game.category && game.category.length > 0
									? game.category.join(", ")
									: "Board Game"}
							</p>
						</div>
					</div>
				</div>
			</div>

			<GameLobbies
				gameIdentifier={GAME_IDENTIFIER}
				initialLobbies={lobbies}
				initialTotal={total}
			/>
		</div>
	);
}
