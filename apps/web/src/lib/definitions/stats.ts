export interface TokenVolume {
	symbol: string;
	volume: number;
	volumeUsd: number;
	lobbyCount: number;
	priceUsd: number;
	imageUrl: string | null;
}

export interface PlatformStats {
	totalUsers: number;
	newUsersCount: number;
	totalLobbies: number;
	activeLobbies: number;
	finishedLobbies: number;
	feeLobbies: number;
	activeFeeLobbies: number;
	totalGames: number;
	totalVolumeUsd: number;
	finishedVolumeUsd: number;
	tokenBreakdown: TokenVolume[];
}
