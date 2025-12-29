import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { JoinRequest, PlayerState } from "@/lib/definitions";
import { formatAddress } from "@/lib/utils";
import Link from "next/link";
import { IoStar } from "react-icons/io5";

interface PlayerProps {
	player: PlayerState | JoinRequest;
	isCreator: boolean;
}

export default function Player({ player, isCreator }: PlayerProps) {
	return (
		<div className="bg-card px-6 p-4 rounded-2xl flex justify-between items-center">
			<Link
				href={`/u/${player.username || player.walletAddress}`}
				className="flex gap-2 items-center max-w-1/2 truncate"
			>
				<Avatar className="size-15 uppercase">
					<AvatarImage src={""} alt="player profile picture" />
					<AvatarFallback>
						{(
							player.displayName ||
							player.username ||
							player.walletAddress
						).slice(0, 2)}
					</AvatarFallback>
				</Avatar>
				<div>
					{player.displayName ? (
						<>
							<p className="text-xl font-medium">
								{player.displayName}
							</p>
							<p className="text-base">
								@
								{player.username ||
									formatAddress(player.walletAddress)}
							</p>
						</>
					) : (
						<p className="text-xl font-medium">
							{player.username ||
								formatAddress(player.walletAddress)}
						</p>
					)}
				</div>
			</Link>

			<div className="flex items-center gap-4">
				{player.isCreator && (
					<Badge variant={"secondary"}>Creator</Badge>
				)}
				<p className="flex items-center gap-1">
					<span>{player.trustRating}</span>{" "}
					<IoStar className="text-yellow-400" />
				</p>
				{isCreator && (
					<div>
						{player.state === "pending" && (
							<div className="flex gap-4">
								<Button className="rounded-full text-base font-medium">
									Accept
								</Button>
								<Button
									variant={"outline"}
									className="rounded-full text-base font-medium"
								>
									Decline
								</Button>
							</div>
						)}
						{player.state === "accepted" && (
							<Button
								variant={"outline"}
								className="rounded-full text-base font-medium"
							>
								Remove
							</Button>
						)}
					</div>
				)}
			</div>
		</div>
	);
}
