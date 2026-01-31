use rust_embed::RustEmbed;
use teloxide::types::InlineKeyboardMarkup;

use crate::route::build_buttons_with_user;

#[derive(RustEmbed)]
#[folder = "src/contents/"]
pub struct Contents;

pub const ADMIN_ID: i64 = 283564928;
pub const TEST_USER_ID: i64 = 108609383;

pub fn build_details(
    text: &str,
    is_external: bool,
) -> Result<(InlineKeyboardMarkup, String), Box<dyn std::error::Error + Send + Sync>> {
    build_details_with_user(text, is_external, None)
}

pub fn build_details_with_user(
    text: &str,
    is_external: bool,
    user_id: Option<i64>,
) -> Result<(InlineKeyboardMarkup, String), Box<dyn std::error::Error + Send + Sync>> {
    let route = text.trim_start_matches('/').replace("/", "-");
    let file_name = format!("{}.md", &route);
    let content = Contents::get(&file_name)
        .ok_or_else(|| format!("File {} not found", file_name))?
        .data;

    let content = String::from_utf8(content.to_vec())?;
    let buttons = build_buttons_with_user(&route, is_external, user_id);

    Ok((buttons, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_details_start() {
        let result = build_details("start", false);
        assert!(result.is_ok(), "start.md should be loadable");
    }
}
