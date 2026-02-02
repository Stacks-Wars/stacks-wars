/**
 * Player Lobbies Store
 *
 * Manages player lobbies browsing state (lobbies data, pagination).
 */

import { create } from "zustand";
import type { LobbyInfo } from "@/lib/definitions";
import { ApiClient } from "@/lib/api/client";

const PAGE_SIZE = 6;

interface PlayerLobbiesActions {
	setInitialData: (lobbies: LobbyInfo[], total: number) => void;
	setUserId: (userId: string) => void;
	fetchLobbies: (userId: string) => Promise<void>;
	setPage: (page: number) => void;
}

interface PlayerLobbiesStore {
	lobbies: LobbyInfo[];
	total: number;
	loading: boolean;
	page: number;
	userId: string | null;
	actions: PlayerLobbiesActions;
}

const usePlayerLobbiesStore = create<PlayerLobbiesStore>((set, get) => ({
	lobbies: [],
	total: 0,
	loading: false,
	page: 1,
	userId: null,
	actions: {
		setInitialData: (lobbies, total) => set({ lobbies, total }),
		setUserId: (userId) => set({ userId }),
		fetchLobbies: async (userId) => {
			set({ loading: true, userId });
			const { page } = get();
			const offset = (page - 1) * PAGE_SIZE;
			try {
				const res = await ApiClient.get<{ 0: LobbyInfo[]; 1: number }>(
					`/api/player-lobby/${userId}?status=waiting,starting,inProgress&limit=${PAGE_SIZE}&offset=${offset}`
				);
				if (res.data) {
					set({
						lobbies: res.data[0],
						total: res.data[1],
					});
				}
			} finally {
				set({ loading: false });
			}
		},
		setPage: (page) => {
			set({ page });
			const { userId } = get();
			if (userId) {
				get().actions.fetchLobbies(userId);
			}
		},
	},
}));

// Export individual state selectors
export const usePlayerLobbies = () =>
	usePlayerLobbiesStore((state) => state.lobbies);
export const usePlayerLobbiesTotal = () =>
	usePlayerLobbiesStore((state) => state.total);
export const usePlayerLobbiesLoading = () =>
	usePlayerLobbiesStore((state) => state.loading);
export const usePlayerLobbiesPage = () =>
	usePlayerLobbiesStore((state) => state.page);
export const usePlayerLobbiesActions = () =>
	usePlayerLobbiesStore((state) => state.actions);