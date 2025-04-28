use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, ChatId, InlineKeyboardMarkup, Me, Message, ParseMode},
    utils::command::BotCommands,
    Bot,
};

use crate::db;
use crate::route::build_buttons;
use crate::users;

use rust_embed::RustEmbed;

const ADMIN_ID: i64 = 283564928;

#[derive(RustEmbed)]
#[folder = "src/contents/"] // Path to your contents directory
struct Contents;

/// These commands are supported:
#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text
    Help,
    /// Start
    Start,
    /// About Us
    About,
    /// Recycling
    Recycling,
    /// Plastic
    Plastic,
    /// Paper
    Paper,
    /// Metal
    Metal,
    /// Glass
    Glass,
    /// Organic
    Organic,
    /// Other
    Other,
    /// GiveAway
    GiveAway,
    /// FAQ
    FAQ,
    /// Broadcast a message to all users (admin only)
    Broadcast,
    /// Send a test message to a specific user
    TestMessage,
}

fn escape_markdown_v2(text: String) -> String {
    text.replace('.', "\\.")
        .replace('-', "\\-")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('!', "\\!")
}

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Message received: {}", msg.text().unwrap());
    // Store user ID
    if let Some(user) = msg.from() {
        let user_id = user.id.0;
        if let Ok(is_new_user) = users::store_user(user_id.try_into().unwrap()).await {
            if is_new_user {
                info!("New user registered from message: {}", user_id);
            }
        }
    }

    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::Broadcast) => {
                if msg.chat.id == ChatId(ADMIN_ID) {
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    let second_part = if parts.len() > 1 { parts[1] } else { "" };

                    let (buttons, content) = build_details(second_part, true)?;
                    // get all users and send
                    let users = users::get_all_users().await?;
                    for user_id in users {
                        log::info!("broadcasting to user: {}", user_id);
                        bot.send_message(ChatId(user_id), &content)
                            .disable_web_page_preview(true)
                            .parse_mode(ParseMode::MarkdownV2)
                            .reply_markup(buttons.clone())
                            .await?;
                        // sleep 2 seconds
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Неизвестная команда: {}. Попробуйте начать сначала написав \"Бот\"",
                            text
                        ),
                    )
                    .await?;
                }
            }
            Ok(Command::TestMessage) => {
                // Send a test message to the specific chat ID
                let test_chat_id = 108609383; // The user's chat ID for testing
                if msg.chat.id == ChatId(ADMIN_ID) {
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    let second_part = if parts.len() > 1 { parts[1] } else { "" };

                    let (buttons, content) = build_details(second_part, true)?;

                    bot.send_message(ChatId(test_chat_id), content)
                        .disable_web_page_preview(true)
                        .parse_mode(ParseMode::MarkdownV2)
                        .reply_markup(buttons)
                        .await?;
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Неизвестная команда: {}. Попробуйте начать сначала написав \"Бот\"",
                            text
                        ),
                    )
                    .await?;
                }
            }
            Ok(
                Command::Start
                | Command::About
                | Command::Recycling
                | Command::GiveAway
                | Command::FAQ
                | Command::Plastic
                | Command::Paper
                | Command::Metal
                | Command::Glass
                | Command::Organic
                | Command::Other,
            ) => {
                let (buttons, content) = build_details(text, false)?;

                bot.send_message(msg.chat.id, content)
                    .disable_web_page_preview(true)
                    .parse_mode(ParseMode::MarkdownV2)
                    .reply_markup(buttons)
                    .await?;
            }
            Err(_) => {
                match text {
                    "бот" | "Бот" => {
                        let (buttons, content) = build_details("start", false)?;
                        bot.send_message(msg.chat.id, content)
                            .disable_web_page_preview(true)
                            .parse_mode(ParseMode::MarkdownV2)
                            .reply_markup(buttons)
                            .await?;
                    }
                    _ => {
                        bot.send_message(msg.chat.id, format!("Неизвестная команда: {}. Попробуйте начать сначала написав \"Бот\"", text))
                            .await?;
                    }
                };
            }
        }
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Store user ID
    let user_id = q.from.id.0;
    if let Ok(is_new_user) = users::store_user(user_id.try_into().unwrap()).await {
        if is_new_user {
            info!("New user registered from callback: {}", user_id);
        }
    }

    if let Some(ref text) = q.data {
        log::info!("callback: {}", text);
        // Tell telegram that we've seen this query, to remove 🕑 icons from the
        // clients. You could also use `answer_callback_query`'s optional
        // parameters to tweak what happens on the client side.
        bot.answer_callback_query(&q.id).await?;

        let (buttons, content) = build_details(text, false)?;

        match bot
            .send_message(q.from.id, content)
            .disable_web_page_preview(true)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(buttons)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                error!("Error sending message: {:?}", e);
                bot.send_message(q.from.id, e.to_string()).await?;
            }
        };
    }

    Ok(())
}

fn build_details(
    text: &str,
    is_external: bool,
) -> Result<(InlineKeyboardMarkup, String), Box<dyn Error + Send + Sync>> {
    let route = text.replace("/", "");
    let file_name = format!("{}.md", &route);
    let content = Contents::get(&file_name)
        .ok_or_else(|| format!("File {} not found", file_name))?
        .data;

    let content = String::from_utf8(content.to_vec())?;
    let buttons = build_buttons(&route, is_external);

    Ok((buttons, escape_markdown_v2(content)))
}
