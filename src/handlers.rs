use std::error::Error;

use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, ChatId, Me, Message, ParseMode},
    utils::command::BotCommands,
    Bot,
};

use crate::commands::{
    AdventCommand, BroadcastCommand, ContentCommand, LocationCommand, StopCommand,
    SubscriptionCommand, ADMIN_ID, TEST_USER_ID,
};
use crate::users;

/// These commands are supported:
#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
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

fn send_unknown_command_message(text: &str) -> String {
    format!(
        "Неизвестная команда: {}. Попробуйте начать сначала написав \"Бот\"",
        text
    )
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
        LocationCommand::handle(&bot, msg.chat.id, location.latitude, location.longitude).await?;
        return Ok(());
    }

    // Handle text messages
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::Broadcast) => {
                if msg.chat.id == ChatId(ADMIN_ID) {
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    let route = if parts.len() > 1 { parts[1] } else { "" };
                    BroadcastCommand::send_to_all(&bot, msg.chat.id, route).await?;
                } else {
                    bot.send_message(msg.chat.id, send_unknown_command_message(text))
                        .await?;
                }
            }
            Ok(Command::TestMessage) => {
                if msg.chat.id == ChatId(ADMIN_ID) {
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    let route = if parts.len() > 1 { parts[1] } else { "" };
                    BroadcastCommand::send_test(&bot, msg.chat.id, TEST_USER_ID, route).await?;
                } else {
                    bot.send_message(msg.chat.id, send_unknown_command_message(text))
                        .await?;
                }
            }
            Ok(Command::AdventTest) => {
                if msg.chat.id == ChatId(ADMIN_ID) {
                    AdventCommand::send_test(&bot, msg.chat.id, TEST_USER_ID).await?;
                } else {
                    bot.send_message(msg.chat.id, send_unknown_command_message(text))
                        .await?;
                }
            }
            Ok(Command::Stop) => {
                if let Some(user) = msg.from() {
                    let user_id: i64 = user.id.0.try_into().unwrap();
                    StopCommand::handle(&bot, msg.chat.id, user_id).await?;
                }
            }
            Ok(Command::Advent) => {
                if msg.chat.id == ChatId(ADMIN_ID) {
                    AdventCommand::send_to_all(&bot, msg.chat.id).await?;
                } else {
                    bot.send_message(msg.chat.id, send_unknown_command_message(text))
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
                ContentCommand::send(&bot, msg.chat.id, text).await?;
            }
            Err(_) => {
                match text {
                    "бот" | "Бот" => {
                        ContentCommand::send(&bot, msg.chat.id, "start").await?;
                    }
                    "стоп" | "Стоп" | "СТОП" => {
                        if let Some(user) = msg.from() {
                            let user_id: i64 = user.id.0.try_into().unwrap();
                            StopCommand::handle(&bot, msg.chat.id, user_id).await?;
                        }
                    }
                    _ => {
                        bot.send_message(msg.chat.id, send_unknown_command_message(text))
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
        bot.answer_callback_query(&q.id).await?;

        // Handle subscribe/unsubscribe actions
        if text.starts_with("/subscribe_") {
            let subscription_type = text.strip_prefix("/subscribe_").unwrap();
            let _ = SubscriptionCommand::subscribe(&bot, q.from.id, subscription_type, text).await;
            return Ok(());
        } else if text.starts_with("/unsubscribe_") {
            let subscription_type = text.strip_prefix("/unsubscribe_").unwrap();
            let _ =
                SubscriptionCommand::unsubscribe(&bot, q.from.id, subscription_type, text).await;
            return Ok(());
        }

        // Handle regular content navigation
        let user_id_i64: i64 = user_id.try_into().unwrap();
        if let Err(e) =
            ContentCommand::send_with_user(&bot, q.from.id.into(), text, user_id_i64).await
        {
            error!("Error sending message: {:?}", e);
            bot.send_message(q.from.id, e.to_string()).await?;
        }
    }

    Ok(())
}
