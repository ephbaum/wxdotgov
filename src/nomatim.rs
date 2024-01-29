/**
 * Nomatim API
 * 
 * This module contains the Nomatim API
 * 
 * It's meant to accept either a postal code, a city, or a city with a state code
 * 
 * It will return a JSON object with the geocoded latitude and longitude for the requested location
 * 
 * If the request is based on a postal code, it will call /search?postalcode={postal_code}&format=json
 * If the request is based on a city it will call /search?city={city}&format=json
 * If the request is based on a city and state it will call /search?city={city}&state={state}&format=json
 * 
 * Nomatim returns an array of objects, each of which contains the geocoded latitude and longitude
 * For now we will only return the first OSM object in the array
 * 
 * Nomatim API docs: https://nominatim.org/release-docs/develop/api/Search/
 */

use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NomatimResponse {
    pub lat: String,
    pub lon: String,
}

pub async fn get_lat_lon(input: &str, base_url: Option<&str>) -> Result<NomatimResponse, Error> {
    let default_base_url = "https://nominatim.openstreetmap.org/search?";
    let base_url = base_url.unwrap_or(default_base_url);
    let format = "&format=json";
    let mut url = String::new();

    // This check should evaluate if postal code is 5 digits or, if 5 digits plus four it should remove the plus 4, if city, or if city and state
    if input.contains(",") {
        let mut split_input = input.split(",");
        let city = split_input.next().unwrap().trim();
        let state = split_input.next().unwrap().trim();
        url.push_str(base_url);
        url.push_str("city=");
        url.push_str(city);
        url.push_str("&state=");
        url.push_str(state);
        url.push_str(format);
    } else if input.len() == 5 && input.chars().all(char::is_numeric) {
        url.push_str(base_url);
        url.push_str("postalcode=");
        url.push_str(input);
        url.push_str(format);
    } else if input.contains("-") {
        let postal_code = input.split("-").next().unwrap();
        url.push_str(base_url);
        url.push_str("postalcode=");
        url.push_str(postal_code);
        url.push_str(format);
    } else {
        url.push_str(base_url);
        url.push_str("city=");
        url.push_str(input.trim());
        url.push_str(format);
    }
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "reqwest")
        .send()
        .await?
        .json::<Vec<NomatimResponse>>()
        .await?;

    Ok(response[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_get_lat_lon_with_postal_code() {
        let mut server = Server::new();

        let _m = server.mock("GET", "/search?format=json&q=90210")
            .with_status(200)
            .with_body(r#"[{"lat": "34.0901", "lon": "-118.4065"}]"#)
            .create();

        let result = get_lat_lon("Los Angeles", Some(server.url().as_str())).await;
        println!("{:?}", result);  // print the result
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.lat, "34.0901");
        assert_eq!(response.lon, "-118.4065");

        _m.assert();
    }

    #[tokio::test]
    async fn test_get_lat_lon_with_city() {
        let mut server = Server::new();

        let _m = server.mock("GET", "/search?format=json&q=Los%20Angeles")
            .with_status(200)
            .with_body(r#"[{"lat": "34.0522", "lon": "-118.2437"}]"#)
            .create();

        let result = get_lat_lon("Los Angeles", Some(server.url().as_str())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.lat, "34.0522");
        assert_eq!(response.lon, "-118.2437");
    }

    #[tokio::test]
    async fn test_get_lat_lon_with_city_and_state() {
        let mut server = Server::new();

        let _m = server.mock("GET", "/search?format=json&q=Los%20Angeles%2CCA")
            .with_status(200)
            .with_body(r#"[{"lat": "34.0522", "lon": "-118.2437"}]"#)
            .create();

        let result = get_lat_lon("Los Angeles,CA", Some(server.url().as_str())).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.lat, "34.0522");
        assert_eq!(response.lon, "-118.2437");
    }
}