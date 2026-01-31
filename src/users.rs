use crate::db;
use anyhow::Result;

/// Store a user ID in the database
pub async fn store_user(user_id: i64) -> Result<bool> {
    db::store_user(user_id).await
}

/// Get all stored user IDs
pub async fn get_all_users() -> Result<Vec<i64>> {
    db::get_all_users().await
}

/// Get all non-blacklisted user IDs
pub async fn get_active_users() -> Result<Vec<i64>> {
    db::get_active_users().await
}

/// Blacklist a user (mark as unable to receive messages)
pub async fn blacklist_user(user_id: i64) -> Result<bool> {
    db::blacklist_user(user_id).await
}
