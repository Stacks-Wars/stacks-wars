"use client";

import type { Game, Season } from "@/lib/definitions";
import Image from "next/image";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import {
	useLeaderboardGameId,
	useLeaderboardSeasonId,
	useLeaderboardActions,
} from "@/lib/stores/leaderboard";

interface GameFilterProps {
	games: Game[];
	seasons: Season[];
	initialSeasonId: number | null;
}

export default function GameFilter({
	games,
	seasons,
	initialSeasonId,
}: GameFilterProps) {
	const gameId = useLeaderboardGameId();
	const seasonId = useLeaderboardSeasonId();
	const actions = useLeaderboardActions();
	const selectedSeasonId = seasonId ?? initialSeasonId;

	return (
		<div className="mb-6 flex flex-col gap-3 sm:flex-row">
			<Select
				value={selectedSeasonId?.toString()}
				onValueChange={(value) => actions.setSeasonId(Number(value))}
			>
				<SelectTrigger className="w-full border-white/10 bg-white/5 sm:w-60">
					<SelectValue placeholder="Select Season" />
				</SelectTrigger>
				<SelectContent>
					{seasons.map((season) => (
						<SelectItem
							key={season.id}
							value={season.id.toString()}
						>
							{season.name}
						</SelectItem>
					))}
				</SelectContent>
			</Select>

			<Select
				value={gameId ?? "all"}
				onValueChange={(value) =>
					actions.setGameId(value === "all" ? null : value)
				}
			>
				<SelectTrigger className="w-full border-white/10 bg-white/5 sm:w-60">
					<SelectValue placeholder="All Games" />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value="all">
						<span className="flex items-center gap-2">
							<span className="text-base">🏆</span>
							All Games
						</span>
					</SelectItem>
					{games.map((game) => (
						<SelectItem key={game.id} value={game.id}>
							<span className="flex items-center gap-2">
								<Image
									src={game.imageUrl}
									alt={game.name}
									width={20}
									height={20}
									className="size-5 rounded-full object-cover"
								/>
								{game.name}
							</span>
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	);
}
