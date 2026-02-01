import { ApiClient } from "@/lib/api/client";
import type { LeaderBoard } from "@/lib/definitions";
import LeaderBoardTable from "./_components/leader-board-table";
import LeaderBoardPodium from "./_components/leader-board-podium";

export default async function LeaderBoardPage() {
	const res = await ApiClient.get<{
		leaderboard: LeaderBoard[];
		total: number;
	}>(`/api/leaderboard`);
	const leaderboard = res.data?.leaderboard || [];
	const total = res.data?.total || 0;

	return (
		<div className="container mx-auto px-4">
			<LeaderBoardPodium leaderboard={leaderboard} />
			<LeaderBoardTable leaderboard={leaderboard} total={total} />
		</div>
	);
}
