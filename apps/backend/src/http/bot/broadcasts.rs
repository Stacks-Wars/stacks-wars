use html_escape::encode_text;
use reqwest::Url;
use teloxide::{
    payloads::{SendMessageSetters, SendPhotoSetters},
    prelude::{Request, Requester},
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MessageId, ParseMode, ReplyParameters},
};
use uuid::Uuid;

use crate::db::{game::GameRepository, lobby::LobbyRepository, lobby_state::LobbyStateRepository, player_state::PlayerStateRepository};
use crate::models::BotNewLobbyPayload;
use crate::state::{AppState, Network};

/// Broadcast lobby creation to Telegram
pub async fn broadcast_lobby_creation_to_tg(
    state: AppState,
    payload: BotNewLobbyPayload,
) {
    tokio::spawn(async move {
        let bot = &state.bot;
        let chat_id = ChatId(state.config.telegram_chat_id.parse().unwrap_or(0));

        // Fetch game details
        let game_repo = GameRepository::new(state.postgres.clone());
        let game = match game_repo.find_by_id(payload.lobby.game_id).await {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to fetch game {}: {}", payload.lobby.game_id, e);
                return;
            }
        };

        let wallet = payload.wallet_address.clone();
        let truncated_wallet = format!("{}...{}", &wallet[0..4], &wallet[wallet.len() - 4..]);

        let lobby_name = format!(
            "🏆 <b>{}</b>",
            encode_text(&payload.lobby.name)
        );

        let game_name = format!("🎮 <b>Game:</b> {}\n", encode_text(&game.name));

        let creator = payload
            .creator_name
            .as_ref()
            .map(|name| format!("👤 <b>Creator:</b> {} ({})\n", encode_text(name), truncated_wallet))
            .unwrap_or_else(|| format!("👤 <b>Creator:</b> {}\n", truncated_wallet));

        let description = payload
            .lobby
            .description
            .as_ref()
            .map(|desc| format!("📝 <b>Description:</b> {}\n", encode_text(desc)))
            .unwrap_or_default();

        let contract_line = payload
            .lobby
            .contract_address
            .as_ref()
            .map(|addr| {
                let network = match state.config.network {
                    Network::Mainnet => "mainnet",
                    Network::Testnet => "testnet",
                };
                format!("🔗 <b>Contract:</b> <a href=\"https://explorer.hiro.so/txid/{}?chain={}\">View on Hiro</a>\n", addr, network)
            })
            .unwrap_or_default();

        let entry_fee_line = if let Some(amount) = payload.lobby.entry_amount {
            if amount > 0.0 {
                let token_symbol = payload.lobby.token_symbol.as_deref().unwrap_or("STX");
                format!("💰 <b>Entry Fee:</b> <code>{:.2} {}</code>\n", amount, token_symbol)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let pool_size_line = if payload.lobby.is_sponsored {
            let token_symbol = payload.lobby.token_symbol.as_deref().unwrap_or("STX");
            format!("💰 <b>Pool Size (Sponsored):</b> <code>{:.2} {}</code>\n", payload.lobby.current_amount.unwrap_or(0.0), token_symbol)
        } else {
            "".to_string()
        };

        let lobby_link = format!(
            "🔗 <b>Lobby Link:</b> <a href=\"https://stackswars.com/room/{}\">Join Now</a>\n",
            payload.lobby.path
        );

        let caption = format!(
            "{}\n{}\n{}{}{}{}{}{}",
            lobby_name,
            game_name,
            creator,
            description,
            contract_line,
            entry_fee_line,
            pool_size_line,
            lobby_link
        );

        let lobby_url: Url = Url::parse(&format!(
            "https://stackswars.com/room/{}",
            payload.lobby.path
        ))
        .unwrap();

        let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
            "🎮 Join Lobby",
            lobby_url,
        )]]);

        let image_url = format!("https://stackswars.com{}", game.image_url);

        let send_result = if game.image_url.to_lowercase().ends_with(".svg") {
            // SVG not supported by send_photo, use send_message instead
            bot.send_message(chat_id, &caption)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .send()
                .await
        } else {
            bot.send_photo(chat_id, InputFile::url(Url::parse(&image_url).unwrap()))
                .caption(&caption)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .send()
                .await
        };

        match send_result {
            Ok(message) => {
                // Update tg_msg_id in Redis
                let lobby_state_repo =LobbyStateRepository::new(state.redis);
                let _ = lobby_state_repo.update_tg_msg_id(payload.lobby.id, message.id.0).await;
                tracing::info!("Broadcasted lobby creation to Telegram for lobby {}", payload.lobby.id);
            }
            Err(e) => {
                tracing::error!("Failed to send lobby creation message to Telegram: {}", e);
            }
        }
    });
}

/// Broadcast lobby winner to Telegram
pub async fn broadcast_lobby_winner_to_tg(
    state: AppState,
    lobby_id: Uuid,
) {
    tokio::spawn(async move {
        let bot = &state.bot;
        let chat_id = ChatId(state.config.telegram_chat_id.parse().unwrap_or(0));

        // Fetch lobby state for tg_msg_id
        let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
        let lobby_state = match lobby_state_repo.get_state(lobby_id).await {
            Ok(ls) => ls,
            Err(e) => {
                tracing::error!("Failed to fetch lobby state {}: {}", lobby_id, e);
                return;
            }
        };

        // Only broadcast if we have tg_msg_id (meaning it was created via Telegram)
        let tg_msg_id = match lobby_state.tg_msg_id {
            Some(id) => id,
            None => return,
        };

        // Fetch lobby and players in parallel
        let lobby_repo = LobbyRepository::new(state.postgres.clone());
        let player_repo = PlayerStateRepository::new(state.redis.clone());

        let (lobby_result, players_result) = tokio::join!(
            lobby_repo.find_by_id(lobby_id),
            player_repo.get_all_in_lobby(lobby_id)
        );

        let lobby = match lobby_result {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to fetch lobby {}: {}", lobby_id, e);
                return;
            }
        };

        let players = match players_result {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to fetch players for lobby {}: {}", lobby_id, e);
                return;
            }
        };

        // Fetch game details
        let game_repo = GameRepository::new(state.postgres.clone());
        let game = match game_repo.find_by_id(lobby.game_id).await {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to fetch game {}: {}", lobby.game_id, e);
                return;
            }
        };

        // Find winner (player with rank 1)
        let winner = players.iter().find(|p| p.rank == Some(1));
        if winner.is_none() {
            tracing::warn!("No winner found for lobby {}", lobby_id);
            return;
        }
        let winner = winner.unwrap();

        let winner_wallet = winner.wallet_address.clone();
        let truncated_winner_wallet = format!(
            "{}...{}",
            &winner_wallet[0..4],
            &winner_wallet[winner_wallet.len() - 4..]
        );

        let winner_display = winner
            .display_name
            .clone()
            .or(winner.username.clone())
            .map(|name| format!("{} ({})", encode_text(&name), truncated_winner_wallet))
            .unwrap_or_else(|| truncated_winner_wallet.clone());

        let mut content = format!(
            "🏆 <b>Game Finished!</b>\n\n🎮 <b>Game:</b> {}\n🏆 <b>Winner:</b> {}\n",
            encode_text(&game.name),
            winner_display
        );

        // Add prize information if there's a prize
        if let Some(prize) = winner.prize {
            if prize > 0.0 {
                let token_symbol = lobby.token_symbol.as_deref().unwrap_or("STX");
                content.push_str(&format!("💰 <b>Prize:</b> <code>{:.2} {}</code>\n", prize, token_symbol));
            }
        }

        // Add runner-ups (rank 2 and 3)
        let runner_ups: Vec<_> = players.iter()
            .filter(|p| p.rank == Some(2) || p.rank == Some(3))
            .collect();

        if !runner_ups.is_empty() {
            content.push_str("\n🏅 <b>Runner-ups:</b>\n");
            for runner_up in &runner_ups {
                let runner_wallet = runner_up.wallet_address.clone();
                let truncated_runner_wallet = format!(
                    "{}...{}",
                    &runner_wallet[0..4],
                    &runner_wallet[runner_wallet.len() - 4..]
                );

                let runner_display = runner_up
                    .display_name
                    .clone()
                    .or(runner_up.username.clone())
                    .map(|name| format!("{} ({})", encode_text(&name), truncated_runner_wallet))
                    .unwrap_or_else(|| truncated_runner_wallet.clone());

                let position = match runner_up.rank {
                    Some(2) => "2nd",
                    Some(3) => "3rd",
                    _ => "Unknown",
                };

                content.push_str(&format!("  {}: {} - ", position, runner_display));

                if let Some(prize) = runner_up.prize {
                    let token_symbol = lobby.token_symbol.as_deref().unwrap_or("STX");
                    content.push_str(&format!("<code>{:.2} {}</code>\n", prize, token_symbol));
                } else {
                    content.push_str("");
                }
            }
        }

        content.push_str(&format!("\n🏆 <b>Lobby:</b> {}\n", encode_text(&lobby.name)));
        content.push_str("🌐 <b>Play at:</b> https://stackswars.com");

        let image_url = format!("https://stackswars.com{}", game.image_url);

        let send_result = if game.image_url.to_lowercase().ends_with(".svg") {
            // SVG not supported by send_photo, use send_message instead
            bot.send_message(chat_id, &content)
                .parse_mode(ParseMode::Html)
                .reply_parameters(ReplyParameters::new(MessageId(tg_msg_id)))
                .await
        } else {
            bot.send_photo(chat_id, InputFile::url(Url::parse(&image_url).unwrap()))
                .caption(&content)
                .parse_mode(ParseMode::Html)
                .reply_parameters(ReplyParameters::new(MessageId(tg_msg_id)))
                .await
        };

        match send_result {
            Ok(_) => {
                tracing::info!("Broadcasted lobby winner to Telegram for lobby {}", lobby_id);
            }
            Err(e) => {
                tracing::error!("Failed to send lobby winner message to Telegram: {}", e);
            }
        }
    });
}

/// Delete lobby creation message from Telegram
pub async fn delete_lobby_creation_message(
    state: AppState,
    tg_msg_id: i32,
) {
    if tg_msg_id == 0 {
        return;
    }
    tokio::spawn(async move {
        let bot = &state.bot;
        let chat_id = ChatId(state.config.telegram_chat_id.parse().unwrap_or(0));

        match bot
            .delete_message(chat_id, MessageId(tg_msg_id))
            .await
        {
            Ok(_) => {
                tracing::debug!("Deleted lobby creation message from Telegram: {}", tg_msg_id);
            }
            Err(e) => {
                tracing::error!("Failed to delete lobby creation message from Telegram: {}", e);
            }
        }
    });
}