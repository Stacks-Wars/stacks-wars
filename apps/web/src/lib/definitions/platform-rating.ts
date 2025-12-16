export interface PlatformRating {
	id: string;
	userId: string;
	seasonId: number;
	rating: number;
	gamesPlayed: number;
	wins: number;
	losses: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreatePlatformRatingRequest {
	userId: string;
	seasonId: number;
	rating?: number;
}

export interface UpdatePlatformRatingRequest {
	rating?: number;
	gamesPlayed?: number;
	wins?: number;
	losses?: number;
}
