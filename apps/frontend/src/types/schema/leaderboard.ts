import { User } from "./user";

export interface LeaderBoard {
	user: User;
	winRate: number;
	rank: number;
	totalMatch: number;
	totalWins: number;
	pnl: number;
}
