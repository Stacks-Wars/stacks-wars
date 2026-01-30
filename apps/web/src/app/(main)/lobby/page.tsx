"use client";

export const runtime = "edge";

import LobbyCard, { LobbyCardSkeleton } from "@/components/main/lobby-card";
import { LobbyFilter } from "@/app/(main)/lobby/_components/lobby-filter";
import {
	useLobbyFilter,
	useLobbyOffset,
	useAppActions,
	useAppHasHydrated,
} from "@/lib/stores/app";
import {
	useLobbyInfo,
	useLobbyTotal,
	useLobbyConnecting,
	useIsLobbyActionLoading,
} from "@/lib/stores/lobby";
import Loading from "@/app/loading";
import type { LobbyStatus } from "@/lib/definitions";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight, Loader2 } from "lucide-react";
import { useLobbyWebSocket } from "@/lib/hooks/useLobbyWebSocket";

const ITEMS_PER_PAGE = 6;

export default function LobbyPage() {
	const hasHydrated = useAppHasHydrated();
	const lobbyFilter = useLobbyFilter();
	const currentOffset = useLobbyOffset();
	const { setLobbyFilter, setLobbyOffset } = useAppActions();

	const lobbyInfo = useLobbyInfo();
	const total = useLobbyTotal();
	const isConnecting = useLobbyConnecting();
	const isLoadingMore = useIsLobbyActionLoading("loadMore");

	const { subscribe, loadMore } = useLobbyWebSocket({
		statusFilter: lobbyFilter,
		limit: ITEMS_PER_PAGE,
		enabled: hasHydrated,
	});

	const handleFilterChange = (newStatuses: LobbyStatus[]) => {
		setLobbyFilter(newStatuses);
		setLobbyOffset(0);
		subscribe(newStatuses);
	};

	const handlePrevious = () => {
		if (currentOffset > 0) {
			const newOffset = Math.max(0, currentOffset - ITEMS_PER_PAGE);
			setLobbyOffset(newOffset);
			loadMore(newOffset);
		}
	};

	const handleNext = () => {
		if (currentOffset + ITEMS_PER_PAGE < total) {
			const newOffset = currentOffset + ITEMS_PER_PAGE;
			setLobbyOffset(newOffset);
			loadMore(newOffset);
		}
	};

	const currentPage = Math.floor(currentOffset / ITEMS_PER_PAGE) + 1;
	const totalPages = Math.ceil(total / ITEMS_PER_PAGE);
	const hasPrevious = currentOffset > 0;
	const hasNext = currentOffset + ITEMS_PER_PAGE < total;

	// Wait for store hydration before rendering (prevents double WS connection)
	if (isConnecting) {
		return <Loading />;
	}

	return (
		<div className="container mx-auto px-4">
			<div className="flex items-center justify-between gap-2 py-6 sm:gap-4 sm:py-8 lg:py-12">
				<h1 className="text-xl font-bold sm:text-2xl lg:text-4xl">
					Available Lobbies
				</h1>
				<LobbyFilter
					value={lobbyFilter}
					onChange={handleFilterChange}
				/>
			</div>

			{isLoadingMore || lobbyInfo === null ? (
				<div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
					{Array.from({ length: 6 }).map((_, i) => (
						<LobbyCardSkeleton key={i} />
					))}
				</div>
			) : lobbyInfo.length === 0 ? (
				<div className="py-16 text-center">
					<p className="text-muted-foreground text-lg font-medium lg:text-xl">
						No lobbies found matching your filters
					</p>
					<p className="text-muted-foreground mt-2 text-sm">
						Try adjusting your filter settings
					</p>
				</div>
			) : (
				<>
					<div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
						{lobbyInfo.map((info) => (
							<LobbyCard key={info.lobby.id} lobbyInfo={info} />
						))}
					</div>

					{totalPages > 1 && (
						<div className="flex items-center justify-center gap-4 py-8">
							<Button
								variant="outline"
								size="sm"
								onClick={handlePrevious}
								disabled={!hasPrevious || isLoadingMore}
							>
								{isLoadingMore && hasPrevious ? (
									<>
										<Loader2 className="mr-2 h-4 w-4 animate-spin" />
										Loading...
									</>
								) : (
									<>
										<ChevronLeft className="mr-2 h-4 w-4" />
										Previous
									</>
								)}
							</Button>

							<span className="text-muted-foreground text-sm">
								Page {currentPage} of {totalPages}
							</span>

							<Button
								variant="outline"
								size="sm"
								onClick={handleNext}
								disabled={!hasNext || isLoadingMore}
							>
								{isLoadingMore && hasNext ? (
									<>
										<Loader2 className="mr-2 h-4 w-4 animate-spin" />
										Loading...
									</>
								) : (
									<>
										Next
										<ChevronRight className="ml-2 h-4 w-4" />
									</>
								)}
							</Button>
						</div>
					)}
				</>
			)}
		</div>
	);
}
