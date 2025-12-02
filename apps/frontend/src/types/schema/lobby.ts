import { AssetString } from "@stacks/transactions";
import { GameType } from "./game";
import { ClaimState, Player } from "./player";
import { User } from "./user";

export type lobbyState = "waiting" | "starting" | "inProgress" | "finished";

export interface Lobby {
	id: string;
	name: string;
	creator: User;
	state: lobbyState;
	game: GameType;
	participants: number;
	createdAt: string;
	description: string | null;
	contractAddress: string | null;
	entryAmount: number | null;
	currentAmount: number | null;
	tokenSymbol: string;
	tokenId: AssetString | null;
	creatorLastPing: number | null;
}

export interface LobbyExtended {
	lobby: Lobby;
	players: Player[];
}

export interface PlayerLobbyInfo extends Lobby {
	prizeAmount: number | null;
	rank: number | null;
	claimState: ClaimState | null;
}

export type JoinState = "pending" | "allowed" | "rejected";

export type PendingJoin = {
	user: User;
	state: JoinState;
};
