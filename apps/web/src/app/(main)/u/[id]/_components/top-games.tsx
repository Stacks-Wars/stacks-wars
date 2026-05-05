import type { UserTopGame } from "@/lib/definitions/player-stats";
import { formatAmount } from "@/lib/utils";
import Image from "next/image";
import {
	Trophy,
	Target,
	Swords,
	TrendingUp,
	TrendingDown,
	Minus,
	Gamepad2,
} from "lucide-react";

interface TopGamesProps {
	topGames: UserTopGame[];
}

export default function TopGames({ topGames }: TopGamesProps) {
	if (!topGames || topGames.length === 0) return null;

	return (
		<div className="mt-8 space-y-4 px-4 sm:mt-12 sm:px-0">
			<div className="flex items-center gap-2">
				<Gamepad2 className="size-6 text-purple-500 sm:size-8" />
				<h2 className="text-xl font-bold sm:text-3xl">
					Favourite Games
				</h2>
			</div>
			<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
				{topGames.map((game) => (
					<div
						key={game.gameId}
						className="group relative overflow-hidden rounded-xl border border-white/10 bg-white/5 transition-all hover:border-white/20 hover:bg-white/[0.07]"
					>
						{/* Game Header */}
						<div className="flex items-center gap-3 border-b border-white/5 p-4">
							<Image
								src={game.gameImageUrl}
								alt={game.gameName}
								width={48}
								height={48}
								className="size-12 rounded-lg object-cover"
							/>
							<div className="min-w-0 flex-1">
								<h3 className="truncate text-sm font-bold sm:text-base">
									{game.gameName}
								</h3>
								<p className="text-xs text-gray-500">
									{game.totalMatches}{" "}
									{game.totalMatches === 1
										? "match"
										: "matches"}{" "}
									played
								</p>
							</div>
							<div className="flex flex-col items-end">
								<span className="text-xs text-gray-500">
									Wars Pts
								</span>
								<span className="text-sm font-bold text-yellow-500">
									{game.points.toFixed(1)}
								</span>
							</div>
						</div>

						{/* Stats Grid */}
						<div className="grid grid-cols-2 gap-3 p-4">
							<div className="flex items-center gap-2">
								<Swords className="size-4 text-blue-500" />
								<div>
									<p className="text-[10px] text-gray-500 uppercase">
										Matches
									</p>
									<p className="text-sm font-semibold">
										{game.totalMatches}
									</p>
								</div>
							</div>
							<div className="flex items-center gap-2">
								<Trophy className="size-4 text-purple-500" />
								<div>
									<p className="text-[10px] text-gray-500 uppercase">
										Wins
									</p>
									<p className="text-sm font-semibold">
										{game.totalWins}
									</p>
								</div>
							</div>
							<div className="flex items-center gap-2">
								<Target className="size-4 text-green-500" />
								<div>
									<p className="text-[10px] text-gray-500 uppercase">
										Win Rate
									</p>
									<p className="text-sm font-semibold text-green-500">
										{game.winRate.toFixed(1)}%
									</p>
								</div>
							</div>
							<div className="flex items-center gap-2">
								{game.totalPnl > 0 ? (
									<TrendingUp className="size-4 text-green-500" />
								) : game.totalPnl < 0 ? (
									<TrendingDown className="size-4 text-red-500" />
								) : (
									<Minus className="size-4 text-gray-500" />
								)}
								<div>
									<p className="text-[10px] text-gray-500 uppercase">
										P&L
									</p>
									<p
										className={`text-sm font-semibold ${
											game.totalPnl > 0
												? "text-green-500"
												: game.totalPnl < 0
													? "text-red-500"
													: "text-gray-500"
										}`}
									>
										{game.totalPnl > 0 ? "+" : ""}$
										{formatAmount(game.totalPnl)} STX
									</p>
								</div>
							</div>
						</div>
					</div>
				))}
			</div>
		</div>
	);
}
