pub use bin_location::*;
use once_cell::sync::Lazy;
use std::env;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
pub use user::*;

mod bin_location;
mod user;

pub static DB: Lazy<Surreal<Client>> = Lazy::new(Surreal::init);

pub async fn connect_db() -> surrealdb::Result<()> {
    let url = env::var("URL").expect("URL must be set in environment");
    let port = env::var("PORT").expect("PORT must be set in environment");
    let username = env::var("USERNAME").expect("USERNAME must be set in environment");
    let password = env::var("PASSWORD").expect("PASSWORD must be set in environment");
    let namespace = env::var("NAMESPACE").expect("NAMESPACE must be set in environment");
    let dbname = env::var("DBNAME").expect("DBNAME must be set in environment");

    let _ = DB.connect::<Ws>(&format!("{url}:{port}")).await?;

    let _ = DB
        .signin(Root {
            username: &username,
            password: &password,
        })
        .await?;

    let _ = DB.use_ns(&namespace).use_db(&dbname).await?;

    Ok(())
}
