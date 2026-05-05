"use client";

import { useState } from "react";
import type { Game } from "@/lib/definitions";
import GameCard from "./game-card";

interface GamesListProps {
	initialGames: Game[];
}

export default function GamesList({ initialGames }: GamesListProps) {
	const [games, setGames] = useState(initialGames);

	const handleGameUpdated = (updatedGame: Game) => {
		setGames((prev) =>
			prev.map((g) => (g.id === updatedGame.id ? updatedGame : g))
		);
	};

	return (
		<div className="grid grid-cols-1 gap-4 pb-8 sm:gap-6">
			{games.length === 0 ? (
				<div className="text-muted-foreground py-12 text-center">
					<p className="text-lg font-medium">No games yet</p>
					<p className="text-sm">
						Create a game first from the Create Game page.
					</p>
				</div>
			) : (
				games.map((game) => (
					<GameCard
						key={game.id}
						game={game}
						onUpdate={handleGameUpdated}
					/>
				))
			)}
		</div>
	);
}
