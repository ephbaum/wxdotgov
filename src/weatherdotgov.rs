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

#[derive(Clone, Debug, Deserialize)]
pub struct WeatherPoint {
    pub properties: Properties,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Properties {
    pub forecast: String,
    // grid_id: String,
    // grid_x: i32,
    // grid_y: i32,
    // forecast_hourly: String,
}

use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum MyError {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Reqwest(e) => write!(f, "Reqwest error: {}", e),
            MyError::Serde(e) => write!(f, "Serde error: {}", e),
        }
    }
}

impl StdError for MyError {}

pub async fn get_weather_point(latitude: &String, longitude: &String) -> Result<WeatherPoint, MyError> {
    println!("Getting weather point for {}, {}", latitude, longitude);
    let url = format!("https://api.weather.gov/points/{},{}", latitude, longitude);
    println!("URL: {}", url);

    let client = reqwest::Client::new();
    let response: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "reqwest")
        .send()
        .await
        .map_err(MyError::Reqwest)?
        .json()
        .await
        .map_err(MyError::Reqwest)?;
    println!("Response: {:?}", response);

    let weather_point: WeatherPoint = serde_json::from_value(response).map_err(MyError::Serde)?;
    Ok(weather_point)
}

pub async fn get_weather_forecast(weather_point: WeatherPoint) -> Result<String, MyError> {
    let response = reqwest::get(&weather_point.properties.forecast)
        .await
        .map_err(MyError::Reqwest)?;
    let forecast: String = response.text().await.map_err(MyError::Reqwest)?;
    Ok(forecast)
}
