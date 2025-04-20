use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatId, ParseMode},
    Bot,
};

// Global static set to store user IDs
pub static USERS: Lazy<Mutex<HashSet<i64>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Store a user ID in the database
pub fn store_user(user_id: i64) -> bool {
    let mut users = USERS.lock().unwrap();
    users.insert(user_id)
}

/// Check if a user ID is already stored
pub fn is_user_stored(user_id: i64) -> bool {
    let users = USERS.lock().unwrap();
    users.contains(&user_id)
}

/// Get all stored user IDs
pub fn get_all_users() -> Vec<i64> {
    let users = USERS.lock().unwrap();
    users.iter().copied().collect()
}

/// Send a message to a specific user
pub async fn send_message_to_user(
    bot: &Bot,
    user_id: i64,
    message: &str,
    markdown: bool,
) -> Result<()> {
    let chat_id = ChatId(user_id);

    let mut msg = bot.send_message(chat_id, message);

    if markdown {
        msg = msg.parse_mode(ParseMode::MarkdownV2);
    }

    msg.await?;
    Ok(())
}

/// Broadcast a message to all stored users
pub async fn broadcast_message(
    bot: &Bot,
    message: &str,
    markdown: bool,
) -> Result<(usize, Vec<i64>)> {
    let users = get_all_users();
    let mut success_count = 0;
    let mut failed_users = Vec::new();

    for user_id in users {
        match send_message_to_user(bot, user_id, message, markdown).await {
            Ok(_) => success_count += 1,
            Err(_) => failed_users.push(user_id),
        }
    }

    Ok((success_count, failed_users))
}
