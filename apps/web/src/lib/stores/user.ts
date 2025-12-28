import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User, LobbyStatus } from "@/lib/definitions";

interface UserActions {
	setUser: (user: User) => void;
	clearUser: () => void;
	updateUser: (user: Partial<User>) => void;
	setLobbyFilter: (filter: LobbyStatus[]) => void;
}

interface UserState {
	user: User | null;
	isAuthenticated: boolean;
	lobbyFilter: LobbyStatus[];

	actions: UserActions;
}

const useUserStore = create<UserState>()(
	persist(
		(set) => ({
			user: null,
			isAuthenticated: false,
			lobbyFilter: ["waiting", "inProgress"],

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
			},
		}),
		{
			name: "user-storage",
			partialize: (state) => ({
				user: state.user,
				isAuthenticated: state.isAuthenticated,
				lobbyFilter: state.lobbyFilter,
			}),
		}
	)
);

export const useUser = () => useUserStore((state) => state.user);
export const useIsAuthenticated = () =>
	useUserStore((state) => state.isAuthenticated);
export const useLobbyFilter = () => useUserStore((state) => state.lobbyFilter);
export const useUserActions = () => useUserStore((state) => state.actions);
