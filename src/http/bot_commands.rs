use teloxide::{
    prelude::*,
    types::{Message, ParseMode},
    utils::command::BotCommands,
};

use crate::{db::leaderboard::get::get_leaderboard, state::RedisClient};

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
    redis: RedisClient,
) -> ResponseResult<()> {
    match cmd {
        Command::Leaderboard => handle_leaderboard_command(bot, msg, redis).await,
    }
}

async fn handle_leaderboard_command(
    bot: Bot,
    msg: Message,
    redis: RedisClient,
) -> ResponseResult<()> {
    tracing::debug!("Processing /leaderboard command from chat {}", msg.chat.id);

    let leaderboard = match get_leaderboard(Some(10), redis).await {
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
        //let rank_emoji = match index + 1 {
        //    1 => "🥇",
        //    2 => "🥈",
        //    3 => "🥉",
        //    _ => "🏅",
        //};

        let display_name = entry
            .user
            .display_name
            .as_ref()
            .or(entry.user.username.as_ref())
            .map(|name| html_escape::encode_text(name).to_string())
            .unwrap_or_else(|| {
                let wallet = &entry.user.wallet_address;
                format!("{}...{}", &wallet[0..4], &wallet[wallet.len() - 4..])
            });

        response.push_str(&format!("<b>{}.</b> {}\n", index + 1, display_name));

        response.push_str(&format!(
            "   📈 Wars Points: <code>{:.1}</code>\n",
            entry.user.wars_point
        ));

        response.push_str(&format!(
            "   🎯 Win Rate: <code>{:.1}%</code> ({}/{})\n",
            entry.win_rate, entry.total_wins, entry.total_match
        ));

        if entry.pnl != 0.0 {
            let pnl_emoji = if entry.pnl > 0.0 { "💰" } else { "💸" };
            response.push_str(&format!(
                "   {} P&L: <code>{:.2} STX</code>\n",
                pnl_emoji, entry.pnl
            ));
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
