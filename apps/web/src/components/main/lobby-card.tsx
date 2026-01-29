import type { LobbyInfo } from "@/lib/definitions";
import { Button } from "../ui/button";
import Image from "next/image";
import { Badge } from "../ui/badge";
import { Lock, LockOpen, Users } from "lucide-react";
import { BiCoinStack } from "react-icons/bi";
import Link from "next/link";
import { displayUserIdentifier } from "@/lib/utils";
import { Skeleton } from "../ui/skeleton";

export default function LobbyCard({ lobbyInfo }: { lobbyInfo: LobbyInfo }) {
	const { lobby, game, creator } = lobbyInfo;

	return (
		<div className="bg-gradient-primary-2 w-full space-y-4 rounded-3xl border p-4 sm:space-y-6 sm:p-6 lg:p-8">
			<div className="space-y-3 sm:space-y-4">
				<div className="flex items-center justify-between gap-2">
					<div className="min-w-0">
						<p className="truncate text-base font-semibold sm:text-lg lg:text-xl">
							{lobby.name}
						</p>
						<Link
							href={`/u/${creator.username || creator.walletAddress}`}
							className="text-muted-foreground truncate text-xs sm:text-sm"
						>
							Creator -{" "}
							<span className="text-foreground">
								@{displayUserIdentifier(creator)}
							</span>
						</Link>
					</div>
					<Badge className="px-2.5 py-1.5 text-xs font-medium sm:px-3.5 sm:py-2 sm:text-sm">
						{lobby.status === "inProgress"
							? "In Progress"
							: lobby.status}
					</Badge>
				</div>

				<div className="flex items-center gap-3 text-xs sm:gap-4 sm:text-sm lg:text-base">
					{lobby.isPrivate ? (
						<p className="flex items-center gap-1.5">
							<Lock className="size-4 lg:size-5" />
							<span>Private</span>
						</p>
					) : (
						<p className="flex items-center gap-1.5">
							<LockOpen className="size-4 lg:size-5" />
							<span>Public</span>
						</p>
					)}
					<p className="flex items-center gap-1.5">
						<Users className="size-4 lg:size-5" />
						<span>
							{lobby.participantCount}/{game.maxPlayers}
						</span>
					</p>
					{lobby.currentAmount && (
						<p className="flex items-center gap-1.5">
							<BiCoinStack className="size-4 lg:size-5" />
							<span>
								{lobby.currentAmount} {lobby.tokenSymbol}
							</span>
						</p>
					)}
					{lobby.entryAmount && (
						<p className="flex items-center gap-1.5">
							<span className="text-muted-foreground">Entry</span>
							<span>
								{lobby.entryAmount} {lobby.tokenSymbol}
							</span>
						</p>
					)}
				</div>

				<Image
					src={game.imageUrl}
					alt="game-cover"
					width={516}
					height={185}
					loading="lazy"
					className="h-30 w-full rounded-3xl lg:h-45"
				/>

				{lobby.description && (
					<p className="line-clamp-2 text-xs sm:text-sm lg:text-base">
						{lobby.description}
					</p>
				)}
			</div>

			<Button
				asChild
				variant={"secondary"}
				className="bg-gradient-secondary-1 hover:bg-gradient-secondary-2 w-full rounded-full py-2.5 text-sm font-medium sm:py-3.5 sm:text-base lg:py-4 lg:text-lg"
			>
				<Link href={`/room/${lobby.path}`}>Open Room</Link>
			</Button>
		</div>
	);
}

export function LobbyCardSkeleton() {
	return (
		<div className="bg-card w-full space-y-4 rounded-3xl border p-4 sm:space-y-6 sm:p-6 lg:p-8">
			<div className="space-y-3 sm:space-y-4">
				<div className="flex items-center justify-between gap-2">
					<div className="min-w-0 space-y-2">
						<Skeleton className="h-5 w-40 sm:h-6 lg:h-7 lg:w-60" />
						<Skeleton className="h-3.5 w-32 sm:h-4 lg:w-48" />
					</div>
					<Skeleton className="h-7 w-20 rounded-full sm:h-8 lg:h-9 lg:w-24" />
				</div>
				<div className="flex items-center gap-3 sm:gap-4">
					<Skeleton className="h-4 w-16 sm:h-5 lg:w-20" />
					<Skeleton className="h-4 w-12 sm:h-5 lg:w-16" />
					<Skeleton className="h-4 w-20 sm:h-5 lg:w-24" />
				</div>
				<Skeleton className="h-30 w-full rounded-3xl lg:h-45" />
				<div className="space-y-2">
					<Skeleton className="h-3.5 w-full sm:h-4" />
					<Skeleton className="h-3.5 w-3/4 sm:h-4" />
				</div>
			</div>
			<Skeleton className="h-9 w-full rounded-full sm:h-10 lg:h-12" />
		</div>
	);
}
