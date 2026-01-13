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
	addPlayer: (playerId: string) => void;
	removePlayer: (playerId: string) => void;
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
			console.log("[Lobby Store] 🚀 setBootstrap", {
				lobby: data.lobbyInfo.lobby.name,
				game: data.lobbyInfo.game.name,
				playersCount: data.players.length,
				joinRequestsCount: data.joinRequests.length,
				chatHistoryCount: data.chatHistory.length,
			});
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
			console.log("[Lobby Store] 🏠 setLobby", lobby.name);
			set({ lobby });
		},

		setGame: (game) => {
			console.log("[Lobby Store] 🎮 setGame", game.name);
			set({ game });
		},

		setCreator: (creator) => {
			console.log("[Lobby Store] 👤 setCreator", creator.username);
			set({ creator });
		},

		setPlayers: (players) => {
			console.log("[Lobby Store] 👥 setPlayers", players.length);
			set({ players });
		},

		setJoinRequests: (requests) => {
			console.log("[Lobby Store] 📋 setJoinRequests", requests.length);
			set({ joinRequests: requests });
		},

		setChatHistory: (history) => {
			console.log("[Lobby Store] 💬 setChatHistory", history.length);
			set({ chatHistory: history });
		},

		addChatMessage: (message) => {
			console.log("[Lobby Store] ➕💬 addChatMessage", {
				user: message.senderId,
				content: message.content.substring(0, 50),
			});
			set((state) => ({
				chatHistory: [...state.chatHistory, message],
			}));
		},

		addReaction: (messageId, userId, emoji) => {
			console.log("[Lobby Store] ➕😀 addReaction", {
				messageId,
				userId,
				emoji,
			});
			set((state) => ({
				chatHistory: state.chatHistory.map((msg) => {
					if (msg.id === messageId) {
						const reactions = msg.reactions || {};
						const userIds = reactions[emoji] || [];
						// Add user if not already present
						if (!userIds.includes(userId)) {
							return {
								...msg,
								reactions: {
									...reactions,
									[emoji]: [...userIds, userId],
								},
							};
						}
					}
					return msg;
				}),
			}));
		},

		removeReaction: (messageId, userId, emoji) => {
			console.log("[Lobby Store] ➖😀 removeReaction", {
				messageId,
				userId,
				emoji,
			});
			set((state) => ({
				chatHistory: state.chatHistory.map((msg) => {
					if (msg.id === messageId) {
						const reactions = msg.reactions || {};
						const userIds = reactions[emoji] || [];
						const newUserIds = userIds.filter(
							(id) => id !== userId
						);

						// Remove emoji key if no users left
						if (newUserIds.length === 0) {
							const { [emoji]: _, ...remainingReactions } =
								reactions;
							return {
								...msg,
								reactions: remainingReactions,
							};
						}

						return {
							...msg,
							reactions: {
								...reactions,
								[emoji]: newUserIds,
							},
						};
					}
					return msg;
				}),
			}));
		},

		addPlayer: (playerId) => {
			console.log("[Lobby Store] ➕👤 addPlayer", playerId);
			set((state) => {
				// Check if player already exists
				if (state.players.some((p) => p.userId === playerId)) {
					console.log(
						"[Lobby Store] ⚠️ Player already exists, skipping"
					);
					return state;
				}
				// Create a basic player state (lobbyId will be set by server)
				const newPlayer: PlayerState = {
					userId: playerId,
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

		removePlayer: (playerId) => {
			console.log("[Lobby Store] ➖👤 removePlayer", playerId);
			set((state) => ({
				players: state.players.filter((p) => p.userId !== playerId),
			}));
		},

		updateLobbyStatus: (status) => {
			console.log("[Lobby Store] 🔄 updateLobbyStatus", status);
			set((state) => ({
				lobby: state.lobby ? { ...state.lobby, status } : null,
			}));
		},

		setCountdown: (seconds) => {
			console.log("[Lobby Store] ⏱️ setCountdown", seconds);
			set({ countdown: seconds });
		},

		setConnected: (connected) => {
			console.log("[Lobby Store] 🔌 setConnected", connected);
			set({ isConnected: connected });
		},

		setConnecting: (connecting) => {
			console.log("[Lobby Store] 🔄 setConnecting", connecting);
			set({ isConnecting: connecting });
		},

		setError: (error) => {
			console.log("[Lobby Store] ❌ setError", error);
			set({ error });
		},

		reset: () => {
			console.log("[Lobby Store] 🔄 reset");
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
