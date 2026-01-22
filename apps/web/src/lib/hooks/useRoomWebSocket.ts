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
	useCountdown,
	useRoomConnected,
	useRoomError,
	useRoomConnecting,
	useRoomLatency,
	useLobbyActions,
} from "../stores/room";
import { useUser } from "../stores/user";
import { WebSocketClient } from "../websocket/wsClient";
import { toast } from "sonner";

interface UseRoomOptions {
	lobbyPath: string;
	wsUrl?: string;
	onActionSuccess?: (action: string, message?: string) => void;
	onActionError?: (
		action: string,
		error: { code: string; message: string }
	) => void;
}

export interface UseRoomWebSocketReturn {
	// Connection state
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;
	latency: number | null;
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
	countdown: number | null;
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
	onActionSuccess,
	onActionError,
}: UseRoomOptions): UseRoomWebSocketReturn {
	const clientRef = useRef<WebSocketClient | null>(null);
	const [gamePlugin, setGamePlugin] = useState<GamePlugin | undefined>();
	const [gameState, setGameState] = useState<unknown>(null);
	const [authenticatedUserId, setAuthenticatedUserId] = useState<
		string | null
	>(null);
	const [isCheckingAuth, setIsCheckingAuth] = useState(true);
	const pendingActionsRef = useRef<Set<string>>(new Set());

	const lobby = useLobby();
	const game = useGame();
	const creator = useCreator();
	const players = usePlayers();
	const joinRequests = useJoinRequests();
	const chatHistory = useChatHistory();
	const countdown = useCountdown();
	const isConnected = useRoomConnected();
	const isConnecting = useRoomConnecting();
	const error = useRoomError();
	const latency = useRoomLatency();
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
				lobbyActions.updateLobbyStatus(
					message.status,
					message.participantCount,
					message.currentAmount
				);
				if (pendingActionsRef.current.has("updateLobbyStatus")) {
					pendingActionsRef.current.delete("updateLobbyStatus");
					lobbyActions.clearActionLoading("updateLobbyStatus");
					onActionSuccess?.(
						"updateLobbyStatus",
						`Lobby status updated to ${message.status}`
					);
				}
				break;

			case "startCountdown":
				lobbyActions.setCountdown(message.secondsRemaining);
				break;

			case "playerJoined":
				if (pendingActionsRef.current.has("join")) {
					pendingActionsRef.current.delete("join");
					lobbyActions.clearActionLoading("join");
					onActionSuccess?.("join", "Successfully joined lobby");
				}
				break;

			case "playerLeft":
				lobbyActions.removePlayer(message.userId);
				if (pendingActionsRef.current.has("leave")) {
					pendingActionsRef.current.delete("leave");
					lobbyActions.clearActionLoading("leave");
					onActionSuccess?.("leave", "Left lobby");
				}
				break;

			case "playerKicked":
				lobbyActions.removePlayer(message.userId);
				if (pendingActionsRef.current.has(`kick-${message.userId}`)) {
					pendingActionsRef.current.delete(`kick-${message.userId}`);
					lobbyActions.clearActionLoading(`kick-${message.userId}`);
					onActionSuccess?.("kick", "Player kicked");
				}
				break;

			case "joinRequestsUpdated":
				lobbyActions.setJoinRequests(message.joinRequests);
				// Check for pending actions
				if (pendingActionsRef.current.has("joinRequest")) {
					const userInList = message.joinRequests.some(
						(jr) => jr.userId === user?.id
					);
					if (userInList) {
						pendingActionsRef.current.delete("joinRequest");
						lobbyActions.clearActionLoading("joinRequest");
						onActionSuccess?.("joinRequest", "Join request sent");
					}
				}
				pendingActionsRef.current.forEach((action) => {
					if (
						action.startsWith("approve-") ||
						action.startsWith("reject-")
					) {
						pendingActionsRef.current.delete(action);
						lobbyActions.clearActionLoading(action);
					}
				});
				break;

			case "joinRequestStatus":
				lobbyActions.updateJoinRequestState(
					message.userId,
					message.accepted ? "accepted" : "rejected"
				);
				if (message.userId === user?.id) {
					if (message.accepted) {
						onActionSuccess?.(
							"joinRequestAccepted",
							"Your join request was approved! You can now join the lobby."
						);
					} else {
						onActionError?.("joinRequestRejected", {
							code: "JOIN_REQUEST_REJECTED",
							message: "Your join request was declined",
						});
					}
				}
				break;

			case "messageReceived":
				lobbyActions.addChatMessage(message.message);
				if (pendingActionsRef.current.has("sendMessage")) {
					pendingActionsRef.current.delete("sendMessage");
					lobbyActions.clearActionLoading("sendMessage");
				}
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
				// Map error codes to actions
				const errorCodeToAction: Record<string, string> = {
					JOIN_FAILED: "join",
					LEAVE_FAILED: "leave",
					LOBBY_STATUS_FAILED: "updateLobbyStatus",
					APPROVE_FAILED: "approve",
					REJECT_FAILED: "reject",
					KICK_FAILED: "kick",
					SEND_MESSAGE_FAILED: "sendMessage",
					REACTION_FAILED: "reaction",
				};
				const action = errorCodeToAction[message.code];
				if (action) {
					// Clear any pending actions related to this error
					pendingActionsRef.current.forEach((pendingAction) => {
						if (
							pendingAction.startsWith(action) ||
							pendingAction === action
						) {
							pendingActionsRef.current.delete(pendingAction);
							lobbyActions.clearActionLoading(pendingAction);
						}
					});
					onActionError?.(action, {
						code: message.code,
						message: message.message,
					});
				}
				break;

			case "pong":
				lobbyActions.setLatency(message.elapsedMs);
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
			toast.warning("Failed to send request", {
				description: "Not connected to lobby",
			});
			console.warn("[Room] Cannot send lobby message: not connected");
			return;
		}

		// Track pending actions
		switch (message.type) {
			case "join":
				pendingActionsRef.current.add("join");
				lobbyActions.setActionLoading("join", true);
				break;
			case "leave":
				console.log(`Sending leave: `);
				pendingActionsRef.current.add("leave");
				lobbyActions.setActionLoading("leave", true);
				break;
			case "updateLobbyStatus":
				pendingActionsRef.current.add("updateLobbyStatus");
				lobbyActions.setActionLoading("updateLobbyStatus", true);
				break;
			case "joinRequest":
				pendingActionsRef.current.add("joinRequest");
				lobbyActions.setActionLoading("joinRequest", true);
				break;
			case "approveJoin":
				pendingActionsRef.current.add(`approve-${message.userId}`);
				lobbyActions.setActionLoading(
					`approve-${message.userId}`,
					true
				);
				break;
			case "rejectJoin":
				pendingActionsRef.current.add(`reject-${message.userId}`);
				lobbyActions.setActionLoading(`reject-${message.userId}`, true);
				break;
			case "kick":
				pendingActionsRef.current.add(`kick-${message.userId}`);
				lobbyActions.setActionLoading(`kick-${message.userId}`, true);
				break;
			case "sendMessage":
				console.log(`Sending message: ${message.content}`);
				pendingActionsRef.current.add("sendMessage");
				lobbyActions.setActionLoading("sendMessage", true);
				break;
			case "addReaction":
				pendingActionsRef.current.add(
					`addReaction-${message.messageId}`
				);
				lobbyActions.setActionLoading(
					`addReaction-${message.messageId}`,
					true
				);
				break;
			case "removeReaction":
				pendingActionsRef.current.add(
					`removeReaction-${message.messageId}`
				);
				lobbyActions.setActionLoading(
					`removeReaction-${message.messageId}`,
					true
				);
				break;
		}

		clientRef.current.sendLobbyMessage(message);
	};

	return {
		// Connection state
		isConnected,
		isConnecting,
		error,
		latency,

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
		countdown,

		// Game state
		gameState,
		gamePlugin,

		// Actions
		sendGameMessage,
		sendLobbyMessage,
	};
}
