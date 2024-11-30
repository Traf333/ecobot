use axum::{routing::get, Router};

mod handlers;
mod route;

use handlers::{callback_handler, message_handler};

use shuttle_runtime::SecretStore;
use teloxide::prelude::*;

async fn health_check() -> &'static str {
    "All works fine. Please check @ecokenigbot in tg!"
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let telegram_bot_token = secret_store
        .get("TELOXIDE_TOKEN")
        .expect("token should be set in secrets");

    let router = build_router(&telegram_bot_token);

    Ok(router.into())
}

fn build_router(token: &str) -> Router {
    let bot = Bot::new(token);
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    tokio::spawn(async move {
        log::info!("Starting ecobot...");

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });

    Router::new().route("/", get(health_check))
}
