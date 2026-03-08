import { ApiClient } from "@/lib/api/client";
import type { LeaderBoard, Game } from "@/lib/definitions";
import LeaderBoardTable from "./_components/leader-board-table";
import LeaderBoardPodium from "./_components/leader-board-podium";
import GameFilter from "./_components/game-filter";

export default async function LeaderBoardPage() {
	const [leaderboardRes, gamesRes] = await Promise.all([
		ApiClient.get<{
			leaderboard: LeaderBoard[];
			total: number;
		}>(`/api/leaderboard`),
		ApiClient.get<Game[]>(`/api/games`),
	]);

	const leaderboard = leaderboardRes.data?.leaderboard || [];
	const total = leaderboardRes.data?.total || 0;
	const games = gamesRes.data || [];

	return (
		<div className="container mx-auto px-4">
			<LeaderBoardPodium leaderboard={leaderboard} />
			<GameFilter games={games} />
			<LeaderBoardTable leaderboard={leaderboard} total={total} />
		</div>
	);
}
