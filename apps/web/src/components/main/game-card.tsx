"use client";

import Image from "next/image";
import { Button } from "../ui/button";
import Link from "next/link";
import type { Game } from "@/lib/definitions";
import { useIsActionLoading } from "@/lib/stores/room";
import { Loader2 } from "lucide-react";

export default function GameCard({
	game,
	action,
	onAction,
	isInLobby,
	isPrivate,
	isJoinRequestPending,
	isJoinRequestAccepted,
	isAuthenticated,
}: {
	game: Game;
	action?: "gamePage" | "createLobbyPage" | "joinLobby";
	onAction?: () => void;
	isInLobby?: boolean;
	isPrivate?: boolean;
	isJoinRequestPending?: boolean;
	isJoinRequestAccepted?: boolean;
	isAuthenticated?: boolean;
}) {
	// Get loading states from store
	const isJoinLoading = useIsActionLoading("join");
	const isLeaveLoading = useIsActionLoading("leave");
	const isJoinRequestLoading = useIsActionLoading("joinRequest");

	return (
		<div className="flex flex-col items-center w-full">
			<div className="flex flex-col-reverse sm:flex-row justify-between sm:items-center bg-card border p-4 sm:p-6 lg:p-8 rounded-3xl w-full">
				<div className="sm:w-1/2 space-y-2 lg:space-y-4">
					<h3 className="lg:text-[40px] text-2xl w-full truncate font-bold">
						{game.name}
					</h3>
					<p className="text-base line-clamp-2 lg:text-2xl sm:font-medium">
						{game.description}
					</p>
					<div className="flex gap-3 w-full overflow-hidden">
						{game.category && (
							<span className="text-xs lg:text-sm bg-foreground/10 rounded-full py-2 md:py-2.5 px-4 md:px-5 md:font-medium">
								{game.category}
							</span>
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
					className="max-w-40 md:max-w-89.5 w-full self-center"
				/>
			</div>
			{action && (
				<Button
					className="-translate-y-1/2 w-full max-w-48 sm:max-w-52 lg:max-w-80 rounded-full text-sm sm:text-base lg:text-xl font-medium -mb-4 sm:-mb-6 lg:-mb-8 py-3 sm:py-3.5 lg:py-4 h-8 sm:h-12 lg:h-16 shadow-sm"
					variant={isInLobby ? "destructive" : "default"}
					asChild={action !== "joinLobby" || !isAuthenticated}
					onClick={
						action === "joinLobby" && isAuthenticated
							? onAction
							: undefined
					}
					disabled={
						action === "joinLobby" &&
						isAuthenticated &&
						(isJoinRequestPending ||
							isJoinLoading ||
							isLeaveLoading ||
							isJoinRequestLoading)
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
							<Link href="/login">Login to Join Lobby</Link>
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
