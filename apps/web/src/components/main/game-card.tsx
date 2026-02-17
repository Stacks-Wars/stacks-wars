"use client";

import Image from "next/image";
import { Button } from "../ui/button";
import Link from "next/link";
import type { Game } from "@/lib/definitions";
import { useIsActionLoading } from "@/lib/stores/room";
import { useUser } from "@/lib/stores/user";
import { Loader2 } from "lucide-react";
import { usePathname } from "next/navigation";

export default function GameCard({
	game,
	action,
	onAction,
	isInLobby,
	isPrivate,
	isJoinRequestPending,
	isJoinRequestAccepted,
	isAuthenticated,
	lobbyStatus,
	playerStatus,
}: {
	game: Game;
	action?: "gamePage" | "createLobbyPage" | "joinLobby";
	onAction?: () => void;
	isInLobby?: boolean;
	isPrivate?: boolean;
	isJoinRequestPending?: boolean;
	isJoinRequestAccepted?: boolean;
	isAuthenticated?: boolean;
	lobbyStatus?: string;
	playerStatus?: string;
}) {
	// Get loading states from store
	const user = useUser();
	const pathname = usePathname();
	const isJoinLoading = useIsActionLoading(`join-${user?.id}`);
	const isLeaveLoading = useIsActionLoading(`leave-${user?.id}`);
	const isJoinRequestLoading = useIsActionLoading("joinRequest");

	return (
		<div className="flex w-full flex-col items-center">
			<div className="bg-gradient-primary flex w-full flex-col-reverse justify-between rounded-3xl border p-4 sm:flex-row sm:items-center sm:p-6 lg:p-8">
				<div className="space-y-2 sm:w-1/2 lg:space-y-4">
					<h3 className="w-full truncate text-2xl font-bold lg:text-[40px]">
						{game.name}
					</h3>
					<p className="line-clamp-2 text-base sm:font-medium lg:text-2xl">
						{game.description}
					</p>
					<div className="flex w-full flex-wrap gap-3 overflow-hidden">
						{game.category && game.category.length > 0 && (
							<>
								{game.category.slice(0, 2).map((cat) => (
									<span
										key={cat}
										className="bg-foreground/10 rounded-full px-4 py-2 text-xs md:px-5 md:py-2.5 md:font-medium lg:text-sm"
									>
										{cat}
									</span>
								))}
								{game.category.length > 2 && (
									<span className="bg-foreground/10 rounded-full px-4 py-2 text-xs md:px-5 md:py-2.5 md:font-medium lg:text-sm">
										+{game.category.length - 2}
									</span>
								)}
							</>
						)}
					</div>
					{/*<div className="text-sm lg:text-xl lg:font-medium flex gap-4 w-full truncate">
						<p>
							<span className="font-medium ">Active Rooms:</span>{" "}
							<span>3</span>
						</p>
						<p className="flex items-center gap-1">
							<span className="font-medium">Ratings:</span>{" "}
							<span>4.5</span>
							<IoStar className="text-yellow-400" />
						</p>
						<p>
							<span className="font-medium">Volume:</span>{" "}
							<span>1K STX</span>{" "}
						</p>
					</div>*/}
				</div>
				<Image
					src={game.imageUrl}
					alt="game logo"
					width={358}
					height={182}
					loading="lazy"
					className="w-full max-w-40 self-center md:max-w-89.5"
				/>
			</div>
			{action && (
				<Button
					className="bg-muted-gradient -mb-4 h-8 w-full max-w-48 -translate-y-1/2 rounded-full py-3 text-sm font-medium shadow-sm transition hover:opacity-90 sm:-mb-6 sm:h-12 sm:max-w-52 sm:py-3.5 sm:text-base lg:-mb-8 lg:h-16 lg:max-w-80 lg:py-4 lg:text-xl"
					variant={isInLobby ? "destructive" : "default"}
					asChild={action !== "joinLobby" || !isAuthenticated}
					onClick={
						action === "joinLobby" && isAuthenticated
							? onAction
							: undefined
					}
					disabled={
						(action === "joinLobby" &&
							isAuthenticated &&
							(isJoinRequestPending ||
								isJoinLoading ||
								isLeaveLoading ||
								isJoinRequestLoading)) ||
						(action === "joinLobby" &&
							lobbyStatus === "inProgress") ||
						(action === "joinLobby" &&
							lobbyStatus === "finished" &&
							(!isInLobby || playerStatus !== "notJoined")) ||
						(action === "joinLobby" &&
							lobbyStatus === "starting")
					}
				>
					{action === "createLobbyPage" ? (
						<Link href={`/games/${game.path}`}>Play Now</Link>
					) : action === "gamePage" ? (
						<Link href={{ pathname: `/game/${game.path}` }}>
							View Game
						</Link>
					) : action === "joinLobby" ? (
						!isAuthenticated ? (
							<Link href={`/login?redirect=${encodeURIComponent(pathname)}`}>Login to Join Lobby</Link>
						) : isJoinLoading ? (
							<span className="flex items-center gap-2">
								<Loader2 className="size-4 animate-spin" />
								Joining...
							</span>
						) : isLeaveLoading ? (
							<span className="flex items-center gap-2">
								<Loader2 className="size-4 animate-spin" />
								Leaving...
							</span>
						) : isJoinRequestLoading ? (
							<span className="flex items-center gap-2">
								<Loader2 className="size-4 animate-spin" />
								Requesting...
							</span>
						) : isInLobby ? (
							<span>Leave Lobby</span>
						) : isJoinRequestPending ? (
							<span>Request Pending</span>
						) : isJoinRequestAccepted ? (
							<span>Join Lobby</span>
						) : isPrivate ? (
							<span>Request to Join Lobby</span>
						) : (
							<span>Join Lobby</span>
						)
					) : null}
				</Button>
			)}
		</div>
	);
}
