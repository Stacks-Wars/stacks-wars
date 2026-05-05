import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions/game";
import type { LobbyInfo, PaginatedLobbiesResponse } from "@/lib/definitions";
import GameLobbies from "../_components/game-lobbies";
import Image from "next/image";

const GAME_IDENTIFIER = "ludo-rush";

export const metadata: Metadata = {
	title: "Ludo Rush",
	description:
		"Fast-paced Ludo variant where capturing opponents instantly finishes your pawn on Stacks Wars.",
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

export default async function LudoRushPage() {
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
						What is Ludo Rush?
					</h2>
					<div className="space-y-4 text-base sm:text-lg">
						<p>
							Ludo Rush is a fast-paced variant of the classic
							Ludo board game for 2-4 players. The core mechanics
							are the same — roll dice, move pawns around the
							52-square track, and race to get all four pawns
							home. But with a devastating twist: capturing an
							opponent&apos;s pawn sends them back to their home
							base AND instantly finishes your attacking pawn!
						</p>
						<p>
							With only 4 safe squares on the entire board (each
							player&apos;s entry point), nowhere is truly safe.
							Games are faster, more aggressive, and every capture
							is a huge power play.
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
								<h4 className="font-semibold">Roll the Dice</h4>
								<p className="text-muted-foreground">
									You have 5 seconds to roll two dice. Choose
									which value to use — die 1, die 2, or their
									sum. Roll a 6 to bring a pawn out of home
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
									You have 15 seconds to pick a pawn and move
									it. Pawns move clockwise around the
									52-square track. Rolling double sixes gives
									you a bonus turn!
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								3
							</div>
							<div>
								<h4 className="font-semibold">Rush Capture!</h4>
								<p className="text-muted-foreground">
									Land on an opponent&apos;s pawn to send it
									back to their home base — AND your attacking
									pawn is instantly finished! Only the 4 entry
									points (one per player) are safe.
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
									enter the home stretch — a 5-square path
									leading to the finish. Get all four pawns
									home to win! Winner takes the entire pot.
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
					<div className="rounded-2xl border border-orange-500/30 bg-orange-500/10 p-4 sm:p-6">
						<h4 className="mb-2 font-semibold text-orange-500">
							⚡ Ludo Rush Rules
						</h4>
						<ul className="space-y-2 text-sm sm:text-base">
							<li className="flex items-start gap-2">
								<span className="text-orange-500">•</span>
								<span>
									<strong>Rush Capture:</strong> Capturing an
									opponent sends them home AND instantly
									finishes your pawn
								</span>
							</li>
							<li className="flex items-start gap-2">
								<span className="text-orange-500">•</span>
								<span>
									<strong>Only 4 Safe Squares:</strong> Only
									each player&apos;s entry point is safe
								</span>
							</li>
							<li className="flex items-start gap-2">
								<span className="text-orange-500">•</span>
								<span>
									<strong>Dual Dice:</strong> Roll two dice
									and choose die 1, die 2, or their sum
								</span>
							</li>
							<li className="flex items-start gap-2">
								<span className="text-orange-500">•</span>
								<span>
									<strong>Double Sixes:</strong> Roll two 6s
									to earn a bonus turn
								</span>
							</li>
							<li className="flex items-start gap-2">
								<span className="text-orange-500">•</span>
								<span>
									<strong>Timers:</strong> 5 seconds to roll,
									15 seconds to move — auto-move on timeout
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
