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

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use colored::*;

mod nomatim;
mod weatherdotgov;

use crate::nomatim::get_lat_lon;
use crate::weatherdotgov::{get_detailed_forecast, get_hourly_forecast, get_weather_point};

#[derive(Debug)]
pub enum LocationInput {
    PostalCode(String),
    PostalCodePlusFour(String, String),
    City(String),
    CityWithState(String, String),
}

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    arg_required_else_help = true,
    group = clap::ArgGroup::new("location")
        .required(true)
        .args(["zip", "city"]),
)]
struct Args {
    /// ZIP code in the U.S.
    #[arg(short, long, group = "location")]
    zip: Option<String>,

    /// City name (when using city/state search)
    #[arg(short, long, group = "location")]
    city: Option<String>,

    /// State abbreviation (e.g., CA)
    #[arg(short, long)]
    state: Option<String>,

    /// Enable pretty output with colors and formatting.
    #[arg(long)]
    pretty: bool,

    /// Forecast type to display. Options: detailed or hourly.
    #[arg(long, value_enum, default_value_t = ForecastType::Detailed)]
    forecast_type: ForecastType,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
enum ForecastType {
    Detailed,
    Hourly,
}

#[cfg(test)]
mod tests {
    mod integration_tests;
    mod api_tests;
    mod app_tests;
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments.
    let args = Args::parse();

    // Build the location input.
    let location_input = if let Some(zip) = args.zip {
        LocationInput::PostalCode(zip)
    } else if let Some(city) = args.city.clone() {
        if let Some(state) = args.state {
            LocationInput::CityWithState(city, state)
        } else {
            LocationInput::City(city)
        }
    } else {
        bail!("Please provide either a ZIP code or both city and state.");
    };

    // Step 1: Geocode with Nominatim.
    let location = get_lat_lon(location_input, None).await?;
    println!("Location found: {}", location.display_name);

    // Step 2: Get points data from Weather.gov.
    let points_resp = get_weather_point(&location.lat, &location.lon).await?;

    // Select the forecast URL based on the chosen forecast type.
    let forecast_url = match args.forecast_type {
        ForecastType::Hourly => points_resp
            .properties
            .forecast_hourly
            .as_ref()
            .context("Hourly forecast not available for this location")?,
        ForecastType::Detailed => &points_resp.properties.forecast,
    };

    println!("Fetching forecast from: {}", forecast_url);

    // Step 3: Fetch and display the forecast.
    if args.forecast_type == ForecastType::Detailed {
        let forecast_resp = get_detailed_forecast(forecast_url).await?;
        if args.pretty {
            println!("\n{}", "Weather Forecast:".bold().underline().bright_white());
        } else {
            println!("\nWeather Forecast:");
        }
        println!();

        // Print each detailed forecast period.
        for period in forecast_resp.properties.periods.iter() {
            if args.pretty {
                // Bold and blue for the period name.
                println!("{}", period.name.bold().blue());
                // Green for detailed forecast.
                println!("{}", period.detailed_forecast.green());
            } else {
                println!("{}: {}", period.name, period.detailed_forecast);
            }
            // Dim the separator line.
            if args.pretty {
                println!("{}", "-------------------------------------".dimmed());
            } else {
                println!("-------------------------------------");
            }
        }
    } else {
        // Hourly forecast branch.
        let hourly_forecast_resp = get_hourly_forecast(forecast_url).await?;
        if args.pretty {
            println!("\n{}", "Hourly Weather Forecast:".bold().underline().bright_white());
        } else {
            println!("\nHourly Weather Forecast:");
        }
        println!();

        // Print each hourly forecast period.
        for period in hourly_forecast_resp.properties.periods.iter() {
            if args.pretty {
                // Bold and blue for the start time.
                println!("{}", period.start_time.bold().blue());
                // Use yellow for temperature and cyan for the rest.
                println!(
                    "{}°{} | {} | Wind: {} {}",
                    period.temperature.to_string().yellow(),
                    period.temperature_unit.yellow(),
                    period.short_forecast.cyan(),
                    period.wind_speed.cyan(),
                    period.wind_direction.cyan()
                );
            } else {
                println!(
                    "{}: {}°{} | {} | Wind: {} {}",
                    period.start_time,
                    period.temperature,
                    period.temperature_unit,
                    period.short_forecast,
                    period.wind_speed,
                    period.wind_direction,
                );
            }
            if args.pretty {
                println!("{}", "-------------------------------------".dimmed());
            } else {
                println!("-------------------------------------");
            }
        }
    }

    Ok(())
}
