#![allow(dead_code)]
#![allow(unused)]

use dotenv::dotenv;
use env_logger::{Builder, Target};
use handlers::{callback_handler, message_handler};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use teloxide::prelude::*;

mod commands;
mod db;
mod handlers;
mod route;
mod users;

fn init_logging() {
    let log_path = std::env::var("LOG_PATH").unwrap_or_else(|_| "ecobot.log".to_string());

    if let Some(parent) = std::path::Path::new(&log_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("cannot create log directory");
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("cannot open log file");

    Builder::new()
        .parse_default_env()
        .target(Target::Pipe(Box::new(file)))
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    init_logging();

    log::info!("Starting bot at time: {}", chrono::Local::now());

    let telegram_bot_token =
        env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN should be set in environment");

    db::connect_db().await.expect("Database connection fails");
    // db::store_esso_points()
    //     .await
    //     .expect("Failed to store ESSO points");

    log::info!("Database connected successfully");

    let bot = Bot::new(&telegram_bot_token);
    log::info!("Bot initialized, starting dispatcher...");

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
