use axum::{routing::get, Router};
use handlers::{callback_handler, message_handler};

use shuttle_runtime::SecretStore;
use teloxide::prelude::*;

mod handlers;
mod route;

async fn health_check() -> &'static str {
    "All works fine. Please check @ecokenigbot in tg!"
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let telegram_bot_token = secret_store
        .get("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN should be set in secrets");
    let sentry_url = secret_store
        .get("SENTRY_URL")
        .expect("SENTRY_URL should be set in secrets");

    let router = build_router(&telegram_bot_token, sentry_url);

    Ok(router.into())
}

fn build_router(token: &str, sentry_url: String) -> Router {
    let bot = Bot::new(token);
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    tokio::spawn(async move {
        log::info!("Starting ecobot...");
        let _guard = sentry::init((
            sentry_url,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                max_breadcrumbs: 50,
                ..Default::default()
            },
        ));

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });

    Router::new().route("/", get(health_check))
}
