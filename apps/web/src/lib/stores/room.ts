/**
 * Lobby Store
 *
 * Manages lobby-level state (players, chat, join requests, lobby info).
 * This is shared across all games.
 */

import { create } from "zustand";
import type {
	ChatMessage,
	Game,
	JoinRequest,
	LobbyBootstrapMessage,
	LobbyExtended,
	LobbyStatus,
	PlayerState,
	User,
} from "@/lib/definitions";

interface LobbyActions {
	setBootstrap: (data: LobbyBootstrapMessage) => void;
	setLobby: (lobby: LobbyExtended) => void;
	setGame: (game: Game) => void;
	setCreator: (creator: User) => void;
	setPlayers: (players: PlayerState[]) => void;
	setJoinRequests: (requests: JoinRequest[]) => void;
	setChatHistory: (history: ChatMessage[]) => void;
	addChatMessage: (message: ChatMessage) => void;
	addReaction: (messageId: string, userId: string, emoji: string) => void;
	removeReaction: (messageId: string, userId: string, emoji: string) => void;
	addPlayer: (userId: string) => void;
	removePlayer: (userId: string) => void;
	updateLobbyStatus: (status: LobbyStatus) => void;
	setCountdown: (seconds: number | null) => void;
	setConnected: (connected: boolean) => void;
	setConnecting: (connecting: boolean) => void;
	setError: (error: string | null) => void;
	reset: () => void;
}

interface LobbyStore {
	// State
	lobby: LobbyExtended | null;
	game: Game | null;
	creator: User | null;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
	countdown: number | null;
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;

	// Actions
	actions: LobbyActions;
}

const initialState = {
	lobby: null,
	game: null,
	creator: null,
	players: [],
	joinRequests: [],
	chatHistory: [],
	countdown: null,
	isConnected: false,
	isConnecting: false,
	error: null,
};

export const useLobbyStore = create<LobbyStore>((set) => ({
	...initialState,

	actions: {
		setBootstrap: (data) => {
			set({
				lobby: data.lobbyInfo.lobby,
				game: data.lobbyInfo.game,
				creator: data.lobbyInfo.creator,
				players: data.players || [],
				joinRequests: data.joinRequests || [],
				chatHistory: data.chatHistory || [],
			});
		},

		setLobby: (lobby) => {
			set({ lobby });
		},

		setGame: (game) => {
			set({ game });
		},

		setCreator: (creator) => {
			set({ creator });
		},

		setPlayers: (players) => {
			set({ players });
		},

		setJoinRequests: (requests) => {
			set({ joinRequests: requests });
		},

		setChatHistory: (history) => {
			set({ chatHistory: history });
		},

		addChatMessage: (message) => {
			set((state) => ({
				chatHistory: [...state.chatHistory, message],
			}));
		},

		addReaction: (messageId, userId, emoji) => {
			set((state) => ({
				chatHistory: state.chatHistory.map((msg) => {
					if (msg.messageId === messageId) {
						const reactions = msg.reactions || [];
						// Add reaction if not already present
						const alreadyReacted = reactions.some(
							(r) => r.userId === userId && r.emoji === emoji
						);
						if (!alreadyReacted) {
							return {
								...msg,
								reactions: [...reactions, { userId, emoji }],
							};
						}
					}
					return msg;
				}),
			}));
		},

		removeReaction: (messageId, userId, emoji) => {
			set((state) => ({
				chatHistory: state.chatHistory.map((msg) => {
					if (msg.messageId === messageId) {
						return {
							...msg,
							reactions: msg.reactions.filter(
								(r) =>
									!(r.userId === userId && r.emoji === emoji)
							),
						};
					}
					return msg;
				}),
			}));
		},

		addPlayer: (userId) => {
			set((state) => {
				// Check if player already exists
				if (state.players.some((p) => p.userId === userId)) {
					return state;
				}
				// Create a basic player state (lobbyId will be set by server)
				const newPlayer: PlayerState = {
					userId,
					lobbyId: state.lobby?.id || "",
					state: "accepted",
					isCreator: false,
					joinedAt: Date.now(),
					status: "joined",
					updatedAt: Date.now(),
					// These can be updated later
					walletAddress: "",
					trustRating: 0,
				};
				return { players: [...state.players, newPlayer] };
			});
		},

		removePlayer: (userId) => {
			set((state) => ({
				players: state.players.filter((p) => p.userId !== userId),
			}));
		},

		updateLobbyStatus: (status) => {
			set((state) => ({
				lobby: state.lobby ? { ...state.lobby, status } : null,
			}));
		},

		setCountdown: (seconds) => {
			set({ countdown: seconds });
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

		reset: () => {
			set(initialState);
		},
	},
}));

// Export individual state selectors
export const useLobby = () => useLobbyStore((state) => state.lobby);
export const useGame = () => useLobbyStore((state) => state.game);
export const useCreator = () => useLobbyStore((state) => state.creator);
export const usePlayers = () => useLobbyStore((state) => state.players);
export const useJoinRequests = () =>
	useLobbyStore((state) => state.joinRequests);
export const useChatHistory = () => useLobbyStore((state) => state.chatHistory);
export const useCountdown = () => useLobbyStore((state) => state.countdown);
export const useRoomConnected = () =>
	useLobbyStore((state) => state.isConnected);
export const useRoomConnecting = () =>
	useLobbyStore((state) => state.isConnecting);
export const useRoomError = () => useLobbyStore((state) => state.error);
export const useLobbyActions = () => useLobbyStore((state) => state.actions);
