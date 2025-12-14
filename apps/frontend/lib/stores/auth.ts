import { create } from "zustand";
import { persist } from "zustand/middleware";
import { User } from "@/lib/definitions";
import { disconnectWallet } from "../wallet";

interface AuthState {
	user: User | null;
	token: string | null;
	isAuthenticated: boolean;
	isLoading: boolean;

	// Actions
	login: (user: User, token: string) => void;
	logout: () => void;
	updateUser: (user: Partial<User>) => void;
	setLoading: (isLoading: boolean) => void;
}

export const useAuthStore = create<AuthState>()(
	persist(
		(set) => ({
			user: null,
			token: null,
			isAuthenticated: false,
			isLoading: false,

			login: (user, token) =>
				set({
					user,
					token,
					isAuthenticated: true,
					isLoading: false,
				}),

			logout: () => {
				disconnectWallet();

				set({
					user: null,
					token: null,
					isAuthenticated: false,
					isLoading: false,
				});
			},

			updateUser: (updates) =>
				set((state) => ({
					user: state.user ? { ...state.user, ...updates } : null,
				})),

			setLoading: (isLoading) => set({ isLoading }),
		}),
		{
			name: "auth-storage",
			partialize: (state) => ({
				user: state.user,
				token: state.token,
				isAuthenticated: state.isAuthenticated,
			}),
		}
	)
);
