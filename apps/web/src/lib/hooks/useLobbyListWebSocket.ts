import { useEffect, useRef, useState, useCallback } from "react";
import { WebSocketClient } from "@/lib/websocket/wsClient";
import type { LobbyExtended, LobbyStatus } from "@/lib/definitions";

const WS_BASE_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001";

interface LobbyListMessage {
	type: string;
	lobbies?: LobbyExtended[];
	total?: number;
	lobby?: LobbyExtended;
	lobbyId?: string;
	error?: {
		code: string;
		message: string;
	};
}

interface UseLobbyListWebSocketOptions {
	statusFilter?: LobbyStatus[];
	limit?: number;
}

export function useLobbyListWebSocket(
	options: UseLobbyListWebSocketOptions = {}
) {
	const { statusFilter = ["waiting", "inProgress"], limit = 12 } = options;
	const [lobbies, setLobbies] = useState<LobbyExtended[] | null>(null);
	const [total, setTotal] = useState(0);
	const [isConnected, setIsConnected] = useState(false);
	const [isConnecting, setIsConnecting] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const wsRef = useRef<WebSocketClient | null>(null);

	const buildWebSocketUrl = useCallback((statuses: LobbyStatus[]) => {
		const statusParam = statuses.join(",");
		return `${WS_BASE_URL}/ws/lobbies?status=${statusParam}`;
	}, []);

	const subscribe = useCallback(
		(statuses: LobbyStatus[]) => {
			if (wsRef.current?.isConnected()) {
				wsRef.current.send({
					type: "subscribe",
					status: statuses,
					limit,
				});
			}
		},
		[limit]
	);

	const loadMore = useCallback((offset: number) => {
		if (wsRef.current?.isConnected()) {
			wsRef.current.send({
				type: "loadMore",
				offset,
			});
		}
	}, []);

	useEffect(() => {
		const ws = new WebSocketClient();
		wsRef.current = ws;

		const wsUrl = buildWebSocketUrl(statusFilter);

		// Handle messages
		const handleMessage = (message: unknown) => {
			const msg = message as LobbyListMessage;

			switch (msg.type) {
				case "lobbyList":
					if (msg.lobbies && msg.total !== undefined) {
						setLobbies(msg.lobbies);
						setTotal(msg.total);
					}
					break;

				case "lobbyCreated":
					if (msg.lobby) {
						setLobbies((prev) => [msg.lobby!, ...(prev || [])]);
						setTotal((prev) => prev + 1);
					}
					break;

				case "lobbyUpdated":
					if (msg.lobby) {
						setLobbies(
							(prev) =>
								prev?.map((l) =>
									l.id === msg.lobby!.id ? msg.lobby! : l
								) || prev
						);
					}
					break;

				case "lobbyRemoved":
					if (msg.lobbyId) {
						setLobbies(
							(prev) =>
								prev?.filter((l) => l.id !== msg.lobbyId) ||
								prev
						);
						setTotal((prev) => prev - 1);
					}
					break;

				case "error":
					if (msg.error) {
						setError(msg.error.message);
					}
					break;
			}
		};

		const handleError = (err: Event | Error) => {
			console.error("[Lobby WS] Error:", err);
			setError("WebSocket connection error");
			setIsConnecting(false);
		};

		const handleClose = () => {
			console.log("[Lobby WS] Connection closed");
			setIsConnected(false);
		};

		// Register handlers
		const unsubMessage = ws.onMessage(handleMessage);
		const unsubError = ws.onError(handleError);
		const unsubClose = ws.onClose(handleClose);

		setIsConnecting(true);
		ws.connect(wsUrl)
			.then(() => {
				setIsConnected(true);
				setIsConnecting(false);
				setError(null);
			})
			.catch((err) => {
				console.error("[Lobby WS] Connection failed:", err);
				setError("Failed to connect to lobby list");
				setIsConnecting(false);
			});

		// Cleanup
		return () => {
			unsubMessage();
			unsubError();
			unsubClose();
			ws.disconnect();
			wsRef.current = null;
		};
	}, [statusFilter, limit, buildWebSocketUrl]);

	return {
		isConnecting,
		lobbies,
		total,
		isConnected,
		error,
		subscribe,
		loadMore,
	};
}
