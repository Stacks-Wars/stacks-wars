"use client";

import LobbyCard, { LobbyCardSkeleton } from "@/components/main/lobby-card";
import { LobbyFilter } from "@/app/(main)/lobby/_components/lobby-filter";
import { useLobbyListWebSocket } from "@/lib/hooks/useLobbyListWebSocket";
import { useLobbyFilter, useUserActions } from "@/lib/stores/user";
import Loading from "@/app/loading";
import type { LobbyStatus } from "@/lib/definitions";

export default function LobbyPage() {
	const lobbyFilter = useLobbyFilter();
	const { setLobbyFilter } = useUserActions();

	const { lobbies, total, isConnected, isConnecting, error, subscribe } =
		useLobbyListWebSocket({
			statusFilter: lobbyFilter,
			limit: 12,
		});

	const handleFilterChange = (newStatuses: LobbyStatus[]) => {
		setLobbyFilter(newStatuses);
		// Send subscribe message to update filter on server
		subscribe(newStatuses);
	};

	if (isConnecting) {
		return <Loading />;
	}

	return (
		<div className="container mx-auto px-4">
			<div className="flex items-center justify-between gap-4 py-4 lg:py-15">
				<h1 className="text-xl lg:text-[40px] font-bold">
					Available Lobbies
				</h1>
				<div className="flex items-center gap-4">
					<LobbyFilter
						value={lobbyFilter}
						onChange={handleFilterChange}
					/>
				</div>
			</div>

			{error && (
				<div className="mb-4 p-4 bg-destructive/10 text-destructive rounded-md">
					{error}
				</div>
			)}

			{lobbies === null ? (
				<div className="grid gap-5 sm:grid-cols-2 justify-items-center max-w-305 mx-auto">
					{Array.from({ length: 6 }).map((_, i) => (
						<LobbyCardSkeleton key={i} />
					))}
				</div>
			) : lobbies.length === 0 ? (
				<div className="text-center py-12">
					<p className="text-xl text-muted-foreground">
						No lobbies found matching your filters
					</p>
					<p className="text-sm text-muted-foreground mt-2">
						Try adjusting your filter settings
					</p>
				</div>
			) : (
				<div className="grid gap-5 sm:grid-cols-2 justify-items-center max-w-305 mx-auto">
					{lobbies.map((lobby) => (
						<LobbyCard key={lobby.id} lobby={lobby} />
					))}
				</div>
			)}

			{total > (lobbies?.length || 0) && (
				<div className="text-center py-4 text-sm text-muted-foreground">
					Showing {lobbies?.length || 0} of {total} lobbies
				</div>
			)}
		</div>
	);
}
