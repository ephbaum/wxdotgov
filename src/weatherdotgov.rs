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

pub async fn get_weather_point(
    latitude: &str,
    longitude: &str,
    base_url: Option<&str>,
) -> Result<PointsResponse> {
    let base_url = base_url.unwrap_or(DEFAULT_BASE_URL);
    let points_url = format!("{base_url}/points/{latitude},{longitude}");
    let client = http::client()?;

    let response = client
        .get(&points_url)
        .header("Accept", "application/geo+json")
        .send()
        .await
        .context("Error sending request to Weather.gov points endpoint")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .context("Error reading error response")?;
        bail!("Weather.gov returned an error for points data: {error_text}");
    }

    let points_resp: PointsResponse = response
        .json()
        .await
        .context("Error parsing JSON from Weather.gov points response")?;
    Ok(points_resp)
}

pub async fn get_detailed_forecast(forecast_url: &str) -> Result<ForecastResponse> {
    let client = http::client()?;
    let response = client
        .get(forecast_url)
        .send()
        .await
        .context("Error sending request to Weather.gov forecast endpoint")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .context("Error reading forecast error response")?;
        bail!("Weather.gov returned an error for forecast: {error_text}");
    }

    let forecast_resp: ForecastResponse = response
        .json()
        .await
        .context("Error parsing JSON from Weather.gov forecast response")?;
    Ok(forecast_resp)
}

pub async fn get_hourly_forecast(forecast_url: &str) -> Result<HourlyForecastResponse> {
    let client = http::client()?;
    let response = client
        .get(forecast_url)
        .send()
        .await
        .context("Error sending request to Weather.gov hourly forecast endpoint")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .context("Error reading hourly forecast error response")?;
        bail!("Weather.gov returned an error for hourly forecast: {error_text}");
    }

    let hourly_forecast_resp: HourlyForecastResponse = response
        .json()
        .await
        .context("Error parsing JSON from Weather.gov hourly forecast response")?;
    Ok(hourly_forecast_resp)
}
