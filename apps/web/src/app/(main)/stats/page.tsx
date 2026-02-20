import { ApiClient } from "@/lib/api/client";
import type { PlatformStats } from "@/lib/definitions";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

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

export default async function StatsPage() {
	const res = await ApiClient.get<PlatformStats>("/api/stats");
	const stats = res.data;

	if (!stats) {
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

	const utilizationRate =
		stats.totalLobbies > 0
			? ((stats.activeLobbies / stats.totalLobbies) * 100).toFixed(1)
			: "0";

	return (
		<div className="container mx-auto px-4 pb-16">
			{/* Header */}
			<div className="py-8 text-center lg:py-12">
				<h1 className="mb-2 text-3xl font-bold tracking-tight md:text-5xl">
					Platform Analytics
				</h1>
				<p className="text-muted-foreground mx-auto max-w-md text-sm md:text-lg">
					Real-time Stacks Wars platform metrics and total value
					locked
				</p>
			</div>

			{/* Quick Stats */}
			<div className="grid grid-cols-2 gap-3 md:grid-cols-4">
				{[
					{
						label: "Total Users",
						value: formatNumber(stats.totalUsers),
					},
					{
						label: "Total Lobbies",
						value: formatNumber(stats.totalLobbies),
					},
					{
						label: "Active Lobbies",
						value: formatNumber(stats.activeLobbies),
					},
					{
						label: "Games Available",
						value: formatNumber(stats.totalGames),
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

			{/* Volume Cards */}
			<div className="mt-6 grid gap-4 md:grid-cols-3">
				<Card className="border-primary/20 bg-primary/5">
					<CardHeader>
						<CardDescription>
							Total Volume (All-Time)
						</CardDescription>
						<CardTitle className="text-3xl md:text-4xl">
							{formatUSD(stats.totalVolumeUsd)}
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p className="text-muted-foreground text-xs">
							Cumulative value across all lobbies
						</p>
					</CardContent>
				</Card>

				<Card className="border-green-500/20 bg-green-500/5">
					<CardHeader>
						<CardDescription>TVL (Active)</CardDescription>
						<CardTitle className="text-3xl text-green-600 md:text-4xl dark:text-green-400">
							{formatUSD(stats.activeVolumeUsd)}
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p className="text-muted-foreground text-xs">
							Currently locked in active lobbies
						</p>
					</CardContent>
				</Card>

				<Card className="border-blue-500/20 bg-blue-500/5">
					<CardHeader>
						<CardDescription>Distributed Volume</CardDescription>
						<CardTitle className="text-3xl text-blue-600 md:text-4xl dark:text-blue-400">
							{formatUSD(stats.finishedVolumeUsd)}
						</CardTitle>
					</CardHeader>
					<CardContent>
						<p className="text-muted-foreground text-xs">
							Total winnings distributed to players
						</p>
					</CardContent>
				</Card>
			</div>

			{/* Token Breakdown */}
			<div className="mt-10">
				<div className="mb-5">
					<h2 className="text-xl font-bold md:text-2xl">
						Token Breakdown
					</h2>
					<p className="text-muted-foreground text-sm">
						Volume distribution by token
					</p>
				</div>

				{stats.tokenBreakdown.length > 0 ? (
					<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
						{stats.tokenBreakdown.map((token) => {
							const share =
								stats.totalVolumeUsd > 0
									? (token.volumeUsd / stats.totalVolumeUsd) *
										100
									: 0;

							return (
								<Card
									key={token.symbol}
									className="overflow-hidden"
								>
									<CardHeader className="pb-3">
										<div className="flex items-center justify-between">
											<div className="flex items-center gap-3">
												{token.imageUrl ? (
													<img
														src={token.imageUrl}
														alt={token.symbol}
														className="h-10 w-10 rounded-full object-cover"
													/>
												) : (
													<div className="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full font-bold">
														{token.symbol.charAt(0)}
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
													{formatUSD(token.volumeUsd)}
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

										{/* Share bar */}
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

			{/* Platform Health */}
			<div className="mt-10">
				<Card>
					<CardHeader>
						<CardTitle>Platform Health</CardTitle>
						<CardDescription>
							Key operational metrics
						</CardDescription>
					</CardHeader>
					<CardContent className="space-y-5">
						<div>
							<div className="mb-2 flex items-center justify-between">
								<span className="text-muted-foreground text-sm">
									Lobby Utilization
								</span>
								<span className="font-semibold">
									{utilizationRate}%
								</span>
							</div>
							<div className="bg-muted h-2 overflow-hidden rounded-full">
								<div
									className="bg-primary h-full rounded-full transition-all"
									style={{
										width: `${Math.min(Number(utilizationRate), 100)}%`,
									}}
								/>
							</div>
						</div>

						<div className="grid grid-cols-2 gap-4 pt-2 md:grid-cols-3">
							<div>
								<p className="text-muted-foreground text-xs">
									Finished Lobbies
								</p>
								<p className="text-lg font-semibold">
									{formatNumber(stats.finishedLobbies)}
								</p>
							</div>
							<div>
								<p className="text-muted-foreground text-xs">
									Avg. Volume / Lobby
								</p>
								<p className="text-lg font-semibold">
									{stats.totalLobbies > 0
										? formatUSD(
												stats.totalVolumeUsd /
													stats.totalLobbies
											)
										: "—"}
								</p>
							</div>
							<div>
								<p className="text-muted-foreground text-xs">
									Avg. Volume / User
								</p>
								<p className="text-lg font-semibold">
									{stats.totalUsers > 0
										? formatUSD(
												stats.totalVolumeUsd /
													stats.totalUsers
											)
										: "—"}
								</p>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>

			{/* Footer */}
			<div className="pt-10 text-center">
				<p className="text-muted-foreground text-xs">
					Data refreshed on each page load. USD prices via{" "}
					<a
						href="https://stxtools.io"
						target="_blank"
						rel="noopener noreferrer"
						className="underline"
					>
						stxtools.io
					</a>{" "}
					(5-min cache).
				</p>
			</div>
		</div>
	);
}
