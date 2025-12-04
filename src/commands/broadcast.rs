use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, ParseMode},
    Bot,
};

use crate::users;

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
            .parse_mode(ParseMode::MarkdownV2)
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
        let blacklisted_ids: Vec<i64> = vec![];

        let users = users::get_all_users().await?;
        info!("Broadcasting to {} users", users.len());

        let mut success_count = 0;
        let mut error_count = 0;

        for user_id in users {
            if blacklisted_ids.contains(&user_id) {
                continue;
            }

            match Self::send_to_user(bot, user_id, route).await {
                Ok(_) => {
                    success_count += 1;
                    info!("Message sent to user: {}", user_id);
                }
                Err(err) => {
                    error_count += 1;
                    error!("Failed to send message to user {}: {:?}", user_id, err);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        bot.send_message(
            admin_chat_id,
            format!(
                "Отправка завершена.\nУспешно: {}\nОшибок: {}",
                success_count, error_count
            ),
        )
        .await?;

        Ok(())
    }
}
