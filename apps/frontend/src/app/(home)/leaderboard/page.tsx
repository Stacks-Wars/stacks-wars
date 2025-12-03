import { apiRequest } from "@/lib/api";
import { LeaderBoard } from "@/types/schema/leaderboard";
import LeaderboardTable from "./_components/leaderboard-table";

export default async function LeaderboardPage() {
	const leaderboard = await apiRequest<LeaderBoard[]>({
		path: "/leaderboard",
		method: "GET",
		auth: false,
		cache: "no-store",
	});

	return (
		<div className="container mx-auto px-4 py-8">
			<div className="space-y-6">
				<div className="space-y-2 text-center">
					<h1 className="text-3xl font-bold tracking-tight">
						🏆 Leaderboard
					</h1>
					{/*<p className="text-muted-foreground">
						Top players competing in Stacks Wars
					</p>*/}
				</div>
				<LeaderboardTable data={leaderboard} />
			</div>
		</div>
	);
}
