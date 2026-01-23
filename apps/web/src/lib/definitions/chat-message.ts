export interface Reaction {
	userId: string;
	emoji: string;
}

export interface ChatMessage {
	messageId: string;
	lobbyId: string;
	userId: string;
	content: string;
	replyTo?: string;
	reactions: Reaction[];
	createdAt: string; // ISO date string
}
