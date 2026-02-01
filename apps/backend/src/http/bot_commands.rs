use teloxide::{
    prelude::*,
    types::{Message, ParseMode},
    utils::command::BotCommands,
};

use crate::{db::user_wars_points::UserWarsPointsRepository, state::AppState};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Show the top 10 leaderboard")]
    Leaderboard,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: AppState,
) -> ResponseResult<()> {
    match cmd {
        Command::Leaderboard => handle_leaderboard_command(bot, msg, state).await,
    }
}

async fn handle_leaderboard_command(
    bot: Bot,
    msg: Message,
    state: AppState,
) -> ResponseResult<()> {
    tracing::debug!("Processing /leaderboard command from chat {}", msg.chat.id);

    let repo = UserWarsPointsRepository::new(state.postgres.clone());
    let leaderboard = match repo.get_leaderboard(None, 10, 0).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to get leaderboard: {}", e);
            bot.send_message(msg.chat.id, "❌ Failed to retrieve leaderboard data")
                .await?;
            return Ok(());
        }
    };

    if leaderboard.is_empty() {
        bot.send_message(msg.chat.id, "📊 No leaderboard data available yet")
            .await?;
        return Ok(());
    }

    let mut response = "🏆 <b>Top 10 Leaderboard</b>\n\n".to_string();

    for (index, entry) in leaderboard.iter().enumerate().take(10) {
        let display_name = entry
            .display_name
            .as_ref()
            .or(entry.username.as_ref())
            .map(|name| html_escape::encode_text(name).to_string())
            .unwrap_or_else(|| {
                let wallet = &entry.wallet_address.0;
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

    bot.send_message(msg.chat.id, response)
        .parse_mode(ParseMode::Html)
        .await?;

    tracing::debug!("Successfully sent leaderboard to chat {}", msg.chat.id);
    Ok(())
}
