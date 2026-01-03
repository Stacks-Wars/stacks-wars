"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import {
	ArrowUpDown,
	ChevronLeft,
	ChevronRight,
	TrendingDown,
	TrendingUp,
} from "lucide-react";

interface LeaderboardEntry {
	userId: string;
	username: string | null;
	displayName: string | null;
	walletAddress: string;
	rank: number;
	warsPoint: number;
	totalMatch: number;
	totalWins: number;
	winRate: number;
	pnl: number;
}

type SortField = "warsPoint" | "winRate" | "totalMatch" | "pnl";
type SortDirection = "asc" | "desc";

export default function LeaderboardPage() {
	const router = useRouter();
	const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [sortField, setSortField] = useState<SortField | null>(null);
	const [sortDirection, setSortDirection] = useState<SortDirection>("desc");
	const [currentPage, setCurrentPage] = useState(1);
	const itemsPerPage = 10;

	useEffect(() => {
		const fetchLeaderboard = async () => {
			try {
				const response = await fetch(`/api/leaderboard/dummy`);
				if (!response.ok) {
					throw new Error(
						`Failed to fetch leaderboard: ${response.status}`
					);
				}
				const data = await response.json();
				setLeaderboard(data);
			} catch (err) {
				setError(
					err instanceof Error ? err.message : "An error occurred"
				);
			} finally {
				setLoading(false);
			}
		};

		fetchLeaderboard();
	}, []);

	const sortedLeaderboard = useMemo(() => {
		if (!sortField) return leaderboard;

		return [...leaderboard].sort((a, b) => {
			const aValue = a[sortField];
			const bValue = b[sortField];

			if (sortDirection === "asc") {
				return aValue > bValue ? 1 : -1;
			}
			return aValue < bValue ? 1 : -1;
		});
	}, [leaderboard, sortField, sortDirection]);

	const totalPages = Math.ceil(sortedLeaderboard.length / itemsPerPage);
	const paginatedLeaderboard = useMemo(() => {
		const startIndex = (currentPage - 1) * itemsPerPage;
		const endIndex = startIndex + itemsPerPage;
		return sortedLeaderboard.slice(startIndex, endIndex);
	}, [sortedLeaderboard, currentPage, itemsPerPage]);

	const topRanked = useMemo(() => {
		const sorted = [...leaderboard].sort((a, b) => a.rank - b.rank);
		return {
			first: sorted[0],
			second: sorted[1],
			third: sorted[2],
		};
	}, [leaderboard]);

	const goToNextPage = () => {
		if (currentPage < totalPages) setCurrentPage(currentPage + 1);
	};

	const goToPreviousPage = () => {
		if (currentPage > 1) setCurrentPage(currentPage - 1);
	};

	const handleSort = (field: SortField) => {
		if (sortField === field) {
			setSortDirection(sortDirection === "asc" ? "desc" : "asc");
			return;
		}
		setSortField(field);
		setSortDirection("desc");
	};

	const formatWallet = (address: string) =>
		`${address.slice(0, 6)}...${address.slice(-4)}`;

	const formatPnL = (pnl: number) => {
		const sign = pnl >= 0 ? "+" : "";
		return `${sign}${pnl.toFixed(2)}`;
	};

	if (loading) {
		return (
			<div className="container mx-auto py-8 px-4">
				<Card className="bg-transparent border-none shadow-none rounded-none">
					<CardContent>
						<div className="space-y-2">
							{[...Array(10)].map((_, i) => (
								<Skeleton key={i} className="h-12 w-full" />
							))}
						</div>
					</CardContent>
				</Card>
			</div>
		);
	}

	if (error) {
		return (
			<div className="container mx-auto py-8 px-4">
				<Card className="bg-transparent border-none shadow-none rounded-none">
					<CardContent className="pt-6">
						<div className="text-center py-8">
							<p className="text-destructive text-lg">
								Error: {error}
							</p>
						</div>
					</CardContent>
				</Card>
			</div>
		);
	}

	return (
		<div className="container mx-auto py-8 md:py-18 px-4">
			<Card className="bg-transparent border-none shadow-none rounded-none max-w-full my-8">
				<CardHeader className="px-0 pb-6">
					<div className="flex flex-wrap justify-center items-end gap-6 md:gap-12">
						{[
							topRanked.second,
							topRanked.first,
							topRanked.third,
						].map((player, idx) => {
							const rankEmoji =
								idx === 0 ? "🥈" : idx === 1 ? "🥇" : "🥉";
							const isFirst = idx === 1;
							return (
								<div
									key={idx}
									className={`relative ${isFirst ? "-translate-y-8 sm:-translate-y-12 md:-translate-y-24" : ""}`}
								>
									<div className="absolute -top-3 sm:-top-4 md:-top-6 left-1/2 -translate-x-1/2 text-3xl sm:text-4xl md:text-6xl z-10">
										{rankEmoji}
									</div>
									<Card className="shadow-md bg-gradient-primary pb-12 sm:pb-16 md:pb-20 border-none rounded-t-full rounded-b-0 overflow-hidden px-2 sm:px-3 pt-2 w-28 sm:w-40 md:w-auto">
										<CardContent className="flex flex-col items-center gap-2 sm:gap-6 md:gap-12 p-2 sm:p-3 md:p-4">
											<Avatar className="w-20 h-20 sm:w-36 sm:h-36 md:w-60 md:h-60">
												<AvatarImage
													alt={`${player?.displayName || player?.username || "Player"}'s avatar`}
												/>
												<AvatarFallback className="text-3xl sm:text-6xl md:text-8xl">
													{(
														player?.displayName ||
														player?.username ||
														player?.walletAddress ||
														"?"
													)
														.slice(0, 2)
														.toUpperCase()}
												</AvatarFallback>
											</Avatar>
											<div className="flex flex-col items-center gap-0">
												<div className="text-[7px] sm:text-[9px] md:text-sm text-muted-foreground uppercase">
													Player Name
												</div>
												<div className="font-semibold text-xs sm:text-xl md:text-3xl text-center">
													{player?.displayName ||
														player?.username ||
														"Anonymous"}
												</div>
											</div>
											<div className="flex items-center justify-center gap-2 sm:gap-6 md:gap-16 text-center">
												<div className="flex flex-col">
													<div className="text-[9px] sm:text-xs md:text-base text-muted-foreground uppercase font-semibold">
														Wins
													</div>
													<div className="font-medium text-xs sm:text-xl md:text-2xl">
														{player
															? player.totalWins
															: "--"}
													</div>
												</div>
												<div className="flex flex-col">
													<div className="text-[9px] sm:text-xs md:text-base text-muted-foreground uppercase font-semibold">
														PTS
													</div>
													<div className="font-medium text-xs sm:text-xl md:text-2xl">
														{player
															? `${formatPnL(player.pnl)}`
															: "--"}
													</div>
												</div>
											</div>
										</CardContent>
									</Card>
								</div>
							);
						})}
					</div>
				</CardHeader>
				<CardContent>
					{leaderboard.length === 0 ? (
						<div className="text-center py-8 text-muted-foreground">
							No leaderboard data available
						</div>
					) : (
						<div>
							<Table className="[&_td]:border-0 [&_th]:border-0">
								<TableHeader className="[&_tr]:py-6 md:[&_tr]:py-10 [&_tr]:pb-8 md:[&_tr]:pb-12">
									<TableRow>
										<TableHead className="w-16 text-center text-lg md:text-2xl">
											Rank
										</TableHead>
										<TableHead className="text-lg md:text-2xl">
											Player
										</TableHead>
										<TableHead className="text-center">
											<Button
												variant="ghost"
												onClick={() =>
													handleSort("warsPoint")
												}
												className="h-auto p-0 hover:bg-transparent font-semibold text-lg md:text-2xl"
											>
												War Points
												<ArrowUpDown className="ml-2 h-5 w-5 md:h-6 md:w-6" />
											</Button>
										</TableHead>
										<TableHead className="text-center">
											<Button
												variant="ghost"
												onClick={() =>
													handleSort("winRate")
												}
												className="h-auto p-0 hover:bg-transparent font-semibold text-lg md:text-2xl"
											>
												Win Rate
												<ArrowUpDown className="ml-2 h-5 w-5 md:h-6 md:w-6" />
											</Button>
										</TableHead>
										<TableHead className="text-center">
											<Button
												variant="ghost"
												onClick={() =>
													handleSort("totalMatch")
												}
												className="h-auto p-0 hover:bg-transparent font-semibold text-lg md:text-2xl"
											>
												Matches
												<ArrowUpDown className="ml-2 h-5 w-5 md:h-6 md:w-6" />
											</Button>
										</TableHead>
										<TableHead className="text-right">
											<Button
												variant="ghost"
												onClick={() =>
													handleSort("pnl")
												}
												className="h-auto p-0 hover:bg-transparent font-semibold ml-auto flex text-lg md:text-2xl"
											>
												P&L
												<ArrowUpDown className="ml-2 h-5 w-5 md:h-6 md:w-6" />
											</Button>
										</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody className="[&_tr]:border-0">
									{paginatedLeaderboard.map((entry) => (
										<TableRow key={entry.userId}>
											<TableCell className="font-bold text-center text-lg md:text-2xl">
												{entry.rank === 1 && "🥇"}
												{entry.rank === 2 && "🥈"}
												{entry.rank === 3 && "🥉"}
												{entry.rank > 3 && entry.rank}
											</TableCell>
											<TableCell
												className="cursor-pointer hover:bg-muted/50 transition-colors"
											onClick={() => router.push(`/u/${entry.username || entry.walletAddress}`)}
											>
												<div className="flex items-center gap-3">
													<Avatar className="size-10 md:size-12">
														<AvatarImage
															alt={`${entry.username || entry.displayName || "User"}'s avatar`}
														/>
														<AvatarFallback>
															{(
																entry.displayName ||
																entry.username ||
																entry.walletAddress
															)
																.slice(0, 2)
																.toUpperCase()}
														</AvatarFallback>
													</Avatar>
													<div className="flex flex-col">
														<div className="font-semibold text-base md:text-lg">
															{entry.displayName ||
																entry.username ||
																"Anonymous"}
														</div>
														<div className="text-sm md:text-base text-muted-foreground font-mono">
															{formatWallet(
																entry.walletAddress
															)}
														</div>
													</div>
												</div>
											</TableCell>
											<TableCell className="text-center font-semibold text-base md:text-lg">
												{entry.warsPoint.toFixed(1)}
											</TableCell>
											<TableCell className="text-center text-base md:text-lg">
												{entry.winRate.toFixed(1)}%
											</TableCell>
											<TableCell className="text-center">
												<div className="flex flex-col">
													<span className="font-medium text-base md:text-lg">
														{entry.totalWins}/
														{entry.totalMatch}
													</span>
												</div>
											</TableCell>
											<TableCell
												className={`text-right font-semibold text-base md:text-lg ${
													entry.pnl > 0
														? "text-green-600 dark:text-green-400"
														: entry.pnl < 0
															? "text-red-600 dark:text-red-400"
															: "text-muted-foreground"
												}`}
											>
												<div className="flex items-center justify-end gap-1">
													{entry.pnl > 0 ? (
														<TrendingUp className="h-4 w-4 md:h-5 md:w-5" />
													) : entry.pnl < 0 ? (
														<TrendingDown className="h-4 w-4 md:h-5 md:w-5" />
													) : (
														<TrendingUp className="h-4 w-4 md:h-5 md:w-5" />
													)}
													<span>
														{formatPnL(entry.pnl)}{" "}
														STX
													</span>
												</div>
											</TableCell>
										</TableRow>
									))}
								</TableBody>
							</Table>
						</div>
					)}
					{leaderboard.length > 0 && (
						<div className="flex items-center justify-between mt-4">
							<div className="text-base md:text-xl text-foreground font-medium">
								Page {currentPage} of {totalPages}
							</div>
							<div className="flex gap-2">
								<Button
									variant="default"
									size="icon"
									onClick={goToPreviousPage}
									disabled={currentPage === 1}
									className="rounded-full h-10 w-10 md:h-12 md:w-12"
								>
									<ChevronLeft className="h-5 w-5 md:h-6 md:w-6" />
								</Button>
								<Button
									variant="default"
									size="icon"
									onClick={goToNextPage}
									disabled={currentPage === totalPages}
									className="rounded-full h-10 w-10 md:h-12 md:w-12"
								>
									<ChevronRight className="h-5 w-5 md:h-6 md:w-6" />
								</Button>
							</div>
						</div>
					)}
				</CardContent>
			</Card>
		</div>
	);
}
