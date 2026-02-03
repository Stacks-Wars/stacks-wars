import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions/game";
import type { LobbyInfo, PaginatedLobbiesResponse } from "@/lib/definitions";
import GameLobbies from "../_components/game-lobbies";
import Image from "next/image";

const GAME_IDENTIFIER = "lexi-wars";

export const metadata: Metadata = {
	title: "Lexi Wars",
	description:
		"Battle with words in this exciting lexical strategy game on Stacks Wars.",
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

export default async function LexiWarsPage() {
	const [game, { lobbies, total }] = await Promise.all([
		getGame(GAME_IDENTIFIER),
		getGameLobbies(GAME_IDENTIFIER),
	]);

	return (
		<div className="container mx-auto px-4 py-8">
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
						What is Lexi Wars?
					</h2>
					<div className="space-y-4 text-base sm:text-lg">
						<p>
							Lexi Wars is a fast-paced, elimination-style word
							game where players compete to submit valid words
							that follow ever-changing rules. It's a battle of
							vocabulary, quick thinking, and strategic word
							creation under time pressure!
						</p>
						<p>
							Players take turns submitting words that must
							satisfy the current rule (like containing a specific
							letter, having minimum length, or ending with
							certain patterns). Words must be valid dictionary
							entries and haven't been used before. The game
							eliminates players who can't submit valid words
							within the time limit, with the last player standing
							claiming victory.
						</p>
					</div>

					<h3 className="text-xl font-semibold sm:text-2xl">
						How the Game Flow Works
					</h3>
					<div className="space-y-4">
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								1
							</div>
							<div>
								<h4 className="font-semibold">Setup Phase</h4>
								<p className="text-muted-foreground">
									Players join a lobby and wait for the game
									to start. Once all players are ready, the
									game begins with the first rule displayed
									and turn rotation established.
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								2
							</div>
							<div>
								<h4 className="font-semibold">
									Rule-Based Word Submission
								</h4>
								<p className="text-muted-foreground">
									On your turn, you have 15 seconds to submit
									a word that follows the current rule (e.g.,
									"must contain the letter 'A'" or "must be at
									least 6 characters"). The word must be in
									the dictionary and not previously used in
									the game.
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								3
							</div>
							<div>
								<h4 className="font-semibold">
									Elimination & Progression
								</h4>
								<p className="text-muted-foreground">
									If you fail to submit a valid word within
									the time limit, you're eliminated and ranked
									based on when you were eliminated. Rules
									cycle through different challenges, and
									after completing all rules, the minimum word
									length increases for higher difficulty.
								</p>
							</div>
						</div>
						<div className="flex items-start gap-4">
							<div className="bg-primary text-primary-foreground flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold">
								4
							</div>
							<div>
								<h4 className="font-semibold">Victory</h4>
								<p className="text-muted-foreground">
									The game continues until only one player
									remains. Prize distribution follows a
									structured system: 1st place gets 50% (70%
									for 2-player games), 2nd gets 30%, and 3rd
									gets 20% of the total pot.
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
								{game.category || "Strategy"}
							</p>
						</div>
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
