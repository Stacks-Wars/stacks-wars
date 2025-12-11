import {
	ChatMessage,
	JoinRequest,
	LobbyExtended,
	LobbyStatus,
	PlayerState,
} from "@/lib/definitions";

// ============================================================================
// Client Messages (sent to server)
// ============================================================================

export type RoomClientMessage =
	| { type: "join" }
	| { type: "leave" }
	| { type: "updateLobbyStatus"; status: LobbyStatus }
	| { type: "joinRequest" }
	| { type: "approveJoin"; playerId: string }
	| { type: "rejectJoin"; playerId: string }
	| { type: "kick"; playerId: string }
	| { type: "sendMessage"; content: string; replyTo?: string }
	| { type: "addReaction"; messageId: string; emoji: string }
	| { type: "removeReaction"; messageId: string; emoji: string }
	| { type: "ping"; ts: number };

// ============================================================================
// Server Messages (received from server)
// ============================================================================

export type RoomServerMessage =
	| {
			type: "lobbyBootstrap";
			lobby: LobbyExtended;
			players: PlayerState[];
			joinRequests: JoinRequest[];
			chatHistory: ChatMessage[];
	  }
	| { type: "lobbyStateChanged"; state: LobbyStatus }
	| { type: "startCountdown"; secondsRemaining: number }
	| { type: "playerJoined"; playerId: string }
	| { type: "playerLeft"; playerId: string }
	| { type: "playerKicked"; playerId: string }
	| { type: "joinRequestsUpdated"; joinRequests: JoinRequest[] }
	| { type: "joinRequestStatus"; playerId: string; accepted: boolean }
	| { type: "messageReceived"; message: ChatMessage }
	| {
			type: "reactionAdded";
			messageId: string;
			emoji: string;
			userId: string;
	  }
	| {
			type: "reactionRemoved";
			messageId: string;
			emoji: string;
			userId: string;
	  }
	| { type: "pong"; clientTs: number; serverTs: number }
	| { type: "error"; error: string; message: string };

// ============================================================================
// WebSocket Client Class
// ============================================================================

export type MessageHandler = (message: RoomServerMessage) => void;
export type ErrorHandler = (error: Event | Error) => void;
export type CloseHandler = () => void;

export class RoomWebSocketClient {
	private ws: WebSocket | null = null;
	private messageHandlers: Set<MessageHandler> = new Set();
	private errorHandlers: Set<ErrorHandler> = new Set();
	private closeHandlers: Set<CloseHandler> = new Set();
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;
	private reconnectDelay = 1000;
	private pingInterval: NodeJS.Timeout | null = null;

	constructor(
		private lobbyPath: string,
		private token?: string
	) {}

	connect(wsUrl: string): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				// Append token as query parameter if available
				const url = this.token ? `${wsUrl}?token=${this.token}` : wsUrl;
				this.ws = new WebSocket(url);

				this.ws.onopen = () => {
					console.log(
						`[RoomWS] Connected to lobby ${this.lobbyPath}`
					);
					this.reconnectAttempts = 0;
					this.startPingInterval();
					resolve();
				};

				this.ws.onmessage = (event) => {
					try {
						const message: RoomServerMessage = JSON.parse(
							event.data
						);
						this.messageHandlers.forEach((handler) =>
							handler(message)
						);
					} catch (err) {
						console.error("[RoomWS] Failed to parse message:", err);
					}
				};

				this.ws.onerror = (event) => {
					console.error("[RoomWS] WebSocket error:", event);
					this.errorHandlers.forEach((handler) => handler(event));
					reject(new Error("WebSocket connection failed"));
				};

				this.ws.onclose = () => {
					console.log("[RoomWS] Connection closed");
					this.stopPingInterval();
					this.closeHandlers.forEach((handler) => handler());
					this.attemptReconnect(wsUrl);
				};
			} catch (err) {
				reject(err);
			}
		});
	}

	private attemptReconnect(wsUrl: string) {
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			console.log("[RoomWS] Max reconnect attempts reached");
			return;
		}

		this.reconnectAttempts++;
		const delay =
			this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

		console.log(
			`[RoomWS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
		);

		setTimeout(() => {
			this.connect(wsUrl).catch((err) => {
				console.error("[RoomWS] Reconnect failed:", err);
			});
		}, delay);
	}

	private startPingInterval() {
		this.pingInterval = setInterval(() => {
			this.send({ type: "ping", ts: Date.now() });
		}, 5000); // Ping every 5 seconds
	}

	private stopPingInterval() {
		if (this.pingInterval) {
			clearInterval(this.pingInterval);
			this.pingInterval = null;
		}
	}

	send(message: RoomClientMessage) {
		if (this.ws && this.ws.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(message));
		} else {
			console.warn("[RoomWS] Cannot send message - not connected");
		}
	}

	onMessage(handler: MessageHandler) {
		this.messageHandlers.add(handler);
		return () => this.messageHandlers.delete(handler);
	}

	onError(handler: ErrorHandler) {
		this.errorHandlers.add(handler);
		return () => this.errorHandlers.delete(handler);
	}

	onClose(handler: CloseHandler) {
		this.closeHandlers.add(handler);
		return () => this.closeHandlers.delete(handler);
	}

	disconnect() {
		this.stopPingInterval();
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
		this.messageHandlers.clear();
		this.errorHandlers.clear();
		this.closeHandlers.clear();
	}

	isConnected(): boolean {
		return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
	}
}
