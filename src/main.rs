/** 
 * wxdotgov
 * 
 * a program that takes a US postal code or a city name and, optionally, a state code and outputs the location's weather
 * if no arguments are passed, print a message and exit
 * the program should accept a US postal code or a city name and, optionally, a state code 
 * if the input is not valid, print a message and exit
 * if the input is valid, get the latitude and longitude of the location from nomatim.openstreetmap.org
 * use the latitude and longitude to get fetch the weather office and grid points from api.weather.gov
 * use the weather office and grid points to get the weather report and output from api.weather.gov
 * 
 * Examples:
 * 
 * $ wxdotgov 12345
 * $ wxdotgov 12345-6789
 * $ wxdotgov "New York"
 * $ wxdotgov "Seattle, WA"
 */

use std::env;
use regex::Regex;


mod nomatim;
mod weatherdotgov;

use crate::nomatim::get_lat_lon;
use crate::weatherdotgov::{get_weather_forecast, get_weather_point};

#[cfg(test)]
mod tests {
    mod integration_tests;
}


#[derive(Debug, PartialEq)]
enum InputType {
    PostalCode(String),
    ExtendedPostalCode(String, String),
    City(String),
    CityWithState(String, String),
}

#[derive(Debug)]
pub enum LocationInput {
    PostalCode(String),
    PostalCodePlusFour(String, String),
    City(String),
    CityWithState(String, String),
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let input = get_args(args);

    let input_type = match extract_input(&input) {
        Some(input_type) => input_type,
        None => {
            println!("Invalid input");
            return;
        }
    };

    let result = match &input_type {
        InputType::PostalCode(code) | InputType::ExtendedPostalCode(code, _) => {
            println!("Got a code: {}", code);
            get_lat_lon(LocationInput::PostalCode(code.clone()), None).await
        }
        InputType::City(city) => {
            println!("Got a city: {}", city);
            get_lat_lon(LocationInput::City(city.clone()), None).await
        }
        InputType::CityWithState(city, state) => {
            println!("Got a city and state: {}, {}", city, state);
            get_lat_lon(LocationInput::CityWithState(city.clone(), state.clone()), None).await
        }
    };

    match result {
        Ok(response) => {
            println!("Got a response: {:?}", response);
            // Assuming the response is a struct with lat and lon fields
            let weather_point = get_weather_point(&response.lat, &response.lon).await;
            match weather_point {
                Ok(wp) => {
                    let forecast = get_weather_forecast(wp).await;
                    match forecast {
                        Ok(f) => {
                            println!("Got a forecast: {}", f);
                        }
                        Err(e) => {
                            println!("Failed to get forecast: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to get weather point: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Got an error: {:?}", e);
        }
    }

    // use lat/lon with weatherdotgov module to get forecast


}

fn get_args(args: Vec<String>) -> String {
    args.into_iter().skip(1).collect::<Vec<String>>().join(" ")
}

fn extract_input(input: &str) -> Option<InputType> {
    let postal_code = Regex::new(r"^\d{5}$").unwrap();
    let extended_postal_code = Regex::new(r"^\d{5}-\d{4}$").unwrap();
    let city_name = Regex::new(r"^[a-zA-Z\s.]+$").unwrap();
    let city_state = Regex::new(r"^[a-zA-Z\s.]+,\s*[a-zA-Z]{2}$").unwrap();

    if postal_code.is_match(input) {
        return Some(InputType::PostalCode(input.to_string()));
    } else if extended_postal_code.is_match(input) {
        let mut split_input = input.split("-");
        let first = split_input.next().unwrap();
        let second = split_input.next().unwrap();
        return Some(InputType::ExtendedPostalCode(first.to_string(), second.to_string()));
    } else if city_name.is_match(input) {
        return Some(InputType::City(input.to_string()));
    } else if city_state.is_match(input) {
        let mut split_input = input.split(",");
        let first = split_input.next().unwrap().trim();
        let second = split_input.next().unwrap().trim();
        return Some(InputType::CityWithState(first.to_string(), second.to_string()));
    } else {
        return None;
    }
}
