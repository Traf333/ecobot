use log::error;
use teloxide::{prelude::Requester, types::ChatId, Bot};

use crate::db;

use super::common::{escape_markdown_v2, Contents};

pub struct StopCommand;

impl StopCommand {
    pub async fn handle(
        bot: &Bot,
        chat_id: ChatId,
        user_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match db::unsubscribe_all(user_id).await {
            Ok(true) => {
                let content = match Contents::get("unsubscribe_advent.md") {
                    Some(file) => match String::from_utf8(file.data.to_vec()) {
                        Ok(content) => escape_markdown_v2(content),
                        Err(e) => {
                            error!("Failed to parse unsubscribe_advent.md: {:?}", e);
                            bot.send_message(
                                chat_id,
                                "Ошибка при загрузке файла unsubscribe_advent.md",
                            )
                            .await?;
                            return Ok(());
                        }
                    },
                    None => {
                        error!("unsubscribe_advent.md not found");
                        bot.send_message(chat_id, "Файл unsubscribe_advent.md не найден")
                            .await?;
                        return Ok(());
                    }
                };
                // Note: content is loaded but not sent - this appears to be a bug in original code
                // Keeping behavior as-is for now
                let _ = content;
            }
            Ok(false) => {
                bot.send_message(chat_id, "Вы не подписаны ни на одну рассылку.")
                    .await?;
            }
            Err(e) => {
                error!("Error unsubscribing user from all: {:?}", e);
                bot.send_message(chat_id, "Произошла ошибка при отписке.")
                    .await?;
            }
        }

        Ok(())
    }
}
