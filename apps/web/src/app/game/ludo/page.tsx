import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions/game";
import type { LobbyInfo, PaginatedLobbiesResponse } from "@/lib/definitions";
import GameLobbies from "../_components/game-lobbies";
import Image from "next/image";

const GAME_IDENTIFIER = "ludo";

export const metadata: Metadata = {
	title: "Ludo",
	description:
		"Classic board game where players race to get all their pawns home on Stacks Wars.",
};

async function getGame(identifier: string): Promise<Game> {
	try {
		const res = await ApiClient.get<Game>(`/api/game/${identifier}`);
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
			`/api/game/${identifier}/lobbies?statuses=waiting,starting,inProgress&limit=6&offset=0`
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

export default async function LudoPage() {
	const [game, { lobbies, total }] = await Promise.all([
		getGame(GAME_IDENTIFIER),
		getGameLobbies(GAME_IDENTIFIER),
	]);

	return (
		<div>
			{/* Game Header */}
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

			{/* Game Explanation */}
			<div className="mb-12">
				<div className="mx-auto max-w-4xl space-y-6">
					<h2 className="text-2xl font-bold sm:text-3xl">
						What is Ludo?
					</h2>
					<div className="space-y-4 text-base sm:text-lg">
						<p>
							Ludo is a classic strategy board game for 2-4
							players where each player races to move their four
							pawns from their starting area, around the board,
							and into their home column. It's a game of luck and
							strategy that has entertained families for
							generations!
						</p>
						<p>
							Players take turns rolling a dice and moving their
							pawns. Rolling a 6 lets you bring a new pawn onto
							the board and grants you another roll. Land on an
							opponent's pawn to send them back to start! The
							first player to get all four pawns home wins.
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
								<h4 className="font-semibold">Roll to Start</h4>
								<p className="text-muted-foreground">
									Roll a 6 to move a pawn from your home base
									onto the starting square. Each player has
									four pawns that begin in their colored home
									base.
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								2
							</div>
							<div>
								<h4 className="font-semibold">
									Move Your Pawns
								</h4>
								<p className="text-muted-foreground">
									Roll the dice and move one of your pawns by
									the number shown. Pawns move clockwise
									around the 52-square track. Rolling a 6
									gives you a bonus turn!
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								3
							</div>
							<div>
								<h4 className="font-semibold">
									Capture Opponents
								</h4>
								<p className="text-muted-foreground">
									Land on an opponent's pawn to send it back
									to their home base. But beware - safe
									squares (marked with stars) protect pawns
									from capture!
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								4
							</div>
							<div>
								<h4 className="font-semibold">Race to Home</h4>
								<p className="text-muted-foreground">
									After completing a full circuit, your pawns
									enter the home stretch - a 6-square path
									leading to the finish. Get all four pawns
									home to win!
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

					{/* Special Rules */}
					<div className="rounded-2xl border border-yellow-500/30 bg-yellow-500/10 p-4 sm:p-6">
						<h4 className="mb-2 font-semibold text-yellow-500">
							🎲 Special Rules
						</h4>
						<ul className="space-y-2 text-sm sm:text-base">
							<li className="flex items-start gap-2">
								<span className="text-yellow-500">•</span>
								<span>
									<strong>Rolling 6:</strong> Get a pawn out
									of home base OR get a bonus turn
								</span>
							</li>
							<li className="flex items-start gap-2">
								<span className="text-yellow-500">•</span>
								<span>
									<strong>Safe Squares:</strong> Pawns on star
									squares cannot be captured
								</span>
							</li>
							<li className="flex items-start gap-2">
								<span className="text-yellow-500">•</span>
								<span>
									<strong>Exact Landing:</strong> You must
									roll the exact number to enter the finish
								</span>
							</li>
							<li className="flex items-start gap-2">
								<span className="text-yellow-500">•</span>
								<span>
									<strong>Turn Timer:</strong> You have 30
									seconds to roll the dice and make your move
								</span>
							</li>
						</ul>
					</div>
				</div>
			</div>

			{/* Active Lobbies */}
			<GameLobbies
				gameIdentifier={GAME_IDENTIFIER}
				initialLobbies={lobbies}
				initialTotal={total}
			/>
		</div>
	);
}
