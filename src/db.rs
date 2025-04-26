use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

pub static DB: Lazy<Surreal<Client>> = Lazy::new(Surreal::init);

pub async fn connect_db() -> surrealdb::Result<()> {
    let url = env::var("URL").expect("URL must be set in environment");
    let port = env::var("PORT").expect("PORT must be set in environment");
    let username = env::var("USERNAME").expect("USERNAME must be set in environment");
    let password = env::var("PASSWORD").expect("PASSWORD must be set in environment");
    let namespace = env::var("NAMESPACE").expect("NAMESPACE must be set in environment");
    let dbname = env::var("DBNAME").expect("DBNAME must be set in environment");

    let _ = DB
        .connect::<Ws>(&format!("{url}:{port}"))
        .await?;

    let _ = DB
        .signin(Root {
            username: &username,
            password: &password,
        })
        .await?;

    let _ = DB
        .use_ns(&namespace)
        .use_db(&dbname)
        .await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Thing,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUser {
    user_id: i64,
    created_at: DateTime<Utc>,
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
    };

    let created: Option<User> = DB
        .create("user")
        .content(user)
        .await
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;

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
