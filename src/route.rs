use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use once_cell::sync::Lazy;
use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub static ROUTES: Lazy<HashMap<String, Route>> = Lazy::new(|| routes().unwrap());

#[derive(Serialize, Deserialize)]
pub struct Route {
    pub path: String,
    pub label: String,
    pub children: Option<Vec<String>>,
    pub external: Option<Vec<String>>,
}

pub fn routes() -> Result<HashMap<String, Route>, serde_json::Error> {
    let routes: HashMap<String, Route> = serde_json::from_str(include_str!("./routes.json"))?;
    Ok(routes)
}

pub fn build_buttons(category: &str, is_external: bool) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];

    let route = ROUTES.get(category).expect("Route not found");

    if let Some(children) = &route.children {
        let mut chunked: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        let mut current_row: Vec<InlineKeyboardButton> = Vec::new();

        for child in children {
            let child_route = ROUTES.get(child).expect("Route not found");
            let button = InlineKeyboardButton::callback(&child_route.label, &child_route.path);

            if child_route.label.chars().count() > 20 {
                // Flush any accumulated small buttons before adding a large button row
                if !current_row.is_empty() {
                    chunked.push(std::mem::take(&mut current_row));
                }
                chunked.push(vec![button]);
            } else {
                current_row.push(button);
                if current_row.len() == 2 {
                    chunked.push(std::mem::take(&mut current_row));
                }
            }
        }

        // Push remaining small buttons (if the count was odd)
        if !current_row.is_empty() {
            chunked.push(current_row);
        }

        buttons.append(&mut chunked);
    } else {
        if is_external {
            if let Some(external) = &route.external {
                let url = Url::parse(&external[1]).unwrap();
                buttons.push(vec![InlineKeyboardButton::url(&external[0], url)]);
            }
        } else {
            buttons.push(vec![InlineKeyboardButton::callback("На Главную", "start")]);
        }
    }
    InlineKeyboardMarkup::new(buttons)
}
