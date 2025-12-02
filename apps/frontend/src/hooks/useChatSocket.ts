import { Player } from "@/types/schema/player";
import { useCallback, useEffect, useRef, useState } from "react";

export interface ChatMessage {
	id: string;
	text: string;
	sender: Player;
	timestamp: string;
}

export type ChatServerMessage =
	| { type: "permitChat"; allowed: boolean }
	| { type: "chat"; message: ChatMessage }
	| { type: "chatHistory"; messages: ChatMessage[] }
	| { type: "pong"; ts: number; pong: number }
	| { type: "error"; message: string };

export type ChatClientMessage =
	| { type: "chat"; text: string }
	| {
			type: "ping";
			ts: number;
	  };

interface QueuedMessage {
	data: ChatClientMessage;
	resolve: () => void;
	reject: (error: Error) => void;
}

interface UseChatSocketProps {
	lobbyId: string;
	userId: string;
}

export interface UseChatSocketType {
	sendMessage: (data: ChatClientMessage) => Promise<void>;
	disconnectChat: () => void;
	readyState: WebSocket["readyState"];
	error: null | Event;
	reconnecting: boolean;
	forceReconnect: () => void;
	messages: ChatMessage[];
	unreadCount: number;
	chatPermitted: boolean;
	userId: string;
	setOpen: (open: boolean) => void;
}

export function useChatSocket({
	lobbyId,
	userId,
}: UseChatSocketProps): UseChatSocketType {
	const socketRef = useRef<WebSocket | null>(null);
	const reconnectAttempts = useRef(0);
	const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const manuallyDisconnectedRef = useRef(false);
	const messageQueue = useRef<QueuedMessage[]>([]);
	const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
	const pingInProgress = useRef(false);
	const PING_INTERVAL = 10000; // 10 seconds
	const [readyState, setReadyState] = useState<WebSocket["readyState"]>(
		WebSocket.CLOSED
	);
	const [error, setError] = useState<null | Event>(null);
	const [reconnecting, setReconnecting] = useState(false);
	const [messages, setMessages] = useState<ChatMessage[]>([]);
	const [chatPermitted, setChatPermitted] = useState(false);
	const [unreadCount, setUnreadCount] = useState(0);
	const [open, setOpen] = useState(false);

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

	const sendMessage = useCallback(
		(data: ChatClientMessage): Promise<void> => {
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
					console.log("⏳ Queuing Chat message (socket not ready)");
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
			console.error("❌ Chat Ping failed:", error);
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
			`${process.env.NEXT_PUBLIC_WS_URL}/ws/chat/${lobbyId}?user_id=${userId}`
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
				const data = JSON.parse(raw) as ChatServerMessage;

				switch (data.type) {
					case "permitChat":
						setChatPermitted(data.allowed);
						if (!data.allowed) setMessages([]);
						break;
					case "chat":
						setMessages((prev) => [...prev, data.message]);
						if (!open && data.message.sender.id !== userId) {
							setUnreadCount((prev) => prev + 1);
						}
						break;
					case "chatHistory":
						setMessages(data.messages);
						break;
					case "pong":
						// Optional latency tracking
						break;
					case "error":
						console.error("Chat error:", data.message);
						break;
					default:
						console.warn("Unknown chat message type", data);
				}
			} catch (err) {
				console.error("❌ Failed to parse Chat message", err);
			}
		};

		ws.onclose = (event) => {
			console.warn("🛑 Chat closed:", event.code, event.reason);
			setReadyState(WebSocket.CLOSED);
			socketRef.current = null;
			pingInProgress.current = false;

			if (pingIntervalRef.current) {
				clearTimeout(pingIntervalRef.current);
				pingIntervalRef.current = null;
			}

			// Check if the close is due to game finished - if so, don't reconnect
			const isGameFinished =
				event.reason && event.reason.toLowerCase().includes("finished");

			if (
				!manuallyDisconnectedRef.current &&
				!isGameFinished &&
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
			//console.error("⚠️ Chat error:", err);
			setError(err);
			setReadyState(WebSocket.CLOSED);

			// Close the broken socket to trigger reconnection
			if (socketRef.current) {
				socketRef.current.close();
				socketRef.current = null;
			}
		};
	}, [lobbyId, userId, schedulePing, processMessageQueue, open]);

	const setOpenWithUnreadReset = useCallback((newOpen: boolean) => {
		setOpen(newOpen);
		if (newOpen) {
			setUnreadCount(0);
		}
	}, []);

	useEffect(() => {
		connectSocket();

		return () => {
			if (reconnectTimeoutRef.current)
				clearTimeout(reconnectTimeoutRef.current);
			socketRef.current?.close();
			socketRef.current = null;
		};
	}, [connectSocket]);

	const disconnectChat = useCallback(() => {
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
		disconnectChat,
		readyState,
		error,
		reconnecting,
		forceReconnect,
		messages,
		unreadCount,
		chatPermitted,
		userId,
		setOpen: setOpenWithUnreadReset,
	};
}
