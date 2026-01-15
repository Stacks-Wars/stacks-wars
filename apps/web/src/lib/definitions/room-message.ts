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
	userId: string;
}

export interface PlayerLeftMessage {
	type: "playerLeft";
	userId: string;
}

export interface PlayerKickedMessage {
	type: "playerKicked";
	userId: string;
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

export interface ErrorMessage {
	type: "error";
	code: string;
	message: string;
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
