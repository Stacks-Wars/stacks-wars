export interface ChatMessage {
	id: string;
	lobbyId: string;
	senderId: string;
	content: string;
	replyTo?: string;
	reactions: Record<string, string[]>; // emoji -> userId[]
	timestamp: number;
}
