import { ApiClient } from "@/lib/api/client";
import type { Game } from "@/lib/definitions";
import GamesList from "./_components/games-list";

export default async function ManageGamesPage() {
	const response = await ApiClient.get<Game[]>("/api/games?limit=100");

	return (
		<div className="container mx-auto px-4">
			<div className="py-4 lg:py-15">
				<h1 className="mb-2 text-center text-2xl font-bold md:text-5xl">
					Manage Games
				</h1>
				<p className="text-center text-xs font-medium md:text-2xl">
					Toggle games on or off for the platform
				</p>
			</div>
			<GamesList initialGames={response.data || []} />
		</div>
	);
}
