"use client";

import { useEffect, useState } from "react";
import { ApiClient } from "@/lib/api/client";
import LobbyCard, { LobbyCardSkeleton } from "@/components/main/lobby-card";
import type { LobbyInfo } from "@/lib/definitions";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight } from "lucide-react";

const PAGE_SIZE = 6;

interface PlayerLobbiesProps {
	userId: string;
}

export default function PlayerLobbies({ userId }: PlayerLobbiesProps) {
	const [lobbies, setLobbies] = useState<LobbyInfo[]>([]);
	const [loading, setLoading] = useState(false);
	const [page, setPage] = useState(1);
	const [total, setTotal] = useState(0);

	useEffect(() => {
		const fetchLobbies = async () => {
			setLoading(true);
			try {
				const offset = (page - 1) * PAGE_SIZE;
				const res = await ApiClient.get<{ 0: LobbyInfo[]; 1: number }>(
					`/api/player-lobby/${userId}?status=waiting,starting,inProgress&limit=${PAGE_SIZE}&offset=${offset}`
				);
				if (res.data) {
					setLobbies(res.data[0]);
					setTotal(res.data[1]);
				}
			} catch (error) {
				console.error("Failed to fetch player lobbies:", error);
			} finally {
				setLoading(false);
			}
		};

		fetchLobbies();
	}, [userId, page]);

	const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

	return (
		<div className="mt-8 px-4 sm:mt-12 sm:px-0">
			<h2 className="mb-4 text-xl font-bold sm:mb-6 sm:text-3xl">
				Active Lobbies
			</h2>
			<div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
				{loading ? (
					Array.from({ length: 2 }).map((_, i) => (
						<LobbyCardSkeleton key={i} />
					))
				) : lobbies.length > 0 ? (
					lobbies.map((lobbyInfo) => (
						<LobbyCard
							key={lobbyInfo.lobby.id}
							lobbyInfo={lobbyInfo}
						/>
					))
				) : (
					<div className="text-muted-foreground col-span-full py-8 text-center sm:py-12">
						<p className="text-sm sm:text-base">
							No active lobbies
						</p>
					</div>
				)}
			</div>
			{total > PAGE_SIZE && (
				<div className="mt-8 flex w-full items-center justify-between">
					{/* Page Indicator */}
					<div className="text-sm font-medium text-gray-400">
						{page} of {totalPages}
					</div>

					{/* Navigation Buttons */}
					<div className="flex gap-4">
						<Button
							variant="ghost"
							size="icon"
							className="size-10 cursor-pointer rounded-full border"
							disabled={page === 1 || loading}
							onClick={() => setPage(page - 1)}
						>
							<ChevronLeft className="size-5" />
						</Button>
						<Button
							variant="ghost"
							size="icon"
							className="size-10 cursor-pointer rounded-full border"
							disabled={page === totalPages || loading}
							onClick={() => setPage(page + 1)}
						>
							<ChevronRight className="size-5" />
						</Button>
					</div>
				</div>
			)}
		</div>
	);
}
