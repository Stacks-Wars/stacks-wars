import { User } from "./user";

type txId = string;

export interface ClaimState {
	status: "claimed" | "notClaimed";
	data: txId | null;
}

export type PlayerStatus = "joined" | "notJoined";

export interface Player {
	id: string;
	state: PlayerStatus;
	rank: number | null;
	usedWords: string[] | null;
	txId: string | null;
	claim: ClaimState | null;
	prize: number | null;
	lastPing: number | null;
	user: User;
}

export interface PlayerStanding {
	player: Player;
	rank: number;
}
