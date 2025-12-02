import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatNumber } from "@/lib/utils";
import { LeaderBoard } from "@/types/schema/leaderboard";
import { Trophy, Target, TrendingUp, TrendingDown, Minus } from "lucide-react";

interface ProfileStatsProps {
	profile: LeaderBoard;
}

function formatPnL(pnl: number): string {
	const sign = pnl > 0 ? "+" : "";
	return `${sign}${formatNumber(pnl)} STX`;
}

function getPnLIcon(pnl: number) {
	if (pnl > 0) return <TrendingUp className="h-4 w-4" />;
	if (pnl < 0) return <TrendingDown className="h-4 w-4" />;
	return <Minus className="h-4 w-4" />;
}

function getPnLColor(pnl: number): string {
	if (pnl > 0) return "text-green-600 dark:text-green-400";
	if (pnl < 0) return "text-red-600 dark:text-red-400";
	return "text-muted-foreground";
}

function getRankIcon(rank: number) {
	switch (rank) {
		case 1:
			return "🥇";
		case 2:
			return "🥈";
		case 3:
			return "🥉";
		default:
			return;
	}
}

export default function ProfileStats({ profile }: ProfileStatsProps) {
	const lossCount = profile.totalMatch - profile.totalWins;

	return (
		<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">
						Leaderboard Rank
					</CardTitle>
					<Trophy className="h-4 w-4 text-yellow-500" />
				</CardHeader>
				<CardContent>
					<div className="text-2xl font-bold flex items-center gap-2">
						<span>#{profile.rank}</span>
						<span>{getRankIcon(profile.rank)}</span>
					</div>
					<p className="text-xs text-muted-foreground">
						Global ranking
					</p>
				</CardContent>
			</Card>

			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">
						Win Rate
					</CardTitle>
					<Target className="h-4 w-4 text-primary" />
				</CardHeader>
				<CardContent>
					<div className="text-2xl font-bold">
						{profile.winRate.toFixed(1)}%
					</div>
					<p className="text-xs text-muted-foreground">
						{profile.totalWins}W - {lossCount}L
					</p>
				</CardContent>
			</Card>

			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">
						Total Matches
					</CardTitle>
					<div className="h-4 w-4 rounded-full bg-primary/20" />
				</CardHeader>
				<CardContent>
					<div className="text-2xl font-bold">
						{profile.totalMatch}
					</div>
					<p className="text-xs text-muted-foreground">
						Games played
					</p>
				</CardContent>
			</Card>

			<Card>
				<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle className="text-sm font-medium">
						Profit & Loss
					</CardTitle>
					{getPnLIcon(profile.pnl)}
				</CardHeader>
				<CardContent>
					<div
						className={`text-2xl font-bold ${getPnLColor(profile.pnl)}`}
					>
						{formatPnL(profile.pnl)}
					</div>
					<p className="text-xs text-muted-foreground">
						Net earnings
					</p>
				</CardContent>
			</Card>
		</div>
	);
}
