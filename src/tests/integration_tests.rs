#[cfg(test)]
mod tests {
    use crate::{extract_input, InputType};
    use crate::nomatim::get_lat_lon;
    use crate::weatherdotgov::{get_weather_point, get_weather_forecast};
    use mockito::{mock, Matcher};

    #[test]
    fn test_extract_input_postal_code() {
        let input = "12345";
        let expected = Some(InputType::PostalCode(input.to_string()));
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_extended_postal_code() {
        let input = "12345-6789";
        let expected = Some(InputType::ExtendedPostalCode("12345".to_string(), "6789".to_string()));
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_city_name() {
        let input = "New York";
        let expected = Some(InputType::City(input.to_string()));
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_city_state() {
        let input = "Seattle, WA";
        let expected = Some(InputType::CityWithState("Seattle".to_string(), "WA".to_string()));
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_invalid_input() {
        let input = "---";
        let expected = None;
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_postal_code_too_long() {
        let input = "123456";
        let expected = None;
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_postal_code_plus_four_too_long() {
        let input = "12345-67890";
        let expected = None;
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_postal_code_too_short() {
        let input = "1234";
        let expected = None;
        assert_eq!(extract_input(input), expected);
    }

    #[test]
    fn test_extract_input_postal_code_plus_four_too_short() {
        let input = "12345-678";
        let expected = None;
        assert_eq!(extract_input(input), expected);
    }

    #[tokio::test]
    async fn test_get_lat_lon_postal_code() {
        let _m = mock("GET", Matcher::Regex(r"^/search\?postalcode=12345&format=json$".to_string()))
            .with_status(200)
            .with_body(r#"[{"lat": "40.7128", "lon": "-74.0060"}]"#)
            .create();

        let result = get_lat_lon(crate::LocationInput::PostalCode("12345".to_string()), Some(&mockito::server_url())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.lat, "40.7128");
        assert_eq!(response.lon, "-74.0060");
    }

    #[tokio::test]
    async fn test_get_weather_point() {
        let _m = mock("GET", Matcher::Regex(r"^/points/40.7128,-74.0060$".to_string()))
            .with_status(200)
            .with_body(r#"{"properties": {"forecast": "https://api.weather.gov/gridpoints/SEW/115,68/forecast"}}"#)
            .create();

        let result = get_weather_point(&"40.7128".to_string(), &"-74.0060".to_string()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.properties.forecast, "https://api.weather.gov/gridpoints/SEW/115,68/forecast");
    }

    #[tokio::test]
    async fn test_get_weather_forecast() {
        let _m = mock("GET", Matcher::Regex(r"^/gridpoints/SEW/115,68/forecast$".to_string()))
            .with_status(200)
            .with_body(r#"{"properties": {"periods": [{"name": "Tonight", "detailedForecast": "A chance of rain."}]}}"#)
            .create();

        let weather_point = crate::weatherdotgov::WeatherPoint {
            properties: crate::weatherdotgov::Properties {
                forecast: format!("{}/gridpoints/SEW/115,68/forecast", &mockito::server_url()),
            },
        };

        let result = get_weather_forecast(weather_point).await;
        assert!(result.is_ok());
        let forecast = result.unwrap();
        assert!(forecast.contains("A chance of rain."));
    }
}
