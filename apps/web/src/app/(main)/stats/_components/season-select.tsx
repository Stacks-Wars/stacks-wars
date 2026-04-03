"use client";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { Season } from "@/lib/definitions";
import { useRouter } from "next/navigation";

interface SeasonSelectProps {
	seasons: Season[];
	selectedSeasonId: number;
}

export default function SeasonSelect({ seasons, selectedSeasonId }: SeasonSelectProps) {
	const router = useRouter();

	return (
		<Select
			value={selectedSeasonId.toString()}
			onValueChange={(value) => {
				router.push(`/stats?seasonId=${value}` as never);
			}}
		>
			<SelectTrigger className="w-full border-border bg-background sm:w-72">
				<SelectValue placeholder="Select season" />
			</SelectTrigger>
			<SelectContent>
				{seasons.map((season) => (
					<SelectItem key={season.id} value={season.id.toString()}>
						{season.name}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
}
