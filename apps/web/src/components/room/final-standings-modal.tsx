"use client";

import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
	useFinalStandings,
	useGameOverData,
	useLobbyActions,
} from "@/lib/stores/room";
import { formatAddress, formatAmount } from "@/lib/utils";
import { IoStar } from "react-icons/io5";
import { Trophy } from "lucide-react";
import { cn } from "@/lib/utils";
import Link from "next/link";

const rankColors: Record<number, string> = {
	1: "bg-yellow-500/10 border-yellow-500/50",
	2: "bg-gray-400/10 border-gray-400/50",
	3: "bg-amber-600/10 border-amber-600/50",
};

const trophyColors: Record<number, string> = {
	1: "text-yellow-500",
	2: "text-gray-400",
	3: "text-amber-600",
};

export default function FinalStandingsModal() {
	const finalStandings = useFinalStandings();
	const gameOverData = useGameOverData();
	const lobbyActions = useLobbyActions();

	// Only show when we have standings AND gameOver modal is closed
	const isOpen = !!finalStandings && !gameOverData;

	const handleClose = () => {
		lobbyActions.setFinalStandings(null);
	};

	if (!finalStandings) return null;

	// Sort standings by rank
	const sortedStandings = [...finalStandings].sort(
		(a, b) => (a.rank ?? Infinity) - (b.rank ?? Infinity)
	);

	return (
		<Dialog open={isOpen} onOpenChange={(open) => !open && handleClose()}>
			<DialogContent className="flex max-h-[80vh] flex-col overflow-hidden sm:max-w-lg">
				<DialogHeader>
					<DialogTitle className="text-center text-2xl font-bold">
						Final Standings
					</DialogTitle>
					<DialogDescription className="text-center">
						Game results for all players
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 space-y-2 overflow-y-auto py-2">
					{sortedStandings.map((player) => {
						const rank = player.rank ?? 0;
						const isTopThree = rank >= 1 && rank <= 3;

						return (
							<Link
								key={player.userId}
								href={`/u/${player.username || player.walletAddress}`}
								className={cn(
									"hover:bg-accent/50 flex items-center gap-3 rounded-xl border p-3 transition-colors",
									isTopThree ? rankColors[rank] : "bg-card"
								)}
							>
								{/* Rank */}
								<div className="flex size-8 shrink-0 items-center justify-center">
									{isTopThree ? (
										<Trophy
											className={cn(
												"size-6",
												trophyColors[rank]
											)}
											strokeWidth={1.5}
										/>
									) : (
										<span className="text-muted-foreground text-lg font-bold">
											#{rank}
										</span>
									)}
								</div>

								{/* Avatar */}
								<Avatar className="size-10 shrink-0 uppercase">
									<AvatarImage src="" alt="player profile" />
									<AvatarFallback>
										{(
											player.displayName ||
											player.username ||
											player.walletAddress
										).slice(0, 2)}
									</AvatarFallback>
								</Avatar>

								{/* Player Info */}
								<div className="min-w-0 flex-1">
									{player.displayName ? (
										<>
											<p className="truncate font-medium">
												{player.displayName}
											</p>
											<p className="text-muted-foreground truncate text-sm">
												@
												{player.username ||
													formatAddress(
														player.walletAddress
													)}
											</p>
										</>
									) : (
										<p className="truncate font-medium">
											{player.username ||
												formatAddress(
													player.walletAddress
												)}
										</p>
									)}
								</div>

								{/* Stats */}
								<div className="flex shrink-0 items-center gap-3">
									{/* Trust Rating */}
									<div className="flex items-center gap-1 text-sm">
										<span>{player.trustRating}</span>
										<IoStar className="size-4 text-yellow-400" />
									</div>

									{/* Prize (if any) */}
									{player.prize != null &&
										player.prize > 0 && (
											<span className="text-sm font-medium text-green-500">
												+{formatAmount(player.prize)}
											</span>
										)}
								</div>
							</Link>
						);
					})}
				</div>

				<div className="border-t pt-4">
					<Button onClick={handleClose} className="w-full">
						Close
					</Button>
				</div>
			</DialogContent>
		</Dialog>
	);
}
