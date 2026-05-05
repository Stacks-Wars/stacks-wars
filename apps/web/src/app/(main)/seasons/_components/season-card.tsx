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
					"bg-card rounded-3xl border p-6 transition-all hover:shadow-md",
					status === "active" && "border-primary bg-primary/5"
				)}
			>
				<div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-start">
					{/* Season Info */}
					<div className="flex-1 space-y-3">
						<div className="flex items-start gap-3">
							<div className="flex-1">
								<div className="mb-1 flex items-center gap-2">
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
												"border-green-500/50 bg-green-500/10 text-green-500"
										)}
									>
										{status.charAt(0).toUpperCase() +
											status.slice(1)}
									</Badge>
								</div>
								{season.description && (
									<p className="text-muted-foreground text-sm">
										{season.description}
									</p>
								)}
							</div>
						</div>

						{/* Dates */}
						<div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
							<div className="flex items-center gap-2 text-sm">
								<Calendar className="text-muted-foreground size-4" />
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
								<Clock className="text-muted-foreground size-4" />
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
						className="shrink-0 gap-2 rounded-full"
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
