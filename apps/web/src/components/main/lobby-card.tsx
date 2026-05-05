import type { LobbyInfo } from "@/lib/definitions";
import { Button } from "../ui/button";
import Image from "next/image";
import { Badge } from "../ui/badge";
import { Lock, LockOpen, Users, Trophy } from "lucide-react";
import { BiCoinStack } from "react-icons/bi";
import Link from "next/link";
import { displayUserIdentifier } from "@/lib/utils";
import { Skeleton } from "../ui/skeleton";
import { useEffect, useState } from "react";

/** Format "last seen" from ms ago */
function formatLastSeen(ms: number): string {
	const seconds = Math.floor(ms / 1000);
	if (seconds < 60) return `${seconds}s ago`;
	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h ago`;
	return `${Math.floor(hours / 24)}d ago`;
}

/** Creator activity for lobby card: active if lastPing within 10s */
function CreatorActivity({ lastPing }: { lastPing?: number }) {
	const [now, setNow] = useState(Date.now());
	useEffect(() => {
		const t = setInterval(() => setNow(Date.now()), 5000);
		return () => clearInterval(t);
	}, []);
	if (!lastPing) {
		return (
			<span className="text-muted-foreground text-[10px] sm:text-xs">
				Offline
			</span>
		);
	}
	const elapsed = now - lastPing;
	const isActive = elapsed <= 10_000;
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

interface LobbyCardProps {
	lobbyInfo: LobbyInfo;
	buttonText?: string;
	prize?: number;
}

export default function LobbyCard({
	lobbyInfo,
	buttonText,
	prize,
}: LobbyCardProps) {
	const { lobby, game, creator } = lobbyInfo;

	return (
		<div className="bg-gradient-primary-2 w-full space-y-4 rounded-3xl border p-4 sm:space-y-6 sm:p-6 lg:p-8">
			<div className="space-y-3 sm:space-y-4">
				<div className="flex items-center justify-between gap-2">
					<div className="min-w-0 flex-1">
						<p className="truncate text-base font-semibold sm:text-lg lg:text-xl">
							{lobby.name}
						</p>
						<div className="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs sm:text-sm">
							<Link
								href={`/u/${creator.username || creator.walletAddress}`}
								className="text-muted-foreground truncate"
							>
								Creator -{" "}
								<span className="text-foreground">
									@{displayUserIdentifier(creator)}
								</span>
							</Link>
							<CreatorActivity lastPing={lobby.creatorLastPing} />
						</div>
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
					{prize && (
						<p className="flex items-center gap-1.5">
							<Trophy className="size-4 lg:size-5" />
							<span>
								Prize: {prize} {lobby.tokenSymbol}
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
				<Link href={`/room/${lobby.path}`}>
					{buttonText || "Open Room"}
				</Link>
			</Button>
		</div>
	);
}

export function LobbyCardSkeleton() {
	return (
		<div className="bg-card w-full space-y-4 rounded-3xl border p-4 sm:space-y-6 sm:p-6 lg:p-8">
			<div className="space-y-3 sm:space-y-4">
				<div className="flex items-center justify-between gap-2">
					<div className="min-w-0 flex-1 space-y-2">
						<Skeleton className="h-5 w-40 sm:h-6 lg:h-7 lg:w-60" />
						<div className="flex flex-wrap items-center gap-x-2 gap-y-0.5">
							<Skeleton className="h-3.5 w-32 sm:h-4 lg:w-48" />
							<Skeleton className="h-3 w-12 sm:h-3.5 sm:w-14" />
						</div>
					</div>
					<Skeleton className="h-7 w-20 shrink-0 rounded-full sm:h-8 lg:h-9 lg:w-24" />
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
