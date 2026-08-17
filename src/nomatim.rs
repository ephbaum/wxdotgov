//! Nomatim API
//!
//! This module contains the Nomatim API
//!
//! It's meant to accept either a postal code, a city, or a city with a state code
//!
//! It will return a JSON object with the geocoded latitude and longitude for the requested location
//!
//! If the request is based on a postal code, it will call /search?postalcode={postal_code}&format=json
//! If the request is based on a city it will call /search?city={city}&format=json
//! If the request is based on a city and state it will call /search?city={city}&state={state}&format=json
//!
//! Nomatim returns an array of objects, each of which contains the geocoded latitude and longitude
//! For now we will only return the first OSM object in the array
//!
//! Nomatim API docs: https://nominatim.org/release-docs/develop/api/Search/

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::http;
use crate::LocationInput;

/// Trim an upstream error body so a full HTML page cannot flood the terminal.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max).collect();
    format!("{}...", head)
}

#[derive(Debug, Deserialize, Clone)]
pub struct NominatimLocation {
    pub lat: String,
    pub lon: String,
    pub display_name: String,
}

pub async fn get_lat_lon(
    input: LocationInput,
    base_url: Option<&str>,
) -> Result<NominatimLocation> {
    let default_base_url = "https://nominatim.openstreetmap.org";
    let base_url = base_url.unwrap_or(default_base_url);
    let client = http::client()?;

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
        .send()
        .await
        .context("Error sending request to Nominatim")?;

    // Check the status before parsing. Nominatim answers a blocked or
    // rate-limited request with an HTML error page, which previously surfaced
    // as a misleading "Error parsing JSON" and pointed at the wrong cause.
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            bail!(
                "Nominatim rate-limited this request (HTTP 429). \
                 Its usage policy allows at most 1 request per second."
            );
        }
        bail!(
            "Nominatim returned an error (HTTP {}): {}",
            status,
            truncate(body.trim(), 200)
        );
    }

    let body = response
        .text()
        .await
        .context("Error reading response body")?;

    let locations: Vec<NominatimLocation> =
        serde_json::from_str(&body).context("Error parsing JSON from Nominatim response")?;

    locations
        .into_iter()
        .next()
        .context("No location found. Make sure your query is correct.")
}
