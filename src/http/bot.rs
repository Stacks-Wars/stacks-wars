use html_escape::encode_text;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use teloxide::{
    Bot,
    payloads::SendPhotoSetters,
    prelude::{Request, Requester},
    sugar::request::RequestReplyExt,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MessageId, ParseMode},
};

use crate::models::game::GameType;
use uuid::Uuid;

pub struct BotNewLobbyPayload {
    pub lobby_id: Uuid,
    pub lobby_name: String,
    pub description: Option<String>,
    pub game: GameType,
    pub contract_address: Option<String>,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub creator_name: Option<String>,
    pub wallet_address: String,
}

#[derive(Serialize, Deserialize)]
pub struct BotLobbyWinnerPayload {
    pub lobby_id: Uuid,
    pub lobby_name: String,
    pub game: GameType,
    pub winner_name: Option<String>,
    pub winner_wallet: String,
    pub winner_prize: Option<f64>,
    pub entry_amount: Option<f64>,
    pub runner_ups: Vec<RunnerUp>,
    pub tg_msg_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct RunnerUp {
    pub name: Option<String>,
    pub wallet: String,
    pub position: String,
    pub prize: Option<f64>,
}

pub async fn broadcast_lobby_created(
    bot: &Bot,
    chat_id: i64,
    payload: BotNewLobbyPayload,
) -> Result<teloxide::types::Message, teloxide::RequestError> {
    let network = std::env::var("STACKS_NETWORK").unwrap_or("testnet".to_string());
    let wallet = payload.wallet_address.clone();
    let truncated_wallet = format!("{}...{}", &wallet[0..4], &wallet[wallet.len() - 4..]);

    let lobby_name = format!(
        "🏷 <b>Lobby Name:</b> {}\n",
        encode_text(&payload.lobby_name)
    );

    let game_name = format!("🎮 <b>Game:</b> {}\n", encode_text(&payload.game.name));

    let creator = payload
        .creator_name
        .as_ref()
        .map(|name| {
            format!(
                "🧑‍🚀 <b>Creator:</b> {} ({})\n",
                encode_text(name),
                encode_text(&truncated_wallet)
            )
        })
        .unwrap_or_else(|| format!("🧑‍🚀 <b>Creator:</b> {}\n", encode_text(&wallet)));

    let description = payload
        .description
        .as_ref()
        .map(|desc| format!("📝 <b>Description:</b> {}\n", encode_text(desc)))
        .unwrap_or_default();

    let contract_line = payload
        .contract_address
        .as_ref()
        .map(|addr| {
            format!(
                "💰 <b>Pool Contract:</b> <a href=\"https://explorer.hiro.so/txid/{}?chain={network}\">View on Hiro</a>\n",
                addr
            )
        })
        .unwrap_or_default();

    let entry_fee_line = match payload.entry_amount {
        Some(amount) if amount == 0.0 => {
            // Sponsored lobby - show pool size instead of entry fee
            let pool_size = payload.current_amount.unwrap_or(0.0);
            let token = payload.token_symbol.as_deref().unwrap_or("STX");
            format!("🎁 <b>Pool Size:</b> {} {} (Sponsored)\n", pool_size, token)
        }
        Some(amount) => {
            // Regular paid lobby - show entry fee
            let token = payload.token_symbol.as_deref().unwrap_or("STX");
            format!("💵 <b>Entry Fee:</b> {} {}\n", amount, token)
        }
        None => String::new(), // No pool configured
    };

    let lobby_link = format!(
        "\n🔗 <b>Link:</b> <code>https://stackswars.com/lobby/{}</code>",
        payload.lobby_id
    );

    let caption = format!(
        "🆕 <b>New Lobby Created</b>\n\n\
        {lobby_name}\
        {game_name}\
        {creator}\
        {description}\
        {contract_line}\
        {entry_fee_line}\
        {lobby_link}",
    );

    tracing::info!("Telegram caption (HTML): {}", caption);

    let lobby_url: Url = Url::parse(&format!(
        "https://stackswars.com/lobby/{}",
        payload.lobby_id
    ))
    .unwrap();

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "🚀 Join Now",
        lobby_url,
    )]]);

    let message = bot
        .send_photo(
            ChatId(chat_id),
            InputFile::url(payload.game.image_url.parse().unwrap()),
        )
        .caption(caption)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .send()
        .await?;

    Ok(message)
}

pub async fn broadcast_lobby_winner(
    bot: &Bot,
    chat_id: i64,
    payload: BotLobbyWinnerPayload,
) -> Result<(), teloxide::RequestError> {
    let winner_wallet = payload.winner_wallet.clone();
    let truncated_winner_wallet = format!(
        "{}...{}",
        &winner_wallet[0..4],
        &winner_wallet[winner_wallet.len() - 4..]
    );

    let winner_display = payload
        .winner_name
        .as_ref()
        .map(|name| {
            format!(
                "{} ({})",
                encode_text(name),
                encode_text(&truncated_winner_wallet)
            )
        })
        .unwrap_or_else(|| encode_text(&winner_wallet).to_string());

    let mut content = format!(
        "🎉 <b>Game Finished!</b>\n\n\
        🏆 <b>Winner:</b> {}\n",
        winner_display
    );

    // Add prize information if there's a prize
    match (payload.winner_prize, payload.entry_amount) {
        (Some(prize), Some(entry)) => {
            let net_prize = prize - entry;
            content.push_str(&format!("💰 <b>Prize Won:</b> {:.2} STX\n", net_prize));
        }
        (Some(prize), None) => {
            // No entry fee, so full prize amount
            content.push_str(&format!("💰 <b>Prize Won:</b> {:.2} STX\n", prize));
        }
        _ => {
            // No prize information available
        }
    }

    // Add runner-ups
    if !payload.runner_ups.is_empty() {
        content.push_str("\n<b>Runner-ups:</b>\n");
        for runner_up in payload.runner_ups {
            let runner_up_wallet = runner_up.wallet.clone();
            let truncated_runner_up_wallet = format!(
                "{}...{}",
                &runner_up_wallet[0..4],
                &runner_up_wallet[runner_up_wallet.len() - 4..]
            );

            let runner_up_display = runner_up
                .name
                .as_ref()
                .map(|name| {
                    format!(
                        "{} ({})",
                        encode_text(name),
                        encode_text(&truncated_runner_up_wallet)
                    )
                })
                .unwrap_or_else(|| encode_text(&runner_up_wallet).to_string());

            // Use appropriate emoji based on position
            let emoji = match runner_up.position.as_str() {
                "2nd" => "🥈",
                "3rd" => "🥉",
                _ => "🏅", // fallback for any other position
            };

            let mut runner_up_line = format!(
                "{} <b>{}:</b> {}",
                emoji, runner_up.position, runner_up_display
            );

            if let Some(prize) = runner_up.prize {
                let net_prize = match payload.entry_amount {
                    Some(entry) => prize - entry,
                    None => prize,
                };

                if net_prize > 0.0 {
                    runner_up_line.push_str(&format!(" - {:.2} STX", net_prize));
                }
            }

            content.push_str(&format!("{}\n", runner_up_line));
        }
    }

    //let lobby_link = format!(
    //    "\n🔗 <b>View Lobby:</b> <code>https://stackswars.com/lobby/{}</code>",
    //    payload.lobby_id
    //);

    //content.push_str(&lobby_link);

    tracing::info!("Telegram winner announcement (HTML): {}", content);

    let game_url: Url = Url::parse(&format!(
        "https://stackswars.com/lexi-wars/{}",
        payload.lobby_id
    ))
    .unwrap();

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "🎮 View Results",
        game_url,
    )]]);

    let _message = bot
        .send_photo(
            ChatId(chat_id),
            InputFile::url(payload.game.image_url.parse().unwrap()),
        )
        .caption(content)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .reply_to(teloxide::types::MessageId(payload.tg_msg_id))
        .await?;

    Ok(())
}

pub async fn delete_lobby_creation_message(
    bot: &Bot,
    chat_id: i64,
    message_id: i32,
) -> Result<(), teloxide::RequestError> {
    tracing::info!("Deleting lobby creation message ID: {}", message_id);

    bot.delete_message(ChatId(chat_id), MessageId(message_id))
        .await?;

    tracing::info!("Successfully deleted lobby creation message");
    Ok(())
}
