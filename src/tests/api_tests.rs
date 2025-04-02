#[cfg(test)]
mod tests {
    use crate::weatherdotgov::{get_weather_point, get_detailed_forecast, get_hourly_forecast};
    use crate::nomatim::get_lat_lon;
    use crate::LocationInput;
    use mockito::Server;

    #[tokio::test]
    async fn test_get_weather_point() {
        let mut server = Server::new();
        let mock_response = r#"{
            "properties": {
                "forecast": "https://api.weather.gov/gridpoints/SEW/115,68/forecast",
                "forecastHourly": "https://api.weather.gov/gridpoints/SEW/115,68/forecast/hourly"
            }
        }"#;

        server.mock("GET", "/points/47.5619,-122.625")
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(mock_response)
            .create();

        let result = get_weather_point("47.5619", "-122.625").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.properties.forecast.contains("/forecast"));
        assert!(response.properties.forecast_hourly.unwrap().contains("/forecast/hourly"));
    }

    #[tokio::test]
    async fn test_get_detailed_forecast() {
        let mut server = Server::new();
        let mock_response = r#"{
            "properties": {
                "periods": [
                    {
                        "name": "Tonight",
                        "detailedForecast": "Partly cloudy with a chance of rain"
                    }
                ]
            }
        }"#;

        server.mock("GET", "/gridpoints/SEW/115,68/forecast")
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(mock_response)
            .create();

        let result = get_detailed_forecast(&format!("{}/gridpoints/SEW/115,68/forecast", server.url())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.properties.periods.len(), 1);
        assert_eq!(response.properties.periods[0].name, "Tonight");
    }

    #[tokio::test]
    async fn test_get_hourly_forecast() {
        let mut server = Server::new();
        let mock_response = r#"{
            "properties": {
                "periods": [
                    {
                        "start_time": "2024-01-29T02:32:45+00:00",
                        "temperature": 51,
                        "temperature_unit": "F",
                        "wind_speed": "5 mph",
                        "wind_direction": "SSW",
                        "short_forecast": "Partly Cloudy"
                    }
                ]
            }
        }"#;

        server.mock("GET", "/gridpoints/SEW/115,68/forecast/hourly")
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(mock_response)
            .create();

        let result = get_hourly_forecast(&format!("{}/gridpoints/SEW/115,68/forecast/hourly", server.url())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.properties.periods.len(), 1);
        assert_eq!(response.properties.periods[0].temperature, 51);
    }

    #[tokio::test]
    async fn test_get_lat_lon() {
        let mut server = Server::new();
        let mock_response = r#"[
            {
                "lat": "47.5619",
                "lon": "-122.625",
                "display_name": "Seattle, King County, Washington, USA"
            }
        ]"#;

        server.mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let input = LocationInput::CityWithState("Seattle".to_string(), "WA".to_string());
        let result = get_lat_lon(input, Some(&server.url())).await;
        
        assert!(result.is_ok());
        let location = result.unwrap();
        assert_eq!(location.lat, "47.5619");
        assert_eq!(location.lon, "-122.625");
    }

    #[tokio::test]
    async fn test_get_weather_point_error() {
        let mut server = Server::new();
        server.mock("GET", "/points/invalid,invalid")
            .with_status(400)
            .with_header("content-type", "application/geo+json")
            .with_body(r#"{"error": "Invalid coordinates"}"#)
            .create();

        let result = get_weather_point("invalid", "invalid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_lat_lon_no_results() {
        let mut server = Server::new();
        let mock_response = r#"[]"#;

        server.mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let input = LocationInput::City("NonexistentCity".to_string());
        let result = get_lat_lon(input, Some(&server.url())).await;
        
        assert!(result.is_err());
    }
} 