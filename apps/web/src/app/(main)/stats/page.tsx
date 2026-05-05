import { ApiClient } from "@/lib/api/client";
import type { GameStats, PlatformStats, Season } from "@/lib/definitions";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import Image from "next/image";
import SeasonSelect from "./_components/season-select";

function formatNumber(n: number): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
	return n.toLocaleString();
}

function formatUSD(n: number): string {
	if (n >= 1_000_000) return `$${(n / 1_000_000).toFixed(2)}M`;
	if (n >= 1_000) return `$${(n / 1_000).toFixed(2)}K`;
	if (n < 0.01 && n > 0) return `$${n.toFixed(6)}`;
	return `$${n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function formatTokenAmount(n: number, symbol: string): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M ${symbol}`;
	if (n >= 1_000) return `${(n / 1_000).toFixed(2)}K ${symbol}`;
	return `${n.toLocaleString(undefined, { maximumFractionDigits: 2 })} ${symbol}`;
}

function formatPrice(n: number): string {
	if (n <= 0) return "—";
	if (n >= 1) return `$${n.toFixed(2)}`;
	if (n >= 0.01) return `$${n.toFixed(4)}`;
	if (n >= 0.0001) return `$${n.toFixed(6)}`;
	return `$${n.toExponential(2)}`;
}

interface StatsPageProps {
	searchParams?: Promise<{
		seasonId?: string;
	}>;
}

function parseSeasonId(value?: string): number | null {
	if (!value) return null;
	const parsed = Number(value);
	return Number.isFinite(parsed) ? parsed : null;
}

export default async function StatsPage({ searchParams }: StatsPageProps) {
	const resolvedSearchParams = await searchParams;
	const [overallRes, seasonsRes] = await Promise.all([
		ApiClient.get<PlatformStats>("/api/stats/"),
		ApiClient.get<Season[]>("/api/seasons?limit=50"),
	]);

	const seasons = seasonsRes.data || [];
	const overallStats = overallRes.data;

	if (!overallStats) {
		return (
			<div className="container mx-auto px-4 py-20 text-center">
				<h1 className="text-2xl font-bold">
					Unable to load platform data
				</h1>
				<p className="text-muted-foreground mt-2">
					Please try again later.
				</p>
			</div>
		);
	}

	const now = new Date();
	const currentSeason = seasons.find((season) => {
		const start = new Date(season.startDate);
		const end = new Date(season.endDate);
		return now >= start && now <= end;
	});
	const currentSeasonId = currentSeason?.id ?? seasons[0]?.id ?? null;
	const requestedSeasonId = parseSeasonId(resolvedSearchParams?.seasonId);
	const selectedSeasonId = requestedSeasonId ?? currentSeasonId;

	const [seasonStatsRes, seasonGameStatsRes] = await Promise.all([
		ApiClient.get<PlatformStats>(
			selectedSeasonId !== null
				? `/api/stats?seasonId=${selectedSeasonId}`
				: "/api/stats/"
		),
		ApiClient.get<GameStats[]>(
			selectedSeasonId !== null
				? `/api/leaderboards/stats/games?seasonId=${selectedSeasonId}`
				: "/api/leaderboards/stats/games"
		),
	]);

	const seasonStats = seasonStatsRes.data || overallStats;
	const seasonGameStats = seasonGameStatsRes.data || [];
	const seasonUtilizationRate =
		seasonStats.feeLobbies > 0
			? (
					(seasonStats.activeFeeLobbies / seasonStats.feeLobbies) *
					100
				).toFixed(1)
			: "0";
	const seasonName =
		selectedSeasonId === null
			? "Overall"
			: seasons.find((season) => season.id === selectedSeasonId)?.name ||
				"Selected Season";

	return (
		<div className="container mx-auto px-4 pb-16">
			<div className="py-8 text-center lg:py-12">
				<h1 className="mb-2 text-3xl font-bold tracking-tight md:text-5xl">
					Platform Analytics
				</h1>
				<p className="text-muted-foreground mx-auto max-w-xl text-sm md:text-lg">
					Simple overall platform metrics at the top, with
					season-specific details below.
				</p>
			</div>

			<section className="bg-card/50 rounded-3xl border p-4 md:p-6">
				<div className="mb-4 flex items-center justify-between gap-3">
					<div>
						<p className="text-muted-foreground text-xs tracking-[0.24em] uppercase">
							Overall
						</p>
						<h2 className="text-xl font-bold md:text-2xl">
							All Seasons
						</h2>
					</div>
					<Badge
						variant="secondary"
						className="rounded-full px-3 py-1"
					>
						Combined totals
					</Badge>
				</div>

				<div className="grid grid-cols-2 gap-3 md:grid-cols-4">
					{[
						{
							label: "Users",
							value: formatNumber(overallStats.totalUsers),
						},
						{
							label: "Lobbies",
							value: formatNumber(overallStats.totalLobbies),
						},
						{
							label: "Volume",
							value: formatUSD(overallStats.totalVolumeUsd),
						},
						{
							label: "Distributed",
							value: formatUSD(overallStats.finishedVolumeUsd),
						},
					].map((stat) => (
						<Card key={stat.label}>
							<CardHeader className="p-4 pb-2 md:p-6 md:pb-2">
								<CardDescription className="text-xs">
									{stat.label}
								</CardDescription>
								<CardTitle className="text-2xl md:text-3xl">
									{stat.value}
								</CardTitle>
							</CardHeader>
						</Card>
					))}
				</div>
			</section>

			<section className="mt-10 space-y-5">
				<div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
					<div>
						<p className="text-muted-foreground text-xs tracking-[0.24em] uppercase">
							Season Snapshot
						</p>
						<h2 className="text-xl font-bold md:text-2xl">
							{seasonName}
						</h2>
						<p className="text-muted-foreground mt-1 text-sm">
							Metrics scoped to the selected season window.
						</p>
					</div>
					{seasons.length > 0 && selectedSeasonId !== null ? (
						<SeasonSelect
							seasons={seasons}
							selectedSeasonId={selectedSeasonId}
						/>
					) : null}
				</div>

				<div className="grid grid-cols-2 gap-3 md:grid-cols-4">
					{[
						{
							label: "New Users",
							value: formatNumber(seasonStats.newUsersCount),
						},
						{
							label: "Lobbies",
							value: formatNumber(seasonStats.totalLobbies),
						},
						{
							label: "Entry-Fee Lobbies",
							value: formatNumber(seasonStats.feeLobbies),
						},
						{
							label: "Games Played",
							value: formatNumber(seasonStats.totalGames),
						},
					].map((stat) => (
						<Card key={stat.label}>
							<CardHeader className="p-4 pb-2 md:p-6 md:pb-2">
								<CardDescription className="text-xs">
									{stat.label}
								</CardDescription>
								<CardTitle className="text-2xl md:text-3xl">
									{stat.value}
								</CardTitle>
							</CardHeader>
						</Card>
					))}
				</div>

				<div className="grid gap-4 md:grid-cols-2">
					<Card className="border-primary/20 bg-primary/5">
						<CardHeader>
							<CardDescription>Season Volume</CardDescription>
							<CardTitle className="text-3xl md:text-4xl">
								{formatUSD(seasonStats.totalVolumeUsd)}
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p className="text-muted-foreground text-xs">
								Total volume for the selected season
							</p>
						</CardContent>
					</Card>

					<Card className="border-blue-500/20 bg-blue-500/5">
						<CardHeader>
							<CardDescription>
								Distributed Volume
							</CardDescription>
							<CardTitle className="text-3xl text-blue-600 md:text-4xl dark:text-blue-400">
								{formatUSD(seasonStats.finishedVolumeUsd)}
							</CardTitle>
						</CardHeader>
						<CardContent>
							<p className="text-muted-foreground text-xs">
								Settled winnings for the selected season
							</p>
						</CardContent>
					</Card>
				</div>

				<div>
					<div className="mb-5">
						<h3 className="text-lg font-bold md:text-xl">
							Token Breakdown
						</h3>
						<p className="text-muted-foreground text-sm">
							Season-specific volume distribution by token
						</p>
					</div>

					{seasonStats.tokenBreakdown.length > 0 ? (
						<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
							{seasonStats.tokenBreakdown.map((token) => {
								const share =
									seasonStats.totalVolumeUsd > 0
										? (token.volumeUsd /
												seasonStats.totalVolumeUsd) *
											100
										: 0;

								return (
									<Card
										key={token.symbol}
										className="overflow-hidden"
									>
										<CardHeader className="pb-3">
											<div className="flex items-center justify-between gap-3">
												<div className="flex items-center gap-3">
													{token.imageUrl ? (
														<img
															src={token.imageUrl}
															alt={token.symbol}
															className="h-10 w-10 rounded-full object-cover"
														/>
													) : (
														<div className="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full font-bold">
															{token.symbol.charAt(
																0
															)}
														</div>
													)}
													<div>
														<CardTitle className="text-lg">
															{token.symbol}
														</CardTitle>
														<p className="text-muted-foreground text-xs">
															{token.priceUsd > 0
																? `${formatPrice(token.priceUsd)} per token`
																: "Price unavailable"}
														</p>
													</div>
												</div>
												<Badge
													variant="secondary"
													className="text-xs"
												>
													{share.toFixed(1)}%
												</Badge>
											</div>
										</CardHeader>
										<CardContent className="space-y-4">
											<div className="grid grid-cols-2 gap-4">
												<div>
													<p className="text-muted-foreground text-xs">
														Volume
													</p>
													<p className="text-sm font-semibold">
														{formatTokenAmount(
															token.volume,
															token.symbol
														)}
													</p>
												</div>
												<div>
													<p className="text-muted-foreground text-xs">
														USD Value
													</p>
													<p className="text-sm font-semibold">
														{formatUSD(
															token.volumeUsd
														)}
													</p>
												</div>
											</div>

											<div className="flex items-center justify-between text-sm">
												<span className="text-muted-foreground">
													{token.lobbyCount}{" "}
													{token.lobbyCount === 1
														? "lobby"
														: "lobbies"}
												</span>
											</div>

											<div className="bg-muted h-1.5 overflow-hidden rounded-full">
												<div
													className="bg-primary h-full rounded-full transition-all"
													style={{
														width: `${Math.min(Math.max(share, 1), 100)}%`,
													}}
												/>
											</div>
										</CardContent>
									</Card>
								);
							})}
						</div>
					) : (
						<Card>
							<CardContent className="py-12 text-center">
								<p className="text-muted-foreground text-sm">
									No token activity yet
								</p>
							</CardContent>
						</Card>
					)}
				</div>

				<div>
					<Card>
						<CardHeader>
							<CardTitle>Paid Lobby Health</CardTitle>
							<CardDescription>
								Entry-fee lobby utilization for the selected
								season
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-5">
							<div>
								<div className="mb-2 flex items-center justify-between">
									<span className="text-muted-foreground text-sm">
										Entry-Fee Lobby Utilization
									</span>
									<span className="font-semibold">
										{seasonUtilizationRate}%
									</span>
								</div>
								<div className="bg-muted h-2 overflow-hidden rounded-full">
									<div
										className="bg-primary h-full rounded-full transition-all"
										style={{
											width: `${Math.min(Number(seasonUtilizationRate), 100)}%`,
										}}
									/>
								</div>
							</div>

							<div className="grid grid-cols-2 gap-4 pt-2 md:grid-cols-3">
								<div>
									<p className="text-muted-foreground text-xs">
										Entry-Fee Lobbies
									</p>
									<p className="text-lg font-semibold">
										{formatNumber(seasonStats.feeLobbies)}
									</p>
								</div>
								<div>
									<p className="text-muted-foreground text-xs">
										Active Entry-Fee Lobbies
									</p>
									<p className="text-lg font-semibold">
										{formatNumber(
											seasonStats.activeFeeLobbies
										)}
									</p>
								</div>
								<div>
									<p className="text-muted-foreground text-xs">
										Avg. Volume / Entry-Fee Lobby
									</p>
									<p className="text-lg font-semibold">
										{seasonStats.feeLobbies > 0
											? formatUSD(
													seasonStats.totalVolumeUsd /
														seasonStats.feeLobbies
												)
											: "—"}
									</p>
								</div>
							</div>
						</CardContent>
					</Card>
				</div>

				<div>
					<div className="mb-5">
						<h3 className="text-lg font-bold md:text-xl">
							Game Breakdown
						</h3>
						<p className="text-muted-foreground text-sm">
							Season-specific statistics and player engagement
						</p>
					</div>

					{seasonGameStats.length > 0 ? (
						<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
							{seasonGameStats.map((game) => {
								const matchShare =
									seasonStats.totalLobbies > 0
										? (game.totalMatches /
												seasonStats.totalLobbies) *
											100
										: 0;

								return (
									<Card
										key={game.gameId}
										className="overflow-hidden"
									>
										<CardHeader className="pb-3">
											<div className="flex items-center justify-between gap-3">
												<div className="flex items-center gap-3">
													<Image
														src={game.gameImageUrl}
														alt={game.gameName}
														width={40}
														height={40}
														className="size-10 rounded-lg object-cover"
													/>
													<div>
														<CardTitle className="text-lg">
															{game.gameName}
														</CardTitle>
														<p className="text-muted-foreground text-xs">
															{game.totalPlayers}{" "}
															active{" "}
															{game.totalPlayers ===
															1
																? "player"
																: "players"}
														</p>
													</div>
												</div>
												<Badge
													variant="secondary"
													className="text-xs"
												>
													{matchShare.toFixed(1)}%
												</Badge>
											</div>
										</CardHeader>
										<CardContent className="space-y-4">
											<div className="grid grid-cols-2 gap-4">
												<div>
													<p className="text-muted-foreground text-xs">
														Total Matches
													</p>
													<p className="text-sm font-semibold">
														{formatNumber(
															game.totalMatches
														)}
													</p>
												</div>
												<div>
													<p className="text-muted-foreground text-xs">
														Total Wins
													</p>
													<p className="text-sm font-semibold">
														{formatNumber(
															game.totalWins
														)}
													</p>
												</div>
												<div>
													<p className="text-muted-foreground text-xs">
														Avg Win Rate
													</p>
													<p className="text-sm font-semibold">
														{game.avgWinRate.toFixed(
															1
														)}
														%
													</p>
												</div>
												<div>
													<p className="text-muted-foreground text-xs">
														Wars Points
													</p>
													<p className="text-sm font-semibold">
														{formatNumber(
															game.totalPoints
														)}
													</p>
												</div>
											</div>

											<div className="bg-muted h-1.5 overflow-hidden rounded-full">
												<div
													className="bg-primary h-full rounded-full transition-all"
													style={{
														width: `${Math.min(Math.max(matchShare, 1), 100)}%`,
													}}
												/>
											</div>
										</CardContent>
									</Card>
								);
							})}
						</div>
					) : (
						<Card>
							<CardContent className="py-12 text-center">
								<p className="text-muted-foreground text-sm">
									No game statistics available yet
								</p>
							</CardContent>
						</Card>
					)}
				</div>
			</section>
		</div>
	);
}
