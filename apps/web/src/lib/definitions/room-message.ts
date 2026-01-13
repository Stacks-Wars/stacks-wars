/**
 * Room WebSocket Message Types
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
}

export interface StartCountdownMessage {
	type: "startCountdown";
	secondsRemaining: number;
}

export interface PlayerJoinedMessage {
	type: "playerJoined";
	playerId: string;
}

export interface PlayerLeftMessage {
	type: "playerLeft";
	playerId: string;
}

export interface PlayerKickedMessage {
	type: "playerKicked";
	playerId: string;
}

export interface JoinRequestsUpdatedMessage {
	type: "joinRequestsUpdated";
	joinRequests: JoinRequest[];
}

export interface JoinRequestStatusMessage {
	type: "joinRequestStatus";
	playerId: string;
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

export interface ErrorMessage {
	type: "error";
	code: string;
	message: string;
}
