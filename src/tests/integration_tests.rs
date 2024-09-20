#[cfg(test)]
mod tests {
    use crate::{extract_input, InputType};

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
}
