import { useEffect, useRef, useCallback } from "react";
import { WebSocketClient } from "@/lib/websocket/wsClient";
import type { LobbyInfo, LobbyStatus } from "@/lib/definitions";
import { useLobbyActions } from "@/lib/stores/lobby";

const WS_BASE_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001";

interface LobbyMessage {
	type: string;
	lobbyInfo?: LobbyInfo[];
	total?: number;
	lobby?: LobbyInfo;
	lobbyId?: string;
	error?: {
		code: string;
		message: string;
	};
}

interface UseLobbyWebSocketOptions {
	statusFilter: LobbyStatus[];
	limit: number;
	onActionSuccess?: (action: string, data?: unknown) => void;
	onActionError?: (action: string, error: string) => void;
}

export function useLobbyWebSocket(options: UseLobbyWebSocketOptions) {
	const { statusFilter, limit, onActionSuccess, onActionError } = options;

	const actions = useLobbyActions();
	const wsRef = useRef<WebSocketClient | null>(null);
	const pendingActionsRef = useRef<Map<string, string>>(new Map());

	const buildWebSocketUrl = useCallback((statuses: LobbyStatus[]) => {
		const statusParam = statuses.join(",");
		return `${WS_BASE_URL}/ws/lobbies?status=${statusParam}`;
	}, []);

	const sendLobbyMessage = useCallback(
		(type: string, payload: Record<string, unknown> = {}) => {
			if (wsRef.current?.isConnected()) {
				wsRef.current.send({ type, ...payload });

				// Track pending actions that need loading states
				if (type === "loadMore") {
					const actionKey = "loadMore";
					pendingActionsRef.current.set(type, actionKey);
					actions.setActionLoading(actionKey, true);
				}
			}
		},
		[actions]
	);

	const subscribe = useCallback(
		(statuses: LobbyStatus[]) => {
			sendLobbyMessage("subscribe", { status: statuses, limit });
		},
		[limit, sendLobbyMessage]
	);

	const loadMore = useCallback(
		(offset: number) => {
			sendLobbyMessage("loadMore", { offset });
		},
		[sendLobbyMessage]
	);

	useEffect(() => {
		const ws = new WebSocketClient();
		wsRef.current = ws;

		const wsUrl = buildWebSocketUrl(statusFilter);

		// Handle messages
		const handleMessage = (message: unknown) => {
			const msg = message as LobbyMessage;

			switch (msg.type) {
				case "lobbyList":
					if (msg.lobbyInfo && msg.total !== undefined) {
						actions.setLobby(msg.lobbyInfo, msg.total);
					}

					// Clear loading state for loadMore
					const actionKey = pendingActionsRef.current.get("loadMore");
					if (actionKey) {
						actions.clearActionLoading(actionKey);
						pendingActionsRef.current.delete("loadMore");
					}
					break;

				case "lobbyCreated":
					if (msg.lobby) {
						actions.addLobby(msg.lobby);
						onActionSuccess?.("lobbyCreated", msg.lobby);
					}
					break;

				case "lobbyUpdated":
					if (msg.lobby) {
						actions.updateLobby(msg.lobby);
					}
					break;

				case "lobbyRemoved":
					if (msg.lobbyId) {
						actions.removeLobby(msg.lobbyId);
					}
					break;

				case "error":
					if (msg.error) {
						actions.setError(msg.error.message);

						// Clear any pending action and notify error handler
						const actionKey =
							pendingActionsRef.current.get("loadMore");
						if (actionKey) {
							actions.clearActionLoading(actionKey);
							pendingActionsRef.current.delete("loadMore");
							onActionError?.("loadMore", msg.error.message);
						}
					}
					break;
			}
		};

		const handleError = (err: Event | Error) => {
			console.error("[Lobby WS] Error:", err);
			actions.setError("WebSocket connection error");
			actions.setConnecting(false);
		};

		const handleClose = () => {
			console.log("[Lobby WS] Connection closed");
			actions.setConnected(false);
		};

		const unsubMessage = ws.onMessage(handleMessage);
		const unsubError = ws.onError(handleError);
		const unsubClose = ws.onClose(handleClose);

		actions.setConnecting(true);
		ws.connect(wsUrl)
			.then(() => {
				actions.setConnected(true);
				actions.setConnecting(false);
				actions.setError(null);
			})
			.catch((err) => {
				console.error("[Lobby WS] Connection failed:", err);
				actions.setError("Failed to connect to lobby ");
				actions.setConnecting(false);
			});

		// Cleanup
		return () => {
			unsubMessage();
			unsubError();
			unsubClose();
			ws.disconnect();
			wsRef.current = null;
		};
	}, [
		statusFilter,
		limit,
		buildWebSocketUrl,
		actions,
		onActionSuccess,
		onActionError,
	]);

	return {
		subscribe,
		loadMore,
	};
}
