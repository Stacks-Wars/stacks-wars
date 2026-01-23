export interface User {
	id: string;
	walletAddress: string;
	username?: string;
	displayName?: string;
	profileImage?: string;
	trustRating: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateUserRequest {
	walletAddress: string;
	username?: string;
	displayName?: string;
}

export interface UpdateUserRequest {
	username?: string;
	displayName?: string;
	profileImage?: string;
}

export type joinState = "pending" | "accepted" | "rejected";

export interface PlayerState {
	userId: string;
	lobbyId: string;
	state: joinState;
	status: "not_joined" | "joined";
	walletAddress: string;
	username?: string;
	displayName?: string;
	trustRating: number;
	txId?: string;
	rank?: number;
	prize?: number;
	claimState?: string;
	lastPing?: number;
	joinedAt: number;
	updatedAt: number;
	isCreator: boolean;
}

export interface JoinRequest {
	userId: string;
	walletAddress: string;
	username?: string;
	displayName?: string;
	trustRating: number;
	state: joinState;
	isCreator: boolean;
}
