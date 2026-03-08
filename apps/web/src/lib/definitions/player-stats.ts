interface UserWarsPoint {
	id: string;
	userId: string;
	seasonId: number;
	points: number;
	rankBadge?: string;
	totalWins: number;
	totalPnl: number;
	totalMatches: number;
	winRate: number;
	createdAt: string;
	updatedAt: string;
}

export interface LeaderBoard extends UserWarsPoint {
	walletAddress: string;
	email: string;
	emailVerified: boolean;
	username?: string;
	displayName?: string;
	profileImage?: string;
	trustRating: number;
}

/** Game-specific leaderboard entry */
export interface GameLeaderBoard {
	id: string;
	gameId: string;
	gameName: string;
	gameImageUrl: string;
	seasonId: number;
	points: number;
	userId: string;
	walletAddress: string;
	username?: string;
	displayName?: string;
	profileImage?: string;
	email: string;
	emailVerified: boolean;
	trustRating: number;
	totalMatches: number;
	totalWins: number;
	totalPnl: number;
	winRate: number;
	createdAt: string;
	updatedAt: string;
}

/** A user's top/most-played game (for profile display) */
export interface UserTopGame {
	gameId: string;
	gameName: string;
	gameImageUrl: string;
	totalMatches: number;
	totalWins: number;
	totalPnl: number;
	winRate: number;
	points: number;
}

/** Aggregated per-game platform stats */
export interface GameStats {
	gameId: string;
	gameName: string;
	gameImageUrl: string;
	totalPlayers: number;
	totalMatches: number;
	totalWins: number;
	avgWinRate: number;
	totalPnl: number;
	totalPoints: number;
}
