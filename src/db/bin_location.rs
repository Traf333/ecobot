use crate::db::DB;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use haversine_rs::{distance, point::Point, units::Unit};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct BinLocation {
    pub id: Thing,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub preset: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateBinLocation {
    latitude: f64,
    longitude: f64,
    address: String,
}

pub async fn get_bin_locations(latitude: f64, longitude: f64) -> Result<Vec<(f64, BinLocation)>> {
    let sql = r#"
    SELECT * FROM bin_location
    WHERE $word NOT IN address
      AND preset != $preset;
    "#;

    let mut response = DB
        .query(sql)
        .bind(("word", "Советск"))
        .bind(("preset", "islands#darkOrangeIcon"))
        .await?;
    let bins: Vec<BinLocation> = response.take(0)?;
    let radius = 1.0;
    let point_a = Point::new(latitude, longitude);
    let mut filtered_bin_locations = Vec::new();
    for bin_location in bins {
        let point_b = Point::new(bin_location.latitude, bin_location.longitude);
        let distance = distance(point_a, point_b, Unit::Kilometers);

        if distance <= radius {
            filtered_bin_locations.push((distance, bin_location));
        }
    }
    filtered_bin_locations
        .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    Ok(filtered_bin_locations)
}

#[derive(Debug, Serialize, Deserialize)]
struct ESSOResponse {
    features: Vec<ESSOFeature>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ESSOFeature {
    id: String,
    properties: ESSOProperties,
    geometry: ESSOGeometry,
    options: ESSOOptions,
}

#[derive(Debug, Serialize, Deserialize)]
struct ESSOProperties {
    name: String,
    description: String,
    iconContent: String,
    iconCaption: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ESSOGeometry {
    coordinates: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ESSOOptions {
    zIndex: i64,
    order: i64,
    preset: String,
}

pub async fn store_esso_points() -> Result<bool> {
    println!("Synchronising ESSO points");
    // Fetch data from URL
    let url = "https://new.esoo39.ru/wp-content/themes/appointment/js/data.js?v=0.72";
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.json::<ESSOResponse>().await?;

    // Extract features array
    let features = response.features;

    let mut success_count = 0;
    let mut total_count = 0;

    // remove all bin locations
    DB.query("DELETE FROM bin_location").await?;
    let result = features
        .into_iter()
        .map(|feature| {
            return serde_json::json!({
                "latitude": feature.geometry.coordinates[1],
                "longitude": feature.geometry.coordinates[0],
                "address": feature.properties.iconCaption,
                "preset": feature.options.preset
            });
        })
        .collect::<Vec<serde_json::Value>>();

    let sql = "INSERT INTO bin_location $data;";
    let mut response = DB.query(sql).bind(("data", result)).await?;

    let inserted: Vec<BinLocation> = response.take(0)?;
    println!("Inserted {} records", inserted.len());
    Ok(true)
}

pub async fn store_rspko_points() -> Result<bool> {
    println!("Synchronising RSPKO points");

    let file = File::open("rspko.json").expect("Failed to open rspko.json");
    let reader = BufReader::new(file);
    let features: Vec<CreateBinLocation> = serde_json::from_reader(reader).unwrap();

    let mut success_count = 0;
    let mut total_count = 0;

    let result = features
        .into_iter()
        .map(|feature| {
            return serde_json::json!({
                "latitude": feature.latitude,
                "longitude": feature.longitude,
                "address": feature.address,
                "preset": "setka"
            });
        })
        .collect::<Vec<serde_json::Value>>();

    let sql = "INSERT INTO bin_location $data;";
    let mut response = DB.query(sql).bind(("data", result)).await?;

    let inserted: Vec<BinLocation> = response.take(0)?;
    println!("Inserted {} records", inserted.len());
    Ok(true)
}
