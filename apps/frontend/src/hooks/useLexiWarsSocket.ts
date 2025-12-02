import { Player, PlayerStanding } from "@/types/schema/player";
import { useCallback, useEffect, useRef, useState } from "react";

export type LexiWarsServerMessage =
	| { type: "turn"; currentTurn: Player; countdown: number }
	| { type: "rule"; rule: string }
	| { type: "countdown"; time: number }
	| { type: "rank"; rank: string }
	| { type: "validate"; msg: string }
	| { type: "wordEntry"; word: string; sender: Player }
	| { type: "usedWord"; word: string }
	| { type: "gameOver" }
	| { type: "finalStanding"; standing: PlayerStanding[] }
	| { type: "prize"; amount: number }
	| { type: "warsPoint"; warsPoint: number }
	| { type: "pong"; ts: number; pong: number }
	| { type: "start"; time: number; started: boolean }
	| { type: "startFailed" }
	| {
			type: "playersCount";
			connectedPlayers: number;
			remainingPlayers: number;
	  }
	| { type: "spectator" };

export type LexiWarsClientMessage =
	| { type: "wordEntry"; word: string }
	| {
			type: "ping";
			ts: number;
	  };

interface QueuedMessage {
	data: LexiWarsClientMessage;
	resolve: () => void;
	reject: (error: Error) => void;
}

interface UseLexiWarsSocketProps {
	lobbyId: string;
	userId: string;
	onMessage?: (msg: LexiWarsServerMessage) => void;
}

export function useLexiWarsSocket({
	lobbyId,
	userId,
	onMessage,
}: UseLexiWarsSocketProps) {
	const socketRef = useRef<WebSocket | null>(null);
	const reconnectAttempts = useRef(0);
	const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const manuallyDisconnectedRef = useRef(false);
	const messageHandlerRef = useRef<typeof onMessage | null>(null);
	const messageQueue = useRef<QueuedMessage[]>([]);
	const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
	const pingInProgress = useRef(false);
	const PING_INTERVAL = 2000; // 2 seconds
	const [readyState, setReadyState] = useState<WebSocket["readyState"]>(
		WebSocket.CLOSED
	);
	const [error, setError] = useState<null | Event>(null);
	const [reconnecting, setReconnecting] = useState(false);

	const maxReconnectAttempts = 5;

	const processMessageQueue = useCallback(() => {
		if (socketRef.current?.readyState === WebSocket.OPEN) {
			while (messageQueue.current.length > 0) {
				const queuedMessage = messageQueue.current.shift();
				if (queuedMessage) {
					try {
						socketRef.current.send(
							JSON.stringify(queuedMessage.data)
						);
						queuedMessage.resolve();
					} catch (error) {
						queuedMessage.reject(error as Error);
					}
				}
			}
		}
	}, []);

	useEffect(() => {
		messageHandlerRef.current = onMessage;
	}, [onMessage]);

	const sendMessage = useCallback(
		(data: LexiWarsClientMessage): Promise<void> => {
			return new Promise((resolve, reject) => {
				const socket = socketRef.current;
				if (socket?.readyState === WebSocket.OPEN) {
					try {
						socket.send(JSON.stringify(data));
						resolve();
					} catch (error) {
						reject(error as Error);
					}
				} else {
					console.log(
						"⏳ Queuing LexiWars message (socket not ready)"
					);
					messageQueue.current.push({ data, resolve, reject });
				}
			});
		},
		[]
	);

	const schedulePing = useCallback(async () => {
		if (pingInProgress.current || !socketRef.current) return;

		pingInProgress.current = true;
		try {
			await sendMessage({ type: "ping", ts: Date.now() });
		} catch (error) {
			console.error("❌ LexiWars Ping failed:", error);
		} finally {
			pingInProgress.current = false;

			// Schedule next ping only after current one completes
			if (socketRef.current?.readyState === WebSocket.OPEN) {
				pingIntervalRef.current = setTimeout(
					schedulePing,
					PING_INTERVAL
				);
			}
		}
	}, [sendMessage]);

	const connectSocket = useCallback(() => {
		if (!lobbyId || !userId) return;
		if (socketRef.current) return; // already connecting or connected

		const ws = new WebSocket(
			`${process.env.NEXT_PUBLIC_WS_URL}/ws/lexiwars/${lobbyId}?user_id=${userId}`
		);

		socketRef.current = ws;

		ws.onopen = () => {
			setReadyState(ws.readyState);
			setError(null);
			setReconnecting(false);
			reconnectAttempts.current = 0;

			// Start the ping cycle
			pingIntervalRef.current = setTimeout(schedulePing, PING_INTERVAL);

			processMessageQueue();
		};

		ws.onmessage = (event) => {
			try {
				const raw = typeof event.data === "string" ? event.data : "";
				const data = JSON.parse(raw) as LexiWarsServerMessage;
				messageHandlerRef.current?.(data);
			} catch (err) {
				console.error("❌ Failed to parse LexiWars message", err);
			}
		};

		ws.onclose = (event) => {
			console.warn("🛑 LexiWarsSocket closed:", event.code, event.reason);
			setReadyState(WebSocket.CLOSED);
			socketRef.current = null;
			pingInProgress.current = false;

			if (pingIntervalRef.current) {
				clearTimeout(pingIntervalRef.current);
				pingIntervalRef.current = null;
			}

			if (
				!manuallyDisconnectedRef.current &&
				reconnectAttempts.current < maxReconnectAttempts
			) {
				reconnectAttempts.current++;
				const timeout = Math.pow(2, reconnectAttempts.current) * 1000;

				setReconnecting(true);
				reconnectTimeoutRef.current = setTimeout(() => {
					connectSocket();
				}, timeout);
			} else {
				// Reject all queued messages if we can't reconnect
				while (messageQueue.current.length > 0) {
					const queuedMessage = messageQueue.current.shift();
					if (queuedMessage) {
						queuedMessage.reject(new Error("Connection failed"));
					}
				}
			}
		};

		ws.onerror = (err) => {
			//console.error("⚠️ LexiWarsSocket error:", err);
			setError(err);
			setReadyState(WebSocket.CLOSED);

			// Close the broken socket to trigger reconnection
			if (socketRef.current) {
				socketRef.current.close();
				socketRef.current = null;
			}
		};
	}, [lobbyId, userId, processMessageQueue, schedulePing]);

	useEffect(() => {
		connectSocket();

		return () => {
			if (reconnectTimeoutRef.current)
				clearTimeout(reconnectTimeoutRef.current);
			socketRef.current?.close();
			socketRef.current = null;
		};
	}, [connectSocket]);

	const disconnect = useCallback(() => {
		manuallyDisconnectedRef.current = true;
		pingInProgress.current = false;

		if (reconnectTimeoutRef.current)
			clearTimeout(reconnectTimeoutRef.current);

		if (pingIntervalRef.current) {
			clearTimeout(pingIntervalRef.current);
			pingIntervalRef.current = null;
		}

		// Reject all queued messages
		while (messageQueue.current.length > 0) {
			const queuedMessage = messageQueue.current.shift();
			if (queuedMessage) {
				queuedMessage.reject(new Error("Socket disconnected"));
			}
		}

		socketRef.current?.close();
		socketRef.current = null;
		setReadyState(WebSocket.CLOSED);
		messageQueue.current = [];
	}, []);

	const forceReconnect = useCallback(() => {
		// Clear any existing timeouts
		if (reconnectTimeoutRef.current) {
			clearTimeout(reconnectTimeoutRef.current);
			reconnectTimeoutRef.current = null;
		}

		// Close existing connection
		if (socketRef.current) {
			socketRef.current.close();
			socketRef.current = null;
		}

		// Reset reconnection attempts and flags
		reconnectAttempts.current = 0;
		manuallyDisconnectedRef.current = false;
		setReconnecting(false);

		// Reconnect immediately
		connectSocket();
	}, [connectSocket]);

	return {
		sendMessage,
		disconnect,
		readyState,
		error,
		reconnecting,
		forceReconnect,
	};
}
