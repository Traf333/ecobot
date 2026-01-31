use crate::db::DB;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Thing,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub subscriptions: Vec<String>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub blacklisted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUser {
    user_id: i64,
    created_at: DateTime<Utc>,
    subscriptions: Vec<String>,
    updated_at: DateTime<Utc>,
    blacklisted: bool,
}

/// Store a user ID in the database
pub async fn store_user(user_id: i64) -> Result<bool> {
    // Check if user already exists
    if is_user_stored(user_id).await? {
        return Ok(false);
    }

    // Create a new user
    let now = Utc::now();
    let user = CreateUser {
        user_id,
        created_at: now,
        subscriptions: vec![],
        updated_at: now,
        blacklisted: false,
    };

    let created: Option<User> = DB
        .create("user")
        .content(user)
        .await
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;
    log::info!("User {} created", user_id);
    Ok(created.is_some())
}

/// Check if a user ID is already stored
pub async fn is_user_stored(user_id: i64) -> Result<bool> {
    // get all users
    let users = get_all_users().await?;
    Ok(users.contains(&user_id))
}

/// Get all stored user IDs
pub async fn get_all_users() -> Result<Vec<i64>> {
    let users: Vec<User> = DB
        .select("user")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?;

    Ok(users.into_iter().map(|user| user.user_id).collect())
}

/// Get user subscriptions
pub async fn get_user_subscriptions(user_id: i64) -> Result<Vec<String>> {
    let users: Vec<User> = DB
        .select("user")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?;

    let user = users
        .into_iter()
        .find(|u| u.user_id == user_id)
        .ok_or_else(|| anyhow!("User not found"))?;

    Ok(user.subscriptions)
}

/// Check if user is subscribed to a specific subscription
pub async fn is_subscribed(user_id: i64, subscription: &str) -> Result<bool> {
    let subscriptions = get_user_subscriptions(user_id).await?;
    Ok(subscriptions.contains(&subscription.to_string()))
}

/// Subscribe user to a subscription type
pub async fn subscribe_user(user_id: i64, subscription: &str) -> Result<bool> {
    // Check if already subscribed
    if is_subscribed(user_id, subscription).await? {
        return Ok(false);
    }

    // Get current subscriptions
    let mut subscriptions = get_user_subscriptions(user_id).await?;
    subscriptions.push(subscription.to_string());

    // Update user
    let users: Vec<User> = DB
        .select("user")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?;

    let user = users
        .into_iter()
        .find(|u| u.user_id == user_id)
        .ok_or_else(|| anyhow!("User not found"))?;

    let record_id = format!("{}:{}", user.id.tb, user.id.id);
    let id_string = user.id.id.to_string();
    log::info!(
        "Subscribing telegram user {} (DB record: {}) to {} with subscriptions: {:?}",
        user_id,
        record_id,
        subscription,
        subscriptions
    );

    let updated: Option<User> = DB
        .update(("user", id_string))
        .merge(serde_json::json!({
            "subscriptions": subscriptions,
            "updated_at": Utc::now()
        }))
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;

    log::info!(
        "Telegram user {} (DB record: {}) subscribed to {} - Update success: {}",
        user_id,
        record_id,
        subscription,
        updated.is_some()
    );
    Ok(true)
}

/// Unsubscribe user from a subscription type
pub async fn unsubscribe_user(user_id: i64, subscription: &str) -> Result<bool> {
    // Check if subscribed
    if !is_subscribed(user_id, subscription).await? {
        return Ok(false);
    }

    // Get current subscriptions and remove the subscription
    let mut subscriptions = get_user_subscriptions(user_id).await?;
    subscriptions.retain(|s| s != subscription);

    // Update user
    let users: Vec<User> = DB
        .select("user")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?;

    let user = users
        .into_iter()
        .find(|u| u.user_id == user_id)
        .ok_or_else(|| anyhow!("User not found"))?;

    let record_id = format!("{}:{}", user.id.tb, user.id.id);
    let id_string = user.id.id.to_string();
    log::info!(
        "Unsubscribing telegram user {} (DB record: {}) from {} with subscriptions: {:?}",
        user_id,
        record_id,
        subscription,
        subscriptions
    );

    let updated: Option<User> = DB
        .update(("user", id_string))
        .merge(serde_json::json!({
            "subscriptions": subscriptions,
            "updated_at": Utc::now()
        }))
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;

    log::info!(
        "Telegram user {} (DB record: {}) unsubscribed from {} - Update success: {}",
        user_id,
        record_id,
        subscription,
        updated.is_some()
    );
    Ok(true)
}

/// Unsubscribe user from all subscriptions
pub async fn unsubscribe_all(user_id: i64) -> Result<bool> {
    // Get current subscriptions
    let subscriptions = get_user_subscriptions(user_id).await?;

    // If no subscriptions, return false
    if subscriptions.is_empty() {
        return Ok(false);
    }

    // Update user to have empty subscriptions
    let users: Vec<User> = DB
        .select("user")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?;

    let user = users
        .into_iter()
        .find(|u| u.user_id == user_id)
        .ok_or_else(|| anyhow!("User not found"))?;

    let record_id = format!("{}:{}", user.id.tb, user.id.id);
    let id_string = user.id.id.to_string();
    log::info!(
        "Unsubscribing telegram user {} (DB record: {}) from all subscriptions",
        user_id,
        record_id
    );

    let updated: Option<User> = DB
        .update(("user", id_string))
        .merge(serde_json::json!({
            "subscriptions": Vec::<String>::new(),
            "updated_at": Utc::now()
        }))
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;

    log::info!(
        "Telegram user {} (DB record: {}) unsubscribed from all - Update success: {}",
        user_id,
        record_id,
        updated.is_some()
    );
    Ok(true)
}

/// Get all users subscribed to a specific subscription type
pub async fn get_users_by_subscription(subscription: &str) -> Result<Vec<i64>> {
    let users: Vec<User> = DB
        .select("user")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?;

    let subscribed_users: Vec<i64> = users
        .into_iter()
        .filter(|user| user.subscriptions.contains(&subscription.to_string()))
        .map(|user| user.user_id)
        .collect();

    Ok(subscribed_users)
}

/// Blacklist a user (mark as unable to receive messages)
pub async fn blacklist_user(user_id: i64) -> Result<bool> {
    let users: Vec<User> = DB
        .select("user")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?;

    let user = users
        .into_iter()
        .find(|u| u.user_id == user_id)
        .ok_or_else(|| anyhow!("User not found"))?;

    if user.blacklisted {
        return Ok(false);
    }

    let id_string = user.id.id.to_string();
    log::info!("Blacklisting user {}", user_id);

    let updated: Option<User> = DB
        .update(("user", id_string))
        .merge(serde_json::json!({
            "blacklisted": true,
            "updated_at": Utc::now()
        }))
        .await
        .map_err(|e| anyhow!("Failed to blacklist user: {}", e))?;

    log::info!(
        "User {} blacklisted - Update success: {}",
        user_id,
        updated.is_some()
    );
    Ok(true)
}

#[derive(serde::Deserialize)]
struct UserIdRow {
    user_id: i64,
}

pub async fn get_active_users() -> Result<Vec<i64>> {
    let rows: Vec<UserIdRow> = DB
        .query("SELECT user_id FROM user WHERE blacklisted = false")
        .await
        .map_err(|e| anyhow!("Failed to query users: {}", e))?
        .take(0)?;

    Ok(rows.into_iter().map(|row| row.user_id).collect())
}
