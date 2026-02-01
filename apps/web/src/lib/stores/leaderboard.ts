/**
 * Leaderboard Store
 *
 * Manages leaderboard browsing state (leaderboard data, sorting, pagination).
 */

import { create } from "zustand";
import type { LeaderBoard } from "@/lib/definitions/player-stats";
import { ApiClient } from "@/lib/api/client";

const PAGE_SIZE = 10;

type SortField = "points" | "winRate" | "matches" | "pnl";

interface LeaderboardActions {
	setInitialData: (leaderboard: LeaderBoard[], total: number) => void;
	fetchLeaderboard: () => Promise<void>;
	setSort: (sortBy: SortField) => void;
	setPage: (page: number) => void;
}

interface LeaderboardStore {
	leaderboard: LeaderBoard[];
	total: number;
	loading: boolean;
	sortBy: SortField;
	order: "asc" | "desc";
	page: number;
	actions: LeaderboardActions;
}

const useLeaderboardStore = create<LeaderboardStore>((set, get) => ({
	leaderboard: [],
	total: 0,
	loading: false,
	sortBy: "points",
	order: "desc",
	page: 1,
	actions: {
		setInitialData: (leaderboard, total) => set({ leaderboard, total }),
		fetchLeaderboard: async () => {
			set({ loading: true });
			const { sortBy, order, page } = get();
			const params = new URLSearchParams({
				sortBy,
				order,
				limit: PAGE_SIZE.toString(),
				offset: ((page - 1) * PAGE_SIZE).toString(),
			});
			try {
				const res = await ApiClient.get<{
					leaderboard: LeaderBoard[];
					total: number;
				}>(`/api/leaderboard?${params.toString()}`);
				set({
					leaderboard: res.data?.leaderboard || [],
					total: res.data?.total || 0,
				});
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
	},
}));

// Export individual state selectors
export const useLeaderboard = () =>
	useLeaderboardStore((state) => state.leaderboard);
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
export const useLeaderboardActions = () =>
	useLeaderboardStore((state) => state.actions);
