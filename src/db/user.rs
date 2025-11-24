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
    pub subscriptions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUser {
    user_id: i64,
    created_at: DateTime<Utc>,
    subscriptions: Vec<String>,
}

/// Store a user ID in the database
pub async fn store_user(user_id: i64) -> Result<bool> {
    // Check if user already exists
    if is_user_stored(user_id).await? {
        return Ok(false);
    }

    // Create a new user
    let user = CreateUser {
        user_id,
        created_at: Utc::now(),
        subscriptions: vec![],
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

    let _: Option<User> = DB
        .update(("user", user.id.id.to_string()))
        .merge(serde_json::json!({ "subscriptions": subscriptions }))
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;

    log::info!("User {} subscribed to {}", user_id, subscription);
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

    let _: Option<User> = DB
        .update(("user", user.id.id.to_string()))
        .merge(serde_json::json!({ "subscriptions": subscriptions }))
        .await
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;

    log::info!("User {} unsubscribed from {}", user_id, subscription);
    Ok(true)
}
