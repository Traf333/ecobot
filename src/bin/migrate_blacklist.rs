use anyhow::Result;
use ecobot::db::{connect_db, DB};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    println!("Connecting to database...");
    connect_db().await?;
    
    println!("Updating all users to set blacklisted=false where not set...");
    
    let result = DB
        .query("UPDATE user SET blacklisted = false WHERE blacklisted IS NONE")
        .await?;
    
    println!("Migration complete!");
    println!("Result: {:?}", result);
    
    // Verify the update
    #[derive(serde::Deserialize, Debug)]
    struct CountRow {
        count: i64,
    }
    
    let count: Vec<CountRow> = DB
        .query("SELECT count() as count FROM user WHERE blacklisted = false GROUP ALL")
        .await?
        .take(0)?;
    
    if let Some(row) = count.first() {
        println!("Users with blacklisted=false: {}", row.count);
    }
    
    let blacklisted_count: Vec<CountRow> = DB
        .query("SELECT count() as count FROM user WHERE blacklisted = true GROUP ALL")
        .await?
        .take(0)?;
    
    if let Some(row) = blacklisted_count.first() {
        println!("Users with blacklisted=true: {}", row.count);
    }
    
    Ok(())
}
