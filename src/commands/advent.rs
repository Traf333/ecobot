use log::{error, info};
use reqwest::Url;
use teloxide::{
    payloads::SendPhotoSetters,
    prelude::Requester,
    types::{ChatId, InputFile, ParseMode},
    Bot,
};

use crate::db;

use super::common::{escape_markdown_v2, Contents};

const PHOTO_URL: &str =
    "https://raw.githubusercontent.com/Traf333/ecobot/refs/heads/main/src/images/advent5.jpg";

pub struct AdventCommand;

impl AdventCommand {
    /// Load and parse advent.md content
    fn load_content() -> Result<String, String> {
        match Contents::get("advent.md") {
            Some(file) => match String::from_utf8(file.data.to_vec()) {
                Ok(content) => Ok(escape_markdown_v2(content)),
                Err(e) => {
                    error!("Failed to parse advent.md: {:?}", e);
                    Err("Ошибка при загрузке файла advent.md".to_string())
                }
            },
            None => {
                error!("advent.md not found");
                Err("Файл advent.md не найден".to_string())
            }
        }
    }

    /// Send advent message to a single user
    async fn send_to_user(bot: &Bot, user_id: i64, content: &str) -> Result<(), String> {
        bot.send_photo(
            ChatId(user_id),
            InputFile::url(Url::parse(PHOTO_URL).unwrap()),
        )
        .caption(content)
        .parse_mode(ParseMode::MarkdownV2)
        .await
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
    }

    /// Send advent message to a test user (admin command)
    pub async fn send_test(
        bot: &Bot,
        admin_chat_id: ChatId,
        test_user_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = match Self::load_content() {
            Ok(c) => c,
            Err(msg) => {
                bot.send_message(admin_chat_id, msg).await?;
                return Ok(());
            }
        };

        info!("Sending test advent.md to user {}", test_user_id);

        match Self::send_to_user(bot, test_user_id, &content).await {
            Ok(_) => {
                bot.send_message(
                    admin_chat_id,
                    format!(
                        "Тестовое сообщение отправлено пользователю {}",
                        test_user_id
                    ),
                )
                .await?;
            }
            Err(err) => {
                error!("Failed to send test advent message: {}", err);
                bot.send_message(
                    admin_chat_id,
                    format!("Ошибка при отправке сообщения: {}", err),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Send advent message to all subscribed users (admin command)
    pub async fn send_to_all(
        bot: &Bot,
        admin_chat_id: ChatId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = match Self::load_content() {
            Ok(c) => c,
            Err(msg) => {
                bot.send_message(admin_chat_id, msg).await?;
                return Ok(());
            }
        };

        // Get all users subscribed to "advent"
        let users = match db::get_users_by_subscription("advent").await {
            Ok(users) => users,
            Err(e) => {
                error!("Failed to get users by subscription: {:?}", e);
                bot.send_message(admin_chat_id, "Ошибка при получении подписчиков")
                    .await?;
                return Ok(());
            }
        };

        info!("Sending advent.md to {} users", users.len());

        bot.send_message(
            admin_chat_id,
            format!("Отправка сообщения {} подписчикам...", users.len()),
        )
        .await?;

        let mut success_count = 0;
        let mut error_count = 0;

        for user_id in users {
            match Self::send_to_user(bot, user_id, &content).await {
                Ok(_) => {
                    success_count += 1;
                    info!("Advent message sent to user: {}", user_id);
                }
                Err(err) => {
                    error_count += 1;
                    error!("Failed to send advent message to user {}: {}", user_id, err);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_content() {
        // This will test that advent.md exists and can be loaded
        let result = AdventCommand::load_content();
        assert!(result.is_ok(), "advent.md should be loadable");
    }
}
