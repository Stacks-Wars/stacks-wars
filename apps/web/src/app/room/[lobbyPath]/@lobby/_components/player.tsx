import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { JoinRequest, PlayerState } from "@/lib/definitions";
import { formatAddress } from "@/lib/utils";
import Link from "next/link";
import { IoStar } from "react-icons/io5";
import { useIsActionLoading } from "@/lib/stores/room";

interface PlayerProps {
	player: PlayerState | JoinRequest;
	isCreator: boolean;
	onApprove?: (userId: string) => void;
	onReject?: (userId: string) => void;
	onKick?: (userId: string) => void;
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
		<div className="bg-card px-3 sm:px-4 lg:px-6 py-3 sm:py-4 rounded-xl sm:rounded-2xl flex justify-between items-center gap-3">
			<Link
				href={`/u/${player.username || player.walletAddress}`}
				className="flex gap-2 sm:gap-3 items-center min-w-0 flex-1"
			>
				<Avatar className="size-10 sm:size-12 lg:size-15 uppercase shrink-0">
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
							<p className="text-sm sm:text-base lg:text-xl font-medium truncate">
								{player.displayName}
							</p>
							<p className="text-xs sm:text-sm lg:text-base text-muted-foreground truncate">
								@
								{player.username ||
									formatAddress(player.walletAddress)}
							</p>
						</>
					) : (
						<p className="text-sm sm:text-base lg:text-xl font-medium truncate">
							{player.username ||
								formatAddress(player.walletAddress)}
						</p>
					)}
				</div>
			</Link>

			<div className="flex items-center gap-2 shrink-0">
				{player.isCreator && (
					<Badge variant={"secondary"} className="text-xs sm:text-sm">
						Creator
					</Badge>
				)}
				<p className="flex items-center gap-1 text-sm sm:text-base">
					<span>{player.trustRating}</span>
					<IoStar className="text-yellow-400 size-3 sm:size-4" />
				</p>
			</div>

			{isCreator && !player.isCreator && (
				<div className="shrink-0">
					{player.state === "pending" && (
						<div className="flex gap-2 sm:gap-3 lg:gap-4">
							<Button
								className="rounded-full text-xs sm:text-sm lg:text-base font-medium px-3 sm:px-4 h-8 sm:h-9 lg:h-10"
								onClick={() => onApprove?.(player.userId)}
								disabled={isApproving || isRejecting}
							>
								{isApproving ? "Accepting..." : "Accept"}
							</Button>
							<Button
								variant={"outline"}
								className="rounded-full text-xs sm:text-sm lg:text-base font-medium px-3 sm:px-4 h-8 sm:h-9 lg:h-10"
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
							className="rounded-full text-xs sm:text-sm lg:text-base font-medium px-3 sm:px-4 h-8 sm:h-9 lg:h-10"
							onClick={() => onKick?.(player.userId)}
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
