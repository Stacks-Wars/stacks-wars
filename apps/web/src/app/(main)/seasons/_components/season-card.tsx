"use client";

import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { Season } from "@/lib/definitions";
import { Calendar, Clock, Edit } from "lucide-react";
import { cn } from "@/lib/utils";
import SeasonDialog from "./season-dialog";

interface SeasonCardProps {
	season: Season;
	onUpdate: (updatedSeason: Season) => void;
}

function formatDateTime(dateStr: string) {
	const date = new Date(dateStr);
	return date.toLocaleString("default", {
		month: "short",
		day: "numeric",
		year: "numeric",
		hour: "2-digit",
		minute: "2-digit",
	});
}

function getSeasonStatus(startDate: string, endDate: string) {
	const now = new Date();
	const start = new Date(startDate);
	const end = new Date(endDate);

	if (now < start) return "upcoming";
	if (now > end) return "past";
	return "active";
}

export default function SeasonCard({ season, onUpdate }: SeasonCardProps) {
	const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
	const status = getSeasonStatus(season.startDate, season.endDate);

	const handleSeasonUpdated = (updatedSeason: Season) => {
		onUpdate(updatedSeason);
		setIsEditDialogOpen(false);
	};

	return (
		<>
			<div
				className={cn(
					"rounded-3xl border bg-card p-6 transition-all hover:shadow-md",
					status === "active" && "border-primary bg-primary/5"
				)}
			>
				<div className="flex flex-col sm:flex-row sm:items-start justify-between gap-4">
					{/* Season Info */}
					<div className="flex-1 space-y-3">
						<div className="flex items-start gap-3">
							<div className="flex-1">
								<div className="flex items-center gap-2 mb-1">
									<h3 className="text-xl font-bold">
										{season.name}
									</h3>
									<Badge
										variant={
											status === "active"
												? "default"
												: status === "upcoming"
													? "secondary"
													: "outline"
										}
										className={cn(
											status === "active" &&
												"bg-green-500/10 text-green-500 border-green-500/50"
										)}
									>
										{status.charAt(0).toUpperCase() +
											status.slice(1)}
									</Badge>
								</div>
								{season.description && (
									<p className="text-sm text-muted-foreground">
										{season.description}
									</p>
								)}
							</div>
						</div>

						{/* Dates */}
						<div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
							<div className="flex items-center gap-2 text-sm">
								<Calendar className="size-4 text-muted-foreground" />
								<div>
									<p className="text-muted-foreground">
										Start Date
									</p>
									<p className="font-medium">
										{formatDateTime(season.startDate)}
									</p>
								</div>
							</div>
							<div className="flex items-center gap-2 text-sm">
								<Clock className="size-4 text-muted-foreground" />
								<div>
									<p className="text-muted-foreground">
										End Date
									</p>
									<p className="font-medium">
										{formatDateTime(season.endDate)}
									</p>
								</div>
							</div>
						</div>
					</div>

					{/* Actions */}
					<Button
						variant="outline"
						onClick={() => setIsEditDialogOpen(true)}
						className="gap-2 shrink-0 rounded-full"
					>
						<Edit className="size-4" />
						Edit
					</Button>
				</div>
			</div>

			<SeasonDialog
				open={isEditDialogOpen}
				onOpenChange={setIsEditDialogOpen}
				season={season}
				onSuccess={handleSeasonUpdated}
			/>
		</>
	);
}
