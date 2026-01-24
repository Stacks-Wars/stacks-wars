/**
 * Game Engine Core Types
 *
 * Defines the plugin architecture for games.
 * Each game implements a plugin that handles its own state and messages.
 */

import type { Game, LobbyExtended, PlayerState, User } from "@/lib/definitions";

// ============================================================================
// Message Wrapper - Game messages from server
// ============================================================================

/**
 * Game messages are wrapped in a "game" object:
 * { "game": { "type": "word_entry", "word": "hello", ... } }
 */
export interface GameMessageWrapper<T = unknown> {
	game: T;
}

// ============================================================================
// Game Plugin Interface
// ============================================================================

export interface GamePlugin<TState = unknown, TMessage = unknown> {
	/** Game path (used for routing and matching) */
	path: string;
	/** Display name */
	name: string;
	/** Game description */
	description?: string;

	/** Initialize game state */
	createInitialState: () => TState;

	/** Handle incoming WebSocket messages for this game */
	handleMessage: (state: TState, message: TMessage) => TState;

	/** React component to render the game UI */
	GameComponent: React.ComponentType<GamePluginProps<TState>>;

	/** Optional: Validate if a message belongs to this game */
	isGameMessage?: (msg: unknown) => msg is TMessage;

	/**
	 * Optional: Apply game state from reconnection.
	 * Called when a player connects/reconnects to an in-progress game.
	 * The rawGameState is the game_state field from GameStateMessage.
	 */
	applyGameState?: (state: TState, rawGameState: unknown) => TState;
}

export interface GamePluginProps<TState = unknown> {
	/** Current game state */
	state: TState;
	/** Send message to server */
	sendMessage: (type: string, payload: unknown) => void;
	/** Lobby information */
	lobby: LobbyExtended;
	/** Game information */
	game: Game;
	/** Creator information */
	creator: User;
	/** Current players */
	players: PlayerState[];
}

// ============================================================================
// Plugin Registry
// ============================================================================

export interface PluginRegistry {
	[gamePath: string]: GamePlugin;
}
