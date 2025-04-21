use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

pub static DB: Lazy<Surreal<Client>> = Lazy::new(Surreal::init);

pub async fn connect_db(secrets: SecretStore) -> surrealdb::Result<()> {
    let _ = DB
        .connect::<Ws>(&format!(
            "{}:{}",
            &secrets.get("URL").expect("database url should be set"),
            &secrets.get("PORT").expect("database port should be set")
        ))
        .await?;

    let _ = DB
        .signin(Root {
            username: &secrets
                .get("USERNAME")
                .expect("database username should be set"),
            password: &secrets
                .get("PASSWORD")
                .expect("database password should be set"),
        })
        .await?;

    let _ = DB
        .use_ns(
            &secrets
                .get("NAMESPACE")
                .expect("database namespace should be set"),
        )
        .use_db(&secrets.get("DBNAME").expect("database name should be set"))
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
