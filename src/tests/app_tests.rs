#[cfg(test)]
mod tests {
    use crate::Args;
    use crate::ForecastType;
    use clap::Parser;

    #[tokio::test]
    async fn test_args_parsing() {
        let args = vec!["wxdotgov", "--zip", "12345"];
        let parsed = Args::try_parse_from(args).unwrap();
        assert_eq!(parsed.zip.unwrap(), "12345");
        assert!(parsed.city.is_none());
        assert!(parsed.state.is_none());
        assert!(!parsed.pretty);
        assert_eq!(parsed.forecast_type, ForecastType::Detailed);
    }

    #[tokio::test]
    async fn test_args_city_state() {
        let args = vec!["wxdotgov", "--city", "Seattle", "--state", "WA"];
        let parsed = Args::try_parse_from(args).unwrap();
        assert!(parsed.zip.is_none());
        assert_eq!(parsed.city.unwrap(), "Seattle");
        assert_eq!(parsed.state.unwrap(), "WA");
    }

    #[tokio::test]
    async fn test_args_hourly_forecast() {
        let args = vec!["wxdotgov", "--zip", "12345", "--forecast-type", "hourly"];
        let parsed = Args::try_parse_from(args).unwrap();
        assert_eq!(parsed.forecast_type, ForecastType::Hourly);
    }

    #[tokio::test]
    async fn test_args_pretty_output() {
        let args = vec!["wxdotgov", "--zip", "12345", "--pretty"];
        let parsed = Args::try_parse_from(args).unwrap();
        assert!(parsed.pretty);
    }

    #[tokio::test]
    async fn test_args_invalid_forecast_type() {
        let args = vec!["wxdotgov", "--zip", "12345", "--forecast-type", "invalid"];
        let result = Args::try_parse_from(args);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_args_missing_required() {
        let args = vec!["wxdotgov"];
        let result = Args::try_parse_from(args);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_args_invalid_combination() {
        let args = vec!["wxdotgov", "--zip", "12345", "--city", "Seattle"];
        let result = Args::try_parse_from(args);
        assert!(result.is_err());
    }
} 