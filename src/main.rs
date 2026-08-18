//! wxdotgov
//!
//! Takes a US postal code, or a city name with an optional state code, and
//! prints that location's weather forecast.
//!
//! The lookup runs in three steps:
//!
//! 1. Geocode the location to a latitude/longitude via nominatim.openstreetmap.org
//! 2. Resolve those coordinates to a forecast office and grid point via api.weather.gov
//! 3. Fetch and print the forecast for that grid point
//!
//! Examples:
//!
//! ```text
//! $ wxdotgov --zip 12345
//! $ wxdotgov --city "New York"
//! $ wxdotgov --city Seattle --state WA
//! $ wxdotgov --city Seattle --state WA --forecast-type hourly --pretty
//! ```

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use colored::*;

mod http;
mod nominatim;
mod weatherdotgov;

use crate::nominatim::get_lat_lon;
use crate::weatherdotgov::{get_detailed_forecast, get_hourly_forecast, get_weather_point};

#[derive(Debug, PartialEq)]
pub enum LocationInput {
    PostalCode(String),
    PostalCodePlusFour(String, String),
    City(String),
    CityWithState(String, String),
}

/// Parse a US ZIP code, accepting both 5-digit and ZIP+4 forms.
///
/// The ZIP+4 form has always been documented but was never parsed, leaving
/// `PostalCodePlusFour` unconstructible outside tests.
fn parse_zip(zip: &str) -> Result<LocationInput> {
    let zip = zip.trim();

    let is_digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());

    if let Some((base, plus_four)) = zip.split_once('-') {
        if base.len() == 5 && is_digits(base) && plus_four.len() == 4 && is_digits(plus_four) {
            return Ok(LocationInput::PostalCodePlusFour(
                base.to_string(),
                plus_four.to_string(),
            ));
        }
    } else if zip.len() == 5 && is_digits(zip) {
        return Ok(LocationInput::PostalCode(zip.to_string()));
    }

    bail!(
        "'{}' is not a valid US ZIP code. Expected 5 digits (12345) \
         or ZIP+4 (12345-6789).",
        zip
    )
}

/// Turn the parsed CLI arguments into a single location query.
fn build_location_input(
    zip: Option<String>,
    city: Option<String>,
    state: Option<String>,
) -> Result<LocationInput> {
    // clap's required ArgGroup guarantees exactly one of zip/city is present.
    match (zip, city) {
        (Some(zip), _) => {
            if state.is_some() {
                // Previously ignored in silence, which looked like the state
                // had been applied to the lookup.
                eprintln!("warning: --state is ignored when --zip is given");
            }
            parse_zip(&zip)
        }
        (None, Some(city)) => Ok(match state {
            Some(state) => LocationInput::CityWithState(city, state),
            None => LocationInput::City(city),
        }),
        (None, None) => unreachable!("clap's required ArgGroup guarantees zip or city"),
    }
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

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments.
    let args = Args::parse();

    // Build the location input.
    let location_input = build_location_input(args.zip, args.city, args.state)?;

    // Step 1: Geocode with Nominatim.
    let location = get_lat_lon(location_input, None).await?;
    println!("Location found: {}", location.display_name);

    // Step 2: Get points data from Weather.gov.
    let points_resp = get_weather_point(&location.lat, &location.lon, None).await?;

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
            println!(
                "\n{}",
                "Weather Forecast:".bold().underline().bright_white()
            );
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
            println!(
                "\n{}",
                "Hourly Weather Forecast:".bold().underline().bright_white()
            );
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

#[cfg(test)]
mod tests {
    mod api_tests;
    mod app_tests;
    mod integration_tests;
    mod location_tests;
}
