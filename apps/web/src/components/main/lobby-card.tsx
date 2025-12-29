import type { LobbyExtended } from "@/lib/definitions";
import { Button } from "../ui/button";
import Image from "next/image";
import { Badge } from "../ui/badge";
import { Lock, LockOpen, Users } from "lucide-react";
import { BiCoinStack } from "react-icons/bi";
import Link from "next/link";
import { formatAddress } from "@/lib/utils";
import { Skeleton } from "../ui/skeleton";

export default function LobbyCard({ lobby }: { lobby: LobbyExtended }) {
	return (
		<div className="bg-card p-6 lg:p-12 rounded-4xl border max-w-150 w-full space-y-4 lg:space-y-8">
			<div className="space-y-4 lg:space-y-6.5">
				<div className="flex justify-between items-center">
					<div className="max-w-50 lg:max-w-100">
						<p className="truncate text-xl lg:text-2xl font-medium">
							{lobby.name}
						</p>
						<Link
							href={`/u/${lobby.creatorUsername || lobby.creatorWalletAddress}`}
							className="truncate text-xs lg:text-lg text-foreground/50"
						>
							Creator -{" "}
							<span className="text-foreground">
								@
								{lobby.creatorDisplayName ||
									lobby.creatorUsername ||
									formatAddress(lobby.creatorWalletAddress)}
							</span>
						</Link>
					</div>
					<Badge className="py-2.5 lg:py-3.5 px-3.5 lg:px-6.5 text-xs lg:text-sm font-medium">
						{lobby.status}
					</Badge>
				</div>
				<div className="text-xs lg:text-base flex items-center justify-center gap-3 lg:gap-5">
					{lobby.isPrivate ? (
						<p className="flex items-center gap-1 lg:gap-2.5">
							<Lock size={20} className="size-4 lg:size-5" />{" "}
							<span>Private</span>
						</p>
					) : (
						<p className="flex items-center gap-1 lg:gap-2.5">
							<LockOpen size={20} className="size-4 lg:size-5" />
							<span>Public</span>
						</p>
					)}
					<p className="flex items-center gap-1 lg:gap-2.5">
						<Users size={20} className="size-4 lg:size-5" />
						<span>{lobby.participantCount}</span>
					</p>
					{lobby.currentAmount && (
						<p className="flex items-center gap-1 lg:gap-2.5">
							<BiCoinStack
								size={20}
								className="size-4 lg:size-5"
							/>
							<span>
								{lobby.currentAmount} {lobby.tokenSymbol}
							</span>
						</p>
					)}
					{lobby.entryAmount && (
						<p>
							Entry Fee: {lobby.entryAmount} {lobby.tokenSymbol}
						</p>
					)}
				</div>
				<Image
					src={lobby.gameImageUrl}
					alt="game-cover"
					width={516}
					height={185}
					loading="lazy"
					className="w-full h-30 lg:h-45 rounded-3xl"
				/>
				<p className="text-sm lg:text-xl line-clamp-2">
					{lobby.description}
				</p>
			</div>
			<Button
				asChild
				variant={"secondary"}
				className="rounded-full w-full text-base lg:text-xl font-medium py-2.5 lg:py-6"
			>
				<Link href={`/room/${lobby.path}`}>Open Room</Link>
			</Button>
		</div>
	);
}

export function LobbyCardSkeleton() {
	return (
		<div className="bg-card p-6 lg:p-12 rounded-4xl border max-w-150 w-full space-y-4 lg:space-y-8">
			<div className="space-y-4 lg:space-y-6.5">
				<div className="flex justify-between items-center">
					<div className="max-w-50 lg:max-w-100 space-y-2">
						<Skeleton className="h-6 lg:h-8 w-40 lg:w-60" />
						<Skeleton className="h-4 lg:h-5 w-32 lg:w-48" />
					</div>
					<Skeleton className="h-8 lg:h-10 w-20 lg:w-24 rounded-full" />
				</div>
				<div className="flex items-center justify-center gap-3 lg:gap-5">
					<Skeleton className="h-5 lg:h-6 w-16 lg:w-20" />
					<Skeleton className="h-5 lg:h-6 w-12 lg:w-16" />
					<Skeleton className="h-5 lg:h-6 w-20 lg:w-24" />
				</div>
				<Skeleton className="w-full h-30 lg:h-45 rounded-3xl" />
				<div className="space-y-2">
					<Skeleton className="h-4 lg:h-5 w-full" />
					<Skeleton className="h-4 lg:h-5 w-3/4" />
				</div>
			</div>
			<Skeleton className="h-10 lg:h-14 w-full rounded-full" />
		</div>
	);
}
