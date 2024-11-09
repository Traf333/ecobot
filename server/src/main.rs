mod route;
use once_cell::sync::Lazy;
use route::{routes, Route};
use std::collections::HashMap;
use std::error::Error;
use teloxide::{
    dispatching::dialogue::GetChatId,
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me,
    },
    utils::command::BotCommands,
};

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
    /// Solid Plastic
    SolidPlastic,
    /// Soft Plastic
    SoftPlastic,
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
}

static ROUTES: Lazy<HashMap<String, Route>> = Lazy::new(|| routes().unwrap());

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting buttons bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

/// Creates a keyboard made by buttons in a big column.
fn make_keyboard(category: &str) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let route = ROUTES.get(category).expect("Route not found");

    if let Some(children) = &route.children {
        for child in children {
            let route = ROUTES.get(child).expect("Route not found");
            keyboard.push(vec![InlineKeyboardButton::callback(
                route.label.to_owned(),
                route.path.to_owned(),
            )]);
        }
    }

    InlineKeyboardMarkup::new(keyboard)
}

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.
async fn message_handler(
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
            Ok(Command::Start) => {
                // Create a list of buttons and send them.
                let keyboard = make_keyboard("home");
                bot.send_message(msg.chat.id, "Виберите категорию чтобы узнать информацию")
                    .reply_markup(keyboard)
                    .await?;
            }
            Ok(Command::About) => {
                bot.send_message(msg.chat.id, "About Us").await?;
            }
            Ok(Command::Recycling) => {
                bot.send_message(msg.chat.id, "Recycling way way")
                    .reply_markup(make_keyboard("recycling"))
                    .await?;
            }
            Ok(Command::SolidPlastic) => {
                bot.send_message(msg.chat.id, "Solid Plastic").await?;
            }
            Ok(Command::SoftPlastic) => {
                bot.send_message(msg.chat.id, "Soft Plastic").await?;
            }
            Ok(Command::Paper) => {
                bot.send_message(msg.chat.id, "Paper").await?;
            }
            Ok(Command::Metal) => {
                bot.send_message(msg.chat.id, "Metal").await?;
            }
            Ok(Command::Glass) => {
                bot.send_message(msg.chat.id, "Glass").await?;
            }
            Ok(Command::Organic) => {
                bot.send_message(msg.chat.id, "Organic").await?;
            }
            Ok(Command::Other) => {
                bot.send_message(msg.chat.id, "Other").await?;
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

async fn inline_query_handler(
    bot: Bot,
    q: InlineQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let choose_debian_version = InlineQueryResultArticle::new(
        "0",
        "Chose debian version",
        InputMessageContent::Text(InputMessageContentText::new("Debian versions:")),
    )
    .reply_markup(make_keyboard("home"));

    bot.answer_inline_query(q.id, vec![choose_debian_version.into()])
        .await?;

    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(ref route) = q.data {
        // Tell telegram that we've seen this query, to remove 🕑 icons from the
        // clients. You could also use `answer_callback_query`'s optional
        // parameters to tweak what happens on the client side.
        bot.answer_callback_query(&q.id).await?;
        let route = route.to_string().replace("/", "");
        log::info!("You chose: {}", route);

        let route = ROUTES.get(&route).expect("Route not found");
        let choose_route = InlineQueryResultArticle::new(
            "1",
            route.label.to_owned(),
            InputMessageContent::Text(InputMessageContentText::new(route.label.to_owned())),
        )
        .reply_markup(make_keyboard(&route.path));

        bot.answer_inline_query(q.id, vec![choose_route.into()])
            .await?;
        // // Edit text of the message to which the buttons were attached
        // if let Some(message) = q.regular_message() {
        //     bot.edit_message_text(q.chat_id().unwrap(), message.id, text)
        //         .await?;
        // } else if let Some(id) = q.inline_message_id {
        //     bot.edit_message_text_inline(id, text).await?;
        // }

        log::info!("You chose: {}", route.label);
    }

    Ok(())
}
