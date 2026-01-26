/**
 * Room Server Message Types
 *
 * Type definitions for messages sent from the server to clients
 * in the room/lobby WebSocket connection.
 *
 * These types match the Rust `RoomServerMessage` enum from the backend.
 */

import type { ChatMessage } from "./chat-message";
import type { JoinRequest, PlayerState } from "./user";
import type { LobbyInfo } from "./lobby";
import type { LobbyStatus } from "./lobby";

/**
 * Discriminated union of all possible room server messages.
 * Each message type has a `type` field that acts as the discriminator.
 */
export type RoomServerMessage =
	| LobbyBootstrapMessage
	| LobbyStatusChangedMessage
	| StartCountdownMessage
	| PlayerJoinedMessage
	| PlayerLeftMessage
	| PlayerKickedMessage
	| JoinRequestsUpdatedMessage
	| JoinRequestStatusMessage
	| MessageReceivedMessage
	| ReactionAddedMessage
	| ReactionRemovedMessage
	| PongMessage
	| PlayerUpdatedMessage
	| GameStateMessage
	| GameStartedMessage
	| GameStartFailedMessage
	| FinalStandingMessage
	| GameOverMessage
	| ErrorMessage;

export interface LobbyBootstrapMessage {
	type: "lobbyBootstrap";
	lobbyInfo: LobbyInfo;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
}

export interface LobbyStatusChangedMessage {
	type: "lobbyStatusChanged";
	status: LobbyStatus;
	participantCount: number;
	currentAmount?: number;
}

export interface StartCountdownMessage {
	type: "startCountdown";
	secondsRemaining: number | null;
}

export interface PlayerJoinedMessage {
	type: "playerJoined";
	player: PlayerState;
}

export interface PlayerLeftMessage {
	type: "playerLeft";
	player: PlayerState;
}

export interface PlayerKickedMessage {
	type: "playerKicked";
	player: PlayerState;
}

export interface JoinRequestsUpdatedMessage {
	type: "joinRequestsUpdated";
	joinRequests: JoinRequest[];
}

export interface JoinRequestStatusMessage {
	type: "joinRequestStatus";
	userId: string;
	accepted: boolean;
}

export interface MessageReceivedMessage {
	type: "messageReceived";
	message: ChatMessage;
}

export interface ReactionAddedMessage {
	type: "reactionAdded";
	messageId: string;
	userId: string;
	emoji: string;
}

export interface ReactionRemovedMessage {
	type: "reactionRemoved";
	messageId: string;
	userId: string;
	emoji: string;
}

export interface PongMessage {
	type: "pong";
	elapsedMs: number;
}

export interface PlayerUpdatedMessage {
	type: "playerUpdated";
	players: PlayerState[];
}

/**
 * Game state for reconnecting players - sent when joining an in-progress game.
 * Contains game-specific state that each game engine provides.
 * Each game plugin should define its own GameStateData type.
 */
export interface GameStateMessage {
	type: "gameState";
	gameState: unknown;
}

export interface ErrorMessage {
	type: "error";
	code: string;
	message: string;
}

// ============================================================================
// Shared Game Events (used across all games)
// ============================================================================

export interface GameStartedMessage {
	type: "gameStarted";
}

export interface GameStartFailedMessage {
	type: "gameStartFailed";
	reason: string;
}

export interface FinalStandingMessage {
	type: "finalStanding";
	standings: PlayerState[];
}

export interface GameOverMessage {
	type: "gameOver";
	rank: number;
	prize: number | null;
	warsPoint: number;
}

/**
 * Room Client Message Types
 *
 * Type definitions for messages sent from clients to the server
 * in the room/lobby WebSocket connection.
 *
 * These types match the Rust `RoomClientMessage` enum from the backend.
 */

/**
 * Discriminated union of all possible room client messages.
 * Each message type has a `type` field that acts as the discriminator.
 */
export type RoomClientMessage =
	| JoinMessage
	| LeaveMessage
	| UpdateLobbyStatusMessage
	| JoinRequestMessage
	| ApproveJoinMessage
	| RejectJoinMessage
	| KickMessage
	| SendChatMessage
	| AddReactionMessage
	| RemoveReactionMessage
	| PingMessage;

export interface JoinMessage {
	type: "join";
}

export interface LeaveMessage {
	type: "leave";
}

export interface UpdateLobbyStatusMessage {
	type: "updateLobbyStatus";
	status: LobbyStatus;
}

export interface JoinRequestMessage {
	type: "joinRequest";
}

export interface ApproveJoinMessage {
	type: "approveJoin";
	userId: string;
}

export interface RejectJoinMessage {
	type: "rejectJoin";
	userId: string;
}

export interface KickMessage {
	type: "kick";
	userId: string;
}

export interface SendChatMessage {
	type: "sendMessage";
	content: string;
	replyTo?: string;
}

export interface AddReactionMessage {
	type: "addReaction";
	messageId: string;
	emoji: string;
}

export interface RemoveReactionMessage {
	type: "removeReaction";
	messageId: string;
	emoji: string;
}

export interface PingMessage {
	type: "ping";
	ts: number;
}
