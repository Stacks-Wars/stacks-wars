/**
 * Game Engine Core Types
 *
 * Defines the plugin architecture for games.
 * Each game implements a plugin that handles its own state and messages.
 */

import {
	LobbyExtended,
	PlayerState,
	JoinRequest,
	ChatMessage,
} from "@/lib/definitions";

// ============================================================================
// Message Wrapper - All WS messages use this structure
// ============================================================================

export interface GameMessage<T = unknown> {
	/** Game identifier (e.g., "lexi-wars", "chess", "coin-flip") */
	game: string;
	/** Message type specific to the game */
	type: string;
	/** Game-specific payload */
	payload: T;
}

// ============================================================================
// Lobby-level messages (not game-specific)
// ============================================================================

export type LobbyMessage =
	| { type: "lobbyBootstrap"; payload: LobbyBootstrapPayload }
	| { type: "lobbyStateChanged"; payload: { state: string } }
	| { type: "playerJoined"; payload: { playerId: string } }
	| { type: "playerLeft"; payload: { playerId: string } }
	| { type: "playerKicked"; payload: { playerId: string } }
	| { type: "joinRequestsUpdated"; payload: { joinRequests: JoinRequest[] } }
	| { type: "messageReceived"; payload: { message: ChatMessage } }
	| { type: "error"; payload: { error: string; message: string } };

export interface LobbyBootstrapPayload {
	lobby: LobbyExtended;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
}

// ============================================================================
// Game Plugin Interface
// ============================================================================

export interface GamePlugin<TState = unknown, TMessage = unknown> {
	/** Unique game identifier */
	id: string;
	/** Display name */
	name: string;
	/** Game description */
	description?: string;

	/** Initialize game state */
	createInitialState: () => TState;

	/** Handle incoming WebSocket messages for this game */
	handleMessage: (state: TState, message: GameMessage<TMessage>) => TState;

	/** React component to render the game UI */
	GameComponent: React.ComponentType<GamePluginProps<TState>>;

	/** Optional: Validate if a message belongs to this game */
	isGameMessage?: (msg: unknown) => msg is GameMessage<TMessage>;
}

export interface GamePluginProps<TState = unknown> {
	/** Current game state */
	state: TState;
	/** Send message to server */
	sendMessage: (type: string, payload: unknown) => void;
	/** Lobby information */
	lobby: LobbyExtended;
	/** Current players */
	players: PlayerState[];
}

// ============================================================================
// Plugin Registry
// ============================================================================

export interface PluginRegistry {
	[gameId: string]: GamePlugin;
}
