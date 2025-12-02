export interface GameType {
	id: string;
	name: string;
	description: string;
	imageUrl: string;
	minPlayers: number;
	tags: string[] | null;
}
