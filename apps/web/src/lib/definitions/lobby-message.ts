import type { LobbyInfo, LobbyStatus } from "./lobby";

// Server -> client messages
export type LobbyServerMessage =
	| LobbyListMessage
	| LobbyCreatedMessage
	| LobbyUpdatedMessage
	| LobbyRemovedMessage
	| LobbyErrorMessage;

export interface LobbyListMessage {
	type: "lobbyList";
	lobbyInfo: LobbyInfo[];
	total: number;
}

export interface LobbyCreatedMessage {
	type: "lobbyCreated";
	lobbyInfo: LobbyInfo;
}

export interface LobbyUpdatedMessage {
	type: "lobbyUpdated";
	lobby: LobbyInfo;
}

export interface LobbyRemovedMessage {
	type: "lobbyRemoved";
	lobbyId: string;
}

export interface LobbyErrorMessage {
	type: "error";
	code: string;
	message: string;
}

// Client -> server messages
export type LobbyClientMessage = SubscribeMessage | LoadMoreMessage;

export interface SubscribeMessage {
	type: "subscribe";
	status?: LobbyStatus[];
	limit?: number;
}

export interface LoadMoreMessage {
	type: "loadMore";
	offset: number;
	limit: number;
}

export default {};
