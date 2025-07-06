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
    let mut filtered_bin_locations = Vec::new();
    for bin_location in bin_locations {
        let point_b = Point::new(bin_location.latitude, bin_location.longitude);
        let distance = distance(point_a, point_b, Unit::Kilometers);
        if distance <= radius {
            filtered_bin_locations.push(bin_location);
        }
    }
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

    // Prepare a vector to collect all bin locations that meet our criteria
    let mut bin_locations = Vec::new();

    // Process each feature
    for feature in features {
        total_count += 1;

        // Extract properties
        let properties = feature.properties;

        // Extract icon caption (address)
        let address = properties.iconCaption;

        // Extract coordinates
        let coordinates = feature.geometry.coordinates;

        let longitude = coordinates[0];
        let latitude = coordinates[1];

        // Extract preset (icon color)
        let options = feature.options;

        let preset = options.preset;

        // Filter out points with "Советск" in the address or yellow icon preset
        if !address.contains("Советск") && !preset.contains("yellow") {
            // Create bin location
            let bin_location = CreateBinLocation {
                latitude,
                longitude,
                address,
            };

            // Add to our collection
            bin_locations.push(bin_location);
            success_count += 1;
        }
    }

    // Create all bin locations at once with a single query
    if !bin_locations.is_empty() {
        let locations_count = bin_locations.len();
        let sql = "CREATE bin_location CONTENT $data;";
        match DB.query(sql).bind(("data", bin_locations)).await {
            Ok(_) => log::info!("Successfully created {} bin locations", locations_count),
            Err(e) => log::error!("Failed to create bin locations: {}", e),
        }
    }

    println!("Stored {success_count} out of {total_count} points");
    Ok(success_count > 0)
}
