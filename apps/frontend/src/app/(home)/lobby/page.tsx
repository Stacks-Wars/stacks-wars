"use client";
import Lobbies from "@/components/home/lobbies";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuTrigger,
	DropdownMenuLabel,
	DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import Loading from "@/app/loading";
import { Plus, Filter, ChevronLeft, ChevronRight, Loader2 } from "lucide-react";
import Link from "next/link";
import { apiRequest } from "@/lib/api";
import { LobbyExtended, lobbyState } from "@/types/schema/lobby";
import { useEffect, useState, useCallback } from "react";

const LOBBIES_PER_PAGE = 12;

export default function LobbyPage() {
	const [lobbies, setLobbies] = useState<LobbyExtended[]>([]);
	const [isLoading, setIsLoading] = useState(true);
	const [currentPage, setCurrentPage] = useState(1);
	const [selectedStates, setSelectedStates] = useState<lobbyState[]>([
		"waiting",
		"starting",
		"inProgress",
	]);
	const [isRefetching, setIsRefetching] = useState(false);

	const filterOptions = [
		{ value: "waiting" as lobbyState, label: "Active" },
		{ value: "starting" as lobbyState, label: "Starting" },
		{ value: "inProgress" as lobbyState, label: "In Progress" },
		{ value: "finished" as lobbyState, label: "Finished" },
	];

	const handleStateChange = (state: lobbyState, checked: boolean) => {
		setCurrentPage(1);
		setSelectedStates((prev) => {
			if (checked) {
				return [...prev, state];
			} else {
				return prev.filter((s) => s !== state);
			}
		});
	};

	const fetchLobbies = useCallback(async () => {
		setIsRefetching(true);
		try {
			const statesQuery =
				selectedStates.length > 0
					? selectedStates.join(",")
					: "waiting";

			const data = await apiRequest<LobbyExtended[]>({
				path: `/lobby/extended?page=${currentPage}&limit=${LOBBIES_PER_PAGE}&lobby_state=${statesQuery}`,
				auth: false,
				cache: "no-store",
			});

			setLobbies(data);
		} catch (error) {
			console.error("Failed to fetch lobbies:", error);
		} finally {
			setIsLoading(false);
			setIsRefetching(false);
		}
	}, [selectedStates, currentPage]);

	useEffect(() => {
		// Initial fetch
		fetchLobbies();

		// Set up interval to fetch every 60 seconds
		const interval = setInterval(fetchLobbies, 60000);

		// Cleanup interval on component unmount
		return () => clearInterval(interval);
	}, [fetchLobbies]);

	const showPagination =
		lobbies.length === LOBBIES_PER_PAGE || currentPage > 1;
	// Show next button if current page has full results or we're not on a page that returned less than 12
	const canGoNext = lobbies.length === LOBBIES_PER_PAGE;

	const handlePrevious = () => {
		if (currentPage > 1) {
			setCurrentPage(currentPage - 1);
		}
	};

	const handleNext = () => {
		if (canGoNext) {
			setCurrentPage(currentPage + 1);
		}
	};

	if (isLoading) {
		return <Loading />;
	}

	return (
		<>
			<section className="w-full py-12 md:py-24 lg:py-32">
				<div className="max-w-5xl mx-auto px-4 md:px-6">
					<div className="flex flex-col items-start gap-4 md:flex-row md:justify-between md:items-center">
						<div>
							<h1 className="text-3xl font-bold tracking-tighter sm:text-4xl md:text-5xl">
								Lobbies
							</h1>
							<p className="mt-2 text-muted-foreground">
								Join a lobby to Battle
								{showPagination && (
									<span className="ml-2">
										• Page {currentPage}
									</span>
								)}
							</p>
						</div>
						<div className="flex gap-2">
							<Button className="gap-1.5" asChild>
								<Link href="/games">
									<Plus className="h-4 w-4" />
									Create A Match
								</Link>
							</Button>
							<DropdownMenu>
								<DropdownMenuTrigger asChild>
									<Button
										variant="outline"
										className="gap-1.5"
										disabled={isRefetching}
									>
										{isRefetching ? (
											<Loader2 className="h-4 w-4 animate-spin" />
										) : (
											<Filter className="h-4 w-4" />
										)}
										Filter ({selectedStates.length})
									</Button>
								</DropdownMenuTrigger>
								<DropdownMenuContent
									className="w-48"
									align="end"
								>
									<DropdownMenuLabel>
										Filter by Status
									</DropdownMenuLabel>
									<DropdownMenuSeparator />
									{filterOptions.map((option) => (
										<div
											key={option.value}
											className="flex items-center space-x-2 px-2 py-1.5 hover:bg-accent rounded-sm"
										>
											<Checkbox
												id={option.value}
												checked={selectedStates.includes(
													option.value
												)}
												onCheckedChange={(
													checked: boolean | string
												) =>
													handleStateChange(
														option.value,
														!!checked
													)
												}
												disabled={isRefetching}
											/>
											<label
												htmlFor={option.value}
												className="text-sm font-medium cursor-pointer"
											>
												{option.label}
											</label>
										</div>
									))}
								</DropdownMenuContent>
							</DropdownMenu>
						</div>
					</div>
					{/*<div className="mt-6 p-4 bg-yellow-500/10 border-2 border-yellow-500/20 rounded-lg">
						<p className="text-yellow-500 text-sm font-medium flex items-center gap-2">
							🚧 This feature is currently under development.
							Check back soon for updates!
						</p>
					</div>*/}
					{isRefetching ? (
						<div className="flex items-center justify-center py-16">
							<div className="flex flex-col items-center gap-2">
								<Loader2 className="h-8 w-8 animate-spin text-primary" />
								<p className="text-sm text-muted-foreground font-medium">
									Updating lobbies...
								</p>
							</div>
						</div>
					) : (
						<div className="grid gap-6 pt-8 md:grid-cols-2 lg:grid-cols-3">
							<Lobbies lobbies={lobbies} />
						</div>
					)}

					{showPagination && (
						<div className="flex justify-center items-center gap-4 mt-8">
							<Button
								variant="outline"
								size="sm"
								onClick={handlePrevious}
								disabled={currentPage === 1}
								className="gap-1.5"
							>
								<ChevronLeft className="h-4 w-4" />
								Previous
							</Button>

							<span className="text-sm text-muted-foreground px-2">
								Page {currentPage}
							</span>

							<Button
								variant="outline"
								size="sm"
								onClick={handleNext}
								disabled={!canGoNext}
								className="gap-1.5"
							>
								Next
								<ChevronRight className="h-4 w-4" />
							</Button>
						</div>
					)}
				</div>
			</section>
		</>
	);
}
