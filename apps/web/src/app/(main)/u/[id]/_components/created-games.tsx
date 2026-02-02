"use client";

import type { Game } from "@/lib/definitions";
import GameCard from "@/components/main/game-card";
import { useUser } from "@/lib/stores/user";
import Link from "next/link";
import { IoAdd } from "react-icons/io5";
import { Button } from "@/components/ui/button";

interface CreatedGamesProps {
	userId: string;
	games: Game[];
}

export default function CreatedGames({ userId, games }: CreatedGamesProps) {
	const user = useUser();

	return (
		<>
			{userId === user?.id || games.length > 0 ? (
				<div className="mt-8 px-4 sm:mt-12 sm:px-0">
					<div className="mb-4 flex items-center justify-between sm:mb-6">
						<h2 className="text-xl font-bold sm:text-3xl">
							Created Games
						</h2>
						{userId === user?.id && (
							<Button
								asChild
								className="h-8 rounded-full text-xs has-[>svg]:px-3.5 sm:h-12 sm:text-base sm:has-[>svg]:px-7"
							>
								<Link href="/create-game">
									<IoAdd className="text-lg sm:text-xl" />{" "}
									Create Game
								</Link>
							</Button>
						)}
					</div>
					{games.length > 0 ? (
						<div className="grid grid-cols-1 gap-4 sm:gap-6">
							{games.map((game) => (
								<GameCard key={game.id} game={game} />
							))}
						</div>
					) : (
						<div className="text-muted-foreground py-8 text-center sm:py-12">
							<p className="text-sm sm:text-base">
								No games created yet
							</p>
						</div>
					)}
				</div>
			) : null}
		</>
	);
}
