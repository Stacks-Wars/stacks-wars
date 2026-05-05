"use client";

import type { Game } from "@/lib/definitions";
import GameCard from "@/components/main/game-card";

interface CreatedGamesProps {
	userId: string;
	games: Game[];
}

export default function CreatedGames({ userId, games }: CreatedGamesProps) {
	if (games.length === 0) return null;

	return (
		<div className="mt-8 px-4 sm:mt-12 sm:px-0">
			<div className="mb-4 sm:mb-6">
				<h2 className="text-xl font-bold sm:text-3xl">Created Games</h2>
			</div>
			<div className="grid grid-cols-1 gap-4 sm:gap-6">
				{games.map((game) => (
					<GameCard key={game.id} game={game} />
				))}
			</div>
		</div>
	);
}
