import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User } from "@/lib/definitions";
import { disconnectWallet } from "../wallet";
import { setAuthToken, getAuthToken, removeAuthToken } from "../auth/cookies";

interface AuthState {
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;

	// Actions
	login: (user: User, token: string) => void;
	logout: () => void;
	updateUser: (user: Partial<User>) => void;
	setLoading: (isLoading: boolean) => void;
	getToken: () => string | null;
}

export const useAuthStore = create<AuthState>()(
	persist(
		(set) => ({
			user: null,
			isAuthenticated: false,
			isLoading: false,

			login: (user, token) => {
				setAuthToken(token);
				set({
					user,
					isAuthenticated: true,
					isLoading: false,
				});
			},

			logout: () => {
				disconnectWallet();
				removeAuthToken();

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

			setLoading: (isLoading) => set({ isLoading }),

			getToken: () => getAuthToken(),
		}),
		{
			name: "auth-storage",
			partialize: (state) => ({
				user: state.user,
				isAuthenticated: state.isAuthenticated,
			}),
		}
	)
);
