use std::error::Error;
use std::fs::File;
use std::i64;
use std::io::{BufRead, BufReader};
use std::path::Path;

use log::{error, info};
use reqwest::Url;
use teloxide::types::InlineKeyboardButton;
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
    /// Find
    Find,
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
    teloxide::utils::markdown::escape(&text).into_owned()
}

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Store user ID
    if let Some(user) = msg.from() {
        let user_id = user.id.0;
        if let Ok(is_new_user) = users::store_user(user_id.try_into().unwrap()).await {
            if is_new_user {
                info!("New user registered from message: {}", user_id);
            }
        }
    }

    // Handle location message
    if let Some(location) = msg.location() {
        let latitude = location.latitude;
        let longitude = location.longitude;
        log::info!("Location received: {} {}", latitude, longitude);
        let bin_locations = db::get_bin_locations(latitude, longitude).await?;

        let mut content = "".to_string();

        if bin_locations.is_empty() {
            content = "*3- и 4-секционные контейнеры РСО в радиусе 1 км не найдены.*".to_string();
            content.push_str("\n👉 Проверить самостоятельно [на сайте обслуживающей компании ЕСОО](https://new.esoo39.ru/%d1%80%d1%81%d0%be/)");
        } else {
            content = "*Ближайшие 3- и 4-секционные контейнеры РСО:*".to_string();
            for (distance, bin_location) in bin_locations.into_iter().take(2) {
                let distance = distance;
                let bin_location = bin_location;

                let link_url = format!(
                    "https://yandex.ru/maps/?rtext={},{}~{},{}&rtt=pedestrian",
                    latitude, longitude, bin_location.latitude, bin_location.longitude
                );
                let glass_text = if bin_location.preset == "islands#darkgreenIcon" {
                    "со стеклом"
                } else {
                    "без стекла"
                };
                let bin_text = format!(
                    "\n{} м [{}]({}) {}",
                    (distance * 1000.0).round(),
                    bin_location.address,
                    link_url,
                    glass_text
                );
                content.push_str(&bin_text);
            }
            content.push_str("\n👉 Проверить самостоятельно [на сайте обслуживающей компании ЕСОО](https://new.esoo39.ru/%d1%80%d1%81%d0%be/)");
        }

        let main_point = db::main_point();
        let distance_to_main = (main_point.distance(latitude, longitude) * 1000.0).round();
        if distance_to_main < 1000.0 {
            content.push_str(
                &format!("\n\nПлощадка раздельного сбора с самым большим перечнем принимаемых фракций находится на [ул. 5-я Причальная 2а](https://yandex.ru/maps/?rtext={},{}~{},{}&rtt=pedestrian) в радиусе {} м.", latitude, longitude, main_point.latitude, main_point.longitude, distance_to_main)
            );
        } else {
            content.push_str(
                &format!("\n\nПлощадка раздельного сбора с самым большим перечнем принимаемых фракций находится на [ул. 5-я Причальная 2а](https://yandex.ru/maps/?text={},{}).", main_point.latitude, main_point.longitude)
            );
        }

        content.push_str(
            "\n\nОтправьте новую геопозицию, если хотите найти другие контейнеры.\nОтправьте «Бот», если хотите вернуться в начало.",
        );

        bot.send_message(msg.chat.id, escape_markdown_v2(content))
            .disable_web_page_preview(true)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        return Ok(());
    }
    // Handle text messages
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
                    let blacklisted_ids: Vec<i64> = vec![
                        1137539828, 245300509, 480442732, 108609383, 370942991, 912211086,
                        1030476206, 439366454, 828969293, 262894147, 473789332, 393940845,
                        5065333159, 1067873349, 683464776, 835102541, 6471579202, 6467162272,
                    ];
                    let (buttons, content) = build_details(second_part, true)?;
                    // get all users and send
                    let users = users::get_all_users().await?;
                    for user_id in users {
                        if blacklisted_ids.contains(&user_id) {
                            continue;
                        }

                        match bot
                            .send_message(ChatId(user_id), &content)
                            .disable_web_page_preview(true)
                            .parse_mode(ParseMode::MarkdownV2)
                            .reply_markup(buttons.clone())
                            .await
                        {
                            Ok(_) => log::info!("Message sent to user: {}", user_id),
                            Err(err) => {
                                log::error!(
                                    "Failed to send message to user {}: {:?}",
                                    user_id,
                                    err
                                );
                                // Continue with the next user if this one has errors
                                continue;
                            }
                        };
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
                | Command::Find
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
