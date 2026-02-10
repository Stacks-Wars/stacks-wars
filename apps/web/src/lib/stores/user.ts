/**
 * User Store
 *
 * User is fetched from the API on app load and cleared on logout.
 * Also manages user token balances and minimum amounts for lobby creation.
 */

import { create } from "zustand";
import type { User, Token, TokenInfo } from "@/lib/definitions";
import { ApiClient } from "@/lib/api/client";

interface UserActions {
	setUser: (user: User) => void;
	clearUser: () => void;
	updateUser: (updates: Partial<User>) => void;
	setLoading: (loading: boolean) => void;
	fetchTokens: (walletAddress: string) => Promise<void>;
	fetchMinimumAmount: (contractId: string) => Promise<void>;
	setSelectedToken: (contractId: string) => void;
}

interface UserStore {
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;

	tokens: Token[];
	minimumAmount: number;
	selectedToken: string;
	tokensLoading: boolean;

	actions: UserActions;
}

const useUserStore = create<UserStore>((set, get) => ({
	user: null,
	isAuthenticated: false,
	isLoading: true, // Start as loading until we check auth

	tokens: [{ name: "STX", balance: 0, contractId: "stx" }],
	minimumAmount: 0,
	selectedToken: "stx",
	tokensLoading: false,

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
				tokens: [{ name: "STX", balance: 0, contractId: "stx" }],
				minimumAmount: 0,
				selectedToken: "stx",
			});
		},

		updateUser: (updates) =>
			set((state) => ({
				user: state.user ? { ...state.user, ...updates } : null,
			})),

		setLoading: (loading) => set({ isLoading: loading }),

		fetchTokens: async (walletAddress) => {
			set({ tokensLoading: true });
			try {
				const response = await ApiClient.get<Token[]>(
					`/api/balance/${walletAddress}`
				);
				if (response.data) {
					const fetchedTokens = response.data;
					const hasSTX = fetchedTokens.some(
						(t) => t.contractId === "stx"
					);
					if (!hasSTX) {
						fetchedTokens.unshift({
							name: "STX",
							balance: 0,
							contractId: "stx",
						});
					}
					set({ tokens: fetchedTokens });
				}
			} finally {
				set({ tokensLoading: false });
			}
		},

		fetchMinimumAmount: async (contractId) => {
			try {
				const response = await ApiClient.get<TokenInfo>(
					`/api/token/${contractId}`
				);
				if (response.data) {
					set({ minimumAmount: response.data.minimumAmount });
				}
			} catch (err) {
				console.error("Failed to fetch minimum amount:", err);
			}
		},

		setSelectedToken: (contractId) => {
			set({ selectedToken: contractId });
			get().actions.fetchMinimumAmount(contractId);
		},
	},
}));

export const useUser = () => useUserStore((state) => state.user);
export const useIsAuthenticated = () =>
	useUserStore((state) => state.isAuthenticated);
export const useUserLoading = () => useUserStore((state) => state.isLoading);
export const useTokens = () => useUserStore((state) => state.tokens);
export const useMinimumAmount = () =>
	useUserStore((state) => state.minimumAmount);
export const useSelectedToken = () =>
	useUserStore((state) => state.selectedToken);
export const useTokensLoading = () =>
	useUserStore((state) => state.tokensLoading);
export const useUserActions = () => useUserStore((state) => state.actions);
