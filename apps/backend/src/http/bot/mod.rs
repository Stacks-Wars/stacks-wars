pub mod broadcasts;
pub mod handlers;

use teloxide::prelude::*;
use crate::state::AppState;

/// Start the Telegram bot and begin listening for commands
pub async fn start_bot(state: AppState) {
    tracing::info!("Starting Telegram bot...");

    let bot = state.bot.clone();

    let handler = Update::filter_message()
        .filter_command::<handlers::LeaderboardCommand>()
        .endpoint(handlers::bot_command_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}