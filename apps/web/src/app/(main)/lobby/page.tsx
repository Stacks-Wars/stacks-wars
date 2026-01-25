"use client";

import { useCallback } from "react";
import LobbyCard, { LobbyCardSkeleton } from "@/components/main/lobby-card";
import { LobbyFilter } from "@/app/(main)/lobby/_components/lobby-filter";
import {
	useLobbyFilter,
	useLobbyOffset,
	useUserActions,
	useHasHydrated,
} from "@/lib/stores/user";
import {
	useLobbyInfo,
	useLobbyTotal,
	useLobbyConnecting,
	useIsLobbyActionLoading,
} from "@/lib/stores/lobby";
import Loading from "@/app/loading";
import type { LobbyStatus, LobbyInfo } from "@/lib/definitions";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { useRouter } from "next/navigation";
import { useLobbyWebSocket } from "@/lib/hooks/useLobbyWebSocket";
import { formatAddress } from "@/lib/utils";

const ITEMS_PER_PAGE = 6;

export default function LobbyPage() {
	const router = useRouter();
	const hasHydrated = useHasHydrated();
	const lobbyFilter = useLobbyFilter();
	const currentOffset = useLobbyOffset();
	const { setLobbyFilter, setLobbyOffset } = useUserActions();

	const lobbyInfo = useLobbyInfo();
	const total = useLobbyTotal();
	const isConnecting = useLobbyConnecting();
	const isLoadingMore = useIsLobbyActionLoading("loadMore");

	const handleActionSuccess = useCallback(
		(action: string, data?: string | LobbyInfo) => {
			if (action === "lobbyCreated") {
				const lobbyInfo = data as LobbyInfo;
				toast.success(`New ${lobbyInfo.game.name} lobby created!`, {
					description: `${lobbyInfo.lobby.name} by ${lobbyInfo.creator.username || formatAddress(lobbyInfo.creator.walletAddress)}`,
					action: {
						label: "Open",
						onClick: () => {
							router.push(`/room/${lobbyInfo.lobby.path}`);
						},
					},
				});
			}
		},
		[router]
	);

	const handleActionError = useCallback(
		(action: string, error: { code: string; message: string }) => {
			if (action === "loadMore") {
				toast.error(`Failed to load more lobbies: ${error.message}`);
			}
		},
		[]
	);

	const { subscribe, loadMore } = useLobbyWebSocket({
		statusFilter: lobbyFilter,
		limit: ITEMS_PER_PAGE,
		enabled: hasHydrated,
		onActionSuccess: handleActionSuccess,
		onActionError: handleActionError,
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
			<div className="flex items-center justify-between gap-2 sm:gap-4 py-6 sm:py-8 lg:py-12">
				<h1 className="text-xl sm:text-2xl lg:text-4xl font-bold">
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
				<div className="text-center py-16">
					<p className="text-lg lg:text-xl font-medium text-muted-foreground">
						No lobbies found matching your filters
					</p>
					<p className="text-sm text-muted-foreground mt-2">
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
										<Loader2 className="h-4 w-4 mr-2 animate-spin" />
										Loading...
									</>
								) : (
									<>
										<ChevronLeft className="h-4 w-4 mr-2" />
										Previous
									</>
								)}
							</Button>

							<span className="text-sm text-muted-foreground">
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
										<Loader2 className="h-4 w-4 mr-2 animate-spin" />
										Loading...
									</>
								) : (
									<>
										Next
										<ChevronRight className="h-4 w-4 ml-2" />
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
