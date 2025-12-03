"use client";

import * as React from "react";
import {
	useReactTable,
	getCoreRowModel,
	getSortedRowModel,
	getFilteredRowModel,
	getPaginationRowModel,
	type ColumnDef,
	type SortingState,
	type ColumnFiltersState,
	flexRender,
} from "@tanstack/react-table";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
	ChevronLeft,
	ChevronRight,
	Search,
	Trophy,
	TrendingUp,
	TrendingDown,
	Minus,
} from "lucide-react";
import { LeaderBoard } from "@/types/schema/leaderboard";
import Link from "next/link";

interface LeaderboardTableProps {
	data: LeaderBoard[];
}

function truncateAddress(address: string): string {
	return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

function formatPnL(pnl: number): string {
	const sign = pnl > 0 ? "+" : "";
	return `${sign}${pnl.toFixed(2)} STX`;
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
			return rank.toString();
	}
}

export default function LeaderboardTable({ data }: LeaderboardTableProps) {
	const [sorting, setSorting] = React.useState<SortingState>([
		{ id: "rank", desc: false },
	]);
	const [columnFilters, setColumnFilters] =
		React.useState<ColumnFiltersState>([]);
	const [globalFilter, setGlobalFilter] = React.useState("");

	const columns: ColumnDef<LeaderBoard>[] = [
		{
			accessorKey: "rank",
			header: "Rank",
			cell: ({ row }) => {
				const rank = row.getValue("rank") as number;
				return (
					<div className="flex items-center justify-center">
						<div className="bg-muted flex h-8 w-8 items-center justify-center rounded-full text-center font-semibold">
							{getRankIcon(rank)}
						</div>
					</div>
				);
			},
		},
		{
			accessorKey: "user",
			header: "Player",
			cell: ({ row }) => {
				const user = row.getValue("user") as LeaderBoard["user"];
				const displayName =
					user.displayName ||
					user.username ||
					truncateAddress(user.walletAddress);
				const identifier = user.username || user.walletAddress;

				return (
					<Link
						href={`/u/${identifier}`}
						className="flex items-center space-x-3"
					>
						<Avatar className="h-8 w-8">
							<AvatarFallback className="bg-primary/10">
								{displayName.charAt(0).toUpperCase()}
							</AvatarFallback>
						</Avatar>
						<div className="min-w-0 flex-1">
							<p className="truncate text-sm leading-none font-medium hover:underline">
								{displayName}
							</p>
							{(user.displayName || user.username) && (
								<p className="text-muted-foreground truncate text-xs hover:underline">
									{truncateAddress(user.walletAddress)}
								</p>
							)}
						</div>
					</Link>
				);
			},
		},
		{
			accessorKey: "warsPoint",
			header: "Wars Points",
			cell: ({ row }) => {
				const user = row.getValue("user") as LeaderBoard["user"];
				const warsPoint = user.warsPoint;
				return (
					<div className="text-center">
						<Badge
							variant={warsPoint >= 0 ? "default" : "destructive"}
							className="font-mono"
						>
							{warsPoint.toFixed(0)}
						</Badge>
					</div>
				);
			},
			sortingFn: (rowA, rowB) => {
				const userA = rowA.getValue("user") as LeaderBoard["user"];
				const userB = rowB.getValue("user") as LeaderBoard["user"];
				return userA.warsPoint - userB.warsPoint;
			},
		},
		{
			accessorKey: "winRate",
			header: "Win Rate",
			cell: ({ row }) => {
				const winRate = row.getValue("winRate") as number;
				return (
					<div className="text-center">
						<Badge variant="outline" className="font-mono">
							{winRate.toFixed(1)}%
						</Badge>
					</div>
				);
			},
		},
		{
			accessorKey: "totalMatch",
			header: "Matches",
			cell: ({ row }) => {
				const totalMatch = row.getValue("totalMatch") as number;
				const totalWins = row.original.totalWins;
				return (
					<div className="text-center">
						<div className="text-sm font-medium">{totalMatch}</div>
						<div className="text-muted-foreground text-xs">
							{totalWins}W - {totalMatch - totalWins}L
						</div>
					</div>
				);
			},
		},
		{
			accessorKey: "pnl",
			header: "P&L",
			cell: ({ row }) => {
				const pnl = row.getValue("pnl") as number;
				return (
					<div className="text-center">
						<div
							className={`flex items-center justify-center gap-1 text-sm font-medium ${getPnLColor(pnl)}`}
						>
							{getPnLIcon(pnl)}
							{formatPnL(pnl)}
						</div>
					</div>
				);
			},
		},
	];

	// Custom global filter function
	const globalFilterFn = React.useMemo(
		() =>
			(
				row: { original: LeaderBoard },
				columnId: string,
				value: string
			) => {
				const user = row.original.user;
				const searchValue = value.toLowerCase();

				// Search in wallet address, username, and display name
				const searchableText = [
					user.walletAddress,
					user.username,
					user.displayName,
				]
					.filter(Boolean) // Remove null/undefined values
					.join(" ")
					.toLowerCase();

				return searchableText.includes(searchValue);
			},
		[]
	);

	const table = useReactTable({
		data,
		columns,
		onSortingChange: setSorting,
		onColumnFiltersChange: setColumnFilters,
		onGlobalFilterChange: setGlobalFilter,
		getCoreRowModel: getCoreRowModel(),
		getPaginationRowModel: getPaginationRowModel(),
		getSortedRowModel: getSortedRowModel(),
		getFilteredRowModel: getFilteredRowModel(),
		globalFilterFn: globalFilterFn,
		state: {
			sorting,
			columnFilters,
			globalFilter,
		},
		initialState: {
			pagination: {
				pageSize: 10,
			},
		},
	});

	return (
		<Card className="bg-primary/30 w-full">
			<CardHeader className="flex flex-col space-y-4 pb-4 sm:flex-row sm:items-center sm:justify-between sm:space-y-0">
				<CardTitle className="flex items-center gap-2">
					<Trophy className="h-5 w-5 text-yellow-500" />
					Rankings
				</CardTitle>
				<div className="flex items-center space-x-2">
					<div className="relative w-full sm:w-[250px]">
						<Search className="text-muted-foreground absolute top-1/2 left-2 h-4 w-4 -translate-y-1/2" />
						<Input
							placeholder="Search by wallet, username..."
							value={globalFilter ?? ""}
							onChange={(e) => setGlobalFilter(e.target.value)}
							className="w-full pl-8"
						/>
					</div>
				</div>
			</CardHeader>
			<CardContent className="p-0">
				<div className="overflow-x-auto">
					<Table>
						<TableHeader>
							{table.getHeaderGroups().map((headerGroup) => (
								<TableRow key={headerGroup.id}>
									{headerGroup.headers.map((header) => (
										<TableHead
											key={header.id}
											className={
												header.id === "user"
													? "text-left"
													: "text-center"
											}
										>
											{header.isPlaceholder
												? null
												: flexRender(
														header.column.columnDef
															.header,
														header.getContext()
													)}
										</TableHead>
									))}
								</TableRow>
							))}
						</TableHeader>
						<TableBody>
							{table.getRowModel().rows?.length ? (
								table.getRowModel().rows.map((row) => (
									<TableRow key={row.id}>
										{row.getVisibleCells().map((cell) => (
											<TableCell
												key={cell.id}
												className="py-4"
											>
												{flexRender(
													cell.column.columnDef.cell,
													cell.getContext()
												)}
											</TableCell>
										))}
									</TableRow>
								))
							) : (
								<TableRow>
									<TableCell
										colSpan={columns.length}
										className="h-24 text-center"
									>
										No players found.
									</TableCell>
								</TableRow>
							)}
						</TableBody>
					</Table>
				</div>

				{/* Pagination */}
				{table.getPageCount() > 1 && (
					<div className="flex items-center justify-between space-x-2 border-t px-6 py-4">
						<div className="text-muted-foreground text-sm">
							Showing {table.getState().pagination.pageIndex + 1}{" "}
							of {table.getPageCount()} pages
						</div>
						<div className="flex items-center space-x-2">
							<Button
								variant="outline"
								size="sm"
								onClick={() => table.previousPage()}
								disabled={!table.getCanPreviousPage()}
							>
								<ChevronLeft className="h-4 w-4" />
								Previous
							</Button>
							<Button
								variant="outline"
								size="sm"
								onClick={() => table.nextPage()}
								disabled={!table.getCanNextPage()}
							>
								Next
								<ChevronRight className="h-4 w-4" />
							</Button>
						</div>
					</div>
				)}
			</CardContent>
		</Card>
	);
}
