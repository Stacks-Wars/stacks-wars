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
	GameMessage,
	GamePlugin,
	JoinRequest,
	LobbyExtended,
	LobbyMessage,
	LobbyStatus,
	PlayerState,
} from "@/lib/definitions";
import { useAuthStore } from "@/lib/stores/auth";
import { useLobbyStore } from "../stores/lobby";
import { webSocketClient } from "../websocket/wsClient";

interface UseRoomOptions {
	lobbyPath: string;
	wsUrl?: string;
}

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001";

export function useRoomWebSocket({
	lobbyPath,
	wsUrl = `${WS_URL}/ws/room`,
}: UseRoomOptions) {
	const { token } = useAuthStore();
	const clientRef = useRef<webSocketClient | null>(null);
	const [gamePlugin, setGamePlugin] = useState<GamePlugin | undefined>();
	const [gameState, setGameState] = useState<unknown>(null);

	const {
		setLobby,
		setPlayers,
		setJoinRequests,
		setChatHistory,
		addChatMessage,
		addPlayer,
		removePlayer,
		updateLobbyStatus,
		setConnected,
		setConnecting,
		setError,
		reset,
		isConnected,
		isConnecting,
		error,
		lobby,
		players,
		joinRequests,
		chatHistory,
	} = useLobbyStore();

	useEffect(() => {
		// Initialize WebSocket connection
		const client = new webSocketClient(lobbyPath, token || undefined);
		clientRef.current = client;
		setConnecting(true);
		setError(null);

		// Connect to WebSocket
		client
			.connect(`${wsUrl}/${lobbyPath}`)
			.then(() => {
				setConnected(true);
				setConnecting(false);
			})
			.catch((err) => {
				console.error("[Room] Connection failed:", err);
				setError("Failed to connect to game server");
				setConnecting(false);
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

				if (msg.game && gamePlugin && msg.game === gamePlugin.id) {
					// Route to game plugin
					console.log(`[Room] Routing message to game: ${msg.game}`, msg);
					setGameState((prevState: unknown) =>
						gamePlugin.handleMessage(prevState, msg as GameMessage),
					);
				} else {
					// Route to lobby handler
					handleLobbyMessage(msg as LobbyMessage);
				}
			} catch (err) {
				console.error("[Room] Failed to handle message:", err);
			}
		});

		// Error handler
		const unsubError = client.onError((err) => {
			console.error("[Room] WebSocket error:", err);
			setError("Connection error");
		});

		// Close handler
		const unsubClose = client.onClose(() => {
			setConnected(false);
		});

		// Cleanup
		return () => {
			unsubscribe();
			unsubError();
			unsubClose();
			client.disconnect();
			reset();
			setGamePlugin(undefined);
			setGameState(null);
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [lobbyPath, token]);

	// Handle lobby-level messages
	const handleLobbyMessage = (message: Record<string, unknown>) => {
		console.log("[Room] Handling lobby message:", message);

		switch (message.type) {
			case "lobbyBootstrap": {
				const lobby = message.lobby as LobbyExtended;
				const players = (message.players || []) as PlayerState[];
				const joinRequests = (message.join_requests || []) as JoinRequest[];
				const chatHistory = (message.chat_history || []) as ChatMessage[];

				setLobby(lobby);
				setPlayers(players);
				setJoinRequests(joinRequests);
				setChatHistory(chatHistory);

				// Load game plugin based on lobby's gamePath
				if (lobby.gamePath) {
					const plugin = getGamePlugin(lobby.gamePath);
					if (plugin) {
						setGamePlugin(plugin);
						setGameState(plugin.createInitialState());
						console.log("[Room] Loaded game plugin:", plugin.id);
					} else {
						console.warn(`[Room] No plugin found for game: ${lobby.gamePath}`);
					}
				}
				break;
			}

			case "lobbyStateChanged":
				updateLobbyStatus(message.state as LobbyStatus);
				break;

			case "playerJoined":
				addPlayer(message.player_id as string);
				break;

			case "playerLeft":
			case "playerKicked":
				removePlayer(message.player_id as string);
				break;

			case "joinRequestsUpdated":
				setJoinRequests((message.join_requests || []) as JoinRequest[]);
				break;

			case "messageReceived":
				addChatMessage(message.message as ChatMessage);
				break;

			case "playerUpdated":
				// Handle full player list update
				if (message.players) {
					setPlayers(message.players as PlayerState[]);
				}
				break;

			case "error":
				setError((message.message as string) || "An error occurred");
				break;

			default:
				console.warn("[Room] Unhandled lobby message:", message);
		}
	};

	// Send a game-specific message
	const sendGameMessage = (type: string, payload: unknown) => {
		if (!clientRef.current || !gamePlugin) {
			console.warn(
				"[Room] Cannot send game message: not connected or no plugin",
			);
			return;
		}
		clientRef.current.sendGameMessage(gamePlugin.id, type, payload);
	};

	// Send a lobby-level message
	const sendLobbyMessage = (type: string, payload?: unknown) => {
		if (!clientRef.current) {
			console.warn("[Room] Cannot send lobby message: not connected");
			return;
		}
		clientRef.current.sendLobbyMessage(type, payload);
	};

	return {
		// Connection state
		isConnected,
		isConnecting,
		error,

		// Lobby state
		lobby,
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
