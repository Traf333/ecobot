use std::error::Error;
use std::fs::File;
use std::i64;
use std::io::{BufRead, BufReader};
use std::path::Path;

use log::{error, info};
use reqwest::Url;
use teloxide::types::InlineKeyboardButton;
use teloxide::{
    payloads::{SendMessageSetters, SendPhotoSetters},
    prelude::Requester,
    types::{CallbackQuery, ChatId, InlineKeyboardMarkup, InputFile, Me, Message, ParseMode},
    utils::command::BotCommands,
    Bot,
};

use crate::db;
use crate::route::{build_buttons, build_buttons_with_user};
use crate::users;

use rust_embed::RustEmbed;

const ADMIN_ID: i64 = 283564928;

#[derive(RustEmbed)]
#[folder = "src/contents/"] // Path to your contents directory
struct Contents;

#[derive(RustEmbed)]
#[folder = "src/images/"] // Path to your images directory
struct Images;

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
    /// Advent calendar
    Advent,
    /// Test advent message to specific user
    AdventTest,
    /// Stop all subscriptions
    Stop,
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
                    let blacklisted_ids: Vec<i64> = vec![];

                    // get all users and send
                    let users = users::get_all_users().await?;
                    for user_id in users {
                        if blacklisted_ids.contains(&user_id) {
                            continue;
                        }

                        // Build personalized buttons for each user
                        let (buttons, content) =
                            build_details_with_user(second_part, true, Some(user_id))?;

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

                    // Build buttons with the test user's ID for personalized subscription buttons
                    let (buttons, content) =
                        build_details_with_user(second_part, true, Some(test_chat_id))?;

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
            Ok(Command::AdventTest) => {
                // Admin only - send advent.md to test user
                let test_chat_id = 108609383;
                if msg.chat.id == ChatId(ADMIN_ID) {
                    // Get content from advent.md
                    let content = match Contents::get("advent.md") {
                        Some(file) => match String::from_utf8(file.data.to_vec()) {
                            Ok(content) => escape_markdown_v2(content),
                            Err(e) => {
                                error!("Failed to parse advent.md: {:?}", e);
                                bot.send_message(
                                    msg.chat.id,
                                    "Ошибка при загрузке файла advent.md",
                                )
                                .await?;
                                return Ok(());
                            }
                        },
                        None => {
                            error!("advent.md not found");
                            bot.send_message(msg.chat.id, "Файл advent.md не найден")
                                .await?;
                            return Ok(());
                        }
                    };

                    info!("Sending test advent.md to user {}", test_chat_id);

                    let photo_url = "https://raw.githubusercontent.com/Traf333/ecobot/refs/heads/main/src/images/advent.jpg";
                    match bot
                        .send_photo(
                            ChatId(test_chat_id),
                            InputFile::url(Url::parse(photo_url).unwrap()),
                        )
                        .caption(&content)
                        .parse_mode(ParseMode::MarkdownV2)
                        .await
                    {
                        Ok(_) => {
                            bot.send_message(
                                msg.chat.id,
                                format!(
                                    "Тестовое сообщение отправлено пользователю {}",
                                    test_chat_id
                                ),
                            )
                            .await?;
                        }
                        Err(err) => {
                            error!("Failed to send test advent message: {:?}", err);
                            bot.send_message(
                                msg.chat.id,
                                format!("Ошибка при отправке сообщения: {:?}", err),
                            )
                            .await?;
                        }
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
            Ok(Command::Stop) => {
                if let Some(user) = msg.from() {
                    let user_id = user.id.0;

                    match db::unsubscribe_all(user_id.try_into().unwrap()).await {
                        Ok(true) => {
                            let content = match Contents::get("unsubscribe_advent.md") {
                                Some(file) => match String::from_utf8(file.data.to_vec()) {
                                    Ok(content) => escape_markdown_v2(content),
                                    Err(e) => {
                                        error!("Failed to parse unsubscribe_advent.md: {:?}", e);
                                        bot.send_message(
                                            msg.chat.id,
                                            "Ошибка при загрузке файла unsubscribe_advent.md",
                                        )
                                        .await?;
                                        return Ok(());
                                    }
                                },
                                None => {
                                    error!("unsubscribe_advent.md not found");
                                    bot.send_message(
                                        msg.chat.id,
                                        "Файл unsubscribe_advent.md не найден",
                                    )
                                    .await?;
                                    return Ok(());
                                }
                            };
                        }
                        Ok(false) => {
                            bot.send_message(msg.chat.id, "Вы не подписаны ни на одну рассылку.")
                                .await?;
                        }
                        Err(e) => {
                            error!("Error unsubscribing user from all: {:?}", e);
                            bot.send_message(msg.chat.id, "Произошла ошибка при отписке.")
                                .await?;
                        }
                    }
                }
            }
            Ok(Command::Advent) => {
                // Admin only - send advent.md to all users subscribed to "advent"
                if msg.chat.id == ChatId(ADMIN_ID) {
                    // Get content from advent.md
                    let content = match Contents::get("advent.md") {
                        Some(file) => match String::from_utf8(file.data.to_vec()) {
                            Ok(content) => escape_markdown_v2(content),
                            Err(e) => {
                                error!("Failed to parse advent.md: {:?}", e);
                                bot.send_message(
                                    msg.chat.id,
                                    "Ошибка при загрузке файла advent.md",
                                )
                                .await?;
                                return Ok(());
                            }
                        },
                        None => {
                            error!("advent.md not found");
                            bot.send_message(msg.chat.id, "Файл advent.md не найден")
                                .await?;
                            return Ok(());
                        }
                    };

                    // Get all users subscribed to "advent"
                    let users = match db::get_users_by_subscription("advent").await {
                        Ok(users) => users,
                        Err(e) => {
                            error!("Failed to get users by subscription: {:?}", e);
                            bot.send_message(msg.chat.id, "Ошибка при получении подписчиков")
                                .await?;
                            return Ok(());
                        }
                    };

                    info!("Sending advent.md to {} users", users.len());

                    bot.send_message(
                        msg.chat.id,
                        format!("Отправка сообщения {} подписчикам...", users.len()),
                    )
                    .await?;

                    let mut success_count = 0;
                    let mut error_count = 0;
                    let photo_url = "https://raw.githubusercontent.com/Traf333/ecobot/refs/heads/main/src/images/advent.jpg";

                    for user_id in users {
                        match bot
                            .send_photo(
                                ChatId(user_id),
                                InputFile::url(Url::parse(photo_url).unwrap()),
                            )
                            .caption(&content)
                            .parse_mode(ParseMode::MarkdownV2)
                            .await
                        {
                            Ok(_) => {
                                success_count += 1;
                                log::info!("Advent message sent to user: {}", user_id);
                            }
                            Err(err) => {
                                error_count += 1;
                                log::error!(
                                    "Failed to send advent message to user {}: {:?}",
                                    user_id,
                                    err
                                );
                            }
                        }

                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }

                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Отправка завершена.\nУспешно: {}\nОшибок: {}",
                            success_count, error_count
                        ),
                    )
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
                    "стоп" | "Стоп" => {}
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

        // Handle subscribe/unsubscribe actions
        if text.starts_with("/subscribe_") {
            let subscription_type = text.strip_prefix("/subscribe_").unwrap();
            match db::subscribe_user(user_id.try_into().unwrap(), subscription_type).await {
                Ok(true) => {
                    let (buttons, content) = build_details(text, false)?;
                    bot.send_message(q.from.id, content)
                        .disable_web_page_preview(true)
                        .parse_mode(ParseMode::MarkdownV2)
                        .reply_markup(buttons)
                        .await?;
                }
                Ok(false) => {
                    bot.send_message(q.from.id, "Вы уже подписаны на эту рассылку.")
                        .await?;
                }
                Err(e) => {
                    error!("Error subscribing user: {:?}", e);
                    bot.send_message(q.from.id, "Произошла ошибка при подписке.")
                        .await?;
                }
            }
            return Ok(());
        } else if text.starts_with("/unsubscribe_") {
            let subscription_type = text.strip_prefix("/unsubscribe_").unwrap();
            match db::unsubscribe_user(user_id.try_into().unwrap(), subscription_type).await {
                Ok(true) => {
                    let (buttons, content) = build_details(text, false)?;
                    bot.send_message(q.from.id, content)
                        .disable_web_page_preview(true)
                        .parse_mode(ParseMode::MarkdownV2)
                        .reply_markup(buttons)
                        .await?;
                }
                Ok(false) => {
                    bot.send_message(q.from.id, "Вы не подписаны на эту рассылку.")
                        .await?;
                }
                Err(e) => {
                    error!("Error unsubscribing user: {:?}", e);
                    bot.send_message(q.from.id, "Произошла ошибка при отписке.")
                        .await?;
                }
            }
            return Ok(());
        }

        let (buttons, content) =
            build_details_with_user(text, false, Some(user_id.try_into().unwrap()))?;

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
    build_details_with_user(text, is_external, None)
}

fn build_details_with_user(
    text: &str,
    is_external: bool,
    user_id: Option<i64>,
) -> Result<(InlineKeyboardMarkup, String), Box<dyn Error + Send + Sync>> {
    let route = text.trim_start_matches('/').replace("/", "-");
    let file_name = format!("{}.md", &route);
    let content = Contents::get(&file_name)
        .ok_or_else(|| format!("File {} not found", file_name))?
        .data;

    let content = String::from_utf8(content.to_vec())?;
    let buttons = build_buttons_with_user(&route, is_external, user_id);

    Ok((buttons, escape_markdown_v2(content)))
}
