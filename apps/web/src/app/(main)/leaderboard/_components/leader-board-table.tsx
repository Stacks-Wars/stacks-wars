"use client";

import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
	TrendingUp,
	TrendingDown,
	Minus,
	ChevronLeft,
	ChevronRight,
	ChevronsUpDown,
} from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import { formatAddress, formatAmount } from "@/lib/utils";
import Link from "next/link";
import {
	useLeaderboard,
	useLeaderboardTotal,
	useLeaderboardLoading,
	useLeaderboardPage,
	useLeaderboardActions,
} from "@/lib/stores/leaderboard";
import Image from "next/image";
import { useEffect } from "react";
import type { LeaderBoard } from "@/lib/definitions/player-stats";

const PAGE_SIZE = 10;

type SortField = "points" | "winRate" | "matches" | "pnl";

interface LeaderBoardTableProps {
	leaderboard: LeaderBoard[];
	total: number;
}

function TableSkeleton() {
	return (
		<tbody>
			{Array.from({ length: PAGE_SIZE }).map((_, i) => (
				<tr key={i} className="border-none">
					{Array.from({ length: 6 }).map((_, j) => (
						<td key={j} className="px-2 py-4">
							<Skeleton className="h-4 w-full" />
						</td>
					))}
				</tr>
			))}
		</tbody>
	);
}

export default function LeaderBoardTable({
	leaderboard: initialLeaderboard,
	total: initialTotal,
}: LeaderBoardTableProps) {
	const leaderboard = useLeaderboard();
	const total = useLeaderboardTotal();
	const loading = useLeaderboardLoading();
	const page = useLeaderboardPage();
	const actions = useLeaderboardActions();

	useEffect(() => {
		actions.setInitialData(initialLeaderboard, initialTotal);
	}, [initialLeaderboard, initialTotal, actions]);

	const handleSort = (field: SortField) => {
		actions.setSort(field);
	};

	const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

	return (
		<div className="">
			<Table>
				<TableHeader className="border-b border-white/10">
					<TableRow className="border-none hover:bg-transparent">
						<TableHead className="text-gray-400">Rank</TableHead>
						<TableHead className="text-gray-400">Player</TableHead>
						<TableHead className="text-right text-gray-400">
							<span
								className="inline-flex cursor-pointer items-center gap-1 select-none"
								onClick={() => handleSort("points")}
							>
								Wars Points
								<ChevronsUpDown className="size-4" />
							</span>
						</TableHead>
						<TableHead className="text-right text-gray-400">
							<span
								className="inline-flex cursor-pointer items-center gap-1 select-none"
								onClick={() => handleSort("winRate")}
							>
								Win Rate
								<ChevronsUpDown className="size-4" />
							</span>
						</TableHead>
						<TableHead className="text-right text-gray-400">
							<span
								className="inline-flex cursor-pointer items-center gap-1 select-none"
								onClick={() => handleSort("matches")}
							>
								Matches
								<ChevronsUpDown className="size-4" />
							</span>
						</TableHead>
						<TableHead className="text-right text-gray-400">
							<span
								className="inline-flex cursor-pointer items-center gap-1 select-none"
								onClick={() => handleSort("pnl")}
							>
								P&L
								<ChevronsUpDown className="size-4" />
							</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				{loading ? (
					<TableSkeleton />
				) : (
					<TableBody>
						{leaderboard.map((row) => (
							<TableRow
								key={row.id}
								className={`border-none hover:bg-white/5`}
							>
								{/* Rank Badge */}
								<TableCell>
									<Image
										width={32}
										height={32}
										src={
											row.rankBadge ||
											"/images/gold-medal.svg"
										}
										alt="rank"
										className="size-8"
									/>
								</TableCell>

								{/* Player Info */}
								<TableCell>
									<Link
										href={`/u/${row.username || row.walletAddress}`}
										className="flex items-center gap-3"
									>
										<Avatar className="size-10 border border-white/20">
											<AvatarImage
												src={row.profileImage}
											/>
											<AvatarFallback>
												{(
													row.displayName ||
													row.username ||
													row.walletAddress
												).slice(0, 2)}
											</AvatarFallback>
										</Avatar>
										<div className="flex flex-col">
											{row.displayName ? (
												<>
													<span className="text-sm font-bold">
														{row.displayName}
													</span>
													<span className="text-[10px] tracking-tighter text-gray-500 uppercase">
														{formatAddress(
															row.walletAddress
														)}
													</span>
												</>
											) : (
												<span className="text-sm font-bold">
													{row.username ||
														formatAddress(
															row.walletAddress
														)}
												</span>
											)}
										</div>
									</Link>
								</TableCell>

								{/* Stats */}
								<TableCell className="text-right font-medium">
									{row.points}
								</TableCell>
								<TableCell className="text-right font-medium">
									{row.winRate.toFixed(2)}%
								</TableCell>
								<TableCell className="text-right font-medium">
									{row.totalMatches}
								</TableCell>

								{/* P&L Column */}
								<TableCell className="text-right">
									<div
										className={`flex items-center justify-end gap-2 font-bold ${
											row.totalPnl > 0
												? "text-green-500"
												: row.totalPnl < 0
													? "text-red-500"
													: "text-gray-600"
										}`}
									>
										{row.totalPnl > 0 ? (
											<TrendingUp size={16} />
										) : row.totalPnl < 0 ? (
											<TrendingDown size={16} />
										) : (
											<Minus size={16} />
										)}
										<span>
											{row.totalPnl > 0 && `+`}$
											{formatAmount(row.totalPnl)} STX
										</span>
									</div>
								</TableCell>
							</TableRow>
						))}
					</TableBody>
				)}
			</Table>
			<div className="flex w-full items-center justify-between">
				{/* Page Indicator */}
				<div className="text-sm font-medium text-gray-400">
					Page {page} of {totalPages}
				</div>

				{/* Navigation Buttons */}
				<div className="flex gap-4">
					<Button
						variant="ghost"
						size="icon"
						className="size-10 cursor-pointer rounded-full border"
						disabled={page === 1 || loading}
						onClick={() => actions.setPage(page - 1)}
					>
						<ChevronLeft className="size-5" />
					</Button>
					<Button
						variant="ghost"
						size="icon"
						className="size-10 cursor-pointer rounded-full border"
						disabled={page === totalPages || loading}
						onClick={() => actions.setPage(page + 1)}
					>
						<ChevronRight className="size-5" />
					</Button>
				</div>
			</div>
		</div>
	);
}
