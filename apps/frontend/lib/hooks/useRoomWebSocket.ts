import { useEffect, useRef, useState } from "react";
import { useAuthStore } from "@/lib/stores/auth";
import {
	ChatMessage,
	JoinRequest,
	LobbyExtended,
	PlayerState,
} from "@/lib/definitions";
import {
	RoomWebSocketClient,
	RoomServerMessage,
	RoomClientMessage,
} from "@/lib/websocket/roomClient";

interface UseRoomWebSocketOptions {
	lobbyPath: string;
	onError?: (error: Event | Error) => void;
	onClose?: () => void;
}

interface RoomState {
	lobby: LobbyExtended | null;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;
}

export function useRoomWebSocket({
	lobbyPath,
	onError,
	onClose,
}: UseRoomWebSocketOptions) {
	const { token } = useAuthStore();
	const clientRef = useRef<RoomWebSocketClient | null>(null);
	const [state, setState] = useState<RoomState>({
		lobby: null,
		players: [],
		joinRequests: [],
		chatHistory: [],
		isConnected: false,
		isConnecting: true,
		error: null,
	});

	useEffect(() => {
		const wsUrl = `${process.env.NEXT_PUBLIC_WS_URL}/ws/room/${lobbyPath}`;
		const client = new RoomWebSocketClient(lobbyPath, token || undefined);
		clientRef.current = client;

		// Set up message handler
		const unsubscribeMessage = client.onMessage(
			(message: RoomServerMessage) => {
				switch (message.type) {
					case "lobbyBootstrap":
						setState((prev) => ({
							...prev,
							lobby: message.lobby,
							players: message.players,
							joinRequests: message.joinRequests,
							chatHistory: message.chatHistory,
							isConnected: true,
							isConnecting: false,
						}));
						break;

					case "lobbyStateChanged":
						setState((prev) => ({
							...prev,
							lobby: prev.lobby
								? { ...prev.lobby, status: message.state }
								: null,
						}));
						break;

					case "playerJoined":
						// Player list will be updated via separate message or re-fetch
						break;

					case "playerLeft":
						setState((prev) => ({
							...prev,
							players: prev.players.filter(
								(p) => p.userId !== message.playerId
							),
						}));
						break;

					case "playerKicked":
						setState((prev) => ({
							...prev,
							players: prev.players.filter(
								(p) => p.userId !== message.playerId
							),
						}));
						break;

					case "joinRequestsUpdated":
						setState((prev) => ({
							...prev,
							joinRequests: message.joinRequests,
						}));
						break;

					case "messageReceived":
						setState((prev) => ({
							...prev,
							chatHistory: [...prev.chatHistory, message.message],
						}));
						break;

					case "reactionAdded":
						setState((prev) => ({
							...prev,
							chatHistory: prev.chatHistory.map((msg) =>
								msg.id === message.messageId
									? {
											...msg,
											reactions: {
												...msg.reactions,
												[message.emoji]: [
													...(msg.reactions[
														message.emoji
													] || []),
													message.userId,
												],
											},
										}
									: msg
							),
						}));
						break;

					case "reactionRemoved":
						setState((prev) => ({
							...prev,
							chatHistory: prev.chatHistory.map((msg) =>
								msg.id === message.messageId
									? {
											...msg,
											reactions: {
												...msg.reactions,
												[message.emoji]: (
													msg.reactions[
														message.emoji
													] || []
												).filter(
													(uid) =>
														uid !== message.userId
												),
											},
										}
									: msg
							),
						}));
						break;

					case "startCountdown":
						// Handle countdown UI updates
						console.log(
							`Game starting in ${message.secondsRemaining} seconds`
						);
						break;

					case "error":
						setState((prev) => ({
							...prev,
							error: message.message,
							isConnecting: false,
						}));
						break;

					case "pong":
						// Handle pong for latency tracking
						break;

					default:
						console.log("[RoomWS] Unhandled message:", message);
				}
			}
		);

		// Set up error handler
		const unsubscribeError = client.onError((error) => {
			setState((prev) => ({
				...prev,
				error:
					error instanceof Error ? error.message : "Connection error",
				isConnected: false,
				isConnecting: false,
			}));
			onError?.(error);
		});

		// Set up close handler
		const unsubscribeClose = client.onClose(() => {
			setState((prev) => ({
				...prev,
				isConnected: false,
			}));
			onClose?.();
		});

		// Connect
		client.connect(wsUrl).catch((err) => {
			setState((prev) => ({
				...prev,
				error: err.message || "Failed to connect",
				isConnecting: false,
			}));
		});

		// Cleanup on unmount
		return () => {
			unsubscribeMessage();
			unsubscribeError();
			unsubscribeClose();
			client.disconnect();
		};
	}, [lobbyPath, token, onError, onClose]);

	const sendMessage = (message: RoomClientMessage) => {
		clientRef.current?.send(message);
	};

	return {
		...state,
		sendMessage,
	};
}
