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
