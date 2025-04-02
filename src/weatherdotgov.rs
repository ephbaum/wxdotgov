/**
 * WeatherDotGov API
 * 
 * This module contains the WeatherDotGov API
 * 
 * It handles two behaviors:
 * 
 * It accepts a latitude and logitude in a float format and returns the weather office and grid points for that location
 * 
 * example request: https://api.weather.gov/points/47.5619,-122.625
 * 
 * example response:  
 * 
 * {
    "@context": [ ... ],
    ...
    "geometry": {
        "type": "Point",
        "coordinates": [
            -122.625,
            47.561900000000001
        ]
    },
    "properties": {
        "@id": "https://api.weather.gov/points/47.5619,-122.625",
        "@type": "wx:Point",
        "cwa": "SEW",
        "forecastOffice": "https://api.weather.gov/offices/SEW",
        "gridId": "SEW",
        "gridX": 115,
        "gridY": 68,
        "forecast": "https://api.weather.gov/gridpoints/SEW/115,68/forecast",
        "forecastHourly": "https://api.weather.gov/gridpoints/SEW/115,68/forecast/hourly",
        "forecastGridData": "https://api.weather.gov/gridpoints/SEW/115,68",
        "observationStations": "https://api.weather.gov/gridpoints/SEW/115,68/stations",
        "relativeLocation": {
            ...
        },
    }
}
 * 
 * It accepts the wxoffice and grid points and returns the weather report and output for that location
 * 
 * example request: https://api.weather.gov/gridpoints/SEW/115,68/forecast
 * 
 * example response:
 * 
 * {
    "@context": [ ... ],
    "type": "Feature",
    "geometry": {
        ...
    },
    "properties": {
        "updated": "2024-01-28T22:40:22+00:00",
        "units": "us",
        "forecastGenerator": "BaselineForecastGenerator",
        "generatedAt": "2024-01-29T02:32:45+00:00",
        "updateTime": "2024-01-28T22:40:22+00:00",
        "validTimes": "2024-01-28T16:00:00+00:00/P7DT12H",
        "elevation": {
            "unitCode": "wmoUnit:m",
            "value": 0
        },
        "periods": [
            {
                "number": 1,
                "name": "Tonight",
                "startTime": "2024-01-28T18:00:00-08:00",
                "endTime": "2024-01-29T06:00:00-08:00",
                "isDaytime": false,
                "temperature": 51,
                "temperatureUnit": "F",
                "temperatureTrend": "rising",
                "probabilityOfPrecipitation": {
                    "unitCode": "wmoUnit:percent",
                    "value": 40
                },
                "dewpoint": {
                    "unitCode": "wmoUnit:degC",
                    "value": 12.222222222222221
                },
                "relativeHumidity": {
                    "unitCode": "wmoUnit:percent",
                    "value": 91
                },
                "windSpeed": "5 mph",
                "windDirection": "SSW",
                "icon": "https://api.weather.gov/icons/land/night/rain,40/rain,20?size=medium",
                "shortForecast": "Chance Light Rain",
                "detailedForecast": "A chance of rain. Mostly cloudy. Low around 51, with temperatures rising to around 53 overnight. South southwest wind around 5 mph. Chance of precipitation is 40%. New rainfall amounts less than a tenth of an inch possible."
            },
            {
                "number": 2,
                ...
            },
            {
                "number": 14,
                "name": "Sunday",
                "startTime": "2024-02-04T06:00:00-08:00",
                "endTime": "2024-02-04T18:00:00-08:00",
                "isDaytime": true,
                "temperature": 44,
                "temperatureUnit": "F",
                "temperatureTrend": null,
                "probabilityOfPrecipitation": {
                    "unitCode": "wmoUnit:percent",
                    "value": null
                },
                "dewpoint": {
                    "unitCode": "wmoUnit:degC",
                    "value": 1.1111111111111112
                },
                "relativeHumidity": {
                    "unitCode": "wmoUnit:percent",
                    "value": 86
                },
                "windSpeed": "6 to 9 mph",
                "windDirection": "NNW",
                "icon": "https://api.weather.gov/icons/land/day/rain?size=medium",
                "shortForecast": "Slight Chance Light Rain",
                "detailedForecast": "A slight chance of rain before 5pm. Partly sunny, with a high near 44."
            }
        ]
    }
}
 * 
 */

use serde::Deserialize;
use anyhow::{bail, Context, Result};

#[derive(Debug, Deserialize)]
pub struct PointsResponse {
    pub properties: PointsProperties,
}

#[derive(Debug, Deserialize)]
pub struct PointsProperties {
    pub forecast: String,
    #[serde(rename = "forecastHourly")]
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
pub struct Period {
    pub name: String,
    #[serde(rename = "detailedForecast")]
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
pub struct HourlyPeriod {
    pub start_time: String,
    pub temperature: i32,
    pub temperature_unit: String,
    pub wind_speed: String,
    pub wind_direction: String,
    pub short_forecast: String,
}

pub async fn get_weather_point(latitude: &str, longitude: &str) -> Result<PointsResponse> {
    let points_url = format!("https://api.weather.gov/points/{},{}", latitude, longitude);
    let client = reqwest::Client::new();
    
    let response = client
        .get(&points_url)
        .header("Accept", "application/geo+json")
        .header("User-Agent", "RustWeatherCLI/0.1 (your_email@example.com)")
        .send()
        .await
        .context("Error sending request to Weather.gov points endpoint")?;

    if !response.status().is_success() {
        let error_text = response.text().await.context("Error reading error response")?;
        bail!("Weather.gov returned an error for points data: {}", error_text);
    }

    let points_resp: PointsResponse = response
        .json()
        .await
        .context("Error parsing JSON from Weather.gov points response")?;
    Ok(points_resp)
}

pub async fn get_detailed_forecast(forecast_url: &str) -> Result<ForecastResponse> {
    let client = reqwest::Client::new();
    let response = client
        .get(forecast_url)
        .header("User-Agent", "RustWeatherCLI/0.1 (your_email@example.com)")
        .send()
        .await
        .context("Error sending request to Weather.gov forecast endpoint")?;

    if !response.status().is_success() {
        let error_text = response.text().await.context("Error reading forecast error response")?;
        bail!("Weather.gov returned an error for forecast: {}", error_text);
    }

    let forecast_resp: ForecastResponse = response
        .json()
        .await
        .context("Error parsing JSON from Weather.gov forecast response")?;
    Ok(forecast_resp)
}

pub async fn get_hourly_forecast(forecast_url: &str) -> Result<HourlyForecastResponse> {
    let client = reqwest::Client::new();
    let response = client
        .get(forecast_url)
        .header("User-Agent", "RustWeatherCLI/0.1 (your_email@example.com)")
        .send()
        .await
        .context("Error sending request to Weather.gov hourly forecast endpoint")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .context("Error reading hourly forecast error response")?;
        bail!("Weather.gov returned an error for hourly forecast: {}", error_text);
    }

    let hourly_forecast_resp: HourlyForecastResponse = response
        .json()
        .await
        .context("Error parsing JSON from Weather.gov hourly forecast response")?;
    Ok(hourly_forecast_resp)
}
