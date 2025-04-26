use dotenv::dotenv;
use handlers::{callback_handler, message_handler};
use std::env;
use teloxide::prelude::*;

mod db;
mod handlers;
mod route;
mod users;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    env_logger::init();
    log::info!("Starting bot");
    // Get environment variables
    let telegram_bot_token =
        env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN should be set in environment");

    // Connect to database
    db::connect_db().await.expect("Database connection fails");

    let bot = Bot::new(&telegram_bot_token);
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
