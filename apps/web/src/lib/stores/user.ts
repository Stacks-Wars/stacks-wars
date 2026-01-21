import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User, LobbyStatus } from "@/lib/definitions";

interface UserActions {
	setUser: (user: User) => void;
	clearUser: () => void;
	updateUser: (user: Partial<User>) => void;
	setLobbyFilter: (filter: LobbyStatus[]) => void;
	setLobbyOffset: (offset: number) => void;
}

interface UserStore {
	user: User | null;
	isAuthenticated: boolean;
	lobbyFilter: LobbyStatus[];
	lobbyOffset: number;

	actions: UserActions;
}

const useUserStore = create<UserStore>()(
	persist(
		(set) => ({
			user: null,
			isAuthenticated: false,
			lobbyFilter: ["waiting", "inProgress"],
			lobbyOffset: 0,

			actions: {
				setUser: (user) => {
					set({
						user,
						isAuthenticated: true,
					});
				},

				clearUser: () => {
					set({
						user: null,
						isAuthenticated: false,
					});
				},

				updateUser: (updates) =>
					set((state) => ({
						user: state.user ? { ...state.user, ...updates } : null,
					})),

				setLobbyFilter: (filter) => set({ lobbyFilter: filter }),
				setLobbyOffset: (offset) => set({ lobbyOffset: offset }),
			},
		}),
		{
			name: "user-storage",
			partialize: (state) => ({
				user: state.user,
				isAuthenticated: state.isAuthenticated,
				lobbyFilter: state.lobbyFilter,
				lobbyOffset: state.lobbyOffset,
			}),
		}
	)
);

export const useUser = () => useUserStore((state) => state.user);
export const useIsAuthenticated = () =>
	useUserStore((state) => state.isAuthenticated);
export const useLobbyFilter = () => useUserStore((state) => state.lobbyFilter);
export const useLobbyOffset = () => useUserStore((state) => state.lobbyOffset);
export const useUserActions = () => useUserStore((state) => state.actions);
