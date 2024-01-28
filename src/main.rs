use std::{env, process::exit};
use regex::Regex;

#[cfg(test)]
mod tests {
    mod integration_tests;
}

// a program that takes a US postal code or a city name and, optionally, a state code and outputs the location's weather
// if no arguments are passed, print a message and exit
// the program should accept a US postal code or a city name and, optionally, a state code 
// if the input is not valid, print a message and exit
// if the input is valid, get the latitude and longitude of the location from nomatim.openstreetmap.org
// use the latitude and longitude to get fetch the weather office and grid points from api.weather.gov
// use the weather office and grid points to get the weather report and output from api.weather.gov

enum InputType {
    PostalCode(String),
    ExtendedPostalCode(String, String),
    CityName(String),
    CityState(String, String),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = get_args(args);

    match extract_input(&input) {
        Some(InputType::PostalCode(code)) => {
            println!("Got a postal code: {}", code);
            // handle postal code
        }
        Some(InputType::ExtendedPostalCode(code, extension)) => {
            println!("Got an extended postal code: {}-{}", code, extension);
            // handle extended postal code
        }
        Some(InputType::CityName(city)) => {
            println!("Got a city name: {}", city);
            // handle city name
        }
        Some(InputType::CityState(city, state)) => {
            println!("Got a city and state: {}, {}", city, state);
            // handle city and state
        
        }
        None => {
            exit_with_message("Invalid input", 1);
        }
    }
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
        return Some(InputType::CityName(input.to_string()));
    } else if city_state.is_match(input) {
        let mut split_input = input.split(",");
        let first = split_input.next().unwrap().trim();
        let second = split_input.next().unwrap().trim();
        return Some(InputType::CityState(first.to_string(), second.to_string()));
    } else {
        return None;
    }
}

// private method that outputs a message and exits the program with a status code appropriate status code
fn exit_with_message(message: &str, status_code: i32) {
    println!("{}", message);
    exit(status_code);
}
