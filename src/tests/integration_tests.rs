#[cfg(test)]
mod tests {
    use crate::{extract_input, InputType};
    use crate::nomatim::get_lat_lon;
    use crate::weatherdotgov::{get_weather_point, get_weather_forecast};
    use crate::main;

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
        let input = LocationInput::PostalCode("12345".to_string());
        let result = get_lat_lon(input, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_lat_lon_city() {
        let input = LocationInput::City("New York".to_string());
        let result = get_lat_lon(input, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_lat_lon_city_with_state() {
        let input = LocationInput::CityWithState("Seattle".to_string(), "WA".to_string());
        let result = get_lat_lon(input, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_weather_point() {
        let latitude = "47.5619".to_string();
        let longitude = "-122.625".to_string();
        let result = get_weather_point(&latitude, &longitude).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_weather_forecast() {
        let latitude = "47.5619".to_string();
        let longitude = "-122.625".to_string();
        let weather_point = get_weather_point(&latitude, &longitude).await.unwrap();
        let result = get_weather_forecast(weather_point).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_main_postal_code() {
        let args = vec!["wxdotgov".to_string(), "12345".to_string()];
        let result = main(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_main_city() {
        let args = vec!["wxdotgov".to_string(), "New York".to_string()];
        let result = main(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_main_city_with_state() {
        let args = vec!["wxdotgov".to_string(), "Seattle, WA".to_string()];
        let result = main(args).await;
        assert!(result.is_ok());
    }
}
