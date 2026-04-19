use teloxide::{
    prelude::*,
    types::ParseMode,
    utils::command::BotCommands,
};

use crate::db::user_wars_points::UserWarsPointsRepository;
use crate::state::AppState;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum LeaderboardCommand {
    #[command(description = "Show top 10 leaderboard")]
    Leaderboard,
}

pub async fn bot_command_handler(
    bot: Bot,
    msg: Message,
    cmd: LeaderboardCommand,
    state: AppState,
) -> ResponseResult<()> {
    match cmd {
        LeaderboardCommand::Leaderboard => {
            handle_leaderboard_command(bot, msg, state).await;
        }
    }

    Ok(())
}

/// Handle leaderboard command
async fn handle_leaderboard_command(
    bot: Bot,
    msg: teloxide::types::Message,
    state: AppState,
) {
    tokio::spawn(async move {
        tracing::debug!("Processing /leaderboard command from chat {}", msg.chat.id);

        let repo = UserWarsPointsRepository::new(state.postgres.clone());
        let leaderboard = match repo.get_leaderboard(None, 10, 0, None, None).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to get leaderboard: {}", e);
                let _ = bot.send_message(msg.chat.id, "❌ Failed to retrieve leaderboard data")
                    .await;
                return;
            }
        };

        if leaderboard.0.is_empty() {
            let _ = bot.send_message(msg.chat.id, "📊 No leaderboard data available yet")
                .await;
            return;
        }

        let mut response = "🏆 <b>Top 10 Leaderboard</b>\n\n".to_string();

        for (index, entry) in leaderboard.0.iter().enumerate().take(10) {
            let display_name = entry
                .display_name
                .as_ref()
                .or(entry.username.as_ref())
                .map(|name| html_escape::encode_text(name).to_string())
                .unwrap_or_else(|| {
                    let wallet = &entry.wallet_address.as_ref();
                    format!("{}...{}", &wallet[0..4], &wallet[wallet.len() - 4..])
                });

            response.push_str(&format!("<b>{}.</b> {}\n", index + 1, display_name));

            response.push_str(&format!(
                "   📈 Wars Points: <code>{:.1}</code>\n",
                entry.points
            ));

            if let Some(ref badge) = entry.rank_badge {
                response.push_str(&format!("   🏅 Rank: <code>{}</code>\n", badge));
            }

            response.push('\n');
        }

        response.push_str("🌐 <b>Join the competition at:</b>\n<code>https://stackswars.com</code>");

        let _ = bot.send_message(msg.chat.id, response)
            .parse_mode(ParseMode::Html)
            .await;

        tracing::debug!("Successfully sent leaderboard to chat {}", msg.chat.id);
    });
}