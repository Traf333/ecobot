use anyhow::Result;
use ecobot::db::{blacklist_user, connect_db, get_active_users};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    println!("Connecting to database...");
    connect_db().await?;
    
    // User IDs to blacklist
    let users_to_blacklist: Vec<i64> = vec![1971527512, 5948878887, 8489394229];
    
    // Step 1: Fetch all active users before blacklisting
    println!("\n=== BEFORE BLACKLISTING ===");
    let active_before = get_active_users().await?;
    println!("Active users count: {}", active_before.len());
    println!("Active user IDs: {:?}", active_before);
    
    // Check which of our target users are currently active
    for user_id in &users_to_blacklist {
        let is_active = active_before.contains(user_id);
        println!("User {} is currently active: {}", user_id, is_active);
    }
    
    // Step 2: Blacklist the users
    println!("\n=== BLACKLISTING USERS ===");
    for user_id in &users_to_blacklist {
        match blacklist_user(*user_id).await {
            Ok(true) => println!("✓ User {} successfully blacklisted", user_id),
            Ok(false) => println!("- User {} was already blacklisted", user_id),
            Err(e) => println!("✗ Failed to blacklist user {}: {}", user_id, e),
        }
    }
    
    // Step 3: Fetch all active users after blacklisting
    println!("\n=== AFTER BLACKLISTING ===");
    let active_after = get_active_users().await?;
    println!("Active users count: {}", active_after.len());
    println!("Active user IDs: {:?}", active_after);
    
    // Verify blacklisted users are no longer in active list
    println!("\n=== VERIFICATION ===");
    let mut all_excluded = true;
    for user_id in &users_to_blacklist {
        let is_active = active_after.contains(user_id);
        if is_active {
            println!("✗ User {} is STILL active (should be excluded)", user_id);
            all_excluded = false;
        } else {
            println!("✓ User {} is correctly excluded from active users", user_id);
        }
    }
    
    if all_excluded {
        println!("\n✓ SUCCESS: All blacklisted users are excluded from active users list");
    } else {
        println!("\n✗ FAILURE: Some blacklisted users are still in active users list");
    }
    
    let removed_count = active_before.len() - active_after.len();
    println!("Users removed from active list: {}", removed_count);
    
    Ok(())
}
