import { Markup, Telegraf } from "telegraf";
import type { IChannelSender, Notification } from "@/types/notifier";

export class TelegramSender implements IChannelSender {
	private bot: Telegraf;
	private botToken: string | undefined;

	constructor(botToken?: string) {
		this.botToken = botToken || process.env.TELEGRAM_BOT_TOKEN;
		if (!this.botToken) {
			console.warn(
				"Telegram Bot Token not provided. TelegramSender will not be able to send messages."
			);
			// Set a dummy bot to prevent errors if methods are called, though isReady should be false.
			this.bot = new Telegraf(
				"YOUR_FALLBACK_BOT_TOKEN_IF_ANY_OR_EMPTY_STRING"
			);
		} else {
			this.bot = new Telegraf(this.botToken);
		}

		// Optional: Add error handling for the bot
		this.bot.catch((err, ctx) => {
			console.error(`Telegraf error for ${ctx.updateType}`, err);
		});
	}

	isReady(): boolean {
		return !!this.botToken;
	}

	async send(notification: Notification): Promise<void> {
		if (!this.isReady()) {
			console.warn(
				"TelegramSender is not ready (missing bot token). Cannot send message."
			);
			return Promise.reject(
				new Error("Telegram Bot Token is not configured.")
			);
		}

		if (!notification.recipient.id) {
			console.error("Recipient ID is missing for Telegram notification");
			return Promise.reject(new Error("Recipient ID is missing."));
		}

		try {
			// eslint-disable-next-line
			const extra: any = {
				parse_mode: notification.parseMode,
				disable_web_page_preview: true,
			};
			if (notification.buttons) {
				extra.reply_markup = Markup.inlineKeyboard(
					notification.buttons
				).reply_markup;
			}

			// The recipient.id for Telegram should be the chat_id
			const result = await this.bot.telegram.sendMessage(
				notification.recipient.id,
				notification.message,
				extra
			);
			console.log(
				`Telegram message sent to ${notification.recipient.id}`
			);
			console.log(result);
		} catch (error) {
			console.error(
				`Failed to send Telegram message to ${notification.recipient.id}:`,
				error
			);
			throw error; // Re-throw the error to be handled by the caller
		}
	}

	// Optional: Method to launch the bot if you need to listen for incoming messages (not strictly needed for sending only)
	// async launch(): Promise<void> {
	//   if (this.isReady()) {
	//     await this.bot.launch();
	//     console.log('Telegram bot started polling...');
	//   }
	// }

	// Optional: Method to stop the bot
	// stop(): void {
	//   if (this.isReady()) {
	//    this.bot.stop('SIGINT'); // Or any other signal
	//    console.log('Telegram bot stopped.');
	//   }
	// }
}
