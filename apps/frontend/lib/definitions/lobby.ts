export type LobbyStatus = "waiting" | "in_progress" | "completed" | "cancelled";

export interface Lobby {
	id: string;
	path: string;
	name: string;
	description?: string;
	gameId: string;
	gamePath: string;
	creatorId: string;
	entryAmount?: number;
	currentAmount?: number;
	tokenSymbol?: string;
	tokenContractId?: string;
	contractAddress?: string;
	isPrivate: boolean;
	isSponsored: boolean;
	status: LobbyStatus;
	createdAt: string;
	updatedAt: string;
}

export interface LobbyExtended extends Lobby {
	participantCount: number;
	creatorLastPing?: number;
	startedAt?: number;
	finishedAt?: number;
}

export interface CreateLobbyRequest {
	name: string;
	description?: string;
	gameId: string;
	gamePath: string;
	entryAmount?: number;
	currentAmount?: number;
	tokenSymbol?: string;
	tokenContractId?: string;
	contractAddress?: string;
	isPrivate?: boolean;
	isSponsored?: boolean;
}
