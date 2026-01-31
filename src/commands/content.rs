use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, ParseMode},
    Bot,
};

use super::common::{build_details, build_details_with_user};

pub struct ContentCommand;

impl ContentCommand {
    /// Send content for a route (used by menu commands like /start, /about, etc.)
    pub async fn send(
        bot: &Bot,
        chat_id: ChatId,
        route: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (buttons, content) = build_details(route, false)?;

        bot.send_message(chat_id, content)
            .disable_web_page_preview(true)
            .parse_mode(ParseMode::Html)
            .reply_markup(buttons)
            .await?;

        Ok(())
    }

    /// Send content for a route with user-specific buttons
    pub async fn send_with_user(
        bot: &Bot,
        chat_id: ChatId,
        route: &str,
        user_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (buttons, content) = build_details_with_user(route, false, Some(user_id))?;

        bot.send_message(chat_id, content)
            .disable_web_page_preview(true)
            .parse_mode(ParseMode::Html)
            .reply_markup(buttons)
            .await?;

        Ok(())
    }
}
