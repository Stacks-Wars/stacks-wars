export interface Season {
	id: number;
	slug: string;
	name: string;
	description?: string;
	startDate: string;
	endDate: string;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateSeasonRequest {
	slug: string;
	name: string;
	description?: string;
	startDate: string;
	endDate: string;
}
