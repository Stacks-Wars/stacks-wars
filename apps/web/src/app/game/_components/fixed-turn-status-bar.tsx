"use client";

import { cn } from "@/lib/utils";

interface FixedTurnStatusBarProps {
	isMyTurn: boolean;
	hasCurrentPlayer: boolean;
	currentPlayerLabel: string;
	timeRemaining: number;
	hideCountdownWhenNotMyTurn?: boolean;
}

export function FixedTurnStatusBar({
	isMyTurn,
	hasCurrentPlayer,
	currentPlayerLabel,
	timeRemaining,
	hideCountdownWhenNotMyTurn = true,
}: FixedTurnStatusBarProps) {
	if (!hasCurrentPlayer) return null;

	const showCountdown = hideCountdownWhenNotMyTurn ? isMyTurn : true;

	return (
		<div className="pointer-events-none fixed right-0 bottom-0 left-0 z-40 flex justify-center pb-5">
			<div
				className={cn(
					"pointer-events-auto flex items-center gap-3 rounded-full border px-4 py-2 shadow-lg backdrop-blur-md",
					isMyTurn
						? "border-primary bg-primary/10"
						: "bg-card/90 border-border"
				)}
			>
				<p className="text-sm font-semibold">
					{isMyTurn
						? "Your turn"
						: `Waiting for ${currentPlayerLabel}`}
				</p>
				{showCountdown && (
					<span
						className={cn(
							"text-xs tabular-nums",
							timeRemaining <= 3
								? "text-red-500"
								: "text-muted-foreground"
						)}
					>
						· Auto move in {timeRemaining}s
					</span>
				)}
			</div>
		</div>
	);
}
