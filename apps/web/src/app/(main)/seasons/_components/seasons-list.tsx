"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import type { Season } from "@/lib/definitions";
import SeasonCard from "./season-card";
import SeasonDialog from "./season-dialog";
import { Plus } from "lucide-react";

interface SeasonsListProps {
	initialSeasons: Season[];
}

export default function SeasonsList({ initialSeasons }: SeasonsListProps) {
	const [seasons, setSeasons] = useState(initialSeasons);
	const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);

	const handleSeasonCreated = (newSeason: Season) => {
		setSeasons((prev) => [newSeason, ...prev]);
		setIsCreateDialogOpen(false);
	};

	const handleSeasonUpdated = (updatedSeason: Season) => {
		setSeasons((prev) =>
			prev.map((s) => (s.id === updatedSeason.id ? updatedSeason : s))
		);
	};

	return (
		<>
			<div className="flex justify-end mb-6">
				<Button
					onClick={() => setIsCreateDialogOpen(true)}
					className="gap-2 rounded-full"
				>
					<Plus className="size-4" />
					Create Season
				</Button>
			</div>

			<div className="grid grid-cols-1 gap-4 sm:gap-6 pb-8">
				{seasons.length === 0 ? (
					<div className="text-center py-12 text-muted-foreground">
						<p className="text-lg font-medium">No seasons yet</p>
						<p className="text-sm">Create the first season!</p>
					</div>
				) : (
					seasons.map((season) => (
						<SeasonCard
							key={season.id}
							season={season}
							onUpdate={handleSeasonUpdated}
						/>
					))
				)}
			</div>

			<SeasonDialog
				open={isCreateDialogOpen}
				onOpenChange={setIsCreateDialogOpen}
				onSuccess={handleSeasonCreated}
			/>
		</>
	);
}
