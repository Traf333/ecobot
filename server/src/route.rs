use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
