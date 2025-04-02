/**
 * Nomatim API
 * 
 * This module contains the Nomatim API
 * 
 * It's meant to accept either a postal code, a city, or a city with a state code
 * 
 * It will return a JSON object with the geocoded latitude and longitude for the requested location
 * 
 * If the request is based on a postal code, it will call /search?postalcode={postal_code}&format=json
 * If the request is based on a city it will call /search?city={city}&format=json
 * If the request is based on a city and state it will call /search?city={city}&state={state}&format=json
 * 
 * Nomatim returns an array of objects, each of which contains the geocoded latitude and longitude
 * For now we will only return the first OSM object in the array
 * 
 * Nomatim API docs: https://nominatim.org/release-docs/develop/api/Search/
 */

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::LocationInput;

#[derive(Debug, Deserialize, Clone)]
pub struct NominatimLocation {
    pub lat: String,
    pub lon: String,
    pub display_name: String,
}

pub async fn get_lat_lon(input: LocationInput, base_url: Option<&str>) -> Result<NominatimLocation> {
    let default_base_url = "https://nominatim.openstreetmap.org";
    let base_url = base_url.unwrap_or(default_base_url);
    let client = reqwest::Client::new();

    let query = match input {
        LocationInput::PostalCode(code) => format!("{}, USA", code),
        LocationInput::PostalCodePlusFour(code, _) => format!("{}, USA", code),
        LocationInput::City(city) => format!("{}, USA", city),
        LocationInput::CityWithState(city, state) => format!("{}, {}, USA", city, state),
    };

    let url = format!("{}/search", base_url);

    let response = client
        .get(&url)
        .query(&[
            ("q", &query),
            ("format", &"json".to_string()),
            ("limit", &"1".to_string()),
        ])
        .header("User-Agent", "RustWeatherCLI/0.1 (your_email@example.com)")
        .send()
        .await
        .context("Error sending request to Nominatim")?;

    let body = response.text().await.context("Error reading response body")?;

    let locations: Vec<NominatimLocation> = serde_json::from_str(&body)
        .context("Error parsing JSON from Nominatim response")?;

    locations
        .into_iter()
        .next()
        .context("No location found. Make sure your query is correct.")
}