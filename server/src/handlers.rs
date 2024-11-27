use std::{error::Error, fs::read_to_string};

use log::info;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, InlineKeyboardMarkup, Me, Message, ParseMode},
    utils::command::BotCommands,
    Bot,
};

use crate::route::build_buttons;

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
    text.replace('.', "\\.") // Escape dots
        .replace('-', "\\-") // Escape hyphens
        .replace('{', "\\{") // Escape curly braces
        .replace('}', "\\}") // Escape curly braces
        .replace('!', "\\!") // Escape exclamation marks
}

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
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
                info!("Sending:  content: {}, text: {}", content, text);
                bot.send_message(msg.chat.id, content)
                    .parse_mode(ParseMode::MarkdownV2)
                    .reply_markup(buttons)
                    .await?;
            }
            Err(_) => {
                match text {
                    "бот" | "Бот" => {
                        let (buttons, content) = build_details("start")?;
                        bot.send_message(msg.chat.id, content)
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
    if let Some(ref text) = q.data {
        // Tell telegram that we've seen this query, to remove 🕑 icons from the
        // clients. You could also use `answer_callback_query`'s optional
        // parameters to tweak what happens on the client side.
        bot.answer_callback_query(&q.id).await?;
        let (buttons, content) = build_details(text)?;
        info!("Sending:  content: {}, text: {}", content, text);
        bot.send_message(q.from.id, content)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(buttons)
            .await?;
    }

    Ok(())
}

fn build_details(
    text: &str,
) -> Result<(InlineKeyboardMarkup, String), Box<dyn Error + Send + Sync>> {
    let route = text.replace("/", "");
    let path = format!("{}/src/contents/{}.md", env!("CARGO_MANIFEST_DIR"), &route);
    let content = read_to_string(path)?;
    let buttons = build_buttons(&route);

    Ok((buttons, escape_markdown_v2(content)))
}
