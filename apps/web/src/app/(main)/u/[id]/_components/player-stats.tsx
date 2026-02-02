import type { LeaderBoard } from "@/lib/definitions";
import { formatAmount } from "@/lib/utils";
import {
	TrendingUp,
	TrendingDown,
	Minus,
	Trophy,
	Target,
	Swords,
	Coins,
} from "lucide-react";

interface PlayerStatsProps {
	stats: LeaderBoard;
}

export default function PlayerStats({ stats }: PlayerStatsProps) {
	const statItems = [
		{
			label: "Wars Points",
			value: stats.points.toFixed(1),
			icon: Trophy,
			color: "text-yellow-500",
		},
		{
			label: "Win Rate",
			value: `${stats.winRate.toFixed(1)}%`,
			icon: Target,
			color: "text-green-500",
		},
		{
			label: "Total Matches",
			value: stats.totalMatches.toString(),
			icon: Swords,
			color: "text-blue-500",
		},
		{
			label: "Total Wins",
			value: stats.totalWins.toString(),
			icon: Trophy,
			color: "text-purple-500",
		},
		{
			label: "P&L",
			value: `${stats.totalPnl > 0 ? "+" : ""}$${formatAmount(stats.totalPnl)} STX`,
			icon:
				stats.totalPnl > 0
					? TrendingUp
					: stats.totalPnl < 0
						? TrendingDown
						: Minus,
			color:
				stats.totalPnl > 0
					? "text-green-500"
					: stats.totalPnl < 0
						? "text-red-500"
						: "text-gray-500",
		},
	];

	return (
		<div className="space-y-4">
			<h2 className="text-xl font-bold sm:text-3xl">Player Stats</h2>
			<div className="grid grid-cols-2 gap-4 sm:grid-cols-3 sm:gap-6">
				{statItems.map((item) => {
					const Icon = item.icon;
					return (
						<div
							key={item.label}
							className="flex flex-col items-center rounded-lg border border-white/10 bg-white/5 p-4 text-center sm:p-6"
						>
							<Icon
								className={`mb-2 size-6 sm:size-8 ${item.color}`}
							/>
							<p className="text-xs font-medium text-gray-400 uppercase sm:text-sm">
								{item.label}
							</p>
							<p
								className={`text-lg font-bold sm:text-xl ${item.color}`}
							>
								{item.value}
							</p>
						</div>
					);
				})}
			</div>
		</div>
	);
}
