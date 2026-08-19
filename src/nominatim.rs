//! Nominatim (OpenStreetMap) geocoding.
//!
//! Turns a `LocationInput` into a latitude/longitude pair so weather.gov can be
//! asked for a forecast.
//!
//! The query is sent as free text via `q`, not as structured `postalcode` /
//! `city` / `state` parameters:
//!
//! ```text
//! GET /search?q=98101,+USA&format=json&limit=1
//! GET /search?q=Seattle,+WA,+USA&format=json&limit=1
//! ```
//!
//! ", USA" is appended to keep results inside the United States, since
//! weather.gov only covers US locations. Nominatim answers with an array
//! ordered by relevance; only the first result is used.
//!
//! Two operational notes:
//!
//! - The usage policy requires an identifying User-Agent with real contact
//!   details and permits blocking clients without one. That header comes from
//!   [`crate::http`], shared with the weather.gov client.
//! - The policy also caps clients at one request per second. This tool makes a
//!   single geocoding request per invocation, so it does not rate-limit
//!   internally; a caller looping over it would need to.
//!
//! Errors carry the HTTP status. A blocked or rate-limited request is answered
//! with an HTML error page rather than JSON, so the status is checked before
//! parsing — otherwise the failure surfaces as a misleading parse error.
//!
//! API docs: <https://nominatim.org/release-docs/develop/api/Search/>

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
    format!("{head}...")
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
        LocationInput::PostalCode(code) => format!("{code}, USA"),
        LocationInput::PostalCodePlusFour(code, _) => format!("{code}, USA"),
        LocationInput::City(city) => format!("{city}, USA"),
        LocationInput::CityWithState(city, state) => format!("{city}, {state}, USA"),
    };

    let url = format!("{base_url}/search");

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
