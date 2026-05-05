import SeasonsList from "./_components/seasons-list";
import { ApiClient } from "@/lib/api/client";
import type { Season } from "@/lib/definitions";

export default async function SeasonsPage() {
	const response = await ApiClient.get<Season[]>("/api/season?limit=50");

	return (
		<div className="container mx-auto px-4">
			<div className="py-4 lg:py-15">
				<h1 className="mb-2 text-center text-2xl font-bold md:text-5xl">
					Seasons
				</h1>
				<p className="text-center text-xs font-medium md:text-2xl">
					Track seasonal competitions and leaderboard periods
				</p>
			</div>
			<SeasonsList initialSeasons={response.data || []} />
		</div>
	);
}
