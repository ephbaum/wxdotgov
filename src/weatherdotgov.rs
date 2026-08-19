//! National Weather Service API client.
//!
//! Forecasts are a two-step lookup. Coordinates resolve to a forecast office
//! and grid square, and that grid square has the forecast URLs:
//!
//! ```text
//! GET /points/{lat},{lon}          -> properties.forecast, properties.forecastHourly
//! GET /gridpoints/{office}/{x},{y}/forecast         -> daily periods
//! GET /gridpoints/{office}/{x},{y}/forecast/hourly  -> hourly periods
//! ```
//!
//! Responses are GeoJSON with the interesting values under `properties`. Only
//! the fields this tool prints are deserialized; everything else is ignored.
//!
//! ```json
//! {
//!   "properties": {
//!     "periods": [
//!       {
//!         "name": "Tonight",
//!         "startTime": "2024-01-28T18:00:00-08:00",
//!         "temperature": 51,
//!         "temperatureUnit": "F",
//!         "windSpeed": "5 mph",
//!         "windDirection": "SSW",
//!         "shortForecast": "Chance Light Rain",
//!         "detailedForecast": "A chance of rain. Mostly cloudy..."
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! Note the field names are camelCase. Every struct here therefore carries
//! `#[serde(rename_all = "camelCase")]`; omitting it on `HourlyPeriod` is what
//! made `--forecast-type hourly` fail against the live API (see #19).
//!
//! API docs: <https://www.weather.gov/documentation/services-web-api>

use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::http;

#[derive(Debug, Deserialize)]
pub struct PointsResponse {
    pub properties: PointsProperties,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointsProperties {
    pub forecast: String,
    pub forecast_hourly: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForecastResponse {
    pub properties: ForecastProperties,
}

#[derive(Debug, Deserialize)]
pub struct ForecastProperties {
    pub periods: Vec<Period>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub name: String,
    pub detailed_forecast: String,
}

#[derive(Debug, Deserialize)]
pub struct HourlyForecastResponse {
    pub properties: HourlyForecastProperties,
}

#[derive(Debug, Deserialize)]
pub struct HourlyForecastProperties {
    pub periods: Vec<HourlyPeriod>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyPeriod {
    pub start_time: String,
    pub temperature: i32,
    pub temperature_unit: String,
    pub wind_speed: String,
    pub wind_direction: String,
    pub short_forecast: String,
}

pub const DEFAULT_BASE_URL: &str = "https://api.weather.gov";

/// GET `url` and deserialize the GeoJSON body into `T`.
///
/// The three endpoints below are the same request shape differing only in the
/// type they decode and the noun they use in errors, so they share one
/// implementation. `what` names the request in messages ("points data",
/// "forecast", "hourly forecast").
async fn get_geojson<T: DeserializeOwned>(url: &str, what: &str) -> Result<T> {
    let client = http::client()?;

    let response = client
        .get(url)
        // GeoJSON is what the API serves by default, but asking for it
        // explicitly on every request keeps the format from being an
        // undeclared dependency on that default. This header was previously
        // sent on /points only.
        .header("Accept", "application/geo+json")
        .send()
        .await
        .with_context(|| format!("Error sending request to Weather.gov for {what}"))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .context("Error reading error response")?;
        bail!("Weather.gov returned an error for {what}: {error_text}");
    }

    response
        .json()
        .await
        .with_context(|| format!("Error parsing JSON from Weather.gov {what} response"))
}

pub async fn get_weather_point(
    latitude: &str,
    longitude: &str,
    base_url: Option<&str>,
) -> Result<PointsResponse> {
    let base_url = base_url.unwrap_or(DEFAULT_BASE_URL);
    get_geojson(
        &format!("{base_url}/points/{latitude},{longitude}"),
        "points data",
    )
    .await
}

pub async fn get_detailed_forecast(forecast_url: &str) -> Result<ForecastResponse> {
    get_geojson(forecast_url, "forecast").await
}

pub async fn get_hourly_forecast(forecast_url: &str) -> Result<HourlyForecastResponse> {
    get_geojson(forecast_url, "hourly forecast").await
}
