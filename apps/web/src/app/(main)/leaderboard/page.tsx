import { ApiClient } from "@/lib/api/client";
import type { LeaderBoard } from "@/lib/definitions";
import LeaderBoardTable from "./_components/leader-board-table";
import LeaderBoardPodium from "./_components/leader-board-podium";
import { toast } from "sonner";

export default async function LeaderBoardPage() {
	const res = await ApiClient.get<{
		leaderboard: LeaderBoard[];
		total: number;
	}>(`/api/leaderboard`);
	const leaderboard = res.data?.leaderboard || [];
	const total = res.data?.total || 0;

	if (res.error) {
		toast.error("Failed to load leaderboard data", {
			description: res.error,
			action: {
				label: "Retry",
				onClick: () => {
					window.location.reload();
				},
			},
		});
	}

	return (
		<div className="container mx-auto px-4">
			<LeaderBoardPodium leaderboard={leaderboard} />
			<LeaderBoardTable leaderboard={leaderboard} total={total} />
		</div>
	);
}
