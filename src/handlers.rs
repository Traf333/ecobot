use std::error::Error;

use log::{error, info};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, InlineKeyboardMarkup, Me, Message, ParseMode},
    utils::command::BotCommands,
    Bot,
};

use crate::route::build_buttons;

use rust_embed::RustEmbed;

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
    if let Some(text) = msg.text() {
        sentry::with_scope(
            |scope| {
                scope.set_tag("chat_id", msg.chat.id.to_string());
                scope.set_tag("command", text);
                scope.set_tag("source", "message");
            },
            || {
                sentry::capture_message(&text, sentry::Level::Info);
            },
        );

        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
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
                let (buttons, content) = build_details(text)?;

                bot.send_message(msg.chat.id, content)
                    .disable_web_page_preview(true)
                    .parse_mode(ParseMode::MarkdownV2)
                    .reply_markup(buttons)
                    .await?;
            }
            Err(_) => {
                match text {
                    "бот" | "Бот" => {
                        let (buttons, content) = build_details("start")?;
                        bot.send_message(msg.chat.id, content)
                            .disable_web_page_preview(true)
                            .parse_mode(ParseMode::MarkdownV2)
                            .reply_markup(buttons)
                            .await?
                    }
                    _ => {
                        bot.send_message(msg.chat.id, format!("Неизвестная команда: {}. Попробуйте начать сначала написав \"Бот\"", text))
                            .await?
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
    sentry::capture_message("Test message", sentry::Level::Info);
    if let Some(ref text) = q.data {
        // Tell telegram that we've seen this query, to remove 🕑 icons from the
        // clients. You could also use `answer_callback_query`'s optional
        // parameters to tweak what happens on the client side.
        bot.answer_callback_query(&q.id).await?;

        let (buttons, content) = build_details(text)?;
        sentry::with_scope(
            |scope| {
                scope.set_tag("chat_id", q.from.id.to_string());
                scope.set_tag("command", text);
                scope.set_tag("source", "callback");
            },
            || {
                sentry::capture_message(&text, sentry::Level::Info);
            },
        );

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
) -> Result<(InlineKeyboardMarkup, String), Box<dyn Error + Send + Sync>> {
    let route = text.replace("/", "");
    let file_name = format!("{}.md", &route);
    let content = Contents::get(&file_name)
        .ok_or_else(|| format!("File {} not found", file_name))?
        .data;

    let content = String::from_utf8(content.to_vec())?;
    let buttons = build_buttons(&route);

    Ok((buttons, escape_markdown_v2(content)))
}
