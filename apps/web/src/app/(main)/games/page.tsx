import GameCard from "@/components/main/game-card";
import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";

export default async function GamesPage() {
	const games = await ApiClient.get<Game[]>("/api/games");

	return (
		<div className="container mx-auto px-4">
			<div className="py-4 lg:py-15">
				<h1 className="text-2xl md:text-5xl font-bold text-center mb-2">
					Available Games
				</h1>
				<p className="text-xs md:text-2xl font-medium text-center">
					Choose from our selection of games to compete and win STX
					rewards
				</p>
			</div>
			{games.data?.map((game, i) => (
				<GameCard key={i} {...game} />
			))}
		</div>
	);
}
