import SeasonsList from "./_components/seasons-list";
import { ApiClient } from "@/lib/api/client";
import type { Season } from "@/lib/definitions";

export default async function SeasonsPage() {
	const response = await ApiClient.get<Season[]>("/api/season?limit=50");

	return (
		<div className="container mx-auto px-4">
			<div className="py-4 lg:py-15">
				<h1 className="text-2xl md:text-5xl font-bold text-center mb-2">
					Seasons
				</h1>
				<p className="text-xs md:text-2xl font-medium text-center">
					Track seasonal competitions and leaderboard periods
				</p>
			</div>
			<SeasonsList initialSeasons={response.data || []} />
		</div>
	);
}
