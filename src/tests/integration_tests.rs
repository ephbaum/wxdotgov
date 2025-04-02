#[cfg(test)]
mod tests {
    use crate::LocationInput;

    #[test]
    fn test_location_input_postal_code() {
        let input = LocationInput::PostalCode("12345".to_string());
        match input {
            LocationInput::PostalCode(code) => assert_eq!(code, "12345"),
            _ => panic!("Expected PostalCode variant"),
        }
    }

    #[test]
    fn test_location_input_postal_code_plus_four() {
        let input = LocationInput::PostalCodePlusFour("12345".to_string(), "6789".to_string());
        match input {
            LocationInput::PostalCodePlusFour(code, plus_four) => {
                assert_eq!(code, "12345");
                assert_eq!(plus_four, "6789");
            }
            _ => panic!("Expected PostalCodePlusFour variant"),
        }
    }

    #[test]
    fn test_location_input_city() {
        let input = LocationInput::City("New York".to_string());
        match input {
            LocationInput::City(city) => assert_eq!(city, "New York"),
            _ => panic!("Expected City variant"),
        }
    }

    #[test]
    fn test_location_input_city_state() {
        let input = LocationInput::CityWithState("Seattle".to_string(), "WA".to_string());
        match input {
            LocationInput::CityWithState(city, state) => {
                assert_eq!(city, "Seattle");
                assert_eq!(state, "WA");
            }
            _ => panic!("Expected CityWithState variant"),
        }
    }
}
