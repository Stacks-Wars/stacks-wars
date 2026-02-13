import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { JoinRequest, PlayerState } from "@/lib/definitions";
import { formatAddress } from "@/lib/utils";
import Link from "next/link";
import { IoStar } from "react-icons/io5";
import { useIsActionLoading } from "@/lib/stores/room";
import { useEffect, useState } from "react";

/** Format a "last seen" duration from milliseconds ago */
function formatLastSeen(ms: number): string {
	const seconds = Math.floor(ms / 1000);
	if (seconds < 60) return `${seconds}s ago`;
	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h ago`;
	return `${Math.floor(hours / 24)}d ago`;
}

/** Activity indicator for a player based on lastPing */
function ActivityStatus({ lastPing }: { lastPing?: number }) {
	const [now, setNow] = useState(Date.now());

	// Re-render every 5 seconds to keep the "last seen" fresh
	useEffect(() => {
		const interval = setInterval(() => setNow(Date.now()), 5000);
		return () => clearInterval(interval);
	}, []);

	if (!lastPing) {
		return (
			<span className="text-muted-foreground text-[10px] sm:text-xs">
				Offline
			</span>
		);
	}

	const elapsed = now - lastPing;
	const isActive = elapsed <= 10_000; // Within 10 seconds

	if (isActive) {
		return (
			<span className="flex items-center gap-1 text-[10px] text-green-500 sm:text-xs">
				<span className="inline-block size-1.5 rounded-full bg-green-500" />
				Active
			</span>
		);
	}

	return (
		<span className="text-muted-foreground text-[10px] sm:text-xs">
			{formatLastSeen(elapsed)}
		</span>
	);
}

interface PlayerProps {
	player: PlayerState | JoinRequest;
	isCreator: boolean;
	onApprove?: (userId: string) => void;
	onReject?: (userId: string) => void;
	onKick?: (userId: string, playerAddress: string) => void;
	kickActionKey?: string;
	approveActionKey?: string;
	rejectActionKey?: string;
}

export default function Player({
	player,
	isCreator,
	onApprove,
	onReject,
	onKick,
	kickActionKey,
	approveActionKey,
	rejectActionKey,
}: PlayerProps) {
	// Get loading states from store if action keys are provided
	const isKickLoading = kickActionKey
		? useIsActionLoading(kickActionKey)
		: false;
	const isApproving = approveActionKey
		? useIsActionLoading(approveActionKey)
		: false;
	const isRejecting = rejectActionKey
		? useIsActionLoading(rejectActionKey)
		: false;
	return (
		<div className="flex items-center justify-between gap-3 rounded-xl border px-3 py-3 sm:rounded-2xl sm:px-4 sm:py-4 lg:px-6">
			<Link
				href={`/u/${player.username || player.walletAddress}`}
				className="flex min-w-0 flex-1 items-center gap-2 sm:gap-3"
			>
				<Avatar className="size-10 shrink-0 border uppercase sm:size-12 lg:size-15">
					<AvatarImage src={""} alt="player profile picture" />
					<AvatarFallback>
						{(
							player.displayName ||
							player.username ||
							player.walletAddress
						).slice(0, 2)}
					</AvatarFallback>
				</Avatar>
				<div className="min-w-0 flex-1">
					{player.displayName ? (
						<>
							<p className="truncate text-sm font-medium sm:text-base lg:text-xl">
								{player.displayName}
							</p>
							<p className="text-muted-foreground truncate text-xs sm:text-sm lg:text-base">
								@
								{player.username ||
									formatAddress(player.walletAddress)}
							</p>
						</>
					) : (
						<p className="truncate text-sm font-medium sm:text-base lg:text-xl">
							{player.username ||
								formatAddress(player.walletAddress)}
						</p>
					)}
				</div>
			</Link>

			<div className="flex shrink-0 items-center gap-2">
				{"lastPing" in player && (
					<ActivityStatus lastPing={player.lastPing} />
				)}
				{player.isCreator && (
					<Badge variant={"secondary"} className="text-xs sm:text-sm">
						Creator
					</Badge>
				)}
				<p className="flex items-center gap-1 text-sm sm:text-base">
					<span>{player.trustRating}</span>
					<IoStar className="size-3 text-yellow-400 sm:size-4" />
				</p>
			</div>

			{isCreator && !player.isCreator && (
				<div className="shrink-0">
					{player.state === "pending" && (
						<div className="flex gap-2 sm:gap-3 lg:gap-4">
							<Button
								className="h-8 rounded-full px-3 text-xs font-medium sm:h-9 sm:px-4 sm:text-sm lg:h-10 lg:text-base"
								onClick={() => onApprove?.(player.userId)}
								disabled={isApproving || isRejecting}
							>
								{isApproving ? "Accepting..." : "Accept"}
							</Button>
							<Button
								variant={"outline"}
								className="h-8 rounded-full px-3 text-xs font-medium sm:h-9 sm:px-4 sm:text-sm lg:h-10 lg:text-base"
								onClick={() => onReject?.(player.userId)}
								disabled={isApproving || isRejecting}
							>
								{isRejecting ? "Declining..." : "Decline"}
							</Button>
						</div>
					)}
					{player.state === "accepted" && (
						<Button
							variant={"outline"}
							className="h-8 rounded-full px-3 text-xs font-medium sm:h-9 sm:px-4 sm:text-sm lg:h-10 lg:text-base"
							onClick={() =>
								onKick?.(player.userId, player.walletAddress)
							}
							disabled={isKickLoading}
						>
							{isKickLoading ? "Removing..." : "Remove"}
						</Button>
					)}
				</div>
			)}
		</div>
	);
}
