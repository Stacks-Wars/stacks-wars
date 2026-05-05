/**
 * Game Lobbies Store
 *
 * Manages game lobbies browsing state (lobbies data, pagination).
 */

import { create } from "zustand";
import type { LobbyInfo } from "@/lib/definitions";
import { ApiClient } from "@/lib/api/client";

const PAGE_SIZE = 6;

interface GameLobbiesActions {
	setInitialData: (lobbies: LobbyInfo[], total: number) => void;
	setGameIdentifier: (gameIdentifier: string) => void;
	fetchLobbies: (gameIdentifier: string) => Promise<void>;
	setPage: (page: number) => void;
}

interface GameLobbiesStore {
	lobbies: LobbyInfo[];
	total: number;
	loading: boolean;
	page: number;
	gameIdentifier: string | null;
	actions: GameLobbiesActions;
}

const useGameLobbiesStore = create<GameLobbiesStore>((set, get) => ({
	lobbies: [],
	total: 0,
	loading: false,
	page: 1,
	gameIdentifier: null,
	actions: {
		setInitialData: (lobbies, total) => set({ lobbies, total }),
		setGameIdentifier: (gameIdentifier) => set({ gameIdentifier }),
		fetchLobbies: async (gameIdentifier) => {
			set({ loading: true, gameIdentifier });
			const { page } = get();
			const offset = (page - 1) * PAGE_SIZE;
			try {
				const res = await ApiClient.get<{
					data: LobbyInfo[];
					total: number;
					limit: number;
					offset: number;
				}>(
					`/api/lobbies/game/${gameIdentifier}/lobbies?statuses=waiting,starting,inProgress&limit=${PAGE_SIZE}&offset=${offset}`
				);
				if (res.data) {
					set({
						lobbies: res.data.data,
						total: res.data.total,
					});
				}
			} finally {
				set({ loading: false });
			}
		},
		setPage: (page) => {
			set({ page });
			const { gameIdentifier } = get();
			if (gameIdentifier) {
				get().actions.fetchLobbies(gameIdentifier);
			}
		},
	},
}));

// Export individual state selectors
export const useGameLobbies = () =>
	useGameLobbiesStore((state) => state.lobbies);
export const useGameLobbiesTotal = () =>
	useGameLobbiesStore((state) => state.total);
export const useGameLobbiesLoading = () =>
	useGameLobbiesStore((state) => state.loading);
export const useGameLobbiesPage = () =>
	useGameLobbiesStore((state) => state.page);
export const useGameLobbiesActions = () =>
	useGameLobbiesStore((state) => state.actions);
