#![allow(dead_code)]
#![allow(unused)]

use dotenv::dotenv;
use env_logger::{Builder, Env, Target};
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Initialize logger with default level "info" if RUST_LOG is not set
    // Logs go to stderr which Docker captures
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .target(Target::Stderr)
        .init();

    // Also write startup info to persistent log file (mounted volume in Docker)
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/var/log/ecobot/ecobot.log")
    {
        let _ = writeln!(file, "=== ECOBOT STARTING at {} ===", chrono::Local::now());
    }

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
