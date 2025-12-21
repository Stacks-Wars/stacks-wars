import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User } from "@/lib/definitions";

interface UserState {
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;

	// Actions
	setUser: (user: User) => void;
	clearUser: () => void;
	updateUser: (user: Partial<User>) => void;
	setLoading: (isLoading: boolean) => void;
}

export const useUserStore = create<UserState>()(
	persist(
		(set) => ({
			user: null,
			isAuthenticated: false,
			isLoading: false,

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

			setLoading: (isLoading) => set({ isLoading }),
		}),
		{
			name: "user-storage",
			partialize: (state) => ({
				user: state.user,
				isAuthenticated: state.isAuthenticated,
			}),
		}
	)
);
