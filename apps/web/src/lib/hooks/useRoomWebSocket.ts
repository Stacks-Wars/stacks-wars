/**
 * Game Engine Hook
 *
 * Central hook that manages:
 * - Single WebSocket connection
 * - Message routing to lobby or game handlers
 * - Lobby state management
 * - Game plugin loading
 */

import { useEffect, useRef, useState } from "react";
import { getGamePlugin } from "@/app/game/registry";
import type {
	ChatMessage,
	Game,
	GameMessage,
	GamePlugin,
	JoinRequest,
	LobbyExtended,
	PlayerState,
	RoomClientMessage,
	RoomServerMessage,
	User,
} from "@/lib/definitions";
import {
	useLobby,
	useGame,
	useCreator,
	usePlayers,
	useJoinRequests,
	useChatHistory,
	useRoomConnected,
	useRoomError,
	useRoomConnecting,
	useLobbyActions,
} from "../stores/room";
import { useUser } from "../stores/user";
import { WebSocketClient } from "../websocket/wsClient";

interface UseRoomOptions {
	lobbyPath: string;
	wsUrl?: string;
}

export interface UseRoomWebSocketReturn {
	// Connection state
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;
	// Auth state
	user: User | null;
	isAuthenticated: boolean;
	// Lobby state
	lobby: LobbyExtended | null;
	game: Game | null;
	creator: User | null;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
	// Game state
	gameState: unknown;
	gamePlugin: GamePlugin | undefined;
	// Actions
	sendGameMessage: (type: string, payload: unknown) => void;
	sendLobbyMessage: (message: RoomClientMessage) => void;
}

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001";

export function useRoomWebSocket({
	lobbyPath,
	wsUrl = `${WS_URL}/ws/room`,
}: UseRoomOptions): UseRoomWebSocketReturn {
	const clientRef = useRef<WebSocketClient | null>(null);
	const [gamePlugin, setGamePlugin] = useState<GamePlugin | undefined>();
	const [gameState, setGameState] = useState<unknown>(null);
	const [authenticatedUserId, setAuthenticatedUserId] = useState<string | null>(null);
	const [isCheckingAuth, setIsCheckingAuth] = useState(true);

	const lobby = useLobby();
	const game = useGame();
	const creator = useCreator();
	const players = usePlayers();
	const joinRequests = useJoinRequests();
	const chatHistory = useChatHistory();
	const isConnected = useRoomConnected();
	const isConnecting = useRoomConnecting();
	const error = useRoomError();
	const lobbyActions = useLobbyActions();
	const user = useUser();

	const isAuthenticated = !isCheckingAuth && !!authenticatedUserId && !!user;

	// Check authentication status
	useEffect(() => {
		async function checkAuth() {
			try {
				const response = await fetch("/api/auth/me");
				const data = await response.json();
				setAuthenticatedUserId(data.userId);
			} catch (error) {
				console.error("Failed to check authentication:", error);
				setAuthenticatedUserId(null);
			} finally {
				setIsCheckingAuth(false);
			}
		}

		checkAuth();
	}, []);

	useEffect(() => {
		// Initialize WebSocket connection
		const client = new WebSocketClient();
		clientRef.current = client;
		lobbyActions.setConnecting(true);
		lobbyActions.setError(null);

		// Connect to WebSocket
		client
			.connect(`${wsUrl}/${lobbyPath}`)
			.then(() => {
				lobbyActions.setConnected(true);
				lobbyActions.setConnecting(false);
			})
			.catch((err) => {
				console.error("[Room] Connection failed:", err);
				lobbyActions.setError("Failed to connect to game server");
				lobbyActions.setConnecting(false);
			});

		// Message router
		const unsubscribe = client.onMessage((message: unknown) => {
			try {
				// Try to parse as wrapped game message
				const msg = message as {
					game?: string;
					type: string;
					payload?: unknown;
				};

				if (msg.game && gamePlugin && msg.game === gamePlugin.path) {
					// Route to game plugin
					console.log(
						`[Room] Routing message to game: ${msg.game}`,
						msg
					);
					setGameState((prevState: unknown) =>
						gamePlugin.handleMessage(prevState, msg as GameMessage)
					);
				} else {
					// Route to lobby handler
					handleLobbyMessage(msg as RoomServerMessage);
				}
			} catch (err) {
				console.error("[Room] Failed to handle message:", err);
			}
		});

		// Error handler
		const unsubError = client.onError((err) => {
			console.error("[Room] WebSocket error:", err);
			lobbyActions.setError("Connection error");
		});

		// Close handler
		const unsubClose = client.onClose(() => {
			lobbyActions.setConnected(false);
		});

		// Cleanup
		return () => {
			unsubscribe();
			unsubError();
			unsubClose();
			client.disconnect();
			lobbyActions.reset();
			setGamePlugin(undefined);
			setGameState(null);
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [lobbyPath]);

	// Handle lobby-level messages
	const handleLobbyMessage = (message: RoomServerMessage) => {
		if (message.type !== "pong")
			console.log("[Room] Handling lobby message:", message);

		switch (message.type) {
			case "lobbyBootstrap": {
				lobbyActions.setBootstrap(message);

				// Load game plugin based on lobby's gamePath
				if (message.lobbyInfo.lobby.gamePath) {
					const plugin = getGamePlugin(
						message.lobbyInfo.lobby.gamePath
					);
					if (plugin) {
						setGamePlugin(plugin);
						setGameState(plugin.createInitialState());
						console.log("[Room] Loaded game plugin:", plugin.id);
					} else {
						console.warn(
							`[Room] No plugin found for game: ${message.lobbyInfo.lobby.gamePath}`
						);
					}
				}
				break;
			}

			case "lobbyStatusChanged":
				lobbyActions.updateLobbyStatus(message.status);
				break;

			case "startCountdown":
				lobbyActions.setCountdown(message.secondsRemaining);
				break;

			case "playerJoined":
				lobbyActions.addPlayer(message.userId);
				break;

			case "playerLeft":
				lobbyActions.removePlayer(message.userId);
				break;

			case "playerKicked":
				lobbyActions.removePlayer(message.userId);
				break;

			case "joinRequestsUpdated":
				lobbyActions.setJoinRequests(message.joinRequests);
				break;

			case "joinRequestStatus":
				console.log(`join request status changed: ${message.accepted}`);
				break;

			case "messageReceived":
				lobbyActions.addChatMessage(message.message);
				break;

			case "reactionAdded":
				lobbyActions.addReaction(
					message.messageId,
					message.userId,
					message.emoji
				);
				break;

			case "reactionRemoved":
				lobbyActions.removeReaction(
					message.messageId,
					message.userId,
					message.emoji
				);
				break;

			case "playerUpdated":
				lobbyActions.setPlayers(message.players);
				break;

			case "error":
				lobbyActions.setError(message.message || "An error occurred");
				break;

			case "pong":
				console.log(`pong: ${message.elapsedMs}`);
				break;
			default:
				console.warn("[Room] Unhandled lobby message:", message);
		}
	};

	// Send a game-specific message
	const sendGameMessage = (type: string, payload: unknown) => {
		if (!clientRef.current || !gamePlugin) {
			console.warn(
				"[Room] Cannot send game message: not connected or no plugin"
			);
			return;
		}
		clientRef.current.sendGameMessage(gamePlugin.path, type, payload);
	};

	// Send a lobby-level message
	const sendLobbyMessage = (message: RoomClientMessage) => {
		if (!clientRef.current) {
			console.warn("[Room] Cannot send lobby message: not connected");
			return;
		}
		const { type, ...payload } = message;
		clientRef.current.sendLobbyMessage(type, payload);
	};

	return {
		// Connection state
		isConnected,
		isConnecting,
		error,

		// Auth state
		user,
		isAuthenticated,

		// Lobby state
		lobby,
		game,
		creator,
		players,
		joinRequests,
		chatHistory,

		// Game state
		gameState,
		gamePlugin,

		// Actions
		sendGameMessage,
		sendLobbyMessage,
	};
}
