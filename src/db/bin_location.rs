use crate::db::DB;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use haversine_rs::{distance, point::Point, units::Unit};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct BinLocation {
    pub id: Thing,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateBinLocation {
    latitude: f64,
    longitude: f64,
    address: String,
}

pub async fn create_bin_location(latitude: f64, longitude: f64, address: String) -> Result<bool> {
    let bin_location = CreateBinLocation {
        latitude,
        longitude,
        address,
    };
    let new_location: Option<BinLocation> = DB.create("bin_location").content(bin_location).await?;
    Ok(new_location.is_some())
}

pub async fn get_bin_locations(latitude: f64, longitude: f64) -> Result<Vec<BinLocation>> {
    let bin_locations: Vec<BinLocation> = DB.select("bin_location").await?;
    let radius = 10.0;
    let point_a = Point::new(latitude, longitude);
    let mut filtered_bin_locations = vec![BinLocation {
        id: Thing::from(("bin_location", "1")),
        latitude,
        longitude,
        address: String::from("You are here"),
        created_at: Utc::now(),
    }];
    for bin_location in bin_locations {
        let point_b = Point::new(bin_location.latitude, bin_location.longitude);
        let distance = distance(point_a, point_b, Unit::Kilometers);
        if distance <= radius {
            filtered_bin_locations.push(bin_location);
        }
    }
    Ok(filtered_bin_locations)
}
