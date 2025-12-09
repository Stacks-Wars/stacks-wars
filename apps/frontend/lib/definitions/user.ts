export interface User {
	id: string;
	walletAddress: string;
	username?: string;
	displayName?: string;
	profileImage?: string;
	trustRating: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateUserRequest {
	walletAddress: string;
	username?: string;
	displayName?: string;
}

export interface UpdateUserRequest {
	username?: string;
	displayName?: string;
	profileImage?: string;
}

export interface AuthResponse {
	user: User;
	token: string;
}
