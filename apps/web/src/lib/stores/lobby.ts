/**
 * Lobby  Store
 *
 * Manages lobby  browsing state (lobby , filters, pagination).
 */

import { create } from "zustand";
import type { LobbyInfo } from "@/lib/definitions";

interface LobbyActions {
	setLobby: (lobbyInfo: LobbyInfo[], total: number) => void;
	addLobby: (lobbyInfo: LobbyInfo) => void;
	updateLobby: (lobbyInfo: LobbyInfo) => void;
	removeLobby: (lobbyId: string) => void;
	setConnected: (connected: boolean) => void;
	setConnecting: (connecting: boolean) => void;
	setError: (error: string | null) => void;
	setActionLoading: (action: string, loading: boolean) => void;
	clearActionLoading: (action: string) => void;
	clearAllLoadingActions: () => void;
	reset: () => void;
}

interface LobbyStore {
	// State
	lobbyInfo: LobbyInfo[] | null;
	total: number;
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;
	loadingActions: Set<string>;

	// Actions
	actions: LobbyActions;
}

const initialState = {
	lobbyInfo: null,
	total: 0,
	isConnected: false,
	isConnecting: false,
	error: null,
	loadingActions: new Set<string>(),
};

export const useLobbyStore = create<LobbyStore>((set) => ({
	...initialState,

	actions: {
		setLobby: (lobbyInfo, total) => {
			set({ lobbyInfo, total });
		},

		addLobby: (lobbyInfo) => {
			set((state) => ({
				lobbyInfo: [lobbyInfo, ...(state.lobbyInfo || [])],
				total: state.total + 1,
			}));
		},

		updateLobby: (lobbyInfo) => {
			set((state) => ({
				lobbyInfo:
					state.lobbyInfo?.map((l) =>
						l.lobby.id === lobbyInfo.lobby.id ? lobbyInfo : l
					) || state.lobbyInfo,
			}));
		},

		removeLobby: (lobbyId) => {
			set((state) => ({
				lobbyInfo:
					state.lobbyInfo?.filter((l) => l.lobby.id !== lobbyId) ||
					state.lobbyInfo,
				total: state.total - 1,
			}));
		},

		setConnected: (connected) => {
			set({ isConnected: connected });
		},

		setConnecting: (connecting) => {
			set({ isConnecting: connecting });
		},

		setError: (error) => {
			set({ error });
		},

		setActionLoading: (action, loading) => {
			set((state) => {
				const newLoadingActions = new Set(state.loadingActions);
				if (loading) {
					newLoadingActions.add(action);
				} else {
					newLoadingActions.delete(action);
				}
				return { loadingActions: newLoadingActions };
			});
		},

		clearActionLoading: (action) => {
			set((state) => {
				const newLoadingActions = new Set(state.loadingActions);
				newLoadingActions.delete(action);
				return { loadingActions: newLoadingActions };
			});
		},

		clearAllLoadingActions: () => {
			set({ loadingActions: new Set<string>() });
		},

		reset: () => {
			set(initialState);
		},
	},
}));

// Export individual state selectors
export const useLobbyInfo = () => useLobbyStore((state) => state.lobbyInfo);
export const useLobbyTotal = () => useLobbyStore((state) => state.total);
export const useLobbyConnected = () =>
	useLobbyStore((state) => state.isConnected);
export const useLobbyConnecting = () =>
	useLobbyStore((state) => state.isConnecting);
export const useLobbyError = () => useLobbyStore((state) => state.error);
export const useLobbyActions = () => useLobbyStore((state) => state.actions);

// Loading state selectors
export const useLobbyLoadingActions = () =>
	useLobbyStore((state) => state.loadingActions);
export const useIsLobbyActionLoading = (action: string) =>
	useLobbyStore((state) => state.loadingActions.has(action));
