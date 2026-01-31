use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, ParseMode},
    Bot,
};

use crate::users::{self, blacklist_user, get_active_users};

use super::common::build_details_with_user;

pub struct BroadcastCommand;

impl BroadcastCommand {
    /// Send message to a single user
    async fn send_to_user(
        bot: &Bot,
        user_id: i64,
        route: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (buttons, content) = build_details_with_user(route, true, Some(user_id))?;

        bot.send_message(ChatId(user_id), &content)
            .disable_web_page_preview(true)
            .parse_mode(ParseMode::Html)
            .reply_markup(buttons)
            .await?;

        Ok(())
    }

    /// Send test message to a specific user (admin command)
    pub async fn send_test(
        bot: &Bot,
        admin_chat_id: ChatId,
        test_user_id: i64,
        route: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match Self::send_to_user(bot, test_user_id, route).await {
            Ok(_) => {
                info!("Test message sent to user: {}", test_user_id);
            }
            Err(err) => {
                error!(
                    "Failed to send test message to user {}: {:?}",
                    test_user_id, err
                );
                bot.send_message(
                    admin_chat_id,
                    format!("Ошибка при отправке сообщения: {:?}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Broadcast message to all users (admin command)
    pub async fn send_to_all(
        bot: &Bot,
        admin_chat_id: ChatId,
        route: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let users = get_active_users().await?;
        info!("Broadcasting to {} active users", users.len());

        let mut success_count = 0;
        let mut error_count = 0;
        let mut blacklisted_count = 0;

        for user_id in users {
            match Self::send_to_user(bot, user_id, route).await {
                Ok(_) => {
                    success_count += 1;
                    info!("Message sent to user: {}", user_id);
                }
                Err(err) => {
                    error_count += 1;
                    error!("Failed to send message to user {}: {:?}", user_id, err);

                    // Blacklist user on send failure
                    if let Err(e) = blacklist_user(user_id).await {
                        error!("Failed to blacklist user {}: {:?}", user_id, e);
                    } else {
                        blacklisted_count += 1;
                        info!("User {} has been blacklisted", user_id);
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        bot.send_message(
            admin_chat_id,
            format!(
                "Отправка завершена.\nУспешно: {}\nОшибок: {}\nЗаблокировано: {}",
                success_count, error_count, blacklisted_count
            ),
        )
        .await?;

        Ok(())
    }
}
