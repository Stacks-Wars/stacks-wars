import type { LobbyInfo } from "@/lib/definitions";
import { Button } from "../ui/button";
import Image from "next/image";
import { Badge } from "../ui/badge";
import { Lock, LockOpen, Users } from "lucide-react";
import { BiCoinStack } from "react-icons/bi";
import Link from "next/link";
import { formatAddress } from "@/lib/utils";
import { Skeleton } from "../ui/skeleton";

export default function LobbyCard({ lobbyInfo }: { lobbyInfo: LobbyInfo }) {
	const { lobby, game, creator } = lobbyInfo;

	return (
		<div className="bg-card border p-4 sm:p-6 lg:p-8 rounded-3xl w-full space-y-4 sm:space-y-6">
			<div className="space-y-3 sm:space-y-4">
				<div className="flex items-center justify-between gap-2">
					<div className="min-w-0">
						<p className="truncate text-base sm:text-lg lg:text-xl font-semibold">
							{lobby.name}
						</p>
						<Link
							href={`/u/${creator.username || creator.walletAddress}`}
							className="truncate text-xs sm:text-sm text-muted-foreground"
						>
							Creator -{" "}
							<span className="text-foreground">
								@
								{creator.displayName ||
									creator.username ||
									formatAddress(creator.walletAddress)}
							</span>
						</Link>
					</div>
					<Badge className="py-1.5 sm:py-2 px-2.5 sm:px-3.5 text-xs sm:text-sm font-medium">
						{lobby.status}
					</Badge>
				</div>

				<div className="text-xs sm:text-sm lg:text-base flex items-center gap-3 sm:gap-4">
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
					className="w-full h-30 lg:h-45 rounded-3xl"
				/>

				{lobby.description && (
					<p className="text-xs sm:text-sm lg:text-base line-clamp-2">
						{lobby.description}
					</p>
				)}
			</div>

			<Button
				asChild
				variant={"secondary"}
				className="rounded-full w-full text-sm sm:text-base lg:text-lg font-medium py-2.5 sm:py-3.5 lg:py-4"
			>
				<Link href={`/room/${lobby.path}`}>Open Room</Link>
			</Button>
		</div>
	);
}

export function LobbyCardSkeleton() {
	return (
		<div className="bg-card border p-4 sm:p-6 lg:p-8 rounded-3xl w-full space-y-4 sm:space-y-6">
			<div className="space-y-3 sm:space-y-4">
				<div className="flex items-center justify-between gap-2">
					<div className="min-w-0 space-y-2">
						<Skeleton className="h-5 sm:h-6 lg:h-7 w-40 lg:w-60" />
						<Skeleton className="h-3.5 sm:h-4 w-32 lg:w-48" />
					</div>
					<Skeleton className="h-7 sm:h-8 lg:h-9 w-20 lg:w-24 rounded-full" />
				</div>
				<div className="flex items-center gap-3 sm:gap-4">
					<Skeleton className="h-4 sm:h-5 w-16 lg:w-20" />
					<Skeleton className="h-4 sm:h-5 w-12 lg:w-16" />
					<Skeleton className="h-4 sm:h-5 w-20 lg:w-24" />
				</div>
				<Skeleton className="w-full h-30 lg:h-45 rounded-3xl" />
				<div className="space-y-2">
					<Skeleton className="h-3.5 sm:h-4 w-full" />
					<Skeleton className="h-3.5 sm:h-4 w-3/4" />
				</div>
			</div>
			<Skeleton className="h-9 sm:h-10 lg:h-12 w-full rounded-full" />
		</div>
	);
}
