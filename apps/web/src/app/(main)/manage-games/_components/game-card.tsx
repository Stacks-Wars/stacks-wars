"use client";

import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import type { Game } from "@/lib/definitions";
import { Gamepad2, Users, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { ApiClient } from "@/lib/api/client";
import { toast } from "sonner";
import Image from "next/image";

interface GameCardProps {
	game: Game;
	onUpdate: (updatedGame: Game) => void;
}

export default function GameCard({ game, onUpdate }: GameCardProps) {
	const [isToggling, setIsToggling] = useState(false);

	const handleToggleActive = async (checked: boolean) => {
		setIsToggling(true);

		try {
			const response = await ApiClient.patch<{
				id: string;
				name: string;
				isActive: boolean;
			}>(`/api/game/${game.id}/active`, { isActive: checked });

			if (response.error || !response.data) {
				toast.error("Failed to update game status", {
					description: response.error || "Unknown error",
				});
				return;
			}

			toast.success(
				`${game.name} ${checked ? "activated" : "deactivated"}`
			);
			onUpdate({ ...game, isActive: response.data.isActive });
		} catch (error) {
			toast.error("An unexpected error occurred");
		} finally {
			setIsToggling(false);
		}
	};

	return (
		<div
			className={cn(
				"bg-card rounded-3xl border p-6 transition-all hover:shadow-md",
				game.isActive
					? "border-primary/20"
					: "border-destructive/20 opacity-75"
			)}
		>
			<div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
				{/* Game Info */}
				<div className="flex flex-1 items-center gap-4">
					{/* Game Image */}
					<div className="relative size-16 shrink-0 overflow-hidden rounded-2xl sm:size-20">
						<Image
							src={game.imageUrl}
							alt={game.name}
							fill
							className="object-cover"
						/>
					</div>

					<div className="flex-1 space-y-1">
						<div className="flex items-center gap-2">
							<h3 className="text-lg font-bold sm:text-xl">
								{game.name}
							</h3>
							<Badge
								variant={
									game.isActive ? "default" : "secondary"
								}
								className={cn(
									game.isActive
										? "border-green-500/50 bg-green-500/10 text-green-500"
										: "border-red-500/50 bg-red-500/10 text-red-500"
								)}
							>
								{game.isActive ? "Active" : "Inactive"}
							</Badge>
						</div>

						<p className="text-muted-foreground line-clamp-2 text-sm">
							{game.description}
						</p>

						{/* Meta info */}
						<div className="flex flex-wrap items-center gap-3 pt-1">
							<div className="text-muted-foreground flex items-center gap-1 text-xs">
								<Users className="size-3" />
								<span>
									{game.minPlayers}-{game.maxPlayers} players
								</span>
							</div>
							{game.category && game.category.length > 0 && (
								<div className="text-muted-foreground flex items-center gap-1 text-xs">
									<Gamepad2 className="size-3" />
									<span>{game.category.join(", ")}</span>
								</div>
							)}
						</div>
					</div>
				</div>

				{/* Toggle */}
				<div className="flex shrink-0 items-center gap-3">
					{isToggling && (
						<Loader2 className="text-muted-foreground size-4 animate-spin" />
					)}
					<label className="flex cursor-pointer items-center gap-2">
						<span className="text-muted-foreground text-sm">
							{game.isActive ? "Active" : "Inactive"}
						</span>
						<Switch
							checked={game.isActive}
							onCheckedChange={handleToggleActive}
							disabled={isToggling}
						/>
					</label>
				</div>
			</div>
		</div>
	);
}
