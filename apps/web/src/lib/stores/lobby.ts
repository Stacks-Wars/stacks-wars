/**
 * Lobby Store
 *
 * Manages lobby-level state (players, chat, join requests, lobby info).
 * This is shared across all games.
 */

import { create } from "zustand";
import type {
	ChatMessage,
	JoinRequest,
	LobbyExtended,
	LobbyStatus,
	PlayerState,
} from "@/lib/definitions";

interface LobbyStore {
	// State
	lobby: LobbyExtended | null;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;

	// Actions
	setLobby: (lobby: LobbyExtended) => void;
	setPlayers: (players: PlayerState[]) => void;
	setJoinRequests: (requests: JoinRequest[]) => void;
	setChatHistory: (history: ChatMessage[]) => void;
	addChatMessage: (message: ChatMessage) => void;
	addPlayer: (playerId: string) => void;
	removePlayer: (playerId: string) => void;
	updateLobbyStatus: (status: LobbyStatus) => void;
	setConnected: (connected: boolean) => void;
	setConnecting: (connecting: boolean) => void;
	setError: (error: string | null) => void;
	reset: () => void;
}

const initialState = {
	lobby: null,
	players: [],
	joinRequests: [],
	chatHistory: [],
	isConnected: false,
	isConnecting: false,
	error: null,
};

export const useLobbyStore = create<LobbyStore>((set) => ({
	...initialState,

	setLobby: (lobby) => set({ lobby }),

	setPlayers: (players) => set({ players }),

	setJoinRequests: (requests) => set({ joinRequests: requests }),

	setChatHistory: (history) => set({ chatHistory: history }),

	addChatMessage: (message) =>
		set((state) => ({
			chatHistory: [...state.chatHistory, message],
		})),

	addPlayer: (playerId) =>
		set((state) => {
			// Check if player already exists
			if (state.players.some((p) => p.userId === playerId)) {
				return state;
			}
			// Create a basic player state (lobbyId will be set by server)
			const newPlayer: PlayerState = {
				userId: playerId,
				lobbyId: state.lobby?.id || "",
				isCreator: false,
				joinedAt: Date.now(),
				status: "joined",
				updatedAt: Date.now(),
				// These can be updated later
				walletAddress: "",
				trustRating: 0,
			};
			return { players: [...state.players, newPlayer] };
		}),

	removePlayer: (playerId) =>
		set((state) => ({
			players: state.players.filter((p) => p.userId !== playerId),
		})),

	updateLobbyStatus: (status) =>
		set((state) => ({
			lobby: state.lobby ? { ...state.lobby, status } : null,
		})),

	setConnected: (connected) => set({ isConnected: connected }),

	setConnecting: (connecting) => set({ isConnecting: connecting }),

	setError: (error) => set({ error }),

	reset: () => set(initialState),
}));
