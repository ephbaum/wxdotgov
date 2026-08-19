#[cfg(test)]
mod tests {
    use crate::nominatim::get_lat_lon;
    use crate::weatherdotgov::{get_detailed_forecast, get_hourly_forecast, get_weather_point};
    use crate::LocationInput;
    use mockito::Server;

    #[tokio::test]
    async fn test_get_weather_point() {
        let mut server = Server::new_async().await;
        let mock_response = r#"{
            "properties": {
                "forecast": "https://api.weather.gov/gridpoints/SEW/115,68/forecast",
                "forecastHourly": "https://api.weather.gov/gridpoints/SEW/115,68/forecast/hourly"
            }
        }"#;

        let mock = server
            .mock("GET", "/points/47.5619,-122.625")
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(mock_response)
            .create();

        let result = get_weather_point("47.5619", "-122.625", Some(&server.url())).await;
        let response = result.expect("points lookup should succeed against the mock");
        assert!(response.properties.forecast.contains("/forecast"));
        assert!(response
            .properties
            .forecast_hourly
            .unwrap()
            .contains("/forecast/hourly"));

        // Proves the request actually reached the mock. Without this the test
        // would still pass if the function bypassed base_url and called the
        // live API, which is exactly the bug this test used to have (#21).
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_detailed_forecast() {
        let mut server = Server::new_async().await;
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

        server
            .mock("GET", "/gridpoints/SEW/115,68/forecast")
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(mock_response)
            .create();

        let result =
            get_detailed_forecast(&format!("{}/gridpoints/SEW/115,68/forecast", server.url()))
                .await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.properties.periods.len(), 1);
        assert_eq!(response.properties.periods[0].name, "Tonight");
    }

    #[tokio::test]
    async fn test_get_hourly_forecast() {
        let mut server = Server::new_async().await;
        // Field names here MUST stay camelCase to match what api.weather.gov
        // actually sends. An earlier snake_case mock silently masked a bug that
        // made --forecast-type hourly fail against the live API (see #19/#20).
        let mock_response = r#"{
            "properties": {
                "periods": [
                    {
                        "number": 1,
                        "startTime": "2024-01-29T02:32:45+00:00",
                        "endTime": "2024-01-29T03:32:45+00:00",
                        "isDaytime": false,
                        "temperature": 51,
                        "temperatureUnit": "F",
                        "windSpeed": "5 mph",
                        "windDirection": "SSW",
                        "shortForecast": "Partly Cloudy"
                    }
                ]
            }
        }"#;

        server
            .mock("GET", "/gridpoints/SEW/115,68/forecast/hourly")
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(mock_response)
            .create();

        let result = get_hourly_forecast(&format!(
            "{}/gridpoints/SEW/115,68/forecast/hourly",
            server.url()
        ))
        .await;
        let response = result.expect("hourly forecast should deserialize real NWS camelCase JSON");
        assert_eq!(response.properties.periods.len(), 1);

        // Assert on every renamed field, not just temperature. Checking only
        // temperature would still pass if the camelCase renames were dropped,
        // because temperature is the one field whose name needs no remapping.
        let period = &response.properties.periods[0];
        assert_eq!(period.start_time, "2024-01-29T02:32:45+00:00");
        assert_eq!(period.temperature, 51);
        assert_eq!(period.temperature_unit, "F");
        assert_eq!(period.wind_speed, "5 mph");
        assert_eq!(period.wind_direction, "SSW");
        assert_eq!(period.short_forecast, "Partly Cloudy");
    }

    #[tokio::test]
    async fn test_get_lat_lon() {
        let mut server = Server::new_async().await;
        let mock_response = r#"[
            {
                "lat": "47.5619",
                "lon": "-122.625",
                "display_name": "Seattle, King County, Washington, USA"
            }
        ]"#;

        server
            .mock("GET", "/search")
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
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/points/invalid,invalid")
            .with_status(400)
            .with_header("content-type", "application/geo+json")
            .with_body(r#"{"error": "Invalid coordinates"}"#)
            .create();

        let result = get_weather_point("invalid", "invalid", Some(&server.url())).await;
        let err = result.expect_err("a 400 from the points endpoint should be an error");

        // Assert on the message, not just is_err(). This test previously hit
        // the live API, where any network failure satisfied is_err() -- it
        // would have passed with the error handling deleted entirely.
        let msg = format!("{err}");
        assert!(
            msg.contains("Weather.gov returned an error for points data"),
            "unexpected error message: {msg}"
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_requests_send_identifying_user_agent() {
        // End-to-end check that the shared client's UA reaches the wire. The
        // mock only matches when the header is present and correct, so a
        // regression to the old placeholder fails at mock.assert().
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/points/47.5619,-122.625")
            .match_header("user-agent", crate::http::user_agent().as_str())
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(r#"{"properties":{"forecast":"https://example.invalid/forecast"}}"#)
            .create();

        get_weather_point("47.5619", "-122.625", Some(&server.url()))
            .await
            .expect("request carrying the shared User-Agent should match the mock");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_lat_lon_rate_limited() {
        let mut server = Server::new_async().await;
        // Nominatim answers a blocked/limited request with HTML, not JSON.
        let mock = server
            .mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(429)
            .with_header("content-type", "text/html")
            .with_body("<html><body>Too Many Requests</body></html>")
            .create();

        let input = LocationInput::City("Seattle".to_string());
        let err = get_lat_lon(input, Some(&server.url()))
            .await
            .expect_err("HTTP 429 should be an error");

        let msg = format!("{err}");
        assert!(msg.contains("rate-limited"), "unexpected message: {msg}");
        // The old code fell through to serde and blamed the parser instead.
        assert!(
            !msg.contains("Error parsing JSON"),
            "rate limiting should not surface as a parse error: {msg}"
        );
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_lat_lon_server_error_reports_status() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/search")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .with_header("content-type", "text/html")
            .with_body("<html><body>Service Unavailable</body></html>")
            .create();

        let input = LocationInput::City("Seattle".to_string());
        let err = get_lat_lon(input, Some(&server.url()))
            .await
            .expect_err("HTTP 503 should be an error");

        let msg = format!("{err}");
        assert!(msg.contains("503"), "status code should be surfaced: {msg}");
        assert!(!msg.contains("Error parsing JSON"), "got: {msg}");
        mock.assert();
    }

    #[tokio::test]
    async fn test_forecast_requests_ask_for_geojson() {
        // The Accept header used to be sent on /points only, leaving the two
        // forecast calls relying on GeoJSON being the server default. The mock
        // matches on the header, so dropping it again fails at mock.assert().
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/gridpoints/SEW/115,68/forecast")
            .match_header("accept", "application/geo+json")
            .with_status(200)
            .with_header("content-type", "application/geo+json")
            .with_body(r#"{"properties":{"periods":[]}}"#)
            .create();

        get_detailed_forecast(&format!("{}/gridpoints/SEW/115,68/forecast", server.url()))
            .await
            .expect("forecast request should carry the GeoJSON Accept header");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_lat_lon_no_results() {
        let mut server = Server::new_async().await;
        let mock_response = r#"[]"#;

        server
            .mock("GET", "/search")
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
