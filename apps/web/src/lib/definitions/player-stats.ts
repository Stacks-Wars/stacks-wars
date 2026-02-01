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
