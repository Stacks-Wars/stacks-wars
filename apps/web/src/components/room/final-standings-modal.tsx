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
import { formatAddress } from "@/lib/utils";
import { useRouter } from "next/navigation";
import { IoStar } from "react-icons/io5";
import { Sparkles, Trophy } from "lucide-react";
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
	const router = useRouter();

	// Only show when we have standings AND gameOver modal is closed
	const isOpen = !!finalStandings && !gameOverData;

	const handleClose = () => {
		lobbyActions.setFinalStandings(null);
		router.push("/lobby");
	};

	if (!finalStandings) return null;

	// Sort standings by rank
	const sortedStandings = [...finalStandings].sort(
		(a, b) => (a.rank ?? Infinity) - (b.rank ?? Infinity)
	);

	return (
		<Dialog open={isOpen} onOpenChange={(open) => !open && handleClose()}>
			<DialogContent className="sm:max-w-lg max-h-[80vh] overflow-hidden flex flex-col">
				<DialogHeader>
					<DialogTitle className="text-2xl font-bold text-center">
						Final Standings
					</DialogTitle>
					<DialogDescription className="text-center">
						Game results for all players
					</DialogDescription>
				</DialogHeader>

				<div className="flex-1 overflow-y-auto py-2 space-y-2">
					{sortedStandings.map((player) => {
						const rank = player.rank ?? 0;
						const isTopThree = rank >= 1 && rank <= 3;

						return (
							<Link
								key={player.userId}
								href={`/u/${player.username || player.walletAddress}`}
								className={cn(
									"flex items-center gap-3 rounded-xl p-3 border transition-colors hover:bg-accent/50",
									isTopThree ? rankColors[rank] : "bg-card"
								)}
							>
								{/* Rank */}
								<div className="flex size-8 items-center justify-center shrink-0">
									{isTopThree ? (
										<Trophy
											className={cn(
												"size-6",
												trophyColors[rank]
											)}
											strokeWidth={1.5}
										/>
									) : (
										<span className="text-lg font-bold text-muted-foreground">
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
								<div className="flex-1 min-w-0">
									{player.displayName ? (
										<>
											<p className="font-medium truncate">
												{player.displayName}
											</p>
											<p className="text-sm text-muted-foreground truncate">
												@
												{player.username ||
													formatAddress(
														player.walletAddress
													)}
											</p>
										</>
									) : (
										<p className="font-medium truncate">
											{player.username ||
												formatAddress(
													player.walletAddress
												)}
										</p>
									)}
								</div>

								{/* Stats */}
								<div className="flex items-center gap-3 shrink-0">
									{/* Trust Rating */}
									<div className="flex items-center gap-1 text-sm">
										<span>{player.trustRating}</span>
										<IoStar className="size-4 text-yellow-400" />
									</div>

									{/* Prize (if any) */}
									{player.prize != null &&
										player.prize > 0 && (
											<span className="text-sm font-medium text-green-500">
												+{player.prize.toFixed(2)}
											</span>
										)}
								</div>
							</Link>
						);
					})}
				</div>

				<div className="pt-4 border-t">
					<Button onClick={handleClose} className="w-full">
						Back to Lobby
					</Button>
				</div>
			</DialogContent>
		</Dialog>
	);
}
