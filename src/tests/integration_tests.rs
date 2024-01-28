#[cfg(test)]
mod tests {
    use crate::{get_args, extract_input, InputType};

    #[test]
    fn test_get_args() {
        let args = vec![
            "program_name".to_string(), // This would be the name of the program
            "12345".to_string(),
        ];
        let input = get_args(args);
        assert_eq!(input, "12345");
    }

    #[test]
    fn test_extract_input_postal_code() {
        let input = "12345";
        match extract_input(input) {
            Some(InputType::PostalCode(code)) => assert_eq!(code, "12345"),
            _ => panic!("Expected PostalCode"),
        }
    }

    #[test]
    fn test_extract_input_extended_postal_code() {
        let input = "12345-6789";
        match extract_input(input) {
            Some(InputType::ExtendedPostalCode(code, extension)) => {
                assert_eq!(code, "12345");
                assert_eq!(extension, "6789");
            }
            _ => panic!("Expected ExtendedPostalCode"),
        }
    }

    #[test]
    fn test_extract_input_city_name() {
        let input = "New York";
        match extract_input(input) {
            Some(InputType::CityName(city)) => assert_eq!(city, "New York"),
            _ => panic!("Expected CityName"),
        }
    }

    #[test]
    fn test_extract_input_city_state() {
        let input = "New York, NY";
        match extract_input(input) {
            Some(InputType::CityState(city, state)) => {
                assert_eq!(city, "New York");
                assert_eq!(state, "NY");
            }
            _ => panic!("Expected CityState"),
        }
    }

    #[test]
    fn test_extract_input_none() {
        let input = "---";
        match extract_input(input) {
            None => (),
            _ => panic!("Expected None"),
        }
    }

    #[test]
    fn test_extract_input_six_digit_postal_code() {
        let input = "123456";
        match extract_input(input) {
            None => (),
            _ => panic!("Expected None"),
        }
    }

    #[test]
    fn test_extract_input_five_digit_postal_code_plus_more() {
        let input = "12345-67890";
        match extract_input(input) {
            None => (),
            _ => panic!("Expected None"),
        }
    }
}