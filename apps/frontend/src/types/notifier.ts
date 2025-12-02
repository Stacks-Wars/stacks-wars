export interface NotificationRecipient {
	id: string; // e.g., Discord user ID, Telegram chat ID, phone number
}

export interface NotificationButton {
	text: string;
	url: string;
}

export interface Notification {
	message: string;
	recipient: NotificationRecipient;
	parseMode?: "MarkdownV2" | "HTML"; // Optional: For Telegram message formatting
	buttons?: NotificationButton[][]; // Array of button rows
}

export type NotificationChannel = "partykit" | "telegram";

export interface INotifier {
	send(
		channel: NotificationChannel,
		notification: Notification
	): Promise<void>;
	// We could also have channel-specific methods if needed, or a more generic send
	// send(notification: Notification, channels: NotificationChannel[]): Promise<void>;
}

export interface IChannelSender {
	send(notification: Notification): Promise<void>;
	isReady(): boolean;
	destroy?: () => void;
}
