use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use once_cell::sync::Lazy;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub static ROUTES: Lazy<HashMap<String, Route>> = Lazy::new(|| routes().unwrap());

#[derive(Serialize, Deserialize)]
pub struct Route {
    pub path: String,
    pub label: String,
    pub children: Option<Vec<String>>,
}

pub fn routes() -> Result<HashMap<String, Route>, serde_json::Error> {
    let routes: HashMap<String, Route> = serde_json::from_str(include_str!("./routes.json"))?;
    Ok(routes)
}

pub fn build_buttons(category: &str) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];

    let route = ROUTES.get(category).expect("Route not found");

    if let Some(children) = &route.children {
        let mut chunked: Vec<Vec<InlineKeyboardButton>> = children
            .chunks(3)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|child| {
                        log::info!("child: {}", child);
                        let route = ROUTES.get(child).expect("Route not found");
                        InlineKeyboardButton::callback(
                            route.label.to_owned(),
                            route.path.to_owned(),
                        )
                    })
                    .collect()
            })
            .collect();
        buttons.append(&mut chunked);
    }
    InlineKeyboardMarkup::new(buttons)
}
