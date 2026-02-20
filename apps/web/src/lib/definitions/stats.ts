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
	totalLobbies: number;
	activeLobbies: number;
	finishedLobbies: number;
	totalGames: number;
	totalVolumeUsd: number;
	activeVolumeUsd: number;
	finishedVolumeUsd: number;
	tokenBreakdown: TokenVolume[];
}
