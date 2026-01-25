/**
 * User Store
 *
 * User is fetched from the API on app load and cleared on logout.
 */

import { create } from "zustand";
import type { User } from "@/lib/definitions";

interface UserActions {
	setUser: (user: User) => void;
	clearUser: () => void;
	updateUser: (updates: Partial<User>) => void;
	setLoading: (loading: boolean) => void;
}

interface UserStore {
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;

	actions: UserActions;
}

const useUserStore = create<UserStore>((set) => ({
	user: null,
	isAuthenticated: false,
	isLoading: true, // Start as loading until we check auth

	actions: {
		setUser: (user) => {
			set({
				user,
				isAuthenticated: true,
				isLoading: false,
			});
		},

		clearUser: () => {
			set({
				user: null,
				isAuthenticated: false,
				isLoading: false,
			});
		},

		updateUser: (updates) =>
			set((state) => ({
				user: state.user ? { ...state.user, ...updates } : null,
			})),

		setLoading: (loading) => set({ isLoading: loading }),
	},
}));

export const useUser = () => useUserStore((state) => state.user);
export const useIsAuthenticated = () =>
	useUserStore((state) => state.isAuthenticated);
export const useUserLoading = () => useUserStore((state) => state.isLoading);
export const useUserActions = () => useUserStore((state) => state.actions);
