import GameCard from "@/components/main/game-card";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";

export default async function GamesPage() {
	const games = await ApiClient.get<Game[]>("/api/games");

	return (
		<div className="container mx-auto px-4">
			<div className="py-4 lg:py-15">
				<h1 className="mb-2 text-center text-2xl font-bold md:text-5xl">
					Available Games
				</h1>
				<p className="text-center text-xs font-medium md:text-2xl">
					Choose from our selection of games to compete and win STX
					rewards
				</p>
			</div>
			<div className="grid grid-cols-1 gap-4 sm:gap-6">
				{games.data?.map((game) => (
					<GameCard
						key={game.id}
						game={game}
						action="createLobbyPage"
					/>
				))}
			</div>
		</div>
	);
}
