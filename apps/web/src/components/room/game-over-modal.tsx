"use client";

import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useGameOverData, useLobby, useLobbyActions } from "@/lib/stores/room";
import { Trophy, Sparkles, Coins } from "lucide-react";
import { cn } from "@/lib/utils";

const rankLabels: Record<number, string> = {
	1: "1st Place",
	2: "2nd Place",
	3: "3rd Place",
};

const rankColors: Record<number, string> = {
	1: "text-yellow-500",
	2: "text-gray-400",
	3: "text-amber-600",
};

export default function GameOverModal() {
	const gameOverData = useGameOverData();
	const lobbyActions = useLobbyActions();
	const lobby = useLobby();

	const handleClose = () => {
		lobbyActions.setGameOver(null);
	};

	if (!gameOverData) return null;

	const { rank, prize, warsPoint } = gameOverData;
	const isTopThree = rank <= 3;
	const rankLabel = rankLabels[rank] || `#${rank}`;
	const rankColor = rankColors[rank] || "text-muted-foreground";

	return (
		<Dialog
			open={!!gameOverData}
			onOpenChange={(open) => !open && handleClose()}
		>
			<DialogContent className="sm:max-w-md" showCloseButton={false}>
				<DialogHeader className="text-center">
					<DialogTitle className="text-2xl font-bold">
						Game Over!
					</DialogTitle>
					<DialogDescription className="sr-only">
						Your game results and rewards
					</DialogDescription>
				</DialogHeader>

				<div className="flex flex-col items-center gap-6 py-4">
					{/* Rank Display */}
					<div className="flex flex-col items-center gap-2">
						{isTopThree && (
							<Trophy
								className={cn("size-16", rankColor)}
								strokeWidth={1.5}
							/>
						)}
						<p
							className={cn(
								"text-4xl font-bold",
								isTopThree ? rankColor : "text-foreground"
							)}
						>
							{rankLabel}
						</p>
						{!isTopThree && (
							<p className="text-muted-foreground text-sm">
								Better luck next time!
							</p>
						)}
					</div>

					{/* Rewards Section */}
					<div className="w-full space-y-3">
						{/* Prize (if won) */}
						{prize != null && prize > 0 && (
							<div className="flex items-center justify-between rounded-lg border bg-card p-4">
								<div className="flex items-center gap-3">
									<div className="flex size-10 items-center justify-center rounded-full bg-green-500/10">
										<Coins className="size-5 text-green-500" />
									</div>
									<span className="font-medium">
										Prize Won
									</span>
								</div>
								<span className="text-xl font-bold text-green-500">
									+{prize.toFixed(2)}{" "}
									{lobby?.tokenSymbol || "STX"}
								</span>
							</div>
						)}

						{/* Wars Points */}
						<div className="flex items-center justify-between rounded-lg border bg-card p-4">
							<div className="flex items-center gap-3">
								<div className="flex size-10 items-center justify-center rounded-full bg-primary/10">
									<Sparkles className="size-5 text-primary" />
								</div>
								<span className="font-medium">Wars Points</span>
							</div>
							<span className="text-xl font-bold text-primary">
								+{warsPoint}
							</span>
						</div>
					</div>
				</div>

				<div className="flex justify-center pt-2">
					<Button onClick={handleClose} className="w-full">
						Close
					</Button>
				</div>
			</DialogContent>
		</Dialog>
	);
}
