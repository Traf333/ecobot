use log::error;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ParseMode, UserId},
    Bot,
};

use crate::db;

use super::common::build_details;

pub struct SubscriptionCommand;

impl SubscriptionCommand {
    pub async fn subscribe(
        bot: &Bot,
        user_id: UserId,
        subscription_type: &str,
        callback_text: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let user_id_i64: i64 = user_id.0.try_into().unwrap();

        match db::subscribe_user(user_id_i64, subscription_type).await {
            Ok(true) => {
                let (buttons, content) = build_details(callback_text, false)?;
                bot.send_message(user_id, content)
                    .disable_web_page_preview(true)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(buttons)
                    .await?;
                Ok(true)
            }
            Ok(false) => {
                bot.send_message(user_id, "Вы уже подписаны на эту рассылку.")
                    .await?;
                Ok(false)
            }
            Err(e) => {
                error!("Error subscribing user: {:?}", e);
                bot.send_message(user_id, "Произошла ошибка при подписке.")
                    .await?;
                Err(e.into())
            }
        }
    }

    pub async fn unsubscribe(
        bot: &Bot,
        user_id: UserId,
        subscription_type: &str,
        callback_text: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let user_id_i64: i64 = user_id.0.try_into().unwrap();

        match db::unsubscribe_user(user_id_i64, subscription_type).await {
            Ok(true) => {
                let (buttons, content) = build_details(callback_text, false)?;
                bot.send_message(user_id, content)
                    .disable_web_page_preview(true)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(buttons)
                    .await?;
                Ok(true)
            }
            Ok(false) => {
                bot.send_message(user_id, "Вы не подписаны на эту рассылку.")
                    .await?;
                Ok(false)
            }
            Err(e) => {
                error!("Error unsubscribing user: {:?}", e);
                bot.send_message(user_id, "Произошла ошибка при отписке.")
                    .await?;
                Err(e.into())
            }
        }
    }
}
