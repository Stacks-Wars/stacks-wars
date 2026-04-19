import { ApiClient } from "@/lib/api/client";
import type { LeaderBoard, Game, Season } from "@/lib/definitions";
import LeaderBoardTable from "./_components/leader-board-table";
import LeaderBoardPodium from "./_components/leader-board-podium";
import GameFilter from "./_components/game-filter";

export default async function LeaderBoardPage() {
	const [leaderboardRes, gamesRes, seasonsRes] = await Promise.all([
		ApiClient.get<{
			leaderboard: LeaderBoard[];
			total: number;
		}>(`/api/leaderboards`),
		ApiClient.get<Game[]>(`/api/games/`),
		ApiClient.get<Season[]>(`/api/seasons?limit=50`),
	]);

	const seasons = seasonsRes.data || [];
	const now = new Date();
	const currentSeason = seasons.find((season) => {
		const start = new Date(season.startDate);
		const end = new Date(season.endDate);
		return now >= start && now <= end;
	});
	const currentSeasonId = currentSeason?.id ?? seasons[0]?.id ?? null;

	const leaderboard = leaderboardRes.data?.leaderboard || [];
	const total = leaderboardRes.data?.total || 0;
	const games = gamesRes.data || [];

	return (
		<div className="container mx-auto px-4">
			<LeaderBoardPodium leaderboard={leaderboard} />
			<GameFilter
				games={games}
				seasons={seasons}
				initialSeasonId={currentSeasonId}
			/>
			<LeaderBoardTable
				leaderboard={leaderboard}
				total={total}
				initialSeasonId={currentSeasonId}
			/>
		</div>
	);
}
