import { useEffect, useRef, useCallback } from "react";
import { WebSocketClient } from "@/lib/websocket/wsClient";
import type { LobbyInfo, LobbyStatus } from "@/lib/definitions";
import type {
	LobbyClientMessage,
	LobbyServerMessage,
} from "@/lib/definitions/lobby-message";
import { useLobbyActions } from "@/lib/stores/lobby";
import { toast } from "sonner";
import { useRouter } from "next/navigation";
import { displayUserIdentifier } from "@/lib/utils";

const WS_BASE_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001";

interface UseLobbyWebSocketOptions {
	statusFilter: LobbyStatus[];
	limit: number;
	enabled: boolean;
}

export function useLobbyWebSocket(options: UseLobbyWebSocketOptions) {
	const { statusFilter, limit, enabled } = options;

	const actions = useLobbyActions();
	const clientRef = useRef<WebSocketClient | null>(null);
	const pendingActionsRef = useRef<Map<string, string>>(new Map());
	const router = useRouter();

	const buildWebSocketUrl = useCallback((statuses: LobbyStatus[]) => {
		const statusParam = statuses.join(",");
		return `${WS_BASE_URL}/ws/lobbies?status=${statusParam}`;
	}, []);

	const sendLobbyMessage = (message: LobbyClientMessage) => {
		if (!clientRef.current) {
			toast.warning("Failed to send request", {
				description: "Not connected to lobby",
			});
			console.warn("[Lobby] Cannot send lobby message: not connected");
			return;
		}

		// Mark 'loadMore' as pending so UI can show loading
		if (message.type === "loadMore") {
			const actionKey = "loadMore";
			pendingActionsRef.current.set(message.type, actionKey);
			actions.setActionLoading(actionKey, true);
		}

		try {
			clientRef.current.send(message);
		} catch (err) {
			console.error("[Lobby] Failed to send message:", err);
			// clear any loadMore loading state if send failed
			const pending = pendingActionsRef.current.get("loadMore");
			if (pending) {
				actions.clearActionLoading(pending);
				pendingActionsRef.current.delete("loadMore");
			}
			toast.error("Failed to send WS message");
		}
	};

	const subscribe = useCallback(
		(statuses: LobbyStatus[]) => {
			sendLobbyMessage({ type: "subscribe", status: statuses, limit });
		},
		[limit]
	);

	const loadMore = useCallback((offset: number) => {
		sendLobbyMessage({ type: "loadMore", offset, limit });
	}, []);

	useEffect(() => {
		if (!enabled) {
			return;
		}

		const client = new WebSocketClient();
		clientRef.current = client;

		const wsUrl = buildWebSocketUrl(statusFilter);

		// Handle messages
		const handleMessage = (message: LobbyServerMessage) => {
			switch (message.type) {
				case "lobbyList": {
					actions.setLobby(message.lobbyInfo, message.total);

					// Clear loading state for loadMore
					const actionKey = pendingActionsRef.current.get("loadMore");
					if (actionKey) {
						actions.clearActionLoading(actionKey);
						pendingActionsRef.current.delete("loadMore");
					}
					break;
				}

				case "lobbyCreated":
					actions.addLobby(message.lobbyInfo);
					toast.success(
						`New ${message.lobbyInfo.game.name} lobby created!`,
						{
							description: `${message.lobbyInfo.lobby.name} by ${displayUserIdentifier(message.lobbyInfo.creator)}`,
							action: {
								label: "Open",
								onClick: () => {
									router.push(
										`/room/${message.lobbyInfo.lobby.path}`
									);
								},
							},
						}
					);
					break;

				case "lobbyUpdated":
					actions.updateLobby(message.lobby);
					break;

				case "lobbyRemoved":
					actions.removeLobby(message.lobbyId);
					break;

				case "error": {
					actions.setError(message.message);

					// Map error codes to logical actions
					const errorCodeToAction: Record<string, string> = {
						FETCH_FAILED: "fetch",
					};

					const action = errorCodeToAction[message.code];
					if (action) {
						// Clear any pending actions related to this error
						pendingActionsRef.current.forEach((pendingAction) => {
							if (
								pendingAction === action ||
								pendingAction.startsWith(action)
							) {
								pendingActionsRef.current.delete(pendingAction);
								actions.clearActionLoading(pendingAction);
							}
						});
					}

					// Show error toast
					if (action === "fetch") {
						toast.error(`Failed to load lobbies`, {
							description: message.message,
							action: {
								label: "Retry",
								onClick: () => {
									subscribe(statusFilter);
								},
							},
						});
					} else {
						toast.error(message.message || "An error occurred");
					}
					break;
				}
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

		const unsubMessage = client.onMessage(handleMessage);
		const unsubError = client.onError(handleError);
		const unsubClose = client.onClose(handleClose);

		actions.setConnecting(true);
		client
			.connect(wsUrl)
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
			client.disconnect();
			clientRef.current = null;
		};
	}, [enabled, statusFilter, limit, buildWebSocketUrl, actions, router]);

	return {
		subscribe,
		loadMore,
	};
}
