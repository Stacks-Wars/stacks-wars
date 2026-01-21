/**
 * WebSocket Client
 *
 * WebSocket client for handling connections with automatic reconnection.
 */

export type MessageHandler = (message: unknown) => void;
export type ErrorHandler = (error: Event | Error) => void;
export type CloseHandler = () => void;

export class WebSocketClient {
	private ws: WebSocket | null = null;
	private messageHandlers: Set<MessageHandler> = new Set();
	private errorHandlers: Set<ErrorHandler> = new Set();
	private closeHandlers: Set<CloseHandler> = new Set();
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;
	private reconnectDelay = 1000;
	private reconnectTimeout: NodeJS.Timeout | null = null;
	private pingInterval: NodeJS.Timeout | null = null;

	constructor() {}

	connect(wsUrl: string): Promise<void> {
		return new Promise((resolve, reject) => {
			try {
				console.log(`[WS] Connecting to ${wsUrl}`);
				this.ws = new WebSocket(wsUrl);

				this.ws.onopen = () => {
					console.log(`[WS] Connected to ${wsUrl}`);
					this.reconnectAttempts = 0;
					this.startPingInterval();
					resolve();
				};

				this.ws.onmessage = (event) => {
					try {
						const message = JSON.parse(event.data);

						// Notify all handlers
						this.messageHandlers.forEach((handler) => {
							try {
								handler(message);
							} catch (err) {
								console.error("[WS] Handler error:", err);
							}
						});
					} catch (error) {
						console.error("[WS] Failed to parse message:", error);
					}
				};

				this.ws.onerror = (error) => {
					console.error("[WS] WebSocket error:", error);
					this.errorHandlers.forEach((handler) => handler(error));
					reject(error);
				};

				this.ws.onclose = () => {
					console.log("[WS] Connection closed");
					this.stopPingInterval();
					this.closeHandlers.forEach((handler) => handler());
					this.attemptReconnect(wsUrl);
				};
			} catch (error) {
				console.error("[WS] Connection failed:", error);
				reject(error);
			}
		});
	}

	send(message: unknown): void {
		if (this.ws && this.ws.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(message));
		} else {
			console.warn("[WS] Cannot send message, not connected");
		}
	}

	/**
	 * Send a game-specific message with proper wrapper format
	 */
	sendGameMessage(game: string, type: string, payload: unknown): void {
		this.send({
			game,
			type,
			payload,
		});
	}

	/**
	 * Send a lobby-level message (no game wrapper)
	 */
	sendLobbyMessage(message: unknown): void {
		this.send(message);
	}

	onMessage(handler: MessageHandler): () => void {
		this.messageHandlers.add(handler);
		return () => this.messageHandlers.delete(handler);
	}

	onError(handler: ErrorHandler): () => void {
		this.errorHandlers.add(handler);
		return () => this.errorHandlers.delete(handler);
	}

	onClose(handler: CloseHandler): () => void {
		this.closeHandlers.add(handler);
		return () => this.closeHandlers.delete(handler);
	}

	disconnect(): void {
		if (this.reconnectTimeout) {
			clearTimeout(this.reconnectTimeout);
			this.reconnectTimeout = null;
		}
		this.stopPingInterval();
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
	}

	isConnected(): boolean {
		return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
	}

	private attemptReconnect(wsUrl: string): void {
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			console.error("[WS] Max reconnection attempts reached");
			return;
		}

		this.reconnectAttempts++;
		const delay =
			this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
		console.log(
			`[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
		);

		this.reconnectTimeout = setTimeout(() => {
			this.connect(wsUrl).catch((error) => {
				console.error("[WS] Reconnection failed:", error);
			});
		}, delay);
	}

	private startPingInterval(): void {
		this.pingInterval = setInterval(() => {
			if (this.isConnected()) {
				this.send({ type: "ping", ts: Date.now() });
			}
		}, 5000); // Ping every 5 seconds
	}

	private stopPingInterval(): void {
		if (this.pingInterval) {
			clearInterval(this.pingInterval);
			this.pingInterval = null;
		}
	}
}
