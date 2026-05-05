"use client";

import { cn } from "@/lib/utils";

interface PlayerIdentifierPillProps {
	label: string;
	isYou?: boolean;
	dotClassName: string;
	isActiveTurn?: boolean;
	activeRingClassName?: string;
	pawnsLeft?: number;
	className?: string;
}

export function PlayerIdentifierPill({
	label,
	isYou = false,
	dotClassName,
	isActiveTurn = false,
	activeRingClassName = "ring-2 ring-offset-1 ring-primary",
	pawnsLeft,
	className,
}: PlayerIdentifierPillProps) {
	return (
		<div
			className={cn(
				"bg-card border-border flex items-center gap-2 rounded-full border px-3 py-1 text-xs",
				isActiveTurn && activeRingClassName,
				className
			)}
		>
			<div className={cn("h-3 w-3 rounded-full", dotClassName)} />
			<span>{isYou ? "You" : label}</span>
			{typeof pawnsLeft === "number" && (
				<span className="text-muted-foreground">({pawnsLeft} left)</span>
			)}
		</div>
	);
}
