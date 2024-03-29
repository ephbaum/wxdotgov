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

use crate::LocationInput;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NomatimResponse {
    pub lat: String,
    pub lon: String,
}

pub async fn get_lat_lon(input: LocationInput, base_url: Option<&str>) -> Result<NomatimResponse, Error> {
    let default_base_url = "https://nominatim.openstreetmap.org/search?";
    let base_url = base_url.unwrap_or(default_base_url);
    let format = "&format=json";
    let mut url = String::new();

    match input {
        LocationInput::PostalCode(code) => {
            url.push_str(base_url);
            url.push_str("postalcode=");
            url.push_str(&code);
            url.push_str(format);
        }
        LocationInput::PostalCodePlusFour(code, _) => {
            url.push_str(base_url);
            url.push_str("postalcode=");
            url.push_str(&code);
            url.push_str(format);
        }
        LocationInput::City(city) => {
            url.push_str(base_url);
            url.push_str("city=");
            url.push_str(&city);
            url.push_str(format);
        }
        LocationInput::CityWithState(city, state) => {
            url.push_str(base_url);
            url.push_str("city=");
            url.push_str(&city);
            url.push_str("&state=");
            url.push_str(&state);
            url.push_str(format);
        }
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