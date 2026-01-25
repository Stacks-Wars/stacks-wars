/**
 * App Store
 *
 * Persisted store for app-level preferences and settings.
 * This includes lobby filters, pagination offsets, and other user preferences.
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { LobbyStatus } from "@/lib/definitions";

interface AppActions {
	setLobbyFilter: (filter: LobbyStatus[]) => void;
	setLobbyOffset: (offset: number) => void;
}

interface AppStore {
	lobbyFilter: LobbyStatus[];
	lobbyOffset: number;
	hasHydrated: boolean;

	actions: AppActions;
}

const useAppStore = create<AppStore>()(
	persist(
		(set) => ({
			lobbyFilter: ["waiting", "inProgress"],
			lobbyOffset: 0,
			hasHydrated: false,

			actions: {
				setLobbyFilter: (filter) => set({ lobbyFilter: filter }),
				setLobbyOffset: (offset) => set({ lobbyOffset: offset }),
			},
		}),
		{
			name: "app-storage",
			partialize: (state) => ({
				lobbyFilter: state.lobbyFilter,
				lobbyOffset: state.lobbyOffset,
			}),
			onRehydrateStorage: () => (state) => {
				if (state) {
					state.hasHydrated = true;
				}
			},
		}
	)
);

export const useLobbyFilter = () => useAppStore((state) => state.lobbyFilter);
export const useLobbyOffset = () => useAppStore((state) => state.lobbyOffset);
export const useAppHasHydrated = () =>
	useAppStore((state) => state.hasHydrated);
export const useAppActions = () => useAppStore((state) => state.actions);
