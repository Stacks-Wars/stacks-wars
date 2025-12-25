export interface Game {
	id: string;
	name: string;
	path: string;
	description: string;
	imageUrl: string;
	minPlayers: number;
	maxPlayers: number;
	category?: string;
	creatorId: string;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateGameRequest {
	name: string;
	path: string;
	description: string;
	imageUrl: string;
	minPlayers: number;
	maxPlayers: number;
	category?: string;
}
