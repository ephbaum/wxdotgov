#[cfg(test)]
mod tests {
    use crate::{extract_input, InputType};
    use crate::nomatim::get_lat_lon;
    use crate::weatherdotgov::{get_weather_point, get_weather_forecast};
    use crate::main;
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
    async fn test_get_lat_lon() {
        let _m = mock("GET", "/search")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("postalcode".into(), "12345".into()),
                Matcher::UrlEncoded("format".into(), "json".into()),
            ]))
            .with_status(200)
            .with_body(r#"[{"lat": "47.5619", "lon": "-122.625"}]"#)
            .create();

        let result = get_lat_lon(crate::LocationInput::PostalCode("12345".to_string()), Some(&mockito::server_url())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.lat, "47.5619");
        assert_eq!(response.lon, "-122.625");
    }

    #[tokio::test]
    async fn test_get_weather_point() {
        let _m = mock("GET", "/points/47.5619,-122.625")
            .with_status(200)
            .with_body(r#"{"properties": {"forecast": "https://api.weather.gov/gridpoints/SEW/115,68/forecast"}}"#)
            .create();

        let result = get_weather_point(&"47.5619".to_string(), &"-122.625".to_string()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.properties.forecast, "https://api.weather.gov/gridpoints/SEW/115,68/forecast");
    }

    #[tokio::test]
    async fn test_get_weather_forecast() {
        let _m = mock("GET", "/gridpoints/SEW/115,68/forecast")
            .with_status(200)
            .with_body(r#"{"properties": {"periods": [{"number": 1, "name": "Tonight", "startTime": "2024-01-28T18:00:00-08:00", "endTime": "2024-01-29T06:00:00-08:00", "isDaytime": false, "temperature": 51, "temperatureUnit": "F", "temperatureTrend": "rising", "probabilityOfPrecipitation": {"unitCode": "wmoUnit:percent", "value": 40}, "dewpoint": {"unitCode": "wmoUnit:degC", "value": 12.222222222222221}, "relativeHumidity": {"unitCode": "wmoUnit:percent", "value": 91}, "windSpeed": "5 mph", "windDirection": "SSW", "icon": "https://api.weather.gov/icons/land/night/rain,40/rain,20?size=medium", "shortForecast": "Chance Light Rain", "detailedForecast": "A chance of rain. Mostly cloudy. Low around 51, with temperatures rising to around 53 overnight. South southwest wind around 5 mph. Chance of precipitation is 40%. New rainfall amounts less than a tenth of an inch possible."}]}}"#)
            .create();

        let weather_point = crate::weatherdotgov::WeatherPoint {
            properties: crate::weatherdotgov::Properties {
                forecast: "https://api.weather.gov/gridpoints/SEW/115,68/forecast".to_string(),
            },
        };

        let result = get_weather_forecast(weather_point).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Chance Light Rain"));
    }

    #[tokio::test]
    async fn test_main() {
        let _m1 = mock("GET", "/search")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("postalcode".into(), "12345".into()),
                Matcher::UrlEncoded("format".into(), "json".into()),
            ]))
            .with_status(200)
            .with_body(r#"[{"lat": "47.5619", "lon": "-122.625"}]"#)
            .create();

        let _m2 = mock("GET", "/points/47.5619,-122.625")
            .with_status(200)
            .with_body(r#"{"properties": {"forecast": "https://api.weather.gov/gridpoints/SEW/115,68/forecast"}}"#)
            .create();

        let _m3 = mock("GET", "/gridpoints/SEW/115,68/forecast")
            .with_status(200)
            .with_body(r#"{"properties": {"periods": [{"number": 1, "name": "Tonight", "startTime": "2024-01-28T18:00:00-08:00", "endTime": "2024-01-29T06:00:00-08:00", "isDaytime": false, "temperature": 51, "temperatureUnit": "F", "temperatureTrend": "rising", "probabilityOfPrecipitation": {"unitCode": "wmoUnit:percent", "value": 40}, "dewpoint": {"unitCode": "wmoUnit:degC", "value": 12.222222222222221}, "relativeHumidity": {"unitCode": "wmoUnit:percent", "value": 91}, "windSpeed": "5 mph", "windDirection": "SSW", "icon": "https://api.weather.gov/icons/land/night/rain,40/rain,20?size=medium", "shortForecast": "Chance Light Rain", "detailedForecast": "A chance of rain. Mostly cloudy. Low around 51, with temperatures rising to around 53 overnight. South southwest wind around 5 mph. Chance of precipitation is 40%. New rainfall amounts less than a tenth of an inch possible."}]}}"#)
            .create();

        let args = vec!["wxdotgov".to_string(), "12345".to_string()];
        let result = main().await;
        assert!(result.is_ok());
    }
}
