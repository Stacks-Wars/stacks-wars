/**
 * Leaderboard Store
 *
 * Manages leaderboard browsing state (leaderboard data, sorting, pagination).
 * Supports both global leaderboard and per-game leaderboard views.
 */

import { create } from "zustand";
import type {
	LeaderBoard,
	GameLeaderBoard,
} from "@/lib/definitions/player-stats";
import { ApiClient } from "@/lib/api/client";

const PAGE_SIZE = 10;

type SortField = "points" | "winRate" | "matches" | "pnl";

interface LeaderboardActions {
	setInitialData: (leaderboard: LeaderBoard[], total: number) => void;
	fetchLeaderboard: () => Promise<void>;
	setSort: (sortBy: SortField) => void;
	setPage: (page: number) => void;
	setGameId: (gameId: string | null) => void;
}

interface LeaderboardStore {
	leaderboard: LeaderBoard[];
	gameLeaderboard: GameLeaderBoard[];
	total: number;
	loading: boolean;
	sortBy: SortField;
	order: "asc" | "desc";
	page: number;
	/** When set, fetches per-game leaderboard instead of global */
	gameId: string | null;
	actions: LeaderboardActions;
}

const useLeaderboardStore = create<LeaderboardStore>((set, get) => ({
	leaderboard: [],
	gameLeaderboard: [],
	total: 0,
	loading: false,
	sortBy: "points",
	order: "desc",
	page: 1,
	gameId: null,
	actions: {
		setInitialData: (leaderboard, total) => set({ leaderboard, total }),
		fetchLeaderboard: async () => {
			set({ loading: true });
			const { sortBy, order, page, gameId } = get();
			const params = new URLSearchParams({
				sortBy,
				order,
				limit: PAGE_SIZE.toString(),
				offset: ((page - 1) * PAGE_SIZE).toString(),
			});
			try {
				if (gameId) {
					const res = await ApiClient.get<{
						leaderboard: GameLeaderBoard[];
						total: number;
					}>(`/api/leaderboard/game/${gameId}?${params.toString()}`);
					set({
						gameLeaderboard: res.data?.leaderboard || [],
						leaderboard: [],
						total: res.data?.total || 0,
					});
				} else {
					const res = await ApiClient.get<{
						leaderboard: LeaderBoard[];
						total: number;
					}>(`/api/leaderboard?${params.toString()}`);
					set({
						leaderboard: res.data?.leaderboard || [],
						gameLeaderboard: [],
						total: res.data?.total || 0,
					});
				}
			} finally {
				set({ loading: false });
			}
		},
		setSort: (newSortBy) => {
			const { sortBy, order } = get();
			if (sortBy === newSortBy) {
				set({ order: order === "asc" ? "desc" : "asc", page: 1 });
			} else {
				set({ sortBy: newSortBy, order: "desc", page: 1 });
			}
			get().actions.fetchLeaderboard();
		},
		setPage: (page) => {
			set({ page });
			get().actions.fetchLeaderboard();
		},
		setGameId: (gameId) => {
			set({ gameId, page: 1, sortBy: "points", order: "desc" });
			get().actions.fetchLeaderboard();
		},
	},
}));

// Export individual state selectors
export const useLeaderboard = () =>
	useLeaderboardStore((state) => state.leaderboard);
export const useGameLeaderboard = () =>
	useLeaderboardStore((state) => state.gameLeaderboard);
export const useLeaderboardTotal = () =>
	useLeaderboardStore((state) => state.total);
export const useLeaderboardLoading = () =>
	useLeaderboardStore((state) => state.loading);
export const useLeaderboardSortBy = () =>
	useLeaderboardStore((state) => state.sortBy);
export const useLeaderboardOrder = () =>
	useLeaderboardStore((state) => state.order);
export const useLeaderboardPage = () =>
	useLeaderboardStore((state) => state.page);
export const useLeaderboardGameId = () =>
	useLeaderboardStore((state) => state.gameId);
export const useLeaderboardActions = () =>
	useLeaderboardStore((state) => state.actions);
