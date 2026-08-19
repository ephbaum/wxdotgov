#[cfg(test)]
mod tests {
    use crate::{build_location_input, parse_zip, LocationInput};

    #[test]
    fn parses_five_digit_zip() {
        assert_eq!(
            parse_zip("98101").unwrap(),
            LocationInput::PostalCode("98101".to_string())
        );
    }

    #[test]
    fn parses_zip_plus_four() {
        // Documented since the first commit but never actually parsed, which
        // left PostalCodePlusFour unconstructible outside tests.
        assert_eq!(
            parse_zip("12345-6789").unwrap(),
            LocationInput::PostalCodePlusFour("12345".to_string(), "6789".to_string())
        );
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(
            parse_zip("  98101 ").unwrap(),
            LocationInput::PostalCode("98101".to_string())
        );
    }

    #[test]
    fn rejects_malformed_zips() {
        // Previously any string was forwarded to Nominatim unvalidated.
        for bad in [
            "abcde",      // not digits
            "1234",       // too short
            "123456",     // too long
            "12345-678",  // +4 too short
            "12345-",     // missing +4
            "-6789",      // missing base
            "12345-abcd", // +4 not digits
            "",
        ] {
            let result = parse_zip(bad);
            assert!(result.is_err(), "expected {bad:?} to be rejected");
            let msg = format!("{}", result.unwrap_err());
            assert!(
                msg.contains("not a valid US ZIP code"),
                "unexpected message for {bad:?}: {msg}"
            );
        }
    }

    #[test]
    fn city_without_state_is_city_only() {
        assert_eq!(
            build_location_input(None, Some("Seattle".to_string()), None).unwrap(),
            LocationInput::City("Seattle".to_string())
        );
    }

    #[test]
    fn city_with_state_is_combined() {
        assert_eq!(
            build_location_input(None, Some("Seattle".to_string()), Some("WA".to_string()))
                .unwrap(),
            LocationInput::CityWithState("Seattle".to_string(), "WA".to_string())
        );
    }

    #[test]
    fn zip_takes_precedence_and_ignores_state() {
        // --state alongside --zip is ignored (now with a warning on stderr);
        // this pins the resulting query so the ZIP path cannot silently start
        // folding state into the lookup.
        assert_eq!(
            build_location_input(Some("98101".to_string()), None, Some("OR".to_string())).unwrap(),
            LocationInput::PostalCode("98101".to_string())
        );
    }

    #[test]
    fn invalid_zip_propagates_through_builder() {
        let err = build_location_input(Some("nope".to_string()), None, None).unwrap_err();
        assert!(format!("{err}").contains("not a valid US ZIP code"));
    }
}
